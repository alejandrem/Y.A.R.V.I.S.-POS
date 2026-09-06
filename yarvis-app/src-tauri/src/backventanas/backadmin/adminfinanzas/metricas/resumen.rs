// ============================================================
// resumen — Resumen del periodo y caché materializada por día.
// ============================================================
//
// `resumen_financiero_diario` es caché (centavos INTEGER): se refresca
// desde las tablas transaccionales con `recalcular_*` / `sincronizar_*`.
// La fuente de verdad siempre son ventas, detalle, productos y gastos.

use super::componentes::{
    calcular_costo_ventas, calcular_gastos_operativos, calcular_impuestos_comisiones,
    calcular_ventas_por_metodo, calcular_ventas_totales, decode_dinero,
};
use crate::backventanas::auth::AuthState;
use crate::backventanas::backadmin::adminfinanzas::models::*;
use crate::dinero::a_centavos;
use chrono::{Datelike, NaiveDate};
use sqlx::{Row, SqlitePool};

#[tauri::command]
pub async fn get_resumen_periodo(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    fecha_inicio: String,
    fecha_fin: String,
) -> Result<ResumenPeriodo, String> {
    auth.require_admin()?;
    let ventas_totales = calcular_ventas_totales(&*state, &fecha_inicio, &fecha_fin).await?;
    let costo_ventas = calcular_costo_ventas(&*state, &fecha_inicio, &fecha_fin).await?;
    let utilidad_bruta = ventas_totales - costo_ventas;
    let gastos_operativos = calcular_gastos_operativos(&*state, &fecha_inicio, &fecha_fin).await?;
    let utilidad_operativa = utilidad_bruta - gastos_operativos;
    let impuestos_comisiones =
        calcular_impuestos_comisiones(&*state, &fecha_inicio, &fecha_fin).await?;
    let utilidad_neta = utilidad_operativa - impuestos_comisiones;
    let margen_promedio_pct = if ventas_totales > 0.0 {
        (utilidad_neta / ventas_totales) * 100.0
    } else {
        0.0
    };

    // Calcular punto de equilibrio (break-even)
    // Gastos fijos mensuales promedio = gastos_operativos / días en periodo * 30
    // Fechas VALIDADAS: un input malformado del front devuelve error claro,
    // no un crash (antes había unwrap() sobre el parseo).
    let fecha_fin_parseada = NaiveDate::parse_from_str(&fecha_fin, "%Y-%m-%d")
        .map_err(|_| format!("Fecha de fin inválida: '{fecha_fin}' (se espera YYYY-MM-DD)"))?;
    let fecha_inicio_parseada = NaiveDate::parse_from_str(&fecha_inicio, "%Y-%m-%d")
        .map_err(|_| format!("Fecha de inicio inválida: '{fecha_inicio}' (se espera YYYY-MM-DD)"))?;
    let dias_periodo = (fecha_fin_parseada - fecha_inicio_parseada).num_days() as f64;
    let gastos_fijos_mensuales = if dias_periodo > 0.0 {
        (gastos_operativos / dias_periodo) * 30.0
    } else {
        0.0
    };

    // Margen de contribución = (Ventas - Costo Variable) / Ventas
    // Asumimos que costo_ventas es variable y gastos_operativos son fijos
    let margen_contribucion_pct = if ventas_totales > 0.0 {
        ((ventas_totales - costo_ventas) / ventas_totales) * 100.0
    } else {
        0.0
    };
    let punto_equilibrio_ventas = if margen_contribucion_pct > 0.0 {
        gastos_fijos_mensuales / (margen_contribucion_pct / 100.0)
    } else {
        0.0
    };

    Ok(ResumenPeriodo {
        periodo_inicio: fecha_inicio,
        periodo_fin: fecha_fin,
        total_ventas: ventas_totales,
        total_costo_ventas: costo_ventas,
        total_utilidad_bruta: utilidad_bruta,
        total_gastos_operativos: gastos_operativos,
        total_utilidad_operativa: utilidad_operativa,
        total_impuestos_comisiones: impuestos_comisiones,
        total_utilidad_neta: utilidad_neta,
        margen_promedio_pct,
        punto_equilibrio_ventas,
    })
}

#[tauri::command]
pub async fn recalcular_resumen_diario(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    fecha: String,
) -> Result<(), String> {
    auth.require_admin()?;
    recalcular_resumen_diario_impl(&*state, &fecha).await
}

/// Refresca una fecha de la caché materializada desde las tablas transaccionales.
pub(crate) async fn recalcular_resumen_diario_impl(
    state: &SqlitePool,
    fecha: &str,
) -> Result<(), String> {
    let ventas_totales = calcular_ventas_totales(&*state, &fecha, &fecha).await?;
    let (ventas_efectivo, ventas_tarjeta, ventas_transferencia) =
        calcular_ventas_por_metodo(&*state, &fecha, &fecha).await?;
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

    // Cortes Z del día
    let row = sqlx::query("SELECT COUNT(*) as count, COALESCE(SUM(diferencia), 0) as diff FROM cortes_caja WHERE tipo_corte = 'Z' AND date(fecha_cierre) = ? AND estado = 'cerrado'")
        .bind(&fecha)
        .fetch_one(&*state)
        .await
        .map_err(|e| e.to_string())?;
    let cortes_z_count: i64 = row.get("count");
    let diferencia_caja_total = decode_dinero(&row, "diff");

    sqlx::query(
        r#"INSERT INTO resumen_financiero_diario (fecha, ventas_totales, ventas_efectivo, ventas_tarjeta, ventas_transferencia, costo_ventas, utilidad_bruta, gastos_operativos, utilidad_operativa, impuestos_comisiones, utilidad_neta, margen_neto_pct, cortes_z_count, diferencia_caja_total, actualizado_en)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now','localtime'))
           ON CONFLICT(fecha) DO UPDATE SET
               ventas_totales = excluded.ventas_totales,
               ventas_efectivo = excluded.ventas_efectivo,
               ventas_tarjeta = excluded.ventas_tarjeta,
               ventas_transferencia = excluded.ventas_transferencia,
               costo_ventas = excluded.costo_ventas,
               utilidad_bruta = excluded.utilidad_bruta,
               gastos_operativos = excluded.gastos_operativos,
               utilidad_operativa = excluded.utilidad_operativa,
               impuestos_comisiones = excluded.impuestos_comisiones,
               utilidad_neta = excluded.utilidad_neta,
               margen_neto_pct = excluded.margen_neto_pct,
               cortes_z_count = excluded.cortes_z_count,
               diferencia_caja_total = excluded.diferencia_caja_total,
               actualizado_en = excluded.actualizado_en"#
    )
    .bind(&fecha)
    // La caché materializada también vive en centavos (INTEGER);
    // margen_neto_pct es porcentaje y sigue siendo REAL.
    .bind(a_centavos(ventas_totales))
    .bind(a_centavos(ventas_efectivo))
    .bind(a_centavos(ventas_tarjeta))
    .bind(a_centavos(ventas_transferencia))
    .bind(a_centavos(costo_ventas))
    .bind(a_centavos(utilidad_bruta))
    .bind(a_centavos(gastos_operativos))
    .bind(a_centavos(utilidad_operativa))
    .bind(a_centavos(impuestos_comisiones))
    .bind(a_centavos(utilidad_neta))
    .bind(margen_neto_pct)
    .bind(cortes_z_count)
    .bind(a_centavos(diferencia_caja_total))
    .execute(&*state)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// Mantiene la caché del mes actual. Se ejecuta al iniciar el job de alertas y
/// cada hora, por lo que ya no depende de que alguien invoque manualmente el
/// comando `recalcular_resumen_diario`.
pub(crate) async fn sincronizar_resumen_mes_actual(state: &SqlitePool) -> Result<(), String> {
    let hoy = chrono::Local::now().date_naive();
    let inicio_mes = NaiveDate::from_ymd_opt(hoy.year(), hoy.month(), 1)
        .ok_or_else(|| "No se pudo calcular el inicio del mes actual".to_string())?;
    let inicio = inicio_mes.format("%Y-%m-%d").to_string();
    let fin = hoy.format("%Y-%m-%d").to_string();

    let rows = sqlx::query(
        r#"SELECT fecha FROM (
               SELECT date(fecha) AS fecha FROM ventas
               WHERE date(fecha) BETWEEN ? AND ?
               UNION
               SELECT date(fecha_pago) AS fecha FROM pagos_gastos
               WHERE date(fecha_pago) BETWEEN ? AND ?
               UNION
               SELECT date(fecha_cierre) AS fecha FROM cortes_caja
               WHERE fecha_cierre IS NOT NULL AND date(fecha_cierre) BETWEEN ? AND ?
           )
           ORDER BY fecha ASC"#,
    )
    .bind(&inicio)
    .bind(&fin)
    .bind(&inicio)
    .bind(&fin)
    .bind(&inicio)
    .bind(&fin)
    .fetch_all(state)
    .await
    .map_err(|e| e.to_string())?;

    for row in rows {
        let fecha: String = row.get("fecha");
        recalcular_resumen_diario_impl(state, &fecha).await?;
    }

    Ok(())
}
