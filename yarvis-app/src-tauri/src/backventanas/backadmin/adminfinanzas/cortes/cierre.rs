// ============================================================
// cierre — Cierre de cortes con recálculo antisabotaje.
// ============================================================
//
// Los totales que envía el cliente se IGNORAN: el servidor recalcula TODO
// desde la tabla `ventas` en transacción, así la "diferencia de caja" no
// es manipulable desde el frontend. Solo cierra cortes ABIERTOS (guard
// contra dobles cierres) y solo acepta entradas/retiros manuales.

use crate::backventanas::auth::AuthState;
use crate::dinero::{a_centavos, a_pesos};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::SqlitePool;

/// Resumen REAL calculado en el servidor al cerrar un corte.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CierreCorte {
    pub total_ventas: f64,
    pub total_efectivo: f64,
    pub total_tarjeta: f64,
    pub total_transferencia: f64,
    pub diferencia: f64,
}

#[tauri::command]
pub async fn cerrar_corte(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    corte_id: i64,
    total_ventas: f64,
    total_efectivo: f64,
    total_tarjeta: f64,
    total_transferencia: f64,
    entradas_manuales: f64,
    retiros_manuales: f64,
) -> Result<CierreCorte, String> {
    auth.require_admin()?;
    // Los totales de venta/métodos que envía el cliente se IGNORAN: si se
    // persistieran tal cual, la "diferencia de caja" sería manipulable desde
    // el frontend. El servidor recalcula TODO desde la tabla ventas.
    // (Los parámetros se conservan por compatibilidad del contrato IPC.)
    let _ = (total_ventas, total_efectivo, total_tarjeta, total_transferencia);

    cerrar_corte_impl(&*state, corte_id, entradas_manuales, retiros_manuales).await
}

/// Núcleo de cierre de corte, testeable sin runtime de Tauri.
///
/// Recalcula ventas totales y por método desde `ventas` dentro de la ventana
/// [fecha_apertura, fecha_cierre] del corte, en una transacción. Solo cierra
/// cortes ABIERTOS (guard contra dobles cierres). Los únicos valores del
/// usuario que se aceptan son entradas/retiros manuales, porque eso es
/// exactamente lo que solo él conoce.
pub async fn cerrar_corte_impl(
    pool: &SqlitePool,
    corte_id: i64,
    entradas_manuales: f64,
    retiros_manuales: f64,
) -> Result<CierreCorte, String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    // El corte debe existir y estar abierto.
    let corte_row = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT fecha_apertura, fecha_cierre FROM cortes_caja WHERE id = ? AND estado = 'abierto'",
    )
    .bind(corte_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "Corte no encontrado o ya cerrado".to_string())?;

    let (apertura, cierre_previo) = corte_row;
    let cierre = cierre_previo.unwrap_or_else(|| {
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
    });

    // Recalcular ventas de la ventana, agrupadas por método (centavos).
    let filas = sqlx::query(
        "SELECT metodo_pago, COALESCE(SUM(total), 0) as total
         FROM ventas
         WHERE datetime(fecha) BETWEEN datetime(?) AND datetime(?)
           AND estado = 'completada'
         GROUP BY metodo_pago",
    )
    .bind(&apertura)
    .bind(&cierre)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    let mut efectivo_c: i64 = 0;
    let mut tarjeta_c: i64 = 0;
    let mut transferencia_c: i64 = 0;
    let mut total_ventas_c: i64 = 0;
    for fila in filas {
        let metodo: String = fila.get("metodo_pago");
        let total: i64 = fila.get("total");
        total_ventas_c += total;
        match metodo.as_str() {
            "efectivo" => efectivo_c += total,
            "tarjeta" => tarjeta_c += total,
            "transferencia" => transferencia_c += total,
            _ => {}
        }
    }

    // Diferencia EXACTA en centavos: lo contado (+entradas −retiros) vs lo vendido.
    let entradas_c = a_centavos(entradas_manuales);
    let retiros_c = a_centavos(retiros_manuales);
    let total_calculado_c = efectivo_c + tarjeta_c + transferencia_c + entradas_c - retiros_c;
    let diferencia_c = total_calculado_c - total_ventas_c;

    let result = sqlx::query(
        r#"UPDATE cortes_caja SET
           fecha_cierre = ?,
           total_ventas = ?,
           total_efectivo = ?,
           total_tarjeta = ?,
           total_transferencia = ?,
           entradas_manuales = ?,
           retiros_manuales = ?,
           diferencia = ?,
           estado = 'cerrado'
           WHERE id = ? AND estado = 'abierto'"#,
    )
    .bind(&cierre)
    .bind(total_ventas_c)
    .bind(efectivo_c)
    .bind(tarjeta_c)
    .bind(transferencia_c)
    .bind(entradas_c)
    .bind(retiros_c)
    .bind(diferencia_c)
    .bind(corte_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    if result.rows_affected() == 0 {
        return Err("Corte no encontrado o ya cerrado".into());
    }

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(CierreCorte {
        total_ventas: a_pesos(total_ventas_c),
        total_efectivo: a_pesos(efectivo_c),
        total_tarjeta: a_pesos(tarjeta_c),
        total_transferencia: a_pesos(transferencia_c),
        diferencia: a_pesos(diferencia_c),
    })
}
