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

pub(crate) mod almacen;
mod archivos;
mod items;
mod procesador;
mod resumen;

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

    const TICKET: &str =
        "TICKET 1\n12/05/2026\n2 TAZAS $60.00 $120.00\n1 PLATO $80.00 $80.00\nTOTAL $200.00\n";

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

    fn escribir(dir: &Path, nombre: &str, contenido: &str) -> String {
        let p = dir.join(nombre);
        std::fs::write(&p, contenido).unwrap();
        p.to_string_lossy().to_string()
    }

    /// Siembra un producto del catálogo con su precio EXACTO (en PESOS; se
    /// escribe en CENTAVOS como la DB migrada): `insertar_venta` matchea
    /// `detalle_ventas`/stock por `nombre + precio_venta` vía la clave de
    /// dedupe en centavos.
    fn sembrar_producto(db: &str, nombre: &str, precio: f64, stock: f64) {
        let conn = Connection::open(db).unwrap();
        conn.execute(
            "INSERT INTO productos (nombre, precio_venta, stock, vendido)
             VALUES (?1, ?2, ?3, 0)",
            params![nombre, (precio * 100.0).round() as i64, stock],
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
    fn un_archivo_con_dos_tickets_reales_crea_dos_ventas_con_folio_y_fecha() {
        let dir = tmp_workspace("tickets_concatenados");
        let db = crear_bd(&dir);
        let texto = r#"
ABARROTES "LA ESQUINA FELIZ"
Ticket: TCK-000001
Fecha: 14/07/2024       Hora: 14:28:37
2 Rockaleta                 $6.00    10%     $10.80
1 Heineken 473ml           $28.00      -     $28.00
1 Tocino 200g              $48.00      -     $48.00
1 Spaghetti 500g           $18.00      -     $18.00
SUBTOTAL:                                      $106.00
DESCUENTO:                                      -$1.20
TOTAL:                                         $104.80
Forma de pago: TARJETA
Ticket: TCK-000002
Fecha: 15/07/2024       Hora: 10:00:00
1 Pan $20.00 $20.00
SUBTOTAL: $20.00
TOTAL: $20.00
Forma de pago: EFECTIVO
"#;
        let archivo = escribir(&dir, "lote.txt", texto);
        let mapeo = MapeoColumnas {
            cantidad: Some(0),
            producto: Some(vec![1]),
            precio_unitario: Some(2),
            total: Some(-1),
            descuento: None,
        };

        let stats = procesar_carpeta_impl(vec![archivo], mapeo, db.clone());
        assert_eq!(stats.exitosos, 1);
        assert_eq!(stats.ventas_creadas, 2);

        let conn = Connection::open(&db).unwrap();
        // La DB guarda CENTAVOS: $104.80 → 10480, $20.00 → 2000.
        let ventas: Vec<(String, String, i64)> = conn
            .prepare("SELECT folio_ticket, fecha, total FROM ventas ORDER BY id")
            .unwrap()
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .unwrap()
            .map(Result::unwrap)
            .collect();

        assert_eq!(ventas.len(), 2);
        assert_eq!(ventas[0].0, "TCK-000001");
        assert_eq!(ventas[0].1, "2024-07-14 14:28:00");
        assert_eq!(ventas[0].2, 10_480);
        assert_eq!(ventas[1].0, "TCK-000002");
        assert_eq!(ventas[1].1, "2024-07-15 10:00:00");
        assert_eq!(ventas[1].2, 2_000);

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
    fn reimportar_la_misma_carpeta_no_duplica_ventas_ni_stock() {
        let dir = tmp_workspace("idempotente");
        let db = crear_bd(&dir);
        sembrar_producto(&db, "TAZAS", 60.0, 100.0);
        sembrar_producto(&db, "PLATO", 80.0, 0.0);

        let ticket_f = |folio: &str| {
            format!("FOLIO: {folio}\n12/05/2026\n2 TAZAS $60.00 $120.00\n1 PLATO $80.00 $80.00\nTOTAL $200.00\n")
        };
        let a = escribir(&dir, "t1.txt", &ticket_f("0001"));
        let b = escribir(&dir, "t2.txt", &ticket_f("0002"));
        // `c` es una COPIA de `a` (mismo ticket, otro nombre de archivo).
        let c = escribir(&dir, "t1_copia.txt", &ticket_f("0001"));

        // Primera corrida: 2 folios nuevos; la copia de `a` se omite AHÍ MISMO.
        let stats1 = procesar_carpeta_impl(
            vec![a.clone(), b, c],
            mapeo(),
            db.clone(),
        );
        assert_eq!(stats1.ventas_creadas, 2);
        assert_eq!(stats1.ventas_omitidas, 1, "copia del folio 0001 no se omitió");
        assert_eq!(stats1.errores, 0, "una omisión total no es un error");

        // Segunda corrida: re-importar lo mismo NO crea nada.
        let stats2 = procesar_carpeta_impl(vec![a], mapeo(), db.clone());
        assert_eq!(stats2.ventas_creadas, 0);
        assert_eq!(stats2.ventas_omitidas, 1);
        assert_eq!(stats2.exitosos, 1, "omitido completo se informa como ok");
        assert_eq!(stats2.errores, 0);

        // La DB queda intacta: dos ventas, cuatro detalles, stock ×1.
        assert_eq!(contar(&db, "ventas"), 2);
        assert_eq!(contar(&db, "detalle_ventas"), 4);
        let conn = Connection::open(&db).unwrap();
        let (stock, vendido): (f64, f64) = conn
            .query_row(
                "SELECT stock, vendido FROM productos WHERE nombre = 'TAZAS'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        // Cada venta descuenta 2 TAZAS: 2 ventas × 2 = 4 (la copia y la
        // re-importación NO volvieron a descontar).
        assert_eq!(stock, 96.0, "el stock se descontó de más");
        assert_eq!(vendido, 4.0);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn carpeta_con_formatos_mezclados_rescata_el_formato_distinto() {
        let dir = tmp_workspace("formatos_mezclados");
        let db = crear_bd(&dir);

        // Formato A (el del mapeo "votado" para la carpeta).
        let a = escribir(
            &dir,
            "formato_a.txt",
            "FOLIO: 001\n12/05/2026\n2 TAZAS 60.00 120.00\n1 PLATO 80.00 80.00\n3 VASO 50.00 150.00\nTOTAL $350.00\n",
        );
        // Formato B (producto primero, cantidad en medio) — otra impresora.
        let b = escribir(
            &dir,
            "formato_b.txt",
            "FOLIO: 002\n13/05/2026\nPAN 3 12.00 36.00\nLECHE 1 22.50 22.50\nJABON 2 15.00 30.00\nTOTAL $88.50\n",
        );

        let stats = procesar_carpeta_impl(vec![a, b], mapeo(), db.clone());

        // El archivo B no lo reconocía el mapeo global: se le detectó uno
        // propio y AMBAS ventas entraron.
        assert_eq!(stats.ventas_creadas, 2, "stats: {stats:?}");
        assert_eq!(stats.archivos_formato_distinto, 1);
        assert_eq!(contar(&db, "detalle_ventas"), 6);

        // Y los nombres del formato B vienen bien ("PAN", no el número).
        let conn = Connection::open(&db).unwrap();
        let nombres: Vec<String> = conn
            .prepare("SELECT producto_nombre FROM detalle_ventas ORDER BY id")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        assert_eq!(
            nombres,
            vec!["TAZAS", "PLATO", "VASO", "PAN", "LECHE", "JABON"]
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn fallo_en_un_archivo_no_afecta_a_los_demas() {
        let dir = tmp_workspace("mixto");
        let db = crear_bd(&dir);
        sembrar_producto(&db, "TAZAS", 60.0, 0.0);
        sembrar_producto(&db, "PLATO", 80.0, 0.0);
        let bueno = escribir(&dir, "bueno.txt", TICKET);
        let solo_cabeceras = escribir(
            &dir,
            "cabeceras.txt",
            "GRACIAS POR SU COMPRA\nCFDI: 4D8F2A1\n",
        );

        let stats = procesar_carpeta_impl(vec![bueno, solo_cabeceras], mapeo(), db.clone());

        assert_eq!(stats.exitosos, 1);
        assert_eq!(stats.errores, 1);
        assert_eq!(stats.tickets_fallidos[0].archivo, "cabeceras.txt");
        assert_eq!(contar(&db, "ventas"), 1);
        assert_eq!(contar(&db, "detalle_ventas"), 2);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
