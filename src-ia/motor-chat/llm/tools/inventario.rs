//! inventario — Tools que leen la tabla `productos` (shapes idénticos al
//! dataset), incluidas las de navegación de inventario (solo lectura,
//! compartidas por todos los proveedores cloud): buscar, explorar categorías
//! y hojear el catálogo para convertir al asistente en un apoyo real de
//! mostrador.

use rusqlite::Connection;
use serde_json::Value;

use super::helpers::{centavos_a_pesos, escape_like, str_arg};

pub(crate) fn query_inventory(conn: &Connection, args: &Value) -> Result<Value, String> {
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

pub(crate) fn get_product_info(conn: &Connection, args: &Value) -> Result<Value, String> {
    let producto = str_arg(args, "product_id", "");
    conn.query_row(
        "SELECT nombre, precio_venta, stock, categoria FROM productos WHERE nombre LIKE ?1 LIMIT 1",
        rusqlite::params![format!("%{producto}%")],
        |r| {
            // precio_venta es INTEGER en centavos desde la migración.
            let precio_centavos: f64 = r.get(1)?;
            Ok(serde_json::json!({
                "producto": r.get::<_, String>(0)?,
                "precio_venta": centavos_a_pesos(precio_centavos),
                "stock": r.get::<_, f64>(2)?,
                "categoria": r.get::<_, Option<String>>(3)?,
            }))
        },
    )
    .map_err(|e| format!("producto no encontrado: {e}"))
}

/// Búsqueda parcial por nombre: devuelve TODAS las coincidencias con precio,
/// stock y categoría. Es el "buscador" del asistente (a diferencia de
/// get_product_info, que devuelve un solo producto).
pub(crate) fn search_products(conn: &Connection, args: &Value) -> Result<Value, String> {
    let query = str_arg(args, "query", "");
    if query.trim().is_empty() {
        return Ok(serde_json::json!({ "error": "falta 'query': texto a buscar en el nombre del producto" }));
    }
    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(15).clamp(1, 50);

    let mut stmt = conn
        .prepare(
            "SELECT nombre, precio_venta, stock, categoria FROM productos
             WHERE nombre LIKE '%' || ?1 || '%' ESCAPE '\\'
             ORDER BY vendido DESC LIMIT ?2",
        )
        .map_err(|e| e.to_string())?;
    let filas = stmt
        .query_map(rusqlite::params![escape_like(query.trim()), limit], |r| {
            let precio_centavos: f64 = r.get(1)?;
            Ok(serde_json::json!({
                "nombre": r.get::<_, String>(0)?,
                "precio_venta": centavos_a_pesos(precio_centavos),
                "stock": r.get::<_, f64>(2)?,
                "categoria": r.get::<_, Option<String>>(3)?,
            }))
        })
        .map_err(|e| e.to_string())?;

    let productos: Vec<Value> = filas.filter_map(|f| f.ok()).collect();
    let encontrados = productos.len();
    Ok(serde_json::json!({
        "consulta": query.trim(),
        "total_encontrados": encontrados,
        "productos": productos,
    }))
}

/// Lista las categorías existentes con cuántos productos tiene cada una y su
/// stock conjunto. Es el punto de partida para "navegar" el inventario.
pub(crate) fn list_categories(conn: &Connection, _args: &Value) -> Result<Value, String> {
    let mut stmt = conn
        .prepare(
            "SELECT COALESCE(categoria, 'Sin categoría'), COUNT(*), SUM(stock)
             FROM productos GROUP BY COALESCE(categoria, 'Sin categoría') ORDER BY 2 DESC",
        )
        .map_err(|e| e.to_string())?;
    let filas = stmt
        .query_map([], |r| {
            Ok(serde_json::json!({
                "categoria": r.get::<_, String>(0)?,
                "productos": r.get::<_, i64>(1)?,
                "stock_total": r.get::<_, f64>(2)?,
            }))
        })
        .map_err(|e| e.to_string())?;

    let categorias: Vec<Value> = filas.filter_map(|f| f.ok()).collect();
    Ok(serde_json::json!({ "categorias": categorias }))
}

/// Hojea los productos de UNA categoría (o todo el catálogo si se omite),
/// ordenados por más vendidos. Para recorrer el inventario por bloques.
pub(crate) fn get_products_by_category(conn: &Connection, args: &Value) -> Result<Value, String> {
    let categoria = str_arg(args, "category", "").trim().to_string();
    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(20).clamp(1, 50);

    // Filtro opcional en un solo SQL (?2 vacío = todas las categorías):
    // evita dos ramas con closures de tipos distintos.
    let sql = "SELECT nombre, precio_venta, stock, COALESCE(categoria,'Sin categoría')
               FROM productos
               WHERE (?2 = '' OR LOWER(COALESCE(categoria,'Sin categoría')) = LOWER(?2))
               ORDER BY vendido DESC LIMIT ?1";

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let filas = stmt
        .query_map(rusqlite::params![limit, categoria], |r| {
            let precio_centavos: f64 = r.get(1)?;
            Ok(serde_json::json!({
                "nombre": r.get::<_, String>(0)?,
                "precio_venta": centavos_a_pesos(precio_centavos),
                "stock": r.get::<_, f64>(2)?,
                "categoria": r.get::<_, String>(3)?,
            }))
        })
        .map_err(|e| e.to_string())?;

    let productos: Vec<Value> = filas.filter_map(|f| f.ok()).collect();
    Ok(serde_json::json!({
        "categoria": if categoria.is_empty() { "todas" } else { categoria.as_str() },
        "total_mostrados": productos.len(),
        "productos": productos,
    }))
}

pub(crate) fn get_restock_analysis(conn: &Connection, args: &Value) -> Result<Value, String> {
    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(10).clamp(1, 30);
    // Productos con venta reciente ordenados por urgencia (stock más bajo).
    // LIMIT parametrizado: aunque el valor viene clampeado a i64, el
    // estándar del módulo es cero interpolación en SQL.
    let sql = "SELECT nombre, stock, stock_minimo, vendido
         FROM productos
         WHERE stock <= stock_minimo
         ORDER BY stock ASC LIMIT ?1";
    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let filas = stmt
        .query_map(rusqlite::params![limit], |r| {
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
