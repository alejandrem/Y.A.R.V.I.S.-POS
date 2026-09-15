// ============================================================
// comun — Piezas compartidas de cortes X/Z: ancla de turno y
// consultas por ventana. Sin comandos: solo helpers.
//
// Ventana del turno: [ancla, fin] filtrada por cajero_id.
// ancla = fecha_cierre del último Z cerrado del empleado;
// si nunca hubo Z → primer_login de hoy (asistencias);
// si tampoco → inicio del día.
// ============================================================

use crate::dinero::{a_pesos, centavos_f64_a_i64};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

/// Fecha-hora actual en formato SQLite local.
pub fn ahora_str() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Ancla del turno del empleado (ver doc del módulo).
pub async fn ancla_turno(pool: &SqlitePool, usuario_id: i64) -> Result<String, String> {
    if let Some((cierre,)) = sqlx::query_as::<_, (String,)>(
        "SELECT fecha_cierre FROM cortes_caja
          WHERE usuario_id = ? AND tipo_corte = 'Z'
            AND estado = 'cerrado' AND fecha_cierre IS NOT NULL
          ORDER BY fecha_cierre DESC LIMIT 1",
    )
    .bind(usuario_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    {
        return Ok(cierre);
    }
    if let Some((primer,)) = sqlx::query_as::<_, (String,)>(
        "SELECT primer_login FROM asistencias
          WHERE empleado_id = ? AND fecha = date('now','localtime')",
    )
    .bind(usuario_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    {
        return Ok(primer);
    }
    let row: (String,) = sqlx::query_as("SELECT date('now','localtime') || ' 00:00:00'")
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(row.0)
}

/// Decodifica una columna monetaria (INTEGER centavos; tolera REAL
/// en DBs viejas) y la devuelve en pesos.
pub fn decode_monto(row: &sqlx::sqlite::SqliteRow, col: &str) -> f64 {
    if let Ok(c) = row.try_get::<i64, _>(col) {
        return a_pesos(c);
    }
    if let Ok(f) = row.try_get::<f64, _>(col) {
        return a_pesos(centavos_f64_a_i64(f));
    }
    0.0
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TicketResumen {
    pub venta_id: i64,
    pub folio: String,
    pub fecha: String,
    pub total: f64,
    pub metodo_pago: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProductoAgregado {
    pub producto_nombre: String,
    pub cantidad: f64,
    pub monto: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TotalesVentana {
    pub total_ventas: f64,
    pub total_efectivo: f64,
    pub total_tarjeta: f64,
    pub total_transferencia: f64,
    pub num_tickets: i64,
}

/// Tickets del empleado en [ancla, fin].
/// (Corte X: se visualiza folio + total por ticket.)
pub async fn tickets_en_ventana(
    pool: &SqlitePool,
    usuario_id: i64,
    ancla: &str,
    fin: &str,
) -> Result<Vec<TicketResumen>, String> {
    let filas = sqlx::query(
        "SELECT id, folio_ticket, fecha, total, metodo_pago FROM ventas
          WHERE cajero_id = ?
            AND datetime(fecha) BETWEEN datetime(?) AND datetime(?)
            AND estado = 'completada'
          ORDER BY fecha ASC",
    )
    .bind(usuario_id)
    .bind(ancla)
    .bind(fin)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(filas
        .into_iter()
        .map(|f| {
            let id: i64 = f.get("id");
            let folio: Option<String> = f.try_get("folio_ticket").ok().flatten();
            TicketResumen {
                venta_id: id,
                folio: folio.unwrap_or_else(|| format!("REM-{id}")),
                fecha: f.get("fecha"),
                total: decode_monto(&f, "total"),
                metodo_pago: f
                    .try_get("metodo_pago")
                    .unwrap_or_else(|_| "efectivo".into()),
            }
        })
        .collect())
}

/// Totales por método con las columnas monto_* (no el string
/// metodo_pago): los mixtos ("efectivo/tarjeta", "mixto", ...)
/// ya no se pierden como pasaba agrupando por metodo_pago.
pub async fn totales_en_ventana(
    pool: &SqlitePool,
    usuario_id: i64,
    ancla: &str,
    fin: &str,
) -> Result<TotalesVentana, String> {
    let row = sqlx::query(
        "SELECT COALESCE(SUM(total),0) AS t,
                COALESCE(SUM(monto_efectivo),0) AS e,
                COALESCE(SUM(monto_tarjeta),0) AS tj,
                COALESCE(SUM(monto_transferencia),0) AS tr,
                COUNT(*) AS n
          FROM ventas
          WHERE cajero_id = ?
            AND datetime(fecha) BETWEEN datetime(?) AND datetime(?)
            AND estado = 'completada'",
    )
    .bind(usuario_id)
    .bind(ancla)
    .bind(fin)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(TotalesVentana {
        total_ventas: decode_monto(&row, "t"),
        total_efectivo: decode_monto(&row, "e"),
        total_tarjeta: decode_monto(&row, "tj"),
        total_transferencia: decode_monto(&row, "tr"),
        num_tickets: row.try_get("n").unwrap_or(0),
    })
}

/// Productos agregados del empleado en [ancla, fin].
/// (Corte Z: nombre + cantidad + monto; concuerda con
/// detalle_ventas y por tanto con los tickets.)
pub async fn productos_en_ventana(
    pool: &SqlitePool,
    usuario_id: i64,
    ancla: &str,
    fin: &str,
) -> Result<Vec<ProductoAgregado>, String> {
    let filas = sqlx::query(
        "SELECT d.producto_nombre AS nombre,
                COALESCE(SUM(d.cantidad),0) AS cant,
                COALESCE(SUM(d.subtotal),0) AS monto
           FROM detalle_ventas d
           JOIN ventas v ON v.id = d.venta_id
          WHERE v.cajero_id = ?
            AND datetime(v.fecha) BETWEEN datetime(?) AND datetime(?)
            AND v.estado = 'completada'
          GROUP BY d.producto_nombre
          ORDER BY monto DESC",
    )
    .bind(usuario_id)
    .bind(ancla)
    .bind(fin)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(filas
        .into_iter()
        .map(|f| ProductoAgregado {
            producto_nombre: f.get("nombre"),
            cantidad: f.try_get("cant").unwrap_or(0.0),
            monto: decode_monto(&f, "monto"),
        })
        .collect())
}
