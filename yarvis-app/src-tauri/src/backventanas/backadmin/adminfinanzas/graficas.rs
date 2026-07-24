use std::sync::Arc;
use sqlx::SqlitePool;
use chrono::Duration;
use crate::backventanas::backadmin::adminfinanzas::models::*;
use crate::sidecar::{AiSidecar, AiStatus};
use crate::backventanas::db::db::DbPath;
use sqlx::Row;

fn decode_f64(row: &sqlx::sqlite::SqliteRow, col: &str) -> f64 {
    row.try_get::<f64, _>(col)
        .or_else(|_| row.try_get::<i64, _>(col).map(|v| v as f64))
        .unwrap_or(0.0)
}

#[tauri::command]
pub async fn get_datos_grafica_pl(state: tauri::State<'_, SqlitePool>, fecha_inicio: String, fecha_fin: String, granularidad: String) -> Result<Vec<DatoGraficaPL>, String> {
    let group_by = match granularidad.as_str() {
        "dia" => "date(fecha)",
        "semana" => "strftime('%Y-%W', fecha)",
        "mes" => "strftime('%Y-%m', fecha)",
        _ => "date(fecha)",
    };

    let rows = sqlx::query(&format!(
        r#"SELECT 
            {} as periodo,
            COALESCE(SUM(CASE WHEN tipo = 'ingreso' THEN monto ELSE 0 END), 0) as ingresos,
            COALESCE(SUM(CASE WHEN tipo = 'gasto' THEN monto ELSE 0 END), 0) as gastos
           FROM (
               SELECT fecha, total as monto, 'ingreso' as tipo FROM ventas WHERE date(fecha) BETWEEN ? AND ? AND estado = 'completada'
               UNION ALL
               SELECT fecha_pago as fecha, monto_pagado as monto, 'gasto' as tipo FROM pagos_gastos WHERE date(fecha_pago) BETWEEN ? AND ?
           )
           GROUP BY {}
           ORDER BY periodo ASC"#, group_by, group_by
    ))
    .bind(&fecha_inicio)
    .bind(&fecha_fin)
    .bind(&fecha_inicio)
    .bind(&fecha_fin)
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    let mut datos: Vec<DatoGraficaPL> = Vec::new();
    for row in rows {
        let ingresos = decode_f64(&row, "ingresos");
        let gastos = decode_f64(&row, "gastos");
        datos.push(DatoGraficaPL {
            fecha: row.get("periodo"),
            ingresos,
            gastos,
            utilidad_neta: ingresos - gastos,
        });
    }

    Ok(datos)
}

#[tauri::command]
pub async fn get_gastos_por_categoria(state: tauri::State<'_, SqlitePool>, fecha_inicio: String, fecha_fin: String) -> Result<Vec<DatoGraficaGastosCategoria>, String> {
    let rows = sqlx::query(
        r#"SELECT gr.categoria, COALESCE(SUM(pg.monto_pagado), 0) as total
           FROM pagos_gastos pg
           JOIN gastos_recurrentes gr ON pg.gasto_id = gr.id
           WHERE date(pg.fecha_pago) BETWEEN ? AND ?
           GROUP BY gr.categoria
           ORDER BY total DESC"#
    )
    .bind(&fecha_inicio)
    .bind(&fecha_fin)
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    let total_general: f64 = rows.iter().map(|r| decode_f64(r, "total")).sum();
    
    Ok(rows.into_iter().map(|row| {
        let monto = decode_f64(&row, "total");
        DatoGraficaGastosCategoria {
            categoria: row.get("categoria"),
            monto,
            porcentaje: if total_general > 0.0 { (monto / total_general) * 100.0 } else { 0.0 },
        }
    }).collect())
}

#[tauri::command]
pub async fn get_tendencia_cortes_z(state: tauri::State<'_, SqlitePool>, fecha_inicio: String, fecha_fin: String) -> Result<Vec<DatoGraficaCortesZ>, String> {
    let rows = sqlx::query(
        r#"SELECT 
            date(c.fecha_cierre) as fecha,
            c.turno,
            COALESCE(c.total_ventas, 0) as total_ventas,
            COALESCE(c.diferencia, 0) as diferencia,
            u.nombre as cajero
           FROM cortes_caja c
           LEFT JOIN usuarios u ON c.usuario_id = u.id
           WHERE c.tipo_corte = 'Z' AND c.estado = 'cerrado'
           AND date(c.fecha_cierre) BETWEEN ? AND ?
           ORDER BY date(c.fecha_cierre) ASC, c.turno"#
    )
    .bind(&fecha_inicio)
    .bind(&fecha_fin)
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|row| DatoGraficaCortesZ {
        fecha: row.get("fecha"),
        turno: row.try_get("turno").unwrap_or_default(),
        total_ventas: decode_f64(&row, "total_ventas"),
        diferencia: decode_f64(&row, "diferencia"),
        cajero: row.get("cajero"),
    }).collect())
}

#[tauri::command]
pub async fn get_ventas_vs_gastos_mensual(state: tauri::State<'_, SqlitePool>, meses: i32) -> Result<Vec<DatoGraficaPL>, String> {
    let fecha_inicio = (chrono::Local::now().date_naive() - Duration::days((meses * 30) as i64)).format("%Y-%m-%d").to_string();
    let fecha_fin = chrono::Local::now().date_naive().format("%Y-%m-%d").to_string();

    let rows = sqlx::query(
        r#"SELECT 
            strftime('%Y-%m', fecha) as mes,
            COALESCE(SUM(CASE WHEN tipo = 'venta' THEN monto ELSE 0 END), 0) as ingresos,
            COALESCE(SUM(CASE WHEN tipo = 'gasto' THEN monto ELSE 0 END), 0) as gastos
           FROM (
               SELECT fecha, total as monto, 'venta' as tipo FROM ventas WHERE date(fecha) BETWEEN ? AND ? AND estado = 'completada'
               UNION ALL
               SELECT fecha_pago as fecha, monto_pagado as monto, 'gasto' as tipo FROM pagos_gastos WHERE date(fecha_pago) BETWEEN ? AND ?
           )
           GROUP BY strftime('%Y-%m', fecha)
           ORDER BY mes ASC"#
    )
    .bind(&fecha_inicio)
    .bind(&fecha_fin)
    .bind(&fecha_inicio)
    .bind(&fecha_fin)
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|row| DatoGraficaPL {
        fecha: row.get("mes"),
        ingresos: decode_f64(&row, "ingresos"),
        gastos: decode_f64(&row, "gastos"),
        utilidad_neta: decode_f64(&row, "ingresos") - decode_f64(&row, "gastos"),
    }).collect())
}

#[tauri::command]
pub async fn get_predicciones_financieras(
    days: i32,
    sidecar: tauri::State<'_, Arc<AiSidecar>>,
    db_path: tauri::State<'_, DbPath>,
) -> Result<serde_json::Value, String> {
    sidecar.check_process_alive();
    if sidecar.get_status() != AiStatus::Ready {
        return Err("Motor de IA no está listo".into());
    }

    let base_url = sidecar.base_url().ok_or("Sidecar sin puerto")?;
    let payload = serde_json::json!({ "db_path": db_path.0, "days": days });

    let resp = sidecar
        .http_client
        .post(format!("{}/recalcular_predicciones", base_url))
        .json(&payload)
        .timeout(std::time::Duration::from_secs(300))
        .send()
        .await
        .map_err(|e| format!("Error llamando a Prophet: {}", e))?;

    let result: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Error decodificando respuesta Prophet: {}", e))?;

    Ok(result)
}