//! tools — Ejecutor de herramientas del fine-tuning de Qwen 1.7B.
//!
//! El modelo aprendió a responder `<tool_call>{"name": ..., "arguments": ...}</tool_call>`
//! (dataset tools_arreglado.jsonl). Este módulo cierra el ciclo:
//!   1. [`detectar_tool_call`] encuentra la llamada en la respuesta cruda.
//!   2. [`ejecutar_tool`] corre el SQL real contra yarvis.db.
//!   3. El backend re-inyecta el resultado como mensaje role:"tool" y el
//!      modelo produce la respuesta final en español.
//!
//! Los shapes de salida JSON espejan EXACTAMENTE los del dataset, para que
//! el modelo sepa leerlos sin re-entrenar.
//!
//! Organización interna:
//! - [`deteccion`]  → parseo del protocolo textual `<tool_call>`.
/// - [`helpers`]    → utilidades puras compartidas (fechas, escape, moneda).
/// - [`ventas`]     → tools que leen ventas/detalle_ventas.
/// - [`inventario`] → tools que leen productos.
/// - [`compras`]    → tools de abasto (proveedores, recepciones, órdenes).
/// - [`operativa`]  → trazabilidad (costos, lotes) y multisucursal.
/// - [`sql`]        → SQL libre de solo lectura + snapshot del schema (#15).
/// - [`tests`]      → suite con DB en memoria.

use rusqlite::Connection;
use serde_json::Value;

mod compras;
mod deteccion;
mod helpers;
mod inventario;
mod operativa;
mod sql;
mod ventas;
#[cfg(test)]
mod tests;

// API público hacia fuera de la crate (motor-chat y yarvis-app consumen
// estas rutas exactas: src_ia::motor_chat::llm::tools::*).
pub use deteccion::{detectar_tool_call, respuesta_final_segura, quitar_tool_calls};

use compras::{get_purchase_detail, query_purchase_orders, query_purchases, query_suppliers};
use inventario::{
    get_product_info, get_products_by_category, get_restock_analysis, list_categories,
    query_inventory, search_products,
};
use operativa::{list_branches, query_branch_stock, query_cost_history, query_expiring};
use sql::sql_readonly;
use ventas::{compare_periods, forecast_sales, get_top_products, query_sales};

/// Snapshot del schema para el prompt del admin (issue #15).
pub use sql::snapshot_schema;

/// Máximo de rondas tool_call→resultado que el backend permite por pregunta.
pub const MAX_RONDAS_TOOLS: usize = 3;

// ─────────────────────────────────────────────────────────────────────────────
// Despacho
// ─────────────────────────────────────────────────────────────────────────────

/// Ejecuta una tool por nombre contra la DB y devuelve su resultado JSON.
/// Los errores de negocio también regresan Ok con {"error": ...}: así el
/// modelo puede disculparse con datos reales en vez de romper el chat.
pub fn ejecutar_tool(nombre: &str, args_json: &str, db_path: &str) -> Result<String, String> {
    let args: Value = serde_json::from_str(args_json).unwrap_or(Value::Null);
    let conn = Connection::open_with_flags(
        db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .map_err(|e| format!("no se pudo abrir la base de datos: {e}"))?;

    let resultado = match nombre {
        "query_sales" => query_sales(&conn, &args),
        "compare_periods" => compare_periods(&conn, &args),
        "get_top_products" => get_top_products(&conn, &args),
        "query_inventory" => query_inventory(&conn, &args),
        "forecast_sales" => forecast_sales(&conn, &args),
        "get_product_info" => get_product_info(&conn, &args),
        "get_restock_analysis" => get_restock_analysis(&conn, &args),
        // Navegación de inventario (lectura, todos los roles)
        "search_products" => search_products(&conn, &args),
        "list_categories" => list_categories(&conn, &args),
        "get_products_by_category" => get_products_by_category(&conn, &args),
        // Abasto: proveedores, recepciones y pedidos (solo lectura)
        "query_suppliers" => query_suppliers(&conn, &args),
        "query_purchases" => query_purchases(&conn, &args),
        "get_purchase_detail" => get_purchase_detail(&conn, &args),
        "query_purchase_orders" => query_purchase_orders(&conn, &args),
        // Trazabilidad y multisucursal (solo lectura)
        "query_cost_history" => query_cost_history(&conn, &args),
        "query_expiring" => query_expiring(&conn, &args),
        "list_branches" => list_branches(&conn, &args),
        "query_branch_stock" => query_branch_stock(&conn, &args),
        // SQL libre de solo lectura (issue #15; el rol se filtra en el backend)
        "sql_readonly" => sql_readonly(&conn, &args),
        otro => Ok(serde_json::json!({ "error": format!("herramienta desconocida: {otro}") })),
    };
    resultado.map(|v| v.to_string())
}
