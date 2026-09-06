// ============================================================
// movimientos — Entradas/retiros manuales de caja y consultas por cajero.
// ============================================================

use super::lecturas::decode_dinero;
use crate::backventanas::auth::AuthState;
use crate::backventanas::backadmin::adminfinanzas::models::*;
use crate::dinero::{a_centavos, a_pesos};
use sqlx::Row;
use sqlx::SqlitePool;

#[tauri::command]
pub async fn agregar_movimiento_caja(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    mov: MovimientoCajaRequest,
) -> Result<i64, String> {
    auth.require_admin()?;
    let result = sqlx::query(
        r#"INSERT INTO movimientos_caja (corte_id, tipo, concepto, monto, metodo_pago, referencia_id)
           VALUES (?, ?, ?, ?, ?, ?)"#
    )
    .bind(mov.corte_id)
    .bind(&mov.tipo)
    .bind(&mov.concepto)
    .bind(a_centavos(mov.monto))
    .bind(&mov.metodo_pago)
    .bind(mov.referencia_id)
    .execute(&*state)
    .await
    .map_err(|e| e.to_string())?;

    Ok(result.last_insert_rowid())
}

#[tauri::command]
pub async fn get_movimientos_corte(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    corte_id: i64,
) -> Result<Vec<MovimientoCaja>, String> {
    auth.require_admin()?;
    let rows = sqlx::query_as::<_, (i64, i64, String, String, i64, Option<String>, Option<i64>, String)>(
        "SELECT id, corte_id, tipo, concepto, monto, metodo_pago, referencia_id, creado_en FROM movimientos_caja WHERE corte_id = ? ORDER BY creado_en ASC"
    )
    .bind(corte_id)
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|row| MovimientoCaja {
            id: row.0,
            corte_id: row.1,
            tipo: row.2,
            concepto: row.3,
            monto: a_pesos(row.4),
            metodo_pago: row.5,
            referencia_id: row.6,
            creado_en: row.7,
        })
        .collect())
}

#[tauri::command]
pub async fn get_cortes_por_cajero_fecha(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    cajero_id: i64,
    fecha_inicio: String,
    fecha_fin: String,
) -> Result<Vec<CorteCaja>, String> {
    auth.require_admin()?;
    let rows = sqlx::query(
        r#"SELECT c.*, u.nombre as usuario_nombre 
           FROM cortes_caja c
           LEFT JOIN usuarios u ON c.usuario_id = u.id
           WHERE c.usuario_id = ? AND date(c.fecha_apertura) BETWEEN ? AND ?
           ORDER BY c.fecha_apertura DESC"#,
    )
    .bind(cajero_id)
    .bind(fecha_inicio)
    .bind(fecha_fin)
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|row| CorteCaja {
            id: row.get("id"),
            fecha_apertura: row.get("fecha_apertura"),
            fecha_cierre: row.try_get("fecha_cierre").ok(),
            monto_inicial: decode_dinero(&row, "monto_inicial"),
            total_ventas: decode_dinero(&row, "total_ventas"),
            total_efectivo: decode_dinero(&row, "total_efectivo"),
            total_tarjeta: decode_dinero(&row, "total_tarjeta"),
            total_transferencia: decode_dinero(&row, "total_transferencia"),
            entradas_manuales: decode_dinero(&row, "entradas_manuales"),
            retiros_manuales: decode_dinero(&row, "retiros_manuales"),
            diferencia: decode_dinero(&row, "diferencia"),
            usuario_id: row.get("usuario_id"),
            usuario_nombre: row.get("usuario_nombre"),
            estado: row.get("estado"),
            tipo_corte: row.get("tipo_corte"),
            turno: row.try_get("turno").ok(),
            observaciones: row.try_get("observaciones").ok(),
        })
        .collect())
}
