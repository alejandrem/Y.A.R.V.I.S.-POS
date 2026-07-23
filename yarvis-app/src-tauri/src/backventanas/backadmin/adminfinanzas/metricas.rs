use sqlx::SqlitePool;
use chrono::NaiveDate;
use crate::backventanas::backadmin::adminfinanzas::models::*;

fn decode_f64(row: &sqlx::sqlite::SqliteRow, col: &str) -> f64 {
    row.try_get::<f64, _>(col)
        .or_else(|_| row.try_get::<i64, _>(col).map(|v| v as f64))
        .unwrap_or(0.0)
}

#[tauri::command]
pub async fn get_metricas_diarias(state: tauri::State<'_, SqlitePool>, fecha_inicio: String, fecha_fin: String) -> Result<Vec<MetricasUtilidad>, String> {
    let rows = sqlx::query(
        r#"SELECT 
            v.fecha,
            COALESCE(SUM(v.total), 0) as ventas_totales,
            COALESCE(SUM(dv.cantidad * p.precio_costo), 0) as costo_ventas,
            COALESCE(SUM(pg.monto_pagado), 0) as gastos_operativos
           FROM ventas v
           LEFT JOIN detalle_ventas dv ON v.id = dv.venta_id
           LEFT JOIN productos p ON dv.producto_id = p.id
           LEFT JOIN pagos_gastos pg ON date(pg.fecha_pago) = date(v.fecha)
           WHERE date(v.fecha) BETWEEN ? AND ? AND v.estado = 'completada'
           GROUP BY date(v.fecha)
           ORDER BY date(v.fecha) ASC"#
    )
    .bind(&fecha_inicio)
    .bind(&fecha_fin)
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    let mut metricas: Vec<MetricasUtilidad> = Vec::new();
    
    for row in rows {
        let ventas_totales = decode_f64(&row, "ventas_totales");
        let costo_ventas = decode_f64(&row, "costo_ventas");
        let gastos_operativos = decode_f64(&row, "gastos_operativos");
        
        let utilidad_bruta = ventas_totales - costo_ventas;
        let utilidad_operativa = utilidad_bruta - gastos_operativos;
        let impuestos_comisiones = ventas_totales * 0.16; // IVA 16% estimado
        let utilidad_neta = utilidad_operativa - impuestos_comisiones;
        let margen_neto_pct = if ventas_totales > 0.0 { (utilidad_neta / ventas_totales) * 100.0 } else { 0.0 };

        metricas.push(MetricasUtilidad {
            fecha: row.get("fecha"),
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

#[tauri::command]
pub async fn get_resumen_periodo(state: tauri::State<'_, SqlitePool>, fecha_inicio: String, fecha_fin: String) -> Result<ResumenPeriodo, String> {
    let metricas = get_metricas_diarias(state.clone(), fecha_inicio.clone(), fecha_fin.clone()).await?;
    
    let total_ventas: f64 = metricas.iter().map(|m| m.ventas_totales).sum();
    let total_costo_ventas: f64 = metricas.iter().map(|m| m.costo_ventas).sum();
    let total_utilidad_bruta: f64 = metricas.iter().map(|m| m.utilidad_bruta).sum();
    let total_gastos_operativos: f64 = metricas.iter().map(|m| m.gastos_operativos).sum();
    let total_utilidad_operativa: f64 = metricas.iter().map(|m| m.utilidad_operativa).sum();
    let total_impuestos_comisiones: f64 = metricas.iter().map(|m| m.impuestos_comisiones).sum();
    let total_utilidad_neta: f64 = metricas.iter().map(|m| m.utilidad_neta).sum();
    let margen_promedio_pct = if total_ventas > 0.0 { (total_utilidad_neta / total_ventas) * 100.0 } else { 0.0 };

    // Calcular punto de equilibrio
    let gastos_fijos_mensuales = sqlx::query_scalar::<_, f64>(
        "SELECT COALESCE(SUM(monto_proyectado), 0) FROM gastos_recurrentes WHERE categoria IN ('servicios', 'operativo') AND frecuencia IN ('mensual', 'quincenal', 'semanal')"
    )
    .fetch_one(&*state)
    .await
    .unwrap_or(0.0);

    let margen_contribucion_pct = if total_ventas > 0.0 { ((total_ventas - total_costo_ventas) / total_ventas) * 100.0 } else { 0.0 };
    let punto_equilibrio_ventas = if margen_contribucion_pct > 0.0 { gastos_fijos_mensuales / (margen_contribucion_pct / 100.0) } else { 0.0 };

    // Tickets promedio
    let tickets_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM ventas WHERE date(fecha) BETWEEN ? AND ? AND estado = 'completada'"
    )
    .bind(&fecha_inicio)
    .bind(&fecha_fin)
    .fetch_one(&*state)
    .await
    .unwrap_or(0);

    let tickets_promedio = if tickets_count > 0 { total_ventas / tickets_count as f64 } else { 0.0 };
    let tickets_necesarios = if tickets_promedio > 0.0 { punto_equilibrio_ventas / tickets_promedio } else { 0.0 };

    Ok(ResumenPeriodo {
        periodo_inicio: fecha_inicio,
        periodo_fin: fecha_fin,
        total_ventas,
        total_costo_ventas,
        total_utilidad_bruta,
        total_gastos_operativos,
        total_utilidad_operativa,
        total_impuestos_comisiones,
        total_utilidad_neta,
        margen_promedio_pct,
        punto_equilibrio_ventas,
    })
}

#[tauri::command]
pub async fn recalcular_resumen_diario(state: tauri::State<'_, SqlitePool>, fecha: String) -> Result<(), String> {
    let ventas_totales: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(total), 0) FROM ventas WHERE date(fecha) = ? AND estado = 'completada'"
    )
    .bind(&fecha)
    .fetch_one(&*state)
    .await
    .map_err(|e| e.to_string())?;

    let costo_ventas: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(dv.cantidad * p.precio_costo), 0)
         FROM detalle_ventas dv
         JOIN ventas v ON dv.venta_id = v.id
         JOIN productos p ON dv.producto_id = p.id
         WHERE date(v.fecha) = ? AND v.estado = 'completada'"
    )
    .bind(&fecha)
    .fetch_one(&*state)
    .await
    .map_err(|e| e.to_string())?;

    let gastos_operativos: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(monto_pagado), 0) FROM pagos_gastos WHERE date(fecha_pago) = ?"
    )
    .bind(&fecha)
    .fetch_one(&*state)
    .await
    .map_err(|e| e.to_string())?;

    let utilidad_bruta = ventas_totales - costo_ventas;
    let utilidad_operativa = utilidad_bruta - gastos_operativos;
    let impuestos_comisiones = ventas_totales * 0.16;
    let utilidad_neta = utilidad_operativa - impuestos_comisiones;
    let margen_neto_pct = if ventas_totales > 0.0 { (utilidad_neta / ventas_totales) * 100.0 } else { 0.0 };

    let cortes_z_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM cortes_caja WHERE tipo_corte = 'Z' AND date(fecha_cierre) = ?"
    )
    .bind(&fecha)
    .fetch_one(&*state)
    .await
    .unwrap_or(0);

    let diferencia_caja_total: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(diferencia), 0) FROM cortes_caja WHERE date(fecha_cierre) = ?"
    )
    .bind(&fecha)
    .fetch_one(&*state)
    .await
    .unwrap_or(0.0);

    sqlx::query(
        r#"INSERT OR REPLACE INTO resumen_financiero_diario 
           (fecha, ventas_totales, costo_ventas, utilidad_bruta, gastos_operativos, 
            utilidad_operativa, impuestos_comisiones, utilidad_neta, margen_neto_pct,
            cortes_z_count, diferencia_caja_total, actualizado_en)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now','localtime'))"#
    )
    .bind(&fecha)
    .bind(ventas_totales)
    .bind(costo_ventas)
    .bind(utilidad_bruta)
    .bind(gastos_operativos)
    .bind(utilidad_operativa)
    .bind(impuestos_comisiones)
    .bind(utilidad_neta)
    .bind(margen_neto_pct)
    .bind(cortes_z_count)
    .bind(diferencia_caja_total)
    .execute(&*state)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_punto_equilibrio(state: tauri::State<'_, SqlitePool>) -> Result<PuntoEquilibrio, String> {
    let gastos_fijos_mensuales = sqlx::query_scalar::<_, f64>(
        "SELECT COALESCE(SUM(monto_proyectado), 0) FROM gastos_recurrentes WHERE frecuencia IN ('mensual', 'quincenal', 'semanal') AND estado_pago != 'pagado'"
    )
    .fetch_one(&*state)
    .await
    .map_err(|e| e.to_string())?;

    let ultimos_30_dias = (chrono::Local::now().date_naive() - chrono::Duration::days(30)).format("%Y-%m-%d").to_string();
    let hoy = chrono::Local::now().date_naive().format("%Y-%m-%d").to_string();
    
    let metricas = get_metricas_diarias(state.clone(), ultimos_30_dias, hoy).await?;
    
    let total_ventas: f64 = metricas.iter().map(|m| m.ventas_totales).sum();
    let total_costo_ventas: f64 = metricas.iter().map(|m| m.costo_ventas).sum();
    let tickets_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM ventas WHERE date(fecha) BETWEEN date('now','-30 days') AND date('now') AND estado = 'completada'"
    )
    .fetch_one(&*state)
    .await
    .unwrap_or(0);

    let margen_contribucion_pct = if total_ventas > 0.0 { ((total_ventas - total_costo_ventas) / total_ventas) * 100.0 } else { 0.0 };
    let ventas_necesarias = if margen_contribucion_pct > 0.0 { gastos_fijos_mensuales / (margen_contribucion_pct / 100.0) } else { 0.0 };
    let tickets_promedio = if tickets_count > 0 { total_ventas / tickets_count as f64 } else { 0.0 };
    let tickets_necesarios = if tickets_promedio > 0.0 { ventas_necesarias / tickets_promedio } else { 0.0 };

    Ok(PuntoEquilibrio {
        gastos_fijos_mensuales,
        margen_contribucion_pct,
        ventas_necesarias,
        tickets_promedio,
        tickets_necesarios,
    })
}