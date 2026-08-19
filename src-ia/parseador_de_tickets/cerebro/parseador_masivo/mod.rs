// ============================================================
// parseador_masivo — Procesamiento masivo de carpetas de tickets
// .txt con transacción propia por archivo. Port de lote.py.
//
//   * archivos.rs    → descubrimiento y lectura de archivos .txt
//   * items.rs       → sumas y redondeo sobre items parseados
//   * almacen.rs     → escritura en SQLite (venta + detalle + stock)
//   * resumen.rs     → modelos de resultado/estadísticas
//   * procesador.rs  → orquestación (stream por canal y modo síncrono)
//
// Sin HTTP: expone funciones puras consumibles desde Tauri.
// ============================================================

mod archivos;
mod items;
mod almacen;
mod resumen;
mod procesador;

pub use archivos::obtener_archivos_txt;
pub use procesador::{procesar_archivos, procesar_carpeta_impl};
pub use resumen::{
    ArchivoResultado, EstadisticasCarpeta, ProductoNuevo, ResumenVenta, TicketFallido,
};

// Imports que los tests del módulo usan vía `use super::*`.
#[cfg(test)]
use crate::cerebro::analizador_tickets::MapeoColumnas;
#[cfg(test)]
use rusqlite::{params, Connection};
#[cfg(test)]
use std::path::Path;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    const MAPEO: &str = r#"{"cantidad": 0, "producto": [1], "precio_unitario": 2, "total": 3}"#;

    const TICKET: &str = "TICKET 1\n12/05/2026\n2 TAZAS $60.00 $120.00\n1 PLATO $80.00 $80.00\nTOTAL $200.00\n";

    fn mapeo() -> MapeoColumnas {
        serde_json::from_str(MAPEO).unwrap()
    }

    fn tmp_workspace(nombre: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("cerebro_lote_{}_{}", nombre, nanos));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn crear_bd(dir: &Path) -> String {
        let path = dir.join("test.db");
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            "CREATE TABLE productos (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                nombre TEXT NOT NULL,
                precio_venta REAL DEFAULT 0,
                stock REAL DEFAULT 0,
                vendido REAL DEFAULT 0
             );
             CREATE TABLE ventas (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                total REAL, subtotal REAL, iva REAL,
                cajero TEXT, metodo_pago TEXT, estado TEXT, fecha TEXT
             );
             CREATE TABLE detalle_ventas (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                venta_id INTEGER, producto_nombre TEXT,
                cantidad REAL, precio_unitario REAL,
                descuento REAL, subtotal REAL
             );",
        )
        .unwrap();
        drop(conn);
        path.to_string_lossy().to_string()
    }

    fn escribir(dir: &Path, nombre: &str, contenido: &str) -> String {
        let p = dir.join(nombre);
        std::fs::write(&p, contenido).unwrap();
        p.to_string_lossy().to_string()
    }

    /// Siembra un producto del catálogo con su precio EXACTO: `insertar_venta`
    /// (regla C) matchea `detalle_ventas`/stock por `nombre + precio_venta`.
    fn sembrar_producto(db: &str, nombre: &str, precio: f64, stock: f64) {
        let conn = Connection::open(db).unwrap();
        conn.execute(
            "INSERT INTO productos (nombre, precio_venta, stock, vendido)
             VALUES (?1, ?2, ?3, 0)",
            params![nombre, precio, stock],
        )
        .unwrap();
        drop(conn);
    }

    fn contar(db: &str, tabla: &str) -> i64 {
        let conn = Connection::open(db).unwrap();
        let n: i64 = conn
            .query_row(&format!("SELECT COUNT(*) FROM {tabla}"), [], |r| r.get(0))
            .unwrap();
        n
    }

    #[test]
    fn sync_crea_ventas_y_detalles() {
        let dir = tmp_workspace("ventas");
        let db = crear_bd(&dir);
        sembrar_producto(&db, "TAZAS", 60.0, 0.0);
        sembrar_producto(&db, "PLATO", 80.0, 0.0);
        let a = escribir(&dir, "ticket1.txt", TICKET);
        let b = escribir(&dir, "ticket2.txt", TICKET);

        let stats = procesar_carpeta_impl(vec![a, b], mapeo(), db.clone());

        assert_eq!(stats.exitosos, 2);
        assert_eq!(stats.errores, 0);
        assert_eq!(stats.ventas_creadas, 2);
        assert_eq!(stats.items_insertados, 4);
        assert_eq!(contar(&db, "ventas"), 2);
        assert_eq!(contar(&db, "detalle_ventas"), 4);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn archivo_vacio_es_error_sin_venta() {
        let dir = tmp_workspace("vacio");
        let db = crear_bd(&dir);
        let a = escribir(&dir, "vacio.txt", "\n\n");

        let stats = procesar_carpeta_impl(vec![a], mapeo(), db.clone());
        assert_eq!(stats.exitosos, 0);
        assert_eq!(stats.errores, 1);
        assert_eq!(contar(&db, "ventas"), 0);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn procesar_archivos_emite_por_canal() {
        let dir = tmp_workspace("canal");
        let db = crear_bd(&dir);
        let a = escribir(&dir, "ticket1.txt", TICKET);
        let b = escribir(&dir, "ticket2.txt", TICKET);

        let (tx, rx) = std::sync::mpsc::channel::<ArchivoResultado>();
        procesar_archivos(&[a, b], &mapeo(), &db, &tx);
        drop(tx);

        let resultados: Vec<ArchivoResultado> = rx.into_iter().collect();
        assert_eq!(resultados.len(), 2);
        assert!(resultados.iter().all(|r| r.ok));
        assert_eq!(resultados.iter().map(|r| r.items).sum::<usize>(), 4);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn linea_duplicada_no_se_descuenta_dos_veces() {
        let dir = tmp_workspace("dup");
        let db = crear_bd(&dir);
        sembrar_producto(&db, "TAZAS", 60.0, 0.0);
        let ticket_dup =
            "TICKET 1\n12/05/2026\n2 TAZAS $60.00 $120.00\n2 TAZAS $60.00 $120.00\nTOTAL $240.00\n";
        let a = escribir(&dir, "dup.txt", ticket_dup);

        let stats = procesar_carpeta_impl(vec![a], mapeo(), db.clone());
        assert_eq!(stats.exitosos, 1);
        assert_eq!(stats.items_insertados, 1);
        assert_eq!(stats.duplicados_detectados, 1);
        assert_eq!(contar(&db, "detalle_ventas"), 1);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn stock_se_actualiza_con_cantidad_del_item() {
        let dir = tmp_workspace("stock");
        let db = crear_bd(&dir);
        sembrar_producto(&db, "TAZAS", 60.0, 100.0);

        let a = escribir(&dir, "ticket_stock.txt", TICKET);
        procesar_carpeta_impl(vec![a], mapeo(), db.clone());

        let conn = Connection::open(&db).unwrap();
        let (stock, vendido): (f64, f64) = conn
            .query_row(
                "SELECT stock, vendido FROM productos WHERE nombre = 'TAZAS'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(stock, 98.0);
        assert_eq!(vendido, 2.0);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn fallo_en_un_archivo_no_afecta_a_los_demas() {
        let dir = tmp_workspace("mixto");
        let db = crear_bd(&dir);
        sembrar_producto(&db, "TAZAS", 60.0, 0.0);
        sembrar_producto(&db, "PLATO", 80.0, 0.0);
        let bueno = escribir(&dir, "bueno.txt", TICKET);
        let solo_cabeceras = escribir(&dir, "cabeceras.txt", "GRACIAS POR SU COMPRA\nCFDI: 4D8F2A1\n");

        let stats =
            procesar_carpeta_impl(vec![bueno, solo_cabeceras], mapeo(), db.clone());

        assert_eq!(stats.exitosos, 1);
        assert_eq!(stats.errores, 1);
        assert_eq!(stats.tickets_fallidos[0].archivo, "cabeceras.txt");
        assert_eq!(contar(&db, "ventas"), 1);
        assert_eq!(contar(&db, "detalle_ventas"), 2);

        let _ = std::fs::remove_dir_all(&dir);
    }
}