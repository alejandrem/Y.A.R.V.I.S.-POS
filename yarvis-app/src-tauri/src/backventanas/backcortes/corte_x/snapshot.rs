// ============================================================
// snapshot — Foto del turno sin efectos secundarios.
// ============================================================

use super::super::comun::{
    ahora_str, ancla_turno, tickets_en_ventana, totales_en_ventana, TicketResumen, TotalesVentana,
};
use crate::backventanas::auth::AuthState;
use crate::dinero::a_centavos;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

/// Reporte X listo para mostrar e imprimir.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CorteXReporte {
    pub corte_id: i64,
    pub cajero_id: i64,
    pub ancla: String,
    pub cierre: String,
    pub tickets: Vec<TicketResumen>,
    pub totales: TotalesVentana,
}

/// Núcleo testeable sin runtime de Tauri: calcula la ventana,
/// guarda la foto como corte X cerrado y la devuelve.
pub async fn corte_x_snapshot_impl(
    pool: &SqlitePool,
    cajero_id: i64,
    observaciones: Option<String>,
) -> Result<CorteXReporte, String> {
    let cierre = ahora_str();
    let ancla = ancla_turno(pool, cajero_id).await?;
    let tickets = tickets_en_ventana(pool, cajero_id, &ancla, &cierre).await?;
    let totales = totales_en_ventana(pool, cajero_id, &ancla, &cierre).await?;

    let total_c = a_centavos(totales.total_ventas);
    let ef_c = a_centavos(totales.total_efectivo);
    let tj_c = a_centavos(totales.total_tarjeta);
    let tr_c = a_centavos(totales.total_transferencia);

    let corte_id = sqlx::query(
        r#"INSERT INTO cortes_caja
           (fecha_apertura, fecha_cierre, monto_inicial, tipo_corte,
            observaciones, usuario_id, estado,
            total_ventas, total_efectivo, total_tarjeta, total_transferencia,
            entradas_manuales, retiros_manuales, diferencia)
           VALUES (?, ?, 0, 'X', ?, ?, 'cerrado', ?, ?, ?, ?, 0, 0, 0)"#,
    )
    .bind(&ancla)
    .bind(&cierre)
    .bind(&observaciones)
    .bind(cajero_id)
    .bind(total_c)
    .bind(ef_c)
    .bind(tj_c)
    .bind(tr_c)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?
    .last_insert_rowid();

    Ok(CorteXReporte {
        corte_id,
        cajero_id,
        ancla,
        cierre,
        tickets,
        totales,
    })
}

/// Comando Tauri: cualquier operador (empleado o admin) puede pedir
/// su propio X cuando quiera. El cajero es siempre el de la sesión:
/// nadie puede pedir el X de otro empleado.
#[tauri::command]
pub async fn corte_x_reporte(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    observaciones: Option<String>,
) -> Result<CorteXReporte, String> {
    let session = auth.require_operator()?;
    corte_x_snapshot_impl(&*state, session.user_id, observaciones).await
}
