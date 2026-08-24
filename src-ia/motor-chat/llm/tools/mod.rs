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

use rusqlite::Connection;
use serde_json::Value;

/// Máximo de rondas tool_call→resultado que el backend permite por pregunta.
pub const MAX_RONDAS_TOOLS: usize = 3;

// ─────────────────────────────────────────────────────────────────────────────
// Detección
// ─────────────────────────────────────────────────────────────────────────────

/// Extrae la PRIMERA llamada `<tool_call>{json}</tool_call>` de una respuesta.
/// Devuelve `(nombre, arguments_serializado)` o None si no hay llamada válida.
pub fn detectar_tool_call(respuesta: &str) -> Option<(String, String)> {
    // Camino principal: bloques <tool_call>{json}</tool_call> (formato entrenado).
    if let Some(ini) = respuesta.find("<tool_call>") {
        let resto = &respuesta[ini + "<tool_call>".len()..];
        if let Some(fin) = resto.find("</tool_call>") {
            if let Ok(v) = serde_json::from_str::<Value>(resto[..fin].trim()) {
                if let Some(nombre) = v.get("name").and_then(|n| n.as_str()) {
                    let args = v.get("arguments").cloned().unwrap_or(Value::Object(Default::default()));
                    return Some((nombre.to_string(), args.to_string()));
                }
            }
        }
    }
    // FALLBACK: el modelo a veces escupe el JSON DESNUDO sin etiquetas
    // (visto en pruebas reales): {"name": "...", "arguments": {...}}
    if let Some(i) = respuesta.find(r#"{"name":"#).or_else(|| respuesta.find(r#"{"name": "#)) {
        if let Some((objeto, _fin)) = extraer_objeto_balanceado(&respuesta[i..]) {
            if let Ok(v) = serde_json::from_str::<Value>(objeto) {
                if let Some(nombre) = v.get("name").and_then(|n| n.as_str()) {
                    if !nombre.is_empty() {
                        println!("[YARVIS-TOOLS] JSON sin etiquetas detectado (fallback)");
                        let args = v.get("arguments").cloned().unwrap_or(Value::Object(Default::default()));
                        return Some((nombre.to_string(), args.to_string()));
                    }
                }
            }
        }
    }
    None
}

/// Extrae el primer objeto {...} BALANCEADO (respeta strings) desde el inicio.
fn extraer_objeto_balanceado(texto: &str) -> Option<(&str, usize)> {
    let bytes = texto.as_bytes();
    if bytes.first() != Some(&b'{') {
        return None;
    }
    let mut profundidad = 0i32;
    let mut en_string = false;
    let mut escape = false;
    for (i, &b) in bytes.iter().enumerate() {
        if escape {
            escape = false;
            continue;
        }
        match b {
            b'\\' if en_string => escape = true,
            b'"' => en_string = !en_string,
            b'{' if !en_string => profundidad += 1,
            b'}' if !en_string => {
                profundidad -= 1;
                if profundidad == 0 {
                    return Some((&texto[..=i], i + 1));
                }
            }
            _ => {}
        }
    }
    None
}

/// Limpieza final + red de seguridad: si tras el ciclo la respuesta queda
/// vacía (el modelo a veces devuelve cadena nula tras un resultado de tool),
/// se entrega un mensaje digno en vez de una burbuja fantasma.
pub fn respuesta_final_segura(texto: &str) -> String {
    let limpio = quitar_tool_calls(texto);
    if limpio.trim().is_empty() {
        "Consulté el sistema pero no obtuve datos para mostrar. Prueba con otra pregunta o revisa que haya información registrada.".to_string()
    } else {
        limpio
    }
}

/// Quita TODOS los bloques <tool_call> de un texto (para mostrar limpio).
pub fn quitar_tool_calls(respuesta: &str) -> String {
    let mut out = String::new();
    let mut restante = respuesta;
    while let Some(i) = restante.find("<tool_call>") {
        out.push_str(&restante[..i]);
        match restante[i..].find("</tool_call>") {
            Some(j) => restante = &restante[i + j + "</tool_call>".len()..],
            None => return out, // bloque sin cerrar: descartar el resto
        }
    }
    out.push_str(restante);
    // Limpieza final: JSONs desnudos de tool_call sin etiquetas.
    let limpio = re_json_desnudo(&out);
    limpio.trim().to_string()
}

/// Elimina objetos {"name": "...", ...} sueltos (fallback de detección).
fn re_json_desnudo(texto: &str) -> String {
    let mut out = String::new();
    let mut restante = texto;
    while let Some(i) = restante.find(r#"{"name":"#).or_else(|| restante.find(r#"{"name": "#)) {
        out.push_str(&restante[..i]);
        let despues = &restante[i..];
        match extraer_objeto_balanceado(despues) {
            Some((objeto, fin_bytes)) => {
                let es_tool = serde_json::from_str::<Value>(objeto)
                    .ok()
                    .and_then(|v| v.get("name").cloned())
                    .is_some();
                if !es_tool {
                    out.push_str(objeto); // no era un tool_call: conservar
                }
                restante = &despues[fin_bytes..];
            }
            None => {
                // JSON incompleto al final: conservarlo tal cual y terminar
                out.push_str(despues);
                restante = "";
            }
        }
    }
    out.push_str(restante);
    out
}

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
        otro => Ok(serde_json::json!({ "error": format!("herramienta desconocida: {otro}") })),
    };
    resultado.map(|v| v.to_string())
}

// ─────────────────────────────────────────────────────────────────────────────
// Rangos de fechas (chrono) → cláusulas SQL
// ─────────────────────────────────────────────────────────────────────────────

struct Rango {
    desde: String,
    hasta: String,
    etiqueta: String,
}

fn rango_de(valor: &str) -> Rango {
    use chrono::{Datelike, Duration, Local};
    let hoy = Local::now().date_naive();
    let fmt = |d: chrono::NaiveDate| d.format("%Y-%m-%d").to_string();

    match valor {
        "today" => Rango { desde: fmt(hoy), hasta: fmt(hoy), etiqueta: valor.into() },
        "yesterday" => {
            let ayer = hoy - Duration::days(1);
            Rango { desde: fmt(ayer), hasta: fmt(ayer), etiqueta: valor.into() }
        }
        "this_week" => {
            let lunes = hoy - Duration::days(hoy.weekday().num_days_from_monday() as i64);
            Rango { desde: fmt(lunes), hasta: fmt(hoy), etiqueta: valor.into() }
        }
        "last_week" => {
            let lunes_esta = hoy - Duration::days(hoy.weekday().num_days_from_monday() as i64);
            let lunes_pasada = lunes_esta - Duration::days(7);
            Rango { desde: fmt(lunes_pasada), hasta: fmt(lunes_esta - Duration::days(1)), etiqueta: valor.into() }
        }
        "this_month" => {
            let primero = chrono::NaiveDate::from_ymd_opt(hoy.year(), hoy.month(), 1).unwrap_or(hoy);
            Rango { desde: fmt(primero), hasta: fmt(hoy), etiqueta: valor.into() }
        }
        "last_month" => {
            let primero_mes = chrono::NaiveDate::from_ymd_opt(hoy.year(), hoy.month(), 1).unwrap_or(hoy);
            let fin_mes_pasado = primero_mes - Duration::days(1);
            let inicio_mes_pasado =
                chrono::NaiveDate::from_ymd_opt(fin_mes_pasado.year(), fin_mes_pasado.month(), 1)
                    .unwrap_or(fin_mes_pasado);
            Rango { desde: fmt(inicio_mes_pasado), hasta: fmt(fin_mes_pasado), etiqueta: valor.into() }
        }
        otro => Rango { desde: fmt(hoy), hasta: fmt(hoy), etiqueta: otro.into() },
    }
}

fn str_arg<'a>(args: &'a Value, clave: &str, default: &'a str) -> String {
    args.get(clave).and_then(|v| v.as_str()).unwrap_or(default).to_string()
}

const MONEDA: &str = "MXN";

// ─────────────────────────────────────────────────────────────────────────────
// Las 7 herramientas (shapes idénticos al dataset)
// ─────────────────────────────────────────────────────────────────────────────

fn query_sales(conn: &Connection, args: &Value) -> Result<Value, String> {
    let rango = rango_de(&str_arg(args, "date_range", "today"));
    let metric = str_arg(args, "metric", "revenue");
    let producto = args.get("product_id").and_then(|v| v.as_str());

    let join_prod = producto.map_or(String::new(), |p| {
        format!(
            " JOIN detalle_ventas d ON d.venta_id = v.id AND d.producto_nombre LIKE '%{p}%' "
        )
    });
    let sql = if metric == "units" {
        // Unidades siempre vienen del detalle de venta.
        let filtro_prod = producto
            .map(|p| format!(" AND d.producto_nombre LIKE '%{p}%'"))
            .unwrap_or_default();
        format!(
            "SELECT COALESCE(SUM(d.cantidad), 0) FROM ventas v JOIN detalle_ventas d ON d.venta_id = v.id WHERE v.estado = 'completada' AND date(v.fecha) BETWEEN ?1 AND ?2{filtro_prod}"
        )
    } else {
        format!(
            "SELECT COALESCE(SUM(v.total), 0) FROM ventas v {join_prod} WHERE v.estado = 'completada' AND date(v.fecha) BETWEEN ?1 AND ?2"
        )
    };

    let total: f64 = conn
        .query_row(&sql, rusqlite::params![rango.desde, rango.hasta], |r| r.get(0))
        .map_err(|e| e.to_string())?;

    let mut out = serde_json::Map::new();
    if metric == "units" {
        out.insert("unidades_totales".into(), serde_json::json!(total));
    } else {
        out.insert("ventas_totales".into(), serde_json::json!(round2(total)));
        out.insert("moneda".into(), serde_json::json!(MONEDA));
    }
    if let Some(p) = producto {
        out.insert("producto".into(), serde_json::json!(p));
    }
    Ok(Value::Object(out))
}

fn compare_periods(conn: &Connection, args: &Value) -> Result<Value, String> {
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

    let (a, b) = (suma(&pa), suma(&pb));
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

fn get_top_products(conn: &Connection, args: &Value) -> Result<Value, String> {
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
            Ok(serde_json::json!({
                "producto": r.get::<_, String>(0)?,
                "ventas_totales": round2(r.get::<_, f64>(1)?),
                "unidades": r.get::<_, f64>(2)?,
            }))
        })
        .map_err(|e| e.to_string())?;

    let productos: Vec<Value> = filas.filter_map(|f| f.ok()).collect();
    Ok(serde_json::json!({ "productos": productos, "orden": order, "rango": rango.etiqueta }))
}

fn query_inventory(conn: &Connection, args: &Value) -> Result<Value, String> {
    let filter = str_arg(args, "filter", "all");
    let producto = args.get("product_id").and_then(|v| v.as_str());

    if let Some(p) = producto {
        // Un producto concreto: shape singular del dataset.
        let fila = conn
            .query_row(
                "SELECT nombre, stock, stock_minimo, precio_venta, categoria FROM productos WHERE nombre LIKE ?1 LIMIT 1",
                rusqlite::params![format!("%{p}%")],
                |r| {
                    Ok(serde_json::json!({
                        "producto": r.get::<_, String>(0)?,
                        "stock": r.get::<_, f64>(1)?,
                        "stock_minimo": r.get::<_, f64>(2)?,
                    }))
                },
            )
            .map_err(|e| e.to_string())?;
        return Ok(fila);
    }

    let condicion = match filter.as_str() {
        "low_stock" | "low" => "WHERE stock <= stock_minimo",
        "out_of_stock" => "WHERE stock <= 0",
        _ => "",
    };
    let sql = format!(
        "SELECT nombre, stock, stock_minimo FROM productos {condicion} ORDER BY stock ASC LIMIT 30"
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let filas = stmt
        .query_map([], |r| {
            Ok(serde_json::json!({
                "producto": r.get::<_, String>(0)?,
                "stock": r.get::<_, f64>(1)?,
                "stock_minimo": r.get::<_, f64>(2)?,
            }))
        })
        .map_err(|e| e.to_string())?;
    let productos: Vec<Value> = filas.filter_map(|f| f.ok()).collect();
    Ok(serde_json::json!({ "filtro": filter, "productos": productos }))
}

fn forecast_sales(conn: &Connection, args: &Value) -> Result<Value, String> {
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

fn get_product_info(conn: &Connection, args: &Value) -> Result<Value, String> {
    let producto = str_arg(args, "product_id", "");
    conn.query_row(
        "SELECT nombre, precio_venta, stock, categoria FROM productos WHERE nombre LIKE ?1 LIMIT 1",
        rusqlite::params![format!("%{producto}%")],
        |r| {
            Ok(serde_json::json!({
                "producto": r.get::<_, String>(0)?,
                "precio_venta": round2(r.get::<_, f64>(1)?),
                "stock": r.get::<_, f64>(2)?,
                "categoria": r.get::<_, Option<String>>(3)?,
            }))
        },
    )
    .map_err(|e| format!("producto no encontrado: {e}"))
}

fn get_restock_analysis(conn: &Connection, args: &Value) -> Result<Value, String> {
    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(10).clamp(1, 30);
    // Productos con venta reciente ordenados por urgencia (stock más bajo).
    let sql = format!(
        "SELECT nombre, stock, stock_minimo, vendido
         FROM productos
         WHERE stock <= stock_minimo
         ORDER BY stock ASC LIMIT {limit}"
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let filas = stmt
        .query_map([], |r| {
            let stock: f64 = r.get(1)?;
            let minimo: f64 = r.get(2)?;
            Ok(serde_json::json!({
                "producto": r.get::<_, String>(0)?,
                "stock_actual": stock,
                "cantidad_sugerida": ((minimo * 2.0) - stock).max(1.0) as i64,
            }))
        })
        .map_err(|e| e.to_string())?;
    let recomendaciones: Vec<Value> = filas.filter_map(|f| f.ok()).collect();
    Ok(serde_json::json!({ "recomendaciones": recomendaciones }))
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests: DB en memoria con esquema mínimo + verificación de detección/shape
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn db_prueba() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE ventas (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                fecha DATETIME DEFAULT CURRENT_TIMESTAMP,
                total REAL, subtotal REAL, metodo_pago TEXT,
                cajero TEXT, estado TEXT DEFAULT 'completada'
            );
            CREATE TABLE detalle_ventas (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                venta_id INTEGER, producto_id INTEGER,
                producto_nombre TEXT, cantidad REAL,
                precio_unitario REAL, descuento REAL, subtotal REAL
            );
            CREATE TABLE productos (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                nombre TEXT, precio_venta REAL DEFAULT 0,
                stock REAL DEFAULT 0, stock_minimo REAL DEFAULT 0,
                vendido REAL DEFAULT 0, categoria TEXT
            );
            INSERT INTO ventas (fecha, total, subtotal, estado) VALUES
              (datetime('now','localtime','+0 hours'), 100.0, 100.0, 'completada'),
              (datetime('now','localtime','-1 hours'), 200.0, 200.0, 'completada'),
              (datetime('now','localtime','-2 hours'), 50.0, 50.0, 'cancelada');
            INSERT INTO detalle_ventas (venta_id, producto_nombre, cantidad, precio_unitario, subtotal) VALUES
              (1, 'Coca-Cola', 4.0, 25.0, 100.0),
              (2, 'Sabritas', 8.0, 25.0, 200.0);
            INSERT INTO productos (nombre, precio_venta, stock, stock_minimo, vendido, categoria) VALUES
              ('Coca-Cola', 25.0, 2.0, 5.0, 40.0, 'Bebidas'),
              ('Pan Bimbo', 42.0, 12.0, 4.0, 15.0, 'Panadería');
            "#,
        )
        .unwrap();
        conn
    }

    #[test]
    fn detecta_tool_call_y_argumentos() {
        let raw = "Pensando...\n<tool_call>\n{\"name\": \"query_sales\", \"arguments\": {\"date_range\": \"today\", \"metric\": \"revenue\"}}\n</tool_call>";
        let (nombre, args) = detectar_tool_call(raw).unwrap();
        assert_eq!(nombre, "query_sales");
        assert!(args.contains("today"));
        assert_eq!(quitar_tool_calls(raw), "Pensando...");
    }

    #[test]
    fn texto_sin_tool_call_devuelve_none() {
        assert!(detectar_tool_call("Llevas $500 vendidos hoy.").is_none());
    }

    #[test]
    fn json_desnudo_sin_etiquetas_tambien_se_detecta() {
        let raw = r#"Claro, consulta: {"name": "get_top_products", "arguments": {"date_range": "this_week"}} y listo"#;
        let (nombre, args) = detectar_tool_call(raw).unwrap();
        assert_eq!(nombre, "get_top_products");
        assert!(args.contains("this_week"));
    }

    #[test]
    fn limpieza_quita_json_desnudo_del_texto_final() {
        let sucio = r#"Estos son: {"name": "get_top_products", "arguments": {}}"#;
        assert_eq!(quitar_tool_calls(sucio), "Estos son:");
    }

    #[test]
    fn query_sales_revenue_shape_del_dataset() {
        let conn = db_prueba();
        let v = query_sales(&conn, &serde_json::json!({"date_range": "today", "metric": "revenue"})).unwrap();
        assert_eq!(v["moneda"], "MXN");
        assert_eq!(v["ventas_totales"], 300.0); // 100 + 200 completadas de hoy
    }

    #[test]
    fn top_products_orden_y_limite() {
        let conn = db_prueba();
        let v = get_top_products(&conn, &serde_json::json!({"date_range": "this_week", "order": "top", "limit": 5})).unwrap();
        let lista = v["productos"].as_array().unwrap();
        assert_eq!(lista.len(), 2);
        assert_eq!(lista[0]["producto"], "Sabritas"); // $200 > $100
    }

    #[test]
    fn inventario_sin_stock_filtra() {
        let conn = db_prueba();
        let v = query_inventory(&conn, &serde_json::json!({"filter": "low_stock"})).unwrap();
        let lista = v["productos"].as_array().unwrap();
        assert_eq!(lista.len(), 1);
        assert_eq!(lista[0]["producto"], "Coca-Cola");
    }

    #[test]
    fn herramienta_desconocida_da_error_legible() {
        let conn = db_prueba();
        let r = ejecutar_tool("hackear_nasa", "{}", ":memory:");
        assert!(r.is_ok()); // nunca rompe el chat
        assert!(r.unwrap().contains("desconocida"));
    }
}
