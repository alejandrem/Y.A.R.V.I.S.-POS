use super::insertar_venta_historica;
use crate::cerebro::analizador_tickets::Item;
use rusqlite::Connection;
use std::collections::HashMap;

fn base() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("CREATE TABLE productos(id INTEGER PRIMARY KEY, nombre TEXT, stock REAL, vendido REAL); CREATE TABLE ventas(id INTEGER PRIMARY KEY, total INTEGER, subtotal INTEGER, iva INTEGER, descuento INTEGER, cajero TEXT, metodo_pago TEXT, estado TEXT, fecha TEXT, folio_ticket TEXT); CREATE TABLE detalle_ventas(id INTEGER PRIMARY KEY, venta_id INTEGER, producto_id INTEGER, producto_nombre TEXT, cantidad REAL, precio_unitario INTEGER, descuento INTEGER, subtotal INTEGER); INSERT INTO productos VALUES(1,'TAZAS',10,3);").unwrap();
    conn
}

#[test]
fn venta_historica_conserva_stock_y_registra_descuento() {
    let conn = base();
    let item = Item {
        producto: "TAZAS".into(),
        cantidad: 2.0,
        precio_unitario: 10.0,
        total: 20.0,
        descuento: Some(2.0),
    };
    let mut mapa = HashMap::new();
    mapa.insert("tazas".to_string(), Some(1));
    insertar_venta_historica(
        &conn,
        &[item],
        "Caja",
        None,
        "EFECTIVO",
        Some("F1"),
        20.0,
        0.0,
        18.0,
        &mapa,
    )
    .unwrap();
    let (stock, vendido): (f64, f64) = conn
        .query_row("SELECT stock,vendido FROM productos WHERE id=1", [], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .unwrap();
    let descuento: i64 = conn
        .query_row("SELECT descuento FROM ventas", [], |r| r.get(0))
        .unwrap();
    let (detalle_desc, detalle_sub): (i64, i64) = conn
        .query_row("SELECT descuento,subtotal FROM detalle_ventas", [], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .unwrap();
    assert_eq!(
        (stock, vendido, descuento, detalle_desc, detalle_sub),
        (10.0, 5.0, 200, 200, 1800)
    );
}

#[test]
fn descuento_global_se_guarda_aunque_no_se_asigne_a_producto() {
    let conn = base();
    let item = Item {
        producto: "TAZAS".into(),
        cantidad: 2.0,
        precio_unitario: 10.0,
        total: 20.0,
        descuento: None,
    };
    insertar_venta_historica(
        &conn,
        &[item],
        "Caja",
        None,
        "EFECTIVO",
        None,
        20.0,
        0.0,
        17.0,
        &HashMap::new(),
    )
    .unwrap();
    let descuento: i64 = conn
        .query_row("SELECT descuento FROM ventas", [], |r| r.get(0))
        .unwrap();
    assert_eq!(descuento, 300);
}
