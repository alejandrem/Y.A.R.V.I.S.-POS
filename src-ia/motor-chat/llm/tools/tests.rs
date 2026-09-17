//! tests — DB en memoria con esquema mínimo + verificación de detección/shape.

use rusqlite::Connection;

use super::deteccion::{detectar_tool_call, quitar_tool_calls};
use super::compras::{
    get_purchase_detail, query_purchase_orders, query_purchases, query_suppliers,
};
use super::operativa::{list_branches, query_branch_stock, query_cost_history, query_expiring};
use super::inventario::{
    get_product_info, get_products_by_category, list_categories, query_inventory, search_products,
};
use super::ventas::{compare_periods, forecast_sales, get_top_products, query_sales};
use super::ejecutar_tool;
use super::sql::sql_readonly;

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

// ── sql_readonly (issue #15): validador + ejecución ──

#[test]
fn sql_acepta_select_y_devuelve_columnas_y_filas() {
    let conn = db_prueba();
    let v = sql_readonly(
        &conn,
        &serde_json::json!({"query": "SELECT nombre, stock FROM productos WHERE stock < 5"}),
    )
    .unwrap();
    assert_eq!(v["columnas"], serde_json::json!(["nombre", "stock"]));
    let filas = v["filas"].as_array().unwrap();
    assert_eq!(filas.len(), 1);
    assert_eq!(filas[0]["nombre"], "Coca-Cola");
}

#[test]
fn sql_agrega_limit_si_falta_y_rechaza_escritura() {
    let conn = db_prueba();
    // Sin LIMIT explícito debe funcionar igual (se agrega LIMIT 100).
    let v = sql_readonly(&conn, &serde_json::json!({"query": "SELECT id FROM ventas"})).unwrap();
    assert_eq!(v["filas"].as_array().unwrap().len(), 3);
    // Escritura, DDL, PRAGMA y multi-sentencia se rechazan.
    for mala in [
        "DELETE FROM ventas",
        "UPDATE productos SET stock = 0",
        "INSERT INTO ventas (total) VALUES (1)",
        "DROP TABLE ventas",
        "PRAGMA table_info(ventas)",
        "SELECT 1; DELETE FROM ventas",
        "SELECT * FROM ventas -- truco",
        "SELECT * FROM sqlite_master",
        "SELECT * FROM ventas LIMIT 5000",
        "SELECT * FROM ventas LIMIT -1",
        "SELECT * FROM tabla_fantasma",
    ] {
        let r = sql_readonly(&conn, &serde_json::json!({"query": mala})).unwrap();
        assert!(r.get("error").is_some(), "debió rechazar: {mala}");
    }
}

#[test]
fn forecast_sales_usa_holt_winters_con_bandas() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "CREATE TABLE ventas (id INTEGER PRIMARY KEY AUTOINCREMENT, fecha DATETIME, total INTEGER, estado TEXT);
         CREATE TABLE detalle_ventas (id INTEGER PRIMARY KEY AUTOINCREMENT, venta_id INTEGER, producto_nombre TEXT, cantidad REAL, subtotal INTEGER);",
    )
    .unwrap();
    // 60 días de $100 diarios (suficiente historia para Holt-Winters).
    for i in 0..60 {
        conn.execute(
            "INSERT INTO ventas (fecha, total, estado) VALUES (date('now', ?1), 10000, 'completada')",
            rusqlite::params![format!("-{} days", 59 - i)],
        )
        .unwrap();
    }
    let v = forecast_sales(&conn, &serde_json::json!({"period": "next_week"})).unwrap();
    assert_eq!(v["motor"], "holt-winters");
    assert_eq!(v["unidad"], "MXN");
    assert_eq!(v["horizonte_dias"], 7);
    let puntos = v["puntos"].as_array().unwrap();
    assert_eq!(puntos.len(), 7);
    assert!(puntos[0].get("minimo").is_some());
    assert!(puntos[0].get("maximo").is_some());
    assert!(v["total_estimado"].as_f64().unwrap() > 0.0);
}

#[test]
fn forecast_sales_sin_historia_da_error_legible() {
    let conn = db_prueba(); // solo 1 día con ventas: insuficiente
    let r = forecast_sales(&conn, &serde_json::json!({"period": "tomorrow"}));
    assert!(r.is_err());
    assert!(r.unwrap_err().contains("sin pronóstico"));
}

#[test]
fn sql_rechaza_tablas_fuera_de_allowlist() {
    let conn = db_prueba();
    // WITH válido sobre tabla permitida sí pasa.
    let v = sql_readonly(
        &conn,
        &serde_json::json!({"query": "WITH t AS (SELECT total FROM ventas) SELECT SUM(total) AS s FROM t"}),
    )
    .unwrap();
    assert!(v.get("error").is_none());
    assert!(v["filas"][0]["s"].as_i64().unwrap() >= 30000);
}

// ── C1: la columna `password` jamás llega al modelo ──

fn db_con_usuarios() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "CREATE TABLE usuarios (id INTEGER PRIMARY KEY, nombre TEXT, password TEXT, rol TEXT);
         INSERT INTO usuarios (nombre, password, rol) VALUES ('admin', 'hash-falso-argon2', 'admin');",
    )
    .unwrap();
    conn
}

#[test]
fn sql_rechaza_pedir_password_por_nombre() {
    let conn = db_con_usuarios();
    for mala in [
        "SELECT nombre, password FROM usuarios",
        "select password from usuarios limit 5",
        "SELECT nombre FROM usuarios WHERE password = 'x'",
        "WITH u AS (SELECT * FROM usuarios) SELECT password FROM u",
    ] {
        let r = sql_readonly(&conn, &serde_json::json!({"query": mala})).unwrap();
        assert!(r.get("error").is_some(), "debió rechazar: {mala}");
    }
}

#[test]
fn sql_select_estrella_recorta_password_de_la_salida() {
    let conn = db_con_usuarios();
    let v = sql_readonly(&conn, &serde_json::json!({"query": "SELECT * FROM usuarios"})).unwrap();
    assert!(v.get("error").is_none());
    assert_eq!(v["columnas"], serde_json::json!(["id", "nombre", "rol"]));
    let fila = &v["filas"][0];
    assert!(fila.get("password").is_none());
    assert_eq!(fila["nombre"], "admin");
}

#[test]
fn sql_snapshot_oculta_columna_password() {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("yarvis_snapshot_test_{}.db", std::process::id()));
    {
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            "CREATE TABLE usuarios (id INTEGER PRIMARY KEY, nombre TEXT, password TEXT, rol TEXT);",
        )
        .unwrap();
    }
    let schema = super::sql::snapshot_schema(path.to_str().unwrap()).unwrap();
    assert!(schema.contains("usuarios("));
    assert!(!schema.to_ascii_uppercase().contains("PASSWORD"), "el schema filtró: {schema}");
    let _ = std::fs::remove_file(&path);
}

// ── Abasto y trazabilidad (migraciones 0012/0013/0018) ──

fn db_abasto() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        r#"
        CREATE TABLE proveedores (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nombre TEXT NOT NULL, telefono TEXT, correo TEXT,
            creado_en DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TABLE compras (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            proveedor_id INTEGER NOT NULL REFERENCES proveedores(id),
            fecha DATETIME DEFAULT CURRENT_TIMESTAMP,
            monto_pagado INTEGER DEFAULT 0, monto_sugerido INTEGER DEFAULT 0,
            metodo_pago TEXT DEFAULT 'efectivo', comentario TEXT,
            cajero_id INTEGER, movimiento_id INTEGER,
            creado_en DATETIME DEFAULT CURRENT_TIMESTAMP,
            orden_id INTEGER
        );
        CREATE TABLE compras_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            compra_id INTEGER NOT NULL REFERENCES compras(id),
            producto_id INTEGER, nombre TEXT NOT NULL,
            presentacion TEXT DEFAULT 'unidad', cantidad REAL NOT NULL,
            precio_sugerido INTEGER DEFAULT 0
        );
        CREATE TABLE ordenes_compra (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            proveedor_id INTEGER NOT NULL REFERENCES proveedores(id),
            fecha DATETIME DEFAULT CURRENT_TIMESTAMP,
            estado TEXT DEFAULT 'pendiente', total_estimado INTEGER DEFAULT 0,
            notas TEXT, creado_en DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TABLE ordenes_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            orden_id INTEGER NOT NULL REFERENCES ordenes_compra(id),
            producto_id INTEGER, nombre TEXT NOT NULL,
            cantidad REAL NOT NULL DEFAULT 0, cantidad_recibida REAL DEFAULT 0,
            costo_unitario INTEGER DEFAULT 0
        );
        CREATE TABLE historial_costos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            producto_id INTEGER, producto_nombre TEXT NOT NULL,
            costo_anterior INTEGER DEFAULT 0, costo_nuevo INTEGER NOT NULL,
            proveedor_id INTEGER, compra_id INTEGER,
            fecha DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TABLE lotes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            producto_id INTEGER, producto_nombre TEXT NOT NULL,
            lote TEXT NOT NULL DEFAULT '', caducidad DATE,
            cantidad REAL DEFAULT 0, compra_id INTEGER,
            creado_en DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TABLE sucursales (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nombre TEXT NOT NULL, direccion TEXT,
            creado_en DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TABLE stock_sucursal (
            sucursal_id INTEGER NOT NULL, producto_id INTEGER NOT NULL,
            stock REAL DEFAULT 0, PRIMARY KEY (sucursal_id, producto_id)
        );
        CREATE TABLE productos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nombre TEXT, precio_venta INTEGER DEFAULT 0, precio_costo INTEGER DEFAULT 0,
            stock REAL DEFAULT 0, stock_minimo REAL DEFAULT 0,
            vendido REAL DEFAULT 0, categoria TEXT
        );
        INSERT INTO proveedores (nombre, telefono) VALUES
          ('Lala Sur', '555-0001'), ('Bimbo Ruta 7', NULL);
        INSERT INTO compras (proveedor_id, fecha, monto_pagado, monto_sugerido, metodo_pago) VALUES
          (1, datetime('now','localtime'), 50000, 48000, 'efectivo'),
          (1, datetime('now','localtime','-40 days'), 30000, 30000, 'transferencia'),
          (2, datetime('now','localtime'), 15000, 15000, 'efectivo');
        INSERT INTO compras_items (compra_id, nombre, cantidad, precio_sugerido) VALUES
          (1, 'Leche Lala 1L', 100.0, 500),
          (2, 'Leche Lala 1L', 60.0, 450);
        INSERT INTO ordenes_compra (proveedor_id, estado, total_estimado) VALUES
          (1, 'pendiente', 20000), (2, 'recibida', 15000);
        INSERT INTO ordenes_items (orden_id, nombre, cantidad, cantidad_recibida, costo_unitario) VALUES
          (1, 'Leche Lala 1L', 40.0, 0.0, 500),
          (2, 'Pan Bimbo', 30.0, 30.0, 400);
        INSERT INTO historial_costos (producto_id, producto_nombre, costo_anterior, costo_nuevo) VALUES
          (1, 'Leche Lala 1L', 400, 500);
        INSERT INTO lotes (producto_nombre, lote, caducidad, cantidad) VALUES
          ('Leche Lala 1L', 'L-001', date('now','-5 days'), 10.0),
          ('Leche Lala 1L', 'L-002', date('now','+10 days'), 20.0),
          ('Pan Bimbo', 'P-009', date('now','+200 days'), 30.0);
        INSERT INTO sucursales (nombre) VALUES ('Centro'), ('Norte');
        INSERT INTO productos (nombre, precio_costo, stock) VALUES
          ('Leche Lala 1L', 500, 50.0), ('Pan Bimbo', 400, 5.0);
        INSERT INTO stock_sucursal (sucursal_id, producto_id, stock) VALUES
          (1, 1, 12.0), (2, 1, 3.0), (2, 2, 0.0);
        "#,
    )
    .unwrap();
    conn
}

#[test]
fn suppliers_con_totales_en_pesos_y_busqueda() {
    let conn = db_abasto();
    let v = query_suppliers(&conn, &serde_json::json!({})).unwrap();
    let lista = v["proveedores"].as_array().unwrap();
    assert_eq!(lista.len(), 2);
    assert_eq!(lista[0]["nombre"], "Lala Sur"); // 800 > 150
    assert_eq!(lista[0]["total_comprado"], 800.0); // 80000 centavos
    assert_eq!(lista[0]["compras"], 2);
    let b = query_suppliers(&conn, &serde_json::json!({"search": "bimbo"})).unwrap();
    assert_eq!(b["proveedores"].as_array().unwrap().len(), 1);
}

#[test]
fn purchases_filtra_por_rango_y_proveedor() {
    let conn = db_abasto();
    let v = query_purchases(&conn, &serde_json::json!({"date_range": "this_month"})).unwrap();
    // Solo las 2 de hoy; la de hace 40 días queda fuera.
    assert_eq!(v["compras"].as_array().unwrap().len(), 2);
    assert_eq!(v["total_pagado"], 650.0);
    let b = query_purchases(
        &conn,
        &serde_json::json!({"date_range": "this_month", "proveedor": "bimbo"}),
    )
    .unwrap();
    assert_eq!(b["compras"].as_array().unwrap().len(), 1);
}

#[test]
fn purchase_detail_con_items_y_error_si_no_existe() {
    let conn = db_abasto();
    let v = get_purchase_detail(&conn, &serde_json::json!({"compra_id": 1})).unwrap();
    assert_eq!(v["proveedor"], "Lala Sur");
    assert_eq!(v["monto_pagado"], 500.0);
    assert_eq!(v["items"][0]["precio_sugerido"], 5.0); // 500 centavos
    assert!(get_purchase_detail(&conn, &serde_json::json!({"compra_id": 999}))
        .unwrap()
        .get("error")
        .is_some());
    assert!(get_purchase_detail(&conn, &serde_json::json!({})).unwrap().get("error").is_some());
}

#[test]
fn purchase_orders_filtra_por_estado_con_renglones() {
    let conn = db_abasto();
    let v = query_purchase_orders(&conn, &serde_json::json!({"estado": "pendiente"})).unwrap();
    let lista = v["ordenes"].as_array().unwrap();
    assert_eq!(lista.len(), 1);
    assert_eq!(lista[0]["items"][0]["cantidad"], 40.0);
    assert_eq!(lista[0]["items"][0]["cantidad_recibida"], 0.0);
    let todas = query_purchase_orders(&conn, &serde_json::json!({"estado": "todas"})).unwrap();
    assert_eq!(todas["ordenes"].as_array().unwrap().len(), 2);
    assert!(query_purchase_orders(&conn, &serde_json::json!({"estado": "volando"}))
        .unwrap()
        .get("error")
        .is_some());
}

#[test]
fn cost_history_con_historial_y_compras() {
    let conn = db_abasto();
    let v = query_cost_history(&conn, &serde_json::json!({"product_id": "lala"})).unwrap();
    assert_eq!(v["producto"], "Leche Lala 1L");
    assert_eq!(v["costo_actual"], 5.0);
    assert_eq!(v["historial"][0]["costo_nuevo"], 5.0);
    assert_eq!(v["ultimas_compras"].as_array().unwrap().len(), 2);
    assert!(query_cost_history(&conn, &serde_json::json!({"product_id": "fantasma"}))
        .unwrap()
        .get("error")
        .is_some());
}

#[test]
fn expiring_separa_vencidos_y_proximos() {
    let conn = db_abasto();
    let v = query_expiring(&conn, &serde_json::json!({"dias": 30})).unwrap();
    assert_eq!(v["caducados"].as_array().unwrap().len(), 1);
    assert_eq!(v["caducados"][0]["lote"], "L-001");
    assert_eq!(v["por_caducar"].as_array().unwrap().len(), 1);
    assert_eq!(v["por_caducar"][0]["lote"], "L-002");
    // El de +200 días queda fuera del horizonte.
    let corto = query_expiring(&conn, &serde_json::json!({"dias": 5})).unwrap();
    assert_eq!(corto["por_caducar"].as_array().unwrap().len(), 0);
}

#[test]
fn branches_y_stock_por_sucursal() {
    let conn = db_abasto();
    let v = list_branches(&conn, &serde_json::json!({})).unwrap();
    assert_eq!(v["sucursales"].as_array().unwrap().len(), 2);
    let s = query_branch_stock(&conn, &serde_json::json!({"sucursal": "norte"})).unwrap();
    assert_eq!(s["sucursal"], "Norte");
    assert_eq!(s["total_lineas"], 2);
    assert!(query_branch_stock(&conn, &serde_json::json!({"sucursal": "sur"}))
        .unwrap()
        .get("error")
        .is_some());
    assert!(query_branch_stock(&conn, &serde_json::json!({})).unwrap().get("error").is_some());
}

#[test]
fn sql_permite_tablas_de_abasto() {
    let conn = db_abasto();
    let v = sql_readonly(&conn, &serde_json::json!({"query": "SELECT nombre FROM proveedores"}))
        .unwrap();
    assert!(v.get("error").is_none());
    assert_eq!(v["filas"].as_array().unwrap().len(), 2);
}
