// ============================================================
// equilibrio — Punto de equilibrio (break-even) del negocio.
// ============================================================
//
// Cuánto hay que vender al mes (y en cuántos tickets) para cubrir los
// gastos, con promedios de los últimos 30 días.

use super::resumen::get_resumen_periodo;
use crate::backventanas::auth::AuthState;
use crate::backventanas::backadmin::adminfinanzas::models::*;
use sqlx::{Row, SqlitePool};

#[tauri::command]
pub async fn get_punto_equilibrio(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
) -> Result<PuntoEquilibrio, String> {
    auth.require_admin()?;
    // Últimos 30 días para calcular promedios
    let fecha_fin = chrono::Local::now()
        .date_naive()
        .format("%Y-%m-%d")
        .to_string();
    let fecha_inicio = (chrono::Local::now().date_naive() - chrono::Duration::days(30))
        .format("%Y-%m-%d")
        .to_string();

    let resumen =
        get_resumen_periodo(state.clone(), auth, fecha_inicio.clone(), fecha_fin.clone()).await?;

    // Calcular gastos fijos mensuales (promedio de últimos 30 días * 30/30)
    let gastos_fijos_mensuales = resumen.total_gastos_operativos;

    // Margen de contribución
    let margen_contribucion_pct = if resumen.total_ventas > 0.0 {
        ((resumen.total_ventas - resumen.total_costo_ventas) / resumen.total_ventas) * 100.0
    } else {
        0.0
    };

    let ventas_necesarias = if margen_contribucion_pct > 0.0 {
        gastos_fijos_mensuales / (margen_contribucion_pct / 100.0)
    } else {
        0.0
    };

    // Ticket promedio
    let row = sqlx::query("SELECT COUNT(*) as count FROM ventas WHERE date(fecha) BETWEEN ? AND ? AND estado = 'completada'")
        .bind(fecha_inicio)
        .bind(fecha_fin)
        .fetch_one(&*state)
        .await
        .map_err(|e| e.to_string())?;
    let ticket_count: i64 = row.get("count");
    let tickets_promedio = if ticket_count > 0 {
        resumen.total_ventas / ticket_count as f64
    } else {
        0.0
    };
    let tickets_necesarios = if tickets_promedio > 0.0 {
        ventas_necesarias / tickets_promedio
    } else {
        0.0
    };

    Ok(PuntoEquilibrio {
        gastos_fijos_mensuales,
        margen_contribucion_pct,
        ventas_necesarias,
        tickets_promedio,
        tickets_necesarios,
    })
}
