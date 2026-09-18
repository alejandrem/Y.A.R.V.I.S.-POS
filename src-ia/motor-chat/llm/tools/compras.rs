//! compras — Tools de abasto: proveedores, recepciones (`compras` +
//! `compras_items`, migraciones 0012/0013) y órdenes de compra (pedidos
//! con estado, migración 0018). Todo SOLO lectura, como el resto del
//! módulo: dinero en centavos en disco, pesos hacia el modelo.

use rusqlite::Connection;
use serde_json::Value;

use super::helpers::{centavos_a_pesos, escape_like, rango_de, str_arg, MONEDA};

/// Lista proveedores con su actividad compradora (conteo + total pagado).
/// `search` filtra por nombre parcial; `limit` 1..50 (default 20).
pub(crate) fn query_suppliers(conn: &Connection, args: &Value) -> Result<Value, String> {
    let search = str_arg(args, "search", "");
    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(20).clamp(1, 50);

    let mut stmt = conn
        .prepare(
            "SELECT p.id, p.nombre, COALESCE(p.telefono, ''),
                    COUNT(c.id), COALESCE(SUM(c.monto_pagado), 0)
             FROM proveedores p
             LEFT JOIN compras c ON c.proveedor_id = p.id
             WHERE (?1 = '' OR p.nombre LIKE '%' || ?1 || '%' ESCAPE '\\')
             GROUP BY p.id
             ORDER BY 5 DESC
             LIMIT ?2",
        )
        .map_err(|e| e.to_string())?;
    let filas = stmt
        .query_map(rusqlite::params![escape_like(search.trim()), limit], |r| {
            let total_centavos: f64 = r.get(4)?;
            Ok(serde_json::json!({
                "id": r.get::<_, i64>(0)?,
                "nombre": r.get::<_, String>(1)?,
                "telefono": r.get::<_, String>(2)?,
                "compras": r.get::<_, i64>(3)?,
                "total_comprado": centavos_a_pesos(total_centavos),
            }))
        })
        .map_err(|e| e.to_string())?;

    let proveedores: Vec<Value> = filas.filter_map(|f| f.ok()).collect();
    Ok(serde_json::json!({
        "total_proveedores": proveedores.len(),
        "moneda": MONEDA,
        "proveedores": proveedores,
    }))
}

/// Recepciones de mercancía por rango (`date_range`, default this_month)
/// y/o proveedor parcial. Totales en pesos.
pub(crate) fn query_purchases(conn: &Connection, args: &Value) -> Result<Value, String> {
    let rango = rango_de(&str_arg(args, "date_range", "this_month"));
    let proveedor = str_arg(args, "proveedor", "");
    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(20).clamp(1, 50);

    let mut stmt = conn
        .prepare(
            "SELECT c.id, date(c.fecha), p.nombre,
                    c.monto_pagado, c.monto_sugerido, c.metodo_pago
             FROM compras c
             JOIN proveedores p ON p.id = c.proveedor_id
             WHERE date(c.fecha) BETWEEN ?1 AND ?2
               AND (?3 = '' OR p.nombre LIKE '%' || ?3 || '%' ESCAPE '\\')
             ORDER BY c.fecha DESC
             LIMIT ?4",
        )
        .map_err(|e| e.to_string())?;
    let filas = stmt
        .query_map(
            rusqlite::params![rango.desde, rango.hasta, escape_like(proveedor.trim()), limit],
            |r| {
                let pagado: f64 = r.get(3)?;
                let sugerido: f64 = r.get(4)?;
                Ok(serde_json::json!({
                    "id": r.get::<_, i64>(0)?,
                    "fecha": r.get::<_, String>(1)?,
                    "proveedor": r.get::<_, String>(2)?,
                    "monto_pagado": centavos_a_pesos(pagado),
                    "monto_sugerido": centavos_a_pesos(sugerido),
                    "metodo_pago": r.get::<_, Option<String>>(5)?,
                }))
            },
        )
        .map_err(|e| e.to_string())?;

    let compras: Vec<Value> = filas.filter_map(|f| f.ok()).collect();
    let total_pagado: f64 = compras
        .iter()
        .filter_map(|c| c.get("monto_pagado").and_then(|v| v.as_f64()))
        .sum();
    Ok(serde_json::json!({
        "rango": rango.etiqueta,
        "total_pagado": (total_pagado * 100.0).round() / 100.0,
        "moneda": MONEDA,
        "compras": compras,
    }))
}

/// Detalle de UNA recepción: cabecera + renglones con costo sugerido.
/// `compra_id` obligatorio; si no existe, error legible (no excepción).
pub(crate) fn get_purchase_detail(conn: &Connection, args: &Value) -> Result<Value, String> {
    let Some(id) = args.get("compra_id").and_then(|v| v.as_i64()) else {
        return Ok(serde_json::json!({ "error": "falta 'compra_id': id de la compra a detallar" }));
    };

    let cabecera: Option<(i64, String, String, f64, String)> = conn
        .query_row(
            "SELECT c.id, date(c.fecha), p.nombre, c.monto_pagado, c.metodo_pago
             FROM compras c
             JOIN proveedores p ON p.id = c.proveedor_id
             WHERE c.id = ?1",
            rusqlite::params![id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .map(Some)
        .or_else(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            otro => Err(otro.to_string()),
        })?;
    let Some((cid, fecha, proveedor, pagado_centavos, metodo)) = cabecera else {
        return Ok(serde_json::json!({ "error": format!("compra no encontrada: {id}") }));
    };

    let mut stmt = conn
        .prepare(
            "SELECT nombre, cantidad, presentacion, precio_sugerido
             FROM compras_items
             WHERE compra_id = ?1
             ORDER BY id ASC",
        )
        .map_err(|e| e.to_string())?;
    let items: Vec<Value> = stmt
        .query_map(rusqlite::params![id], |r| {
            let sugerido: f64 = r.get(3)?;
            Ok(serde_json::json!({
                "nombre": r.get::<_, String>(0)?,
                "cantidad": r.get::<_, f64>(1)?,
                "presentacion": r.get::<_, Option<String>>(2)?,
                "precio_sugerido": centavos_a_pesos(sugerido),
            }))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|f| f.ok())
        .collect();

    Ok(serde_json::json!({
        "id": cid,
        "fecha": fecha,
        "proveedor": proveedor,
        "monto_pagado": centavos_a_pesos(pagado_centavos),
        "metodo_pago": metodo,
        "moneda": MONEDA,
        "items": items,
    }))
}

/// Pedidos a proveedor por estado (`pendiente`/`parcial`/`recibida`/
/// `cancelada`/`todas`, default `pendiente`) con sus renglones.
pub(crate) fn query_purchase_orders(conn: &Connection, args: &Value) -> Result<Value, String> {
    let estado = str_arg(args, "estado", "pendiente");
    if !["pendiente", "parcial", "recibida", "cancelada", "todas"].contains(&estado.as_str()) {
        return Ok(serde_json::json!({
            "error": "estado inválido: usa pendiente, parcial, recibida, cancelada o todas"
        }));
    }
    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(20).clamp(1, 50);

    let mut stmt = conn
        .prepare(
            "SELECT o.id, p.nombre, o.estado, date(o.fecha), o.total_estimado
             FROM ordenes_compra o
             JOIN proveedores p ON p.id = o.proveedor_id
             WHERE (?1 = 'todas' OR o.estado = ?1)
             ORDER BY o.fecha DESC
             LIMIT ?2",
        )
        .map_err(|e| e.to_string())?;
    let cabeceras: Vec<(i64, String, String, String, f64)> = stmt
        .query_map(rusqlite::params![estado, limit], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|f| f.ok())
        .collect();

    let mut ordenes = Vec::with_capacity(cabeceras.len());
    for (oid, proveedor, est, fecha, total_centavos) in cabeceras {
        let mut items_stmt = conn
            .prepare(
                "SELECT nombre, cantidad, cantidad_recibida, costo_unitario
                 FROM ordenes_items
                 WHERE orden_id = ?1
                 ORDER BY id ASC",
            )
            .map_err(|e| e.to_string())?;
        let items: Vec<Value> = items_stmt
            .query_map(rusqlite::params![oid], |r| {
                let costo: f64 = r.get(3)?;
                Ok(serde_json::json!({
                    "nombre": r.get::<_, String>(0)?,
                    "cantidad": r.get::<_, f64>(1)?,
                    "cantidad_recibida": r.get::<_, f64>(2)?,
                    "costo_unitario": centavos_a_pesos(costo),
                }))
            })
            .map_err(|e| e.to_string())?
            .filter_map(|f| f.ok())
            .collect();
        ordenes.push(serde_json::json!({
            "id": oid,
            "proveedor": proveedor,
            "estado": est,
            "fecha": fecha,
            "total_estimado": centavos_a_pesos(total_centavos),
            "items": items,
        }));
    }

    Ok(serde_json::json!({
        "estado": estado,
        "total_ordenes": ordenes.len(),
        "moneda": MONEDA,
        "ordenes": ordenes,
    }))
}
