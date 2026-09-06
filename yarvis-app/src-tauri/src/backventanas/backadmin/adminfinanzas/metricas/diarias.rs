// ============================================================
// diarias — Métricas día por día para gráficas y tablas.
// ============================================================

use super::componentes::{
    calcular_costo_ventas, calcular_gastos_operativos, calcular_impuestos_comisiones,
    calcular_ventas_totales,
};
use crate::backventanas::auth::AuthState;
use crate::backventanas::backadmin::adminfinanzas::models::*;
use sqlx::{Row, SqlitePool};

#[tauri::command]
pub async fn get_metricas_diarias(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    fecha_inicio: String,
    fecha_fin: String,
) -> Result<Vec<MetricasUtilidad>, String> {
    auth.require_admin()?;
    let rows = sqlx::query(
        r#"SELECT DISTINCT date(fecha) as fecha FROM ventas WHERE date(fecha) BETWEEN ? AND ? AND estado = 'completada'
           UNION
           SELECT DISTINCT date(fecha_pago) as fecha FROM pagos_gastos WHERE date(fecha_pago) BETWEEN ? AND ?
           ORDER BY fecha ASC"#
    )
    .bind(&fecha_inicio)
    .bind(&fecha_fin)
    .bind(&fecha_inicio)
    .bind(&fecha_fin)
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    let mut metricas = Vec::new();
    for row in rows {
        let fecha: String = row.get("fecha");

        let ventas_totales = calcular_ventas_totales(&*state, &fecha, &fecha).await?;
        let costo_ventas = calcular_costo_ventas(&*state, &fecha, &fecha).await?;
        let utilidad_bruta = ventas_totales - costo_ventas;
        let gastos_operativos = calcular_gastos_operativos(&*state, &fecha, &fecha).await?;
        let utilidad_operativa = utilidad_bruta - gastos_operativos;
        let impuestos_comisiones = calcular_impuestos_comisiones(&*state, &fecha, &fecha).await?;
        let utilidad_neta = utilidad_operativa - impuestos_comisiones;
        let margen_neto_pct = if ventas_totales > 0.0 {
            (utilidad_neta / ventas_totales) * 100.0
        } else {
            0.0
        };

        metricas.push(MetricasUtilidad {
            fecha,
            ventas_totales,
            costo_ventas,
            utilidad_bruta,
            gastos_operativos,
            utilidad_operativa,
            impuestos_comisiones,
            utilidad_neta,
            margen_neto_pct,
        });
    }

    Ok(metricas)
}
