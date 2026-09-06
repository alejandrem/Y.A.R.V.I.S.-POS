// Prueba de carga del pipeline real de importación masiva.
//
// Qué hace:
//   1. Genera N tickets sintéticos (mismo formato, folios y fechas
//      distintos) en una carpeta temporal.
//   2. Crea una DB SQLite con el esquema que espera el procesador.
//   3. Corre `procesar_carpeta_impl` (detección + segmentación + parseo +
//      escritura) midiendo el tiempo.
//   4. Re-importa la misma carpeta y verifica que no duplique nada.
//
// Uso:
//   cargo run --release --example bench1000 -- [N]
//
// N default = 1000. En `--release` el tiempo es representativo de la app
// real; en debug todo es ~10x más lento y solo sirve para verificar que
// no truena. La carpeta temporal se borra sola al terminar.

use std::time::Instant;

const ESQUEMA: &str = "
CREATE TABLE productos (id INTEGER PRIMARY KEY AUTOINCREMENT, nombre TEXT NOT NULL,
  precio_venta INTEGER DEFAULT 0, precio_costo INTEGER DEFAULT 0, stock REAL DEFAULT 0,
  stock_minimo REAL DEFAULT 5, vendido REAL DEFAULT 0, categoria TEXT DEFAULT '');
CREATE TABLE ventas (id INTEGER PRIMARY KEY AUTOINCREMENT, total INTEGER, subtotal INTEGER,
  iva INTEGER, cajero TEXT, metodo_pago TEXT, estado TEXT, fecha TEXT);
CREATE TABLE detalle_ventas (id INTEGER PRIMARY KEY AUTOINCREMENT, venta_id INTEGER,
  producto_id INTEGER, producto_nombre TEXT, cantidad REAL, precio_unitario INTEGER,
  descuento INTEGER, subtotal INTEGER);";

fn mapeo_clasico() -> src_ia::cerebro::analizador_tickets::MapeoColumnas {
    src_ia::cerebro::analizador_tickets::MapeoColumnas {
        cantidad: Some(0),
        producto: Some(vec![1, -3]),
        precio_unitario: Some(-2),
        total: Some(-1),
        descuento: None,
    }
}

fn main() {
    let n: usize = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(1000);
    let dir = std::env::temp_dir().join(format!("bench_yarvis_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let productos = [
        ("COCA COLA 600ML", 18.0),
        ("SABRITAS ADOBADAS 45G", 18.50),
        ("MASECA 1KG", 20.70),
        ("HUEVO SAN JUAN 18PZA", 58.00),
        ("JARRITOS TAMARINDO 400", 14.00),
    ];
    for i in 0..n {
        let dia = 1 + (i % 28);
        let hora = 8 + (i % 12);
        let min = i % 60;
        let mut t = format!(
            "ABARROTES LA ESQUINA\nFOLIO: {i:06}          FECHA: {dia:02}/05/2026\nCAJERO: MARIA G.         HORA: {hora:02}:{min:02}:00\n"
        );
        for (k, (nombre, precio)) in productos.iter().enumerate() {
            let cant = (1 + ((i + k) % 3)) as f64;
            t.push_str(&format!("{cant} {nombre} {precio:.2} {:.2}\n", cant * precio));
        }
        t.push_str("TOTAL: 300.00\nEFECTIVO: 500.00\nGRACIAS POR SU COMPRA\n");
        std::fs::write(dir.join(format!("ticket_{i:04}.txt")), t).unwrap();
    }

    let db = dir.join("bench.db");
    let conn = rusqlite::Connection::open(&db).unwrap();
    conn.execute_batch(ESQUEMA).unwrap();
    drop(conn);
    let db = db.to_string_lossy().to_string();

    // 1. Detección global sobre la carpeta generada (debe pasar holgada).
    let muestra: Vec<String> = (0..15)
        .map(|i| std::fs::read_to_string(dir.join(format!("ticket_{i:04}.txt"))).unwrap())
        .collect();
    let refs: Vec<&str> = muestra
        .iter()
        .flat_map(|t| t.lines().take(60))
        .collect();
    match src_ia::cerebro::analizador_tickets::detectar_mapeo(&refs) {
        Some(d) => println!("detección: confianza={:.3} ({} líneas)", d.confianza, d.lineas_evaluadas),
        None => println!("detección: SIN DETECCION (la importación por archivo igual rescata)"),
    }

    // 2. Importación masiva cronometrada.
    let archivos = src_ia::cerebro::parseador_masivo::obtener_archivos_txt(&dir.to_string_lossy());
    assert_eq!(archivos.len(), n);
    let t0 = Instant::now();
    let stats =
        src_ia::cerebro::parseador_masivo::procesar_carpeta_impl(archivos, mapeo_clasico(), db.clone());
    let dt = t0.elapsed();
    println!(
        "importación: {dt:?} para {n} tickets ({}ms/archivo) ventas={} items={} omitidas={} errores={} nuevos={}",
        dt.as_millis() as f64 / n as f64,
        stats.ventas_creadas,
        stats.items_insertados,
        stats.ventas_omitidas,
        stats.errores,
        stats.productos_nuevos,
    );
    assert_eq!(stats.ventas_creadas, n, "faltaron ventas");
    assert_eq!(stats.errores, 0, "hubo archivos con error");

    // 3. Re-importación: no debe crear nada.
    let archivos2 = src_ia::cerebro::parseador_masivo::obtener_archivos_txt(&dir.to_string_lossy());
    let t1 = Instant::now();
    let stats2 =
        src_ia::cerebro::parseador_masivo::procesar_carpeta_impl(archivos2, mapeo_clasico(), db.clone());
    println!(
        "re-importación: {:?} ventas={} omitidas={} errores={}",
        t1.elapsed(),
        stats2.ventas_creadas,
        stats2.ventas_omitidas,
        stats2.errores,
    );
    assert_eq!(stats2.ventas_creadas, 0, "la re-importación duplicó");
    assert_eq!(stats2.ventas_omitidas, n);

    let _ = std::fs::remove_dir_all(&dir);
    println!("OK: {n} tickets importados y re-importación idempotente.");
}
