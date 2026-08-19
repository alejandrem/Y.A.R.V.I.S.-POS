use rusqlite::{params, Connection};
use std::collections::HashSet;

use super::items::round2;
use crate::cerebro::analizador_tickets::Item;

/// Asegura la columna `folio_ticket` en `ventas` (las DBs viejas creadas con
/// `CREATE TABLE IF NOT EXISTS` no se migran solas). Idempotente vía PRAGMA.
pub(super) fn garantizar_columna_folio(conn: &Connection) {
    let ok = match conn.prepare("PRAGMA table_info(ventas)") {
        Ok(mut stmt) => stmt
            .query_map([], |r| r.get::<_, String>(1))
            .map(|rows| {
                rows.flatten()
                    .any(|c| c.eq_ignore_ascii_case("folio_ticket"))
            })
            .unwrap_or(false),
        Err(_) => false,
    };
    if !ok {
        let _ = conn.execute_batch("ALTER TABLE ventas ADD COLUMN folio_ticket TEXT");
    }
}

/// Precarga el set de claves `NOMBRE|precio` ya existentes en `productos`.
pub(super) fn cargar_estado(db_path: &str) -> HashSet<String> {
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

/// Fecha/hora UTC actual "YYYY-MM-DD HH:MM:SS" (mismo formato que el
/// `CURRENT_TIMESTAMP` de SQLite). Fallback para tickets sin fecha: así la
/// venta no se pierde de ventas_diarias / conteos por día.
fn ahora_iso_utc() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let dias = (secs / 86400) as i64;
    let resto = secs % 86400;

    // Algoritmo civil_from_days (Howard Hinnant), sin dependencias.
    let z = dias + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let anio = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let dia = doy - (153 * mp + 2) / 5 + 1;
    let mes = if mp < 10 { mp + 3 } else { mp - 9 };
    let anio = if mes <= 2 { anio + 1 } else { anio };

    let hora = resto / 3600;
    let min = (resto % 3600) / 60;
    let seg = resto % 60;
    format!("{anio:04}-{mes:02}-{dia:02} {hora:02}:{min:02}:{seg:02}")
}

/// Inserta una venta con sus totales YA resueltos (reales del ticket o
/// cálculo) y el folio del ticket si se detectó. Requiere que el llamador
/// haya corrido [`garantizar_columna_folio`] al abrir la conexión.
#[allow(clippy::too_many_arguments)]
pub(super) fn insertar_venta(
    conn: &Connection,
    items: &[Item],
    cajero: &str,
    fecha_iso: Option<&str>,
    metodo_pago: &str,
    folio: Option<&str>,
    subtotal: f64,
    iva: f64,
    total: f64,
) -> Result<i64, rusqlite::Error> {
    conn.execute(
        "INSERT INTO ventas (total, subtotal, iva, cajero, metodo_pago, estado, fecha, folio_ticket)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            total,
            subtotal,
            iva,
            cajero,
            metodo_pago,
            "completada",
            // Ticket sin fecha → ahora (UTC), no NULL (se perdería del conteo).
            fecha_iso.map(str::to_owned).unwrap_or_else(ahora_iso_utc),
            folio
        ],
    )?;
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
