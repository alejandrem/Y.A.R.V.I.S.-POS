//! soporte.rs — Helpers compartidos de la suite de estrés (`estres`).
//!
//! Todo es `pub(crate)`: los hermanos (`fuzzing`, `masivo`) lo traen vía
//! `crate::soporte::*`.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;

use src_ia::cerebro::analizador_tickets::{MapeoColumnas, PRECIO_MAXIMO};

pub(crate) fn mapeo() -> MapeoColumnas {
    serde_json::from_str(r#"{"cantidad": 0, "producto": [1], "precio_unitario": 2, "total": 3}"#)
        .unwrap()
}

pub(crate) fn finito(x: f64) -> bool {
    x.is_finite() && x.abs() <= PRECIO_MAXIMO
}

pub(crate) fn tmp_workspace(nombre: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("estres_{}_{}", nombre, nanos));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

pub(crate) fn crear_bd(dir: &Path) -> String {
    let path = dir.join("estres.db");
    let conn = Connection::open(&path).unwrap();
    conn.execute_batch(
        // Espejo del esquema migrado: dinero en INTEGER CENTAVOS.
        "CREATE TABLE productos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nombre TEXT NOT NULL,
            precio_venta INTEGER DEFAULT 0,
            stock REAL DEFAULT 0,
            vendido REAL DEFAULT 0
         );
         CREATE TABLE ventas (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            total INTEGER, subtotal INTEGER, iva INTEGER,
            cajero TEXT, metodo_pago TEXT, estado TEXT, fecha TEXT
         );
         CREATE TABLE detalle_ventas (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            venta_id INTEGER, producto_id INTEGER, producto_nombre TEXT,
            cantidad REAL, precio_unitario INTEGER,
            descuento INTEGER, subtotal INTEGER
         );",
    )
    .unwrap();
    drop(conn);
    path.to_string_lossy().to_string()
}

pub(crate) fn contar(db: &str, tabla: &str) -> i64 {
    let conn = Connection::open(db).unwrap();
    conn.query_row(&format!("SELECT COUNT(*) FROM {tabla}"), [], |r| r.get(0))
        .unwrap()
}
