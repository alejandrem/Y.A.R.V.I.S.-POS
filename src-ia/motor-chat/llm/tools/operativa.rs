//! operativa — Tools de trazabilidad y multisucursal: historial de
//! costo por producto (`historial_costos` + recepciones, migración 0018),
//! lotes con caducidad (`lotes`) y stock por sucursal (`sucursales` +
//! `stock_sucursal`). Todo SOLO lectura; dinero en pesos hacia el modelo.
//!
//! Tablas vacías = respuesta vacía con shape estable (el modelo lo explica
//! en palabras, jamás inventa filas).

use rusqlite::Connection;
use serde_json::Value;

use super::helpers::{centavos_a_pesos, escape_like, str_arg, MONEDA};

/// Evolución del costo de UN producto: costo actual + cambios registrados
/// por el trigger + últimos precios pagados en recepciones como contexto.
/// `product_id` = nombre (parcial) del producto, obligatorio.
pub(crate) fn query_cost_history(conn: &Connection, args: &Value) -> Result<Value, String> {
    let producto = str_arg(args, "product_id", "");
    if producto.trim().is_empty() {
        return Ok(
            serde_json::json!({ "error": "falta 'product_id': nombre del producto a rastrear" })
        );
    }

    let actual: Option<(i64, String, f64)> = conn
        .query_row(
            "SELECT id, nombre, precio_costo FROM productos
             WHERE nombre LIKE '%' || ?1 || '%' ESCAPE '\\'
             LIMIT 1",
            rusqlite::params![escape_like(producto.trim())],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .map(Some)
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            otro => Err(otro.to_string()),
        })?;
    let Some((pid, nombre, costo_centavos)) = actual else {
        return Ok(serde_json::json!({ "error": format!("producto no encontrado: {producto}") }));
    };

    let mut hist_stmt = conn
        .prepare(
            "SELECT datetime(fecha), costo_anterior, costo_nuevo
             FROM historial_costos
             WHERE producto_id = ?1
             ORDER BY fecha DESC
             LIMIT 20",
        )
        .map_err(|e| e.to_string())?;
    let historial: Vec<Value> = hist_stmt
        .query_map(rusqlite::params![pid], |r| {
            let anterior: f64 = r.get(1)?;
            let nuevo: f64 = r.get(2)?;
            Ok(serde_json::json!({
                "fecha": r.get::<_, String>(0)?,
                "costo_anterior": centavos_a_pesos(anterior),
                "costo_nuevo": centavos_a_pesos(nuevo),
            }))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|f| f.ok())
        .collect();

    // Contexto: últimos precios pagados en recepciones (precio_sugerido).
    let mut comp_stmt = conn
        .prepare(
            "SELECT date(c.fecha), i.precio_sugerido
             FROM compras_items i
             JOIN compras c ON c.id = i.compra_id
             WHERE i.nombre LIKE '%' || ?1 || '%' ESCAPE '\\'
             ORDER BY c.fecha DESC
             LIMIT 5",
        )
        .map_err(|e| e.to_string())?;
    let ultimas_compras: Vec<Value> = comp_stmt
        .query_map(rusqlite::params![escape_like(&nombre)], |r| {
            let precio: f64 = r.get(1)?;
            Ok(serde_json::json!({
                "fecha": r.get::<_, String>(0)?,
                "precio_pagado": centavos_a_pesos(precio),
            }))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|f| f.ok())
        .collect();

    Ok(serde_json::json!({
        "producto": nombre,
        "costo_actual": centavos_a_pesos(costo_centavos),
        "moneda": MONEDA,
        "historial": historial,
        "ultimas_compras": ultimas_compras,
    }))
}

/// Lotes vencidos + por vencer en `dias` (default 30, 1..365), opcional
/// filtro por producto. Sin lotes = listas vacías (shape estable).
pub(crate) fn query_expiring(conn: &Connection, args: &Value) -> Result<Value, String> {
    let dias = args.get("dias").and_then(|v| v.as_i64()).unwrap_or(30).clamp(1, 365);
    let producto = str_arg(args, "product_id", "");

    let mut stmt = conn
        .prepare(
            "SELECT producto_nombre, lote, caducidad, cantidad,
                    CAST(julianday(caducidad) - julianday('now', 'localtime') AS INTEGER)
             FROM lotes
             WHERE caducidad IS NOT NULL
               AND (?1 = '' OR producto_nombre LIKE '%' || ?1 || '%' ESCAPE '\\')
             ORDER BY caducidad ASC
             LIMIT 50",
        )
        .map_err(|e| e.to_string())?;
    let filas = stmt
        .query_map(rusqlite::params![escape_like(producto.trim())], |r| {
            Ok(serde_json::json!({
                "producto": r.get::<_, String>(0)?,
                "lote": r.get::<_, String>(1)?,
                "caducidad": r.get::<_, String>(2)?,
                "cantidad": r.get::<_, f64>(3)?,
                "dias_restantes": r.get::<_, i64>(4)?,
            }))
        })
        .map_err(|e| e.to_string())?;

    let mut caducados = Vec::new();
    let mut por_caducar = Vec::new();
    for fila in filas.filter_map(|f| f.ok()) {
        let dias_rest = fila.get("dias_restantes").and_then(|v| v.as_i64()).unwrap_or(i64::MAX);
        if dias_rest < 0 {
            caducados.push(fila);
        } else if dias_rest <= dias {
            por_caducar.push(fila);
        }
    }
    Ok(serde_json::json!({
        "dias": dias,
        "caducados": caducados,
        "por_caducar": por_caducar,
    }))
}

/// Catálogo de sucursales con conteo de líneas y stock conjunto.
/// El stock GLOBAL vive en `productos.stock` (tienda original).
pub(crate) fn list_branches(conn: &Connection, _args: &Value) -> Result<Value, String> {
    let mut stmt = conn
        .prepare(
            "SELECT s.id, s.nombre, COUNT(st.producto_id), COALESCE(SUM(st.stock), 0)
             FROM sucursales s
             LEFT JOIN stock_sucursal st ON st.sucursal_id = s.id
             GROUP BY s.id
             ORDER BY s.nombre ASC",
        )
        .map_err(|e| e.to_string())?;
    let sucursales: Vec<Value> = stmt
        .query_map([], |r| {
            Ok(serde_json::json!({
                "id": r.get::<_, i64>(0)?,
                "nombre": r.get::<_, String>(1)?,
                "lineas": r.get::<_, i64>(2)?,
                "stock_total": r.get::<_, f64>(3)?,
            }))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|f| f.ok())
        .collect();

    Ok(serde_json::json!({ "sucursales": sucursales }))
}

/// Stock de UNA sucursal (por nombre parcial o id). `product_id` opcional
/// filtra a un producto; `limit` 1..50 (default 20).
pub(crate) fn query_branch_stock(conn: &Connection, args: &Value) -> Result<Value, String> {
    let sucursal = str_arg(args, "sucursal", "");
    if sucursal.trim().is_empty() {
        return Ok(serde_json::json!({
            "error": "falta 'sucursal': nombre o id (usa list_branches para verlas)"
        }));
    }
    let producto = str_arg(args, "product_id", "");
    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(20).clamp(1, 50);

    let nombre_suc: Option<String> = conn
        .query_row(
            "SELECT nombre FROM sucursales
             WHERE nombre LIKE '%' || ?1 || '%' ESCAPE '\\' OR CAST(id AS TEXT) = ?1
             LIMIT 1",
            rusqlite::params![escape_like(sucursal.trim())],
            |r| r.get(0),
        )
        .map(Some)
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            otro => Err(otro.to_string()),
        })?;
    let Some(nombre_suc) = nombre_suc else {
        return Ok(serde_json::json!({ "error": format!("sucursal no encontrada: {sucursal}") }));
    };

    let mut stmt = conn
        .prepare(
            "SELECT p.nombre, st.stock
             FROM stock_sucursal st
             JOIN productos p ON p.id = st.producto_id
             JOIN sucursales s ON s.id = st.sucursal_id
             WHERE s.nombre = ?1
               AND (?2 = '' OR p.nombre LIKE '%' || ?2 || '%' ESCAPE '\\')
             ORDER BY st.stock ASC
             LIMIT ?3",
        )
        .map_err(|e| e.to_string())?;
    let productos: Vec<Value> = stmt
        .query_map(
            rusqlite::params![nombre_suc, escape_like(producto.trim()), limit],
            |r| {
                Ok(serde_json::json!({
                    "nombre": r.get::<_, String>(0)?,
                    "stock": r.get::<_, f64>(1)?,
                }))
            },
        )
        .map_err(|e| e.to_string())?
        .filter_map(|f| f.ok())
        .collect();

    Ok(serde_json::json!({
        "sucursal": nombre_suc,
        "total_lineas": productos.len(),
        "productos": productos,
    }))
}
