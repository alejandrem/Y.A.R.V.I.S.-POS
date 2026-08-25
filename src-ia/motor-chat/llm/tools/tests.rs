//! tests — DB en memoria con esquema mínimo + verificación de detección/shape.

use rusqlite::Connection;

use super::deteccion::{detectar_tool_call, quitar_tool_calls};
use super::inventario::{
    get_product_info, get_products_by_category, list_categories, query_inventory, search_products,
};
use super::ventas::{compare_periods, get_top_products, query_sales};
use super::ejecutar_tool;

fn db_prueba() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        r#"
        -- Espejo del esquema migrado: dinero en INTEGER CENTAVOS.
        CREATE TABLE ventas (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            fecha DATETIME DEFAULT CURRENT_TIMESTAMP,
            total INTEGER, subtotal INTEGER, metodo_pago TEXT,
            cajero TEXT, estado TEXT DEFAULT 'completada'
        );
        CREATE TABLE detalle_ventas (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            venta_id INTEGER, producto_id INTEGER,
            producto_nombre TEXT, cantidad REAL,
            precio_unitario INTEGER, descuento INTEGER, subtotal INTEGER
        );
        CREATE TABLE productos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nombre TEXT, precio_venta INTEGER DEFAULT 0,
            stock REAL DEFAULT 0, stock_minimo REAL DEFAULT 0,
            vendido REAL DEFAULT 0, categoria TEXT
        );
        INSERT INTO ventas (fecha, total, subtotal, estado) VALUES
          (datetime('now','localtime','+0 hours'), 10000, 10000, 'completada'),
          (datetime('now','localtime','-1 hours'), 20000, 20000, 'completada'),
          (datetime('now','localtime','-2 hours'), 5000, 5000, 'cancelada');
        INSERT INTO detalle_ventas (venta_id, producto_nombre, cantidad, precio_unitario, subtotal) VALUES
          (1, 'Coca-Cola', 4.0, 2500, 10000),
          (2, 'Sabritas', 8.0, 2500, 20000);
        INSERT INTO productos (nombre, precio_venta, stock, stock_minimo, vendido, categoria) VALUES
          ('Coca-Cola', 2500, 2.0, 5.0, 40.0, 'Bebidas'),
          ('Pan Bimbo', 4200, 12.0, 4.0, 15.0, 'Panadería');
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
    // 10000 + 20000 centavos completados de hoy → $300.00 en pesos.
    assert_eq!(v["ventas_totales"], 300.0);
    assert_eq!(v["ventas_totales"].as_f64().unwrap() * 100.0, 30000.0);
}

#[test]
fn get_product_info_convierte_precio_de_centavos_a_pesos() {
    let conn = db_prueba();
    let v = get_product_info(&conn, &serde_json::json!({"product_id": "Coca-Cola"})).unwrap();
    // 2500 centavos en DB → $25.0 para el LLM.
    assert_eq!(v["precio_venta"], 25.0);
}

#[test]
fn top_products_orden_y_limite() {
    let conn = db_prueba();
    let v = get_top_products(&conn, &serde_json::json!({"date_range": "this_week", "order": "top", "limit": 5})).unwrap();
    let lista = v["productos"].as_array().unwrap();
    assert_eq!(lista.len(), 2);
    assert_eq!(lista[0]["producto"], "Sabritas"); // $200 > $100
    assert_eq!(lista[0]["ventas_totales"], 200.0); // 20000 centavos → $200
}

#[test]
fn compare_periods_revenue_devuelve_pesos_no_centavos() {
    let conn = db_prueba();
    let v = compare_periods(&conn, &serde_json::json!({"period1": "today", "period2": "yesterday"})).unwrap();
    assert_eq!(v["ventas_a"], 300.0);
    assert_eq!(v["ventas_b"], 0.0);
    assert_eq!(v["diferencia"], 300.0);
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
    // ejecutar_tool abre su propia conexión read-only (:memory:).
    let r = ejecutar_tool("hackear_nasa", "{}", ":memory:");
    assert!(r.is_ok()); // nunca rompe el chat
    assert!(r.unwrap().contains("desconocida"));
}

// ── Navegación de inventario: search / categories / by_category ──

#[test]
fn search_products_encuentra_parciales_con_precios_en_pesos() {
    let conn = db_prueba();
    // Búsqueda parcial y sin mayúsculas: "coca" debe hallar Coca-Cola.
    let v = search_products(&conn, &serde_json::json!({"query": "coca"})).unwrap();
    assert_eq!(v["total_encontrados"], 1);
    let p = &v["productos"][0];
    assert_eq!(p["nombre"], "Coca-Cola");
    assert_eq!(p["precio_venta"], 25.0); // 2500 centavos → pesos
    assert_eq!(p["stock"], 2.0);
    assert_eq!(p["categoria"], "Bebidas");
}

#[test]
fn search_products_escapea_comodines_del_input() {
    let conn = db_prueba();
    // Un "%" del LLM NO debe convertirse en comodín: buscaría todo.
    let v = search_products(&conn, &serde_json::json!({"query": "%"})).unwrap();
    assert_eq!(v["total_encontrados"], 0);
    // Y sin query no truena: devuelve error legible para el modelo.
    let v2 = search_products(&conn, &serde_json::json!({})).unwrap();
    assert!(v2.get("error").is_some());
}

#[test]
fn list_categories_cuenta_productos_y_stock() {
    let conn = db_prueba();
    let v = list_categories(&conn, &serde_json::json!({})).unwrap();
    let cats = v["categorias"].as_array().unwrap();
    assert_eq!(cats.len(), 2);
    let nombres: Vec<&str> = cats.iter().map(|c| c["categoria"].as_str().unwrap()).collect();
    assert!(nombres.contains(&"Bebidas"));
    assert!(nombres.contains(&"Panadería"));
    for c in cats {
        assert_eq!(c["productos"], 1);
    }
}

#[test]
fn products_by_category_filtra_case_insensitive_y_ordena_por_vendido() {
    let conn = db_prueba();
    let v = get_products_by_category(
        &conn,
        &serde_json::json!({"category": "bebidas"}), // minúsculas: debe encontrar 'Bebidas'
    )
    .unwrap();
    let lista = v["productos"].as_array().unwrap();
    assert_eq!(lista.len(), 1);
    assert_eq!(lista[0]["nombre"], "Coca-Cola");

    // Sin categoría → catálogo completo ordenado por más vendidos.
    let v2 = get_products_by_category(&conn, &serde_json::json!({})).unwrap();
    let lista2 = v2["productos"].as_array().unwrap();
    assert_eq!(lista2.len(), 2);
    assert_eq!(lista2[0]["nombre"], "Coca-Cola"); // vendido 40 > 15
}
