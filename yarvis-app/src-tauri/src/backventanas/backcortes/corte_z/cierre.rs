// ============================================================
// cierre — Cierre Z del turno del empleado.
// ============================================================

use super::super::comun::{
    ahora_str, ancla_turno, productos_en_ventana, tickets_en_ventana, totales_en_ventana,
    ProductoAgregado, TicketResumen, TotalesVentana,
};
use crate::backventanas::auth::AuthState;
use crate::dinero::a_centavos;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

/// Reporte Z listo para mostrar e imprimir.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CorteZReporte {
    pub corte_id: i64,
    pub cajero_id: i64,
    pub ancla: String,
    pub cierre: String,
    pub tickets: Vec<TicketResumen>,
    pub productos: Vec<ProductoAgregado>,
    pub totales: TotalesVentana,
}

/// Núcleo testeable sin runtime de Tauri: calcula la ventana desde
/// el ancla, guarda el Z cerrado (nuevo ancla para el próximo turno)
/// y lo devuelve. Si no hubo ventas en el turno, el Z se guarda
/// con totales en 0 (queda constancia del cierre).
pub async fn corte_z_cierre_impl(
    pool: &SqlitePool,
    cajero_id: i64,
    observaciones: Option<String>,
) -> Result<CorteZReporte, String> {
    let cierre = ahora_str();
    let ancla = ancla_turno(pool, cajero_id).await?;
    let tickets = tickets_en_ventana(pool, cajero_id, &ancla, &cierre).await?;
    let productos = productos_en_ventana(pool, cajero_id, &ancla, &cierre).await?;
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
           VALUES (?, ?, 0, 'Z', ?, ?, 'cerrado', ?, ?, ?, ?, 0, 0, 0)"#,
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

    // #26 — El Z cierra el turno automáticamente: se estampa la salida
    // en asistencias (último registro de actividad del día). El próximo
    // conteo parte de este cierre (ver `comun::ancla_turno`).
    let salida = sqlx::query(
        "UPDATE asistencias SET ultimo_login = ?
          WHERE empleado_id = ? AND fecha = date(?,'localtime')",
    )
    .bind(&cierre)
    .bind(cajero_id)
    .bind(&cierre)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    if salida.rows_affected() == 0 {
        sqlx::query(
            "INSERT INTO asistencias (empleado_id, fecha, primer_login, ultimo_login)
             VALUES (?, date(?,'localtime'), ?, ?)",
        )
        .bind(cajero_id)
        .bind(&cierre)
        .bind(&cierre)
        .bind(&cierre)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(CorteZReporte {
        corte_id,
        cajero_id,
        ancla,
        cierre,
        tickets,
        productos,
        totales,
    })
}

/// Comando Tauri: el empleado cierra SU propio turno (o el admin el
/// suyo). El cajero es siempre el de la sesión.
#[tauri::command]
pub async fn corte_z_cierre(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    observaciones: Option<String>,
) -> Result<CorteZReporte, String> {
    let session = auth.require_operator()?;
    corte_z_cierre_impl(&*state, session.user_id, observaciones).await
}
