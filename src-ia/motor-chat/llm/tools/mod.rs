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
//! - [`helpers`]    → utilidades puras compartidas (fechas, escape, moneda).
//! - [`ventas`]     → tools que leen ventas/detalle_ventas.
//! - [`inventario`] → tools que leen productos.
//! - [`tests`]      → suite con DB en memoria.

use rusqlite::Connection;
use serde_json::Value;

mod deteccion;
mod helpers;
mod inventario;
mod ventas;
#[cfg(test)]
mod tests;

// API público hacia fuera de la crate (motor-chat y yarvis-app consumen
// estas rutas exactas: src_ia::motor_chat::llm::tools::*).
pub use deteccion::{detectar_tool_call, respuesta_final_segura, quitar_tool_calls};

use inventario::{
    get_product_info, get_products_by_category, get_restock_analysis, list_categories,
    query_inventory, search_products,
};
use ventas::{compare_periods, forecast_sales, get_top_products, query_sales};

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
        otro => Ok(serde_json::json!({ "error": format!("herramienta desconocida: {otro}") })),
    };
    resultado.map(|v| v.to_string())
}
