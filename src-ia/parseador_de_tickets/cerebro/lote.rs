//! lote.rs — Port de `yarvis-IA/parseador_de_tickets/cerebro/lote.py`
//!
//! Procesa carpetas de tickets .txt con transacción propia por archivo.
//! Sin HTTP: expone las funciones puras de parseo/inserción, consumibles
//! desde un servidor axum o los comandos de Tauri (la capa HTTP es aparte).
//!
//! Depende de `cerebro::analizador` (parsear_linea, fecha/hora, pago) que a
//! su vez conecta con `cerebro::filtrador` (limpiar_producto).

use rusqlite::{params, Connection};
use std::collections::HashSet;
use std::path::Path;
use std::sync::mpsc::Sender;

use super::analizador::{extraer_fecha_hora_regex, extraer_metodo_pago, parsear_linea, Item, MapeoColumnas};

// ---------------------------------------------------------------------------
// Tipos
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProductoNuevo {
    pub nombre: String,
    pub precio: f64,
}

/// Resultado de procesar UN archivo (equivalente a un `yield` de Python).
#[derive(Debug, Clone, serde::Serialize)]
pub struct ArchivoResultado {
    pub archivo: String,
    pub ok: bool,
    pub motivo: Option<String>,
    pub items: usize,
    pub duplicados: usize,
    pub nuevos: Vec<ProductoNuevo>,
    pub existentes: usize,
    pub venta_id: Option<i64>,
    pub total: f64,
}

impl ArchivoResultado {
    fn info(ok: bool, motivo: Option<String>) -> Self {
        Self {
            archivo: String::new(),
            ok,
            motivo,
            items: 0,
            duplicados: 0,
            nuevos: Vec::new(),
            existentes: 0,
            venta_id: None,
            total: 0.0,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ResumenVenta {
    pub archivo: String,
    pub venta_id: Option<i64>,
    pub items: usize,
    pub total: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TicketFallido {
    pub archivo: String,
    pub motivo: Option<String>,
}

#[derive(Debug, Default, serde::Serialize)]
pub struct EstadisticasCarpeta {
    pub total_archivos: usize,
    pub procesados: usize,
    pub exitosos: usize,
    pub errores: usize,
    pub ventas_creadas: usize,
    pub items_insertados: usize,
    pub productos_nuevos: usize,
    pub productos_existentes: usize,
    pub duplicados_detectados: usize,
    pub productos_nuevos_lista: Vec<ProductoNuevo>,
    pub resumen_ventas: Vec<ResumenVenta>,
    pub tickets_fallidos: Vec<TicketFallido>,
}

// ---------------------------------------------------------------------------
// Utilidades
// ---------------------------------------------------------------------------

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

/// Lista de archivos .txt en la carpeta (ordenados por nombre).
pub fn obtener_archivos_txt(carpeta: &str) -> Vec<String> {
    let mut archivos: Vec<String> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(carpeta) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let es_txt = path
                .extension()
                .map(|e| e.to_string_lossy().to_ascii_lowercase() == "txt")
                .unwrap_or(false);
            if es_txt {
                archivos.push(path.to_string_lossy().to_string());
            }
        }
    }
    archivos.sort();
    archivos
}

fn nombre_de_archivo(ruta: &str) -> String {
    Path::new(ruta)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| ruta.to_string())
}

/// Igual que Python `open(..., errors="ignore")`: bytes inválidos se descartan.
fn leer_archivo_tolerante(ruta: &str) -> std::io::Result<String> {
    let bytes = std::fs::read(ruta)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// Suma de totales de línea. Espejo de `sum(i.get("total",0) or cant*precio)`.
fn calcular_subtotal(items: &[Item]) -> f64 {
    items
        .iter()
        .map(|i| {
            if i.total != 0.0 {
                i.total
            } else {
                i.cantidad * i.precio_unitario
            }
        })
        .sum()
}

fn extraer_cajero(texto: &str) -> String {
    for linea in texto.lines().take(10) {
        let lower = linea.to_lowercase();
        if lower.contains("cajero") || lower.contains("empleado") || lower.contains("vendedor") {
            if let Some(idx) = linea.find(':') {
                return linea[idx + 1..].trim().to_string();
            }
        }
    }
    "SISTEMA".to_string()
}

/// Precarga el set de claves `NOMBRE|precio` ya existentes en `productos`.
fn cargar_estado(db_path: &str) -> HashSet<String> {
    let mut vistos = HashSet::new();
    if let Ok(conn) = Connection::open(db_path) {
        if let Ok(mut stmt) = conn.prepare("SELECT nombre, precio_venta FROM productos") {
            if let Ok(rows) = stmt.query_map([], |row| {
                let nombre: String = row.get(0)?;
                let precio: f64 = row.get(1)?;
                Ok((nombre, precio))
            }) {
                for row in rows.flatten() {
                    let (nombre, precio) = row;
                    vistos.insert(format!("{}|{:.2}", nombre.trim().to_uppercase(), precio));
                }
            }
        }
    }
    vistos
}

fn insertar_venta(
    conn: &Connection,
    items: &[Item],
    cajero: &str,
    fecha_iso: Option<&str>,
    metodo_pago: &str,
) -> Result<i64, rusqlite::Error> {
    let subtotal = calcular_subtotal(items);
    let iva = round2(subtotal * 0.16);
    let total = round2(subtotal + iva);

    match fecha_iso {
        Some(fecha) => {
            conn.execute(
                "INSERT INTO ventas (total, subtotal, iva, cajero, metodo_pago, estado, fecha)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![total, subtotal, iva, cajero, metodo_pago, "completada", fecha],
            )?;
        }
        None => {
            conn.execute(
                "INSERT INTO ventas (total, subtotal, iva, cajero, metodo_pago, estado)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![total, subtotal, iva, cajero, metodo_pago, "completada"],
            )?;
        }
    }
    let venta_id = conn.last_insert_rowid();

    for item in items {
        let sub = round2(item.cantidad * item.precio_unitario - item.descuento.unwrap_or(0.0));

        conn.execute(
            "INSERT INTO detalle_ventas (venta_id, producto_nombre, cantidad, precio_unitario, descuento, subtotal)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                venta_id,
                item.producto,
                item.cantidad,
                item.precio_unitario,
                item.descuento.unwrap_or(0.0),
                sub
            ],
        )?;

        // Actualizar stock y vendido (case-insensitive).
        conn.execute(
            "UPDATE productos SET stock = stock - ?1 WHERE LOWER(nombre) = LOWER(?2)",
            params![item.cantidad, item.producto],
        )?;
        conn.execute(
            "UPDATE productos SET vendido = vendido + ?1 WHERE LOWER(nombre) = LOWER(?2)",
            params![item.cantidad, item.producto],
        )?;
    }

    Ok(venta_id)
}

// ---------------------------------------------------------------------------
// Núcleo: procesa cada archivo con transacción propia y emite un resultado
// por canal (equivalente al generador `_procesar_archivos` de Python).
// ---------------------------------------------------------------------------

pub fn procesar_archivos(
    archivos: &[String],
    mapeo: &MapeoColumnas,
    db_path: &str,
    tx: &Sender<ArchivoResultado>,
) {
    let mut productos_vistos = cargar_estado(db_path);

    let conn = match Connection::open(db_path) {
        Ok(c) => c,
        Err(e) => {
            for archivo in archivos {
                let mut res = ArchivoResultado::info(
                    false,
                    Some(format!("no se pudo abrir la base de datos: {e}")),
                );
                res.archivo = nombre_de_archivo(archivo);
                let _ = tx.send(res);
            }
            return;
        }
    };

    for archivo in archivos {
        let nombre_archivo = nombre_de_archivo(archivo);

        let texto = match leer_archivo_tolerante(archivo) {
            Ok(t) => t,
            Err(e) => {
                let mut res = ArchivoResultado::info(false, Some(format!("error inesperado: {e}")));
                res.archivo = nombre_archivo;
                let _ = tx.send(res);
                continue;
            }
        };

        if texto.trim().is_empty() {
            let mut res = ArchivoResultado::info(false, Some("archivo vacío".to_string()));
            res.archivo = nombre_archivo;
            let _ = tx.send(res);
            continue;
        }

        let lineas: Vec<&str> = texto
            .trim()
            .lines()
            .filter(|l| !l.trim().is_empty())
            .collect();
        if lineas.is_empty() {
            let mut res = ArchivoResultado::info(false, Some("sin líneas útiles".to_string()));
            res.archivo = nombre_archivo;
            let _ = tx.send(res);
            continue;
        }

        let total_cols = lineas
            .iter()
            .map(|l| l.split_whitespace().count())
            .max()
            .unwrap_or(0);

        let mut items: Vec<Item> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        let mut duplicados = 0usize;
        let mut existentes = 0usize;
        let mut nuevos: Vec<ProductoNuevo> = Vec::new();

        for linea in &lineas {
            let Some(item) = parsear_linea(linea, mapeo, total_cols) else {
                continue;
            };

            let dup_key = format!("{}|{:.2}", item.producto, item.precio_unitario);
            if seen.contains(&dup_key) {
                duplicados += 1;
                continue;
            }
            seen.insert(dup_key.clone());

            if productos_vistos.contains(&dup_key) {
                existentes += 1;
            } else {
                productos_vistos.insert(dup_key);
                nuevos.push(ProductoNuevo {
                    nombre: item.producto.clone(),
                    precio: item.precio_unitario,
                });
            }
            items.push(item);
        }

        if items.is_empty() {
            let mut res = ArchivoResultado::info(
                false,
                Some("ningún producto reconocido con el mapeo actual".to_string()),
            );
            res.archivo = nombre_archivo;
            let _ = tx.send(res);
            continue;
        }

        let (fecha, hora) = extraer_fecha_hora_regex(&texto);
        let fecha_iso = fecha.map(|f| match &hora {
            Some(h) => format!("{f} {h}:00"),
            None => format!("{f} 00:00:00"),
        });

        let cajero = extraer_cajero(&texto);
        let metodo_pago = extraer_metodo_pago(&texto);
        let total_ticket = round2(calcular_subtotal(&items) * 1.16);

        if let Err(e) = conn.execute_batch("BEGIN") {
            let mut res = ArchivoResultado::info(
                false,
                Some(format!("error al insertar en DB: no se pudo iniciar la transacción ({e})")),
            );
            res.archivo = nombre_archivo;
            res.items = items.len();
            res.duplicados = duplicados;
            res.nuevos = nuevos;
            res.existentes = existentes;
            let _ = tx.send(res);
            continue;
        }

        match insertar_venta(&conn, &items, &cajero, fecha_iso.as_deref(), &metodo_pago) {
            Ok(venta_id) => {
                let _ = conn.execute_batch("COMMIT");
                let mut res = ArchivoResultado::info(true, None);
                res.archivo = nombre_archivo;
                res.items = items.len();
                res.duplicados = duplicados;
                res.nuevos = nuevos;
                res.existentes = existentes;
                res.venta_id = Some(venta_id);
                res.total = total_ticket;
                let _ = tx.send(res);
            }
            Err(e) => {
                let _ = conn.execute_batch("ROLLBACK");
                let mut res =
                    ArchivoResultado::info(false, Some(format!("error al insertar en DB: {e}")));
                res.archivo = nombre_archivo;
                res.items = items.len();
                res.duplicados = duplicados;
                res.nuevos = nuevos;
                res.existentes = existentes;
                let _ = tx.send(res);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Modo síncrono: agrega los resultados por archivo en estadísticas totales
// ---------------------------------------------------------------------------

pub fn procesar_carpeta_impl(
    archivos: Vec<String>,
    mapeo: MapeoColumnas,
    db_path: String,
) -> EstadisticasCarpeta {
    let mut stats = EstadisticasCarpeta {
        total_archivos: archivos.len(),
        ..Default::default()
    };

    let mut nombres_nuevos_vistos: HashSet<String> = HashSet::new();
    let (tx, rx) = std::sync::mpsc::channel::<ArchivoResultado>();

    procesar_archivos(&archivos, &mapeo, &db_path, &tx);
    drop(tx);

    for res in rx {
        stats.procesados += 1;
        if res.ok {
            stats.exitosos += 1;
            stats.ventas_creadas += 1;
            stats.items_insertados += res.items;
            stats.duplicados_detectados += res.duplicados;
            stats.productos_existentes += res.existentes;
            stats.productos_nuevos += res.nuevos.len();
            for nuevo in res.nuevos {
                if nombres_nuevos_vistos.insert(nuevo.nombre.clone()) {
                    stats.productos_nuevos_lista.push(nuevo);
                }
            }
            stats.resumen_ventas.push(ResumenVenta {
                archivo: res.archivo,
                venta_id: res.venta_id,
                items: res.items,
                total: res.total,
            });
        } else {
            stats.errores += 1;
            stats.tickets_fallidos.push(TicketFallido {
                archivo: res.archivo,
                motivo: res.motivo,
            });
        }
    }

    stats.productos_nuevos_lista.truncate(100);
    stats.tickets_fallidos.truncate(500);
    stats
}

// ---------------------------------------------------------------------------
// Tests (espejo de test_lote.py, con BD SQLite temporal)
// ---------------------------------------------------------------------------

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
        let conn = Connection::open(&db).unwrap();
        conn.execute(
            "INSERT INTO productos (nombre, stock, vendido) VALUES ('TAZAS', 100, 0)",
            [],
        )
        .unwrap();
        drop(conn);

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