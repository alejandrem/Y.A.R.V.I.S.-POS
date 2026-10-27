use super::asegurar_producto;
use crate::cerebro::analizador_tickets::Item;
use rusqlite::Connection;
use std::collections::{HashMap, HashSet};

#[test]
fn producto_nuevo_se_registra_en_el_mapa_y_no_duplica_vendido() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("CREATE TABLE productos(id INTEGER PRIMARY KEY, nombre TEXT, precio_venta INTEGER, precio_costo INTEGER, stock REAL, stock_minimo REAL, vendido REAL, categoria TEXT);").unwrap();
    let item = Item {
        producto: "TAZAS".into(),
        cantidad: 2.0,
        precio_unitario: 60.0,
        total: 120.0,
        descuento: None,
    };
    let (mut mapa, mut vistos) = (HashMap::new(), HashSet::new());
    assert!(
        asegurar_producto(&conn, &item, "TAZAS|6000", &mut vistos, &mut mapa, &[])
            .unwrap()
            .is_some()
    );
    assert_eq!(mapa.get("tazas").copied().flatten(), Some(1));
    assert!(
        asegurar_producto(&conn, &item, "TAZAS|6000", &mut vistos, &mut mapa, &[])
            .unwrap()
            .is_none()
    );
    let (stock, vendido): (f64, f64) = conn
        .query_row("SELECT stock, vendido FROM productos", [], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .unwrap();
    assert_eq!((stock, vendido), (0.0, 0.0));
}
