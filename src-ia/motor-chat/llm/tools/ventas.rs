//! ventas — Tools que leen ventas y detalle_ventas (shapes idénticos al
//! dataset del fine-tuning).

use rusqlite::Connection;
use serde_json::Value;

use super::helpers::{centavos_a_pesos, escape_like, rango_de, round2, str_arg, MONEDA};

// ─────────────────────────────────────────────────────────────────────────────
// Las herramientas de ventas (shapes idénticos al dataset)
// ─────────────────────────────────────────────────────────────────────────────

pub(crate) fn query_sales(conn: &Connection, args: &Value) -> Result<Value, String> {
    let rango = rango_de(&str_arg(args, "date_range", "today"));
    let metric = str_arg(args, "metric", "revenue");
    let producto = args.get("product_id").and_then(|v| v.as_str());

    // SEGURIDAD: el product_id viene de los `arguments` que el LLM escribe
    // en <tool_call> (input semiconfiado). TODO va parametrizado con
    // placeholders ?1..?3 — jamás interpolar strings en el SQL. Los
    // comodines LIKE del input se escapan para que se traten literalmente.
    let filtro_prod =
        producto.map(|_| " AND d.producto_nombre LIKE '%' || ?3 || '%' ESCAPE '\\'").unwrap_or("");
    let sql = if metric == "units" {
        format!(
            "SELECT COALESCE(SUM(d.cantidad), 0) FROM ventas v JOIN detalle_ventas d ON d.venta_id = v.id WHERE v.estado = 'completada' AND date(v.fecha) BETWEEN ?1 AND ?2{filtro_prod}"
        )
    } else {
        let join_prod = producto
            .map(|_| " JOIN detalle_ventas d ON d.venta_id = v.id AND d.producto_nombre LIKE '%' || ?3 || '%' ESCAPE '\\' ")
            .unwrap_or("");
        format!(
            "SELECT COALESCE(SUM(v.total), 0) FROM ventas v{join_prod} WHERE v.estado = 'completada' AND date(v.fecha) BETWEEN ?1 AND ?2"
        )
    };

    // SUM(v.total) viene en CENTAVOS (columna INTEGER); se lee como f64 por
    // si una DB vieja mezcla REAL×INTEGER y se convierte a pesos.
    let total_centavos: f64 = if let Some(p) = producto {
        conn.query_row(&sql, rusqlite::params![rango.desde, rango.hasta, escape_like(p)], |r| {
            r.get(0)
        })
        .map_err(|e| e.to_string())?
    } else {
        conn.query_row(&sql, rusqlite::params![rango.desde, rango.hasta], |r| r.get(0))
            .map_err(|e| e.to_string())?
    };

    let mut out = serde_json::Map::new();
    if metric == "units" {
        out.insert("unidades_totales".into(), serde_json::json!(total_centavos));
    } else {
        out.insert("ventas_totales".into(), serde_json::json!(centavos_a_pesos(total_centavos)));
        out.insert("moneda".into(), serde_json::json!(MONEDA));
    }
    if let Some(p) = producto {
        out.insert("producto".into(), serde_json::json!(p));
    }
    Ok(Value::Object(out))
}

pub(crate) fn compare_periods(conn: &Connection, args: &Value) -> Result<Value, String> {
    // El dataset usa DOS variantes de nombres: period1/period2 y period_a/period_b.
    let pa = str_arg(args, "period_a", "");
    let pb = str_arg(args, "period_b", "");
    let (pa, pb) = if pa.is_empty() || pb.is_empty() {
        (str_arg(args, "period1", "this_week"), str_arg(args, "period2", "last_week"))
    } else {
        (pa, pb)
    };
    let metric = str_arg(args, "metric", "revenue");

    let suma = |valor: &str| -> f64 {
        let r = rango_de(valor);
        let sql = if metric == "units" {
            "SELECT COALESCE(SUM(d.cantidad), 0) FROM ventas v JOIN detalle_ventas d ON d.venta_id = v.id WHERE v.estado = 'completada' AND date(v.fecha) BETWEEN ?1 AND ?2"
        } else {
            "SELECT COALESCE(SUM(v.total), 0) FROM ventas v WHERE v.estado = 'completada' AND date(v.fecha) BETWEEN ?1 AND ?2"
        };
        conn.query_row(sql, rusqlite::params![r.desde, r.hasta], |row| row.get::<_, f64>(0))
            .unwrap_or(0.0)
    };

    // En metric "revenue" la suma viene en CENTAVOS: convertir a pesos antes
    // de restar y serializar (así la diferencia también queda en pesos).
    let a_pesos = |v: f64| if metric == "units" { v } else { centavos_a_pesos(v) };
    let (a, b) = (a_pesos(suma(&pa)), a_pesos(suma(&pb)));
    let diferencia = round2(a - b);

    let mut out = serde_json::Map::new();
    if metric == "units" {
        out.insert("unidades_a".into(), serde_json::json!(a));
        out.insert("unidades_b".into(), serde_json::json!(b));
    } else {
        out.insert("ventas_a".into(), serde_json::json!(round2(a)));
        out.insert("ventas_b".into(), serde_json::json!(round2(b)));
        out.insert("moneda".into(), serde_json::json!(MONEDA));
    }
    out.insert("diferencia".into(), serde_json::json!(diferencia));
    out.insert("periodo_a".into(), serde_json::json!(pa));
    out.insert("periodo_b".into(), serde_json::json!(pb));
    Ok(Value::Object(out))
}

pub(crate) fn get_top_products(conn: &Connection, args: &Value) -> Result<Value, String> {
    let rango = rango_de(&str_arg(args, "date_range", "this_week"));
    let order = str_arg(args, "order", "top");
    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(10).clamp(1, 50);
    let dir = if order == "bottom" { "ASC" } else { "DESC" };

    let sql = format!(
        "SELECT d.producto_nombre AS producto,
                COALESCE(SUM(d.subtotal), 0) AS ventas,
                COALESCE(SUM(d.cantidad), 0) AS unidades
         FROM detalle_ventas d
         JOIN ventas v ON v.id = d.venta_id
         WHERE v.estado = 'completada' AND date(v.fecha) BETWEEN ?1 AND ?2
         GROUP BY d.producto_nombre
         ORDER BY ventas {dir}
         LIMIT ?3"
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let filas = stmt
        .query_map(rusqlite::params![rango.desde, rango.hasta, limit], |r| {
            // SUM(d.subtotal) está en centavos → pesos para el LLM.
            let ventas_centavos: f64 = r.get(1)?;
            Ok(serde_json::json!({
                "producto": r.get::<_, String>(0)?,
                "ventas_totales": centavos_a_pesos(ventas_centavos),
                "unidades": r.get::<_, f64>(2)?,
            }))
        })
        .map_err(|e| e.to_string())?;

    let productos: Vec<Value> = filas.filter_map(|f| f.ok()).collect();
    Ok(serde_json::json!({ "productos": productos, "orden": order, "rango": rango.etiqueta }))
}

pub(crate) fn forecast_sales(conn: &Connection, args: &Value) -> Result<Value, String> {
    let periodo = str_arg(args, "period", "next_week");
    let producto = str_arg(args, "product_id", "");
    let dias_horizonte = if periodo == "tomorrow" { 1 } else { 7 };

    // Pronóstico simple: promedio de unidades vendidas en los últimos 7 días.
    let base: f64 = if producto.is_empty() {
        conn.query_row(
            "SELECT COALESCE(SUM(d.cantidad), 0) / 7.0
             FROM detalle_ventas d JOIN ventas v ON v.id = d.venta_id
             WHERE v.estado = 'completada' AND date(v.fecha) >= date('now','localtime','-7 day')",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0.0)
    } else {
        conn.query_row(
            "SELECT COALESCE(SUM(d.cantidad), 0) / 7.0
             FROM detalle_ventas d JOIN ventas v ON v.id = d.venta_id
             WHERE v.estado = 'completada' AND d.producto_nombre LIKE ?1
               AND date(v.fecha) >= date('now','localtime','-7 day')",
            rusqlite::params![format!("%{producto}%")],
            |r| r.get(0),
        )
        .unwrap_or(0.0)
    };

    let sugerida = (base * dias_horizonte as f64).ceil();
    Ok(serde_json::json!({
        "producto": producto,
        "cantidad_sugerida": sugerida as i64,
        "confianza": "media",
        "periodo": periodo,
    }))
}
