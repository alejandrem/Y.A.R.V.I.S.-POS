use sqlx::SqlitePool;
use chrono::{NaiveDate, Duration, Datelike, Weekday};
use crate::backventanas::backadmin::adminfinanzas::models::*;

fn decode_f64(row: &sqlx::sqlite::SqliteRow, col: &str) -> f64 {
    row.try_get::<f64, _>(col)
        .or_else(|_| row.try_get::<i64, _>(col).map(|v| v as f64))
        .unwrap_or(0.0)
}

fn calcular_proxima_fecha(fecha_inicio: &str, frecuencia: &str, dia_pago: Option<i32>, intervalo_dias: Option<i32>, desde: NaiveDate) -> Option<String> {
    let inicio = NaiveDate::parse_from_str(fecha_inicio, "%Y-%m-%d").ok()?;
    
    match frecuencia {
        "semanal" => {
            let target_weekday = dia_pago.unwrap_or(1) as u32;
            let mut fecha = inicio;
            while fecha <= desde {
                fecha += Duration::days(7);
            }
            Some(fecha.format("%Y-%m-%d").to_string())
        }
        "quincenal" => {
            let dia = dia_pago.unwrap_or(1).clamp(1, 15);
            let mut fecha = NaiveDate::from_ymd_opt(desde.year(), desde.month(), dia)?;
            if fecha <= desde {
                if desde.month() == 12 {
                    fecha = NaiveDate::from_ymd_opt(desde.year() + 1, 1, dia)?;
                } else {
                    fecha = NaiveDate::from_ymd_opt(desde.year(), desde.month() + 1, dia)?;
                }
            }
            Some(fecha.format("%Y-%m-%d").to_string())
        }
        "mensual" => {
            let dia = dia_pago.unwrap_or(1).clamp(1, 28);
            let mut fecha = NaiveDate::from_ymd_opt(desde.year(), desde.month(), dia)?;
            if fecha <= desde {
                if desde.month() == 12 {
                    fecha = NaiveDate::from_ymd_opt(desde.year() + 1, 1, dia)?;
                } else {
                    fecha = NaiveDate::from_ymd_opt(desde.year(), desde.month() + 1, dia)?;
                }
            }
            Some(fecha.format("%Y-%m-%d").to_string())
        }
        "trimestral" => {
            let dia = dia_pago.unwrap_or(1).clamp(1, 28);
            let mut mes = ((desde.month() - 1) / 3 + 1) * 3 + 1;
            let mut anio = desde.year();
            if mes > 12 {
                mes = 1;
                anio += 1;
            }
            let mut fecha = NaiveDate::from_ymd_opt(anio, mes, dia)?;
            if fecha <= desde {
                mes += 3;
                if mes > 12 {
                    mes -= 12;
                    anio += 1;
                }
                fecha = NaiveDate::from_ymd_opt(anio, mes, dia)?;
            }
            Some(fecha.format("%Y-%m-%d").to_string())
        }
        "personalizado" => {
            let intervalo = intervalo_dias.unwrap_or(30) as i64;
            let mut fecha = inicio;
            while fecha <= desde {
                fecha += Duration::days(intervalo);
            }
            Some(fecha.format("%Y-%m-%d").to_string())
        }
        _ => None,
    }
}

fn calcular_dias_para_vencer(proxima_fecha: Option<&str>) -> Option<i32> {
    let proxima = NaiveDate::parse_from_str(proxima_fecha?, "%Y-%m-%d").ok()?;
    let hoy = chrono::Local::now().date_naive();
    let diff = (proxima - hoy).num_days();
    Some(diff as i32)
}

fn map_row_to_gasto(row: sqlx::sqlite::SqliteRow) -> GastoRecurrente {
    let id: i64 = row.get("id");
    let fecha_inicio: String = row.get("fecha_inicio");
    let frecuencia: String = row.get("frecuencia");
    let dia_pago: Option<i32> = row.try_get("dia_pago").ok();
    let intervalo_dias: Option<i32> = row.try_get("intervalo_dias").ok();
    let hoy = chrono::Local::now().date_naive();
    
    let proxima_fecha = calcular_proxima_fecha(&fecha_inicio, &frecuencia, dia_pago, intervalo_dias, hoy);
    let dias_vencer = proxima_fecha.as_ref().and_then(|f| calcular_dias_para_vencer(Some(f)));
    
    GastoRecurrente {
        id,
        nombre: row.get("nombre"),
        tipo: row.get("tipo"),
        categoria: row.get("categoria"),
        monto_proyectado: decode_f64(&row, "monto_proyectado"),
        monto_real: decode_f64(&row, "monto_real"),
        frecuencia,
        dia_pago,
        intervalo_dias,
        fecha_inicio,
        fecha_fin: row.try_get("fecha_fin").ok(),
        estado_pago: row.get("estado_pago"),
        folio_comprobante: row.try_get("folio_comprobante").ok(),
        comprobante_url: row.try_get("comprobante_url").ok(),
        notas: row.try_get("notas").ok(),
        creado_en: row.get("creado_en"),
        actualizado_en: row.get("actualizado_en"),
        proxima_fecha_pago: proxima_fecha,
        dias_para_vencer: dias_vencer,
    }
}

#[tauri::command]
pub async fn get_gastos_recurrentes(state: tauri::State<'_, SqlitePool>) -> Result<Vec<GastoRecurrente>, String> {
    let rows = sqlx::query(
        "SELECT * FROM gastos_recurrentes ORDER BY fecha_inicio DESC"
    )
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(map_row_to_gasto).collect())
}

#[tauri::command]
pub async fn crear_gasto(
    state: tauri::State<'_, SqlitePool>,
    gasto: CrearGastoRequest,
) -> Result<i64, String> {
    let result = sqlx::query(
        r#"INSERT INTO gastos_recurrentes 
           (nombre, tipo, categoria, monto_proyectado, frecuencia, dia_pago, intervalo_dias, fecha_inicio, fecha_fin, folio_comprobante, notas)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#
    )
    .bind(&gasto.nombre)
    .bind(&gasto.tipo)
    .bind(&gasto.categoria)
    .bind(gasto.monto_proyectado)
    .bind(&gasto.frecuencia)
    .bind(gasto.dia_pago)
    .bind(gasto.intervalo_dias)
    .bind(&gasto.fecha_inicio)
    .bind(&gasto.fecha_fin)
    .bind(&gasto.folio_comprobante)
    .bind(&gasto.notas)
    .execute(&*state)
    .await
    .map_err(|e| e.to_string())?;

    Ok(result.last_insert_rowid())
}

#[tauri::command]
pub async fn actualizar_gasto(
    state: tauri::State<'_, SqlitePool>,
    id: i64,
    gasto: CrearGastoRequest,
) -> Result<(), String> {
    sqlx::query(
        r#"UPDATE gastos_recurrentes SET
           nombre = ?, tipo = ?, categoria = ?, monto_proyectado = ?, frecuencia = ?,
           dia_pago = ?, intervalo_dias = ?, fecha_inicio = ?, fecha_fin = ?,
           folio_comprobante = ?, notas = ?, actualizado_en = datetime('now','localtime')
           WHERE id = ?"#
    )
    .bind(&gasto.nombre)
    .bind(&gasto.tipo)
    .bind(&gasto.categoria)
    .bind(gasto.monto_proyectado)
    .bind(&gasto.frecuencia)
    .bind(gasto.dia_pago)
    .bind(gasto.intervalo_dias)
    .bind(&gasto.fecha_inicio)
    .bind(&gasto.fecha_fin)
    .bind(&gasto.folio_comprobante)
    .bind(&gasto.notas)
    .bind(id)
    .execute(&*state)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn eliminar_gasto(state: tauri::State<'_, SqlitePool>, id: i64) -> Result<(), String> {
    sqlx::query("DELETE FROM gastos_recurrentes WHERE id = ?")
        .bind(id)
        .execute(&*state)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn registrar_pago_gasto(
    state: tauri::State<'_, SqlitePool>,
    pago: RegistrarPagoRequest,
) -> Result<i64, String> {
    let mut tx = state.begin().await.map_err(|e| e.to_string())?;

    let result = sqlx::query(
        r#"INSERT INTO pagos_gastos (gasto_id, fecha_pago, monto_pagado, metodo_pago, folio_comprobante, notas)
           VALUES (?, ?, ?, ?, ?, ?)"#
    )
    .bind(pago.gasto_id)
    .bind(&pago.fecha_pago)
    .bind(pago.monto_pagado)
    .bind(&pago.metodo_pago)
    .bind(&pago.folio_comprobante)
    .bind(&pago.notas)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    let pago_id = result.last_insert_rowid();

    sqlx::query(
        "UPDATE gastos_recurrentes SET monto_real = monto_real + ?, estado_pago = 'pagado', actualizado_en = datetime('now','localtime') WHERE id = ?"
    )
    .bind(pago.monto_pagado)
    .bind(pago.gasto_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(pago_id)
}

#[tauri::command]
pub async fn get_pagos_gasto(state: tauri::State<'_, SqlitePool>, gasto_id: i64) -> Result<Vec<PagoGasto>, String> {
    let rows = sqlx::query_as::<_, (i64, i64, String, f64, Option<String>, Option<String>, Option<String>, String)>(
        "SELECT id, gasto_id, fecha_pago, monto_pagado, metodo_pago, folio_comprobante, comprobante_url, notas, creado_en FROM pagos_gastos WHERE gasto_id = ? ORDER BY fecha_pago DESC"
    )
    .bind(gasto_id)
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|row| PagoGasto {
        id: row.0,
        gasto_id: row.1,
        fecha_pago: row.2,
        monto_pagado: row.3,
        metodo_pago: row.4,
        folio_comprobante: row.5,
        comprobante_url: row.6,
        notas: row.7,
        creado_en: row.8,
    }).collect())
}

#[tauri::command]
pub async fn get_proximos_vencimientos(state: tauri::State<'_, SqlitePool>, dias: i32) -> Result<Vec<GastoRecurrente>, String> {
    let hoy = chrono::Local::now().date_naive();
    let limite = hoy + Duration::days(dias as i64);
    
    let rows = sqlx::query(
        "SELECT * FROM gastos_recurrentes WHERE estado_pago IN ('pendiente', 'proximo_vencer') AND fecha_inicio <= ? ORDER BY fecha_inicio ASC"
    )
    .bind(limite.format("%Y-%m-%d").to_string())
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    let gastos: Vec<GastoRecurrente> = rows.into_iter().map(map_row_to_gasto).collect();
    Ok(gastos.into_iter().filter(|g| g.dias_para_vencer.unwrap_or(999) <= dias).collect())
}

#[tauri::command]
pub async fn actualizar_estados_gastos(state: tauri::State<'_, SqlitePool>) -> Result<(), String> {
    let hoy = chrono::Local::now().date_naive();
    let en_3_dias = hoy + Duration::days(3);

    sqlx::query(
        "UPDATE gastos_recurrentes SET estado_pago = 'vencido', actualizado_en = datetime('now','localtime')
         WHERE estado_pago IN ('pendiente', 'proximo_vencer')
         AND fecha_fin IS NULL
         AND (
             (frecuencia = 'semanal' AND date(fecha_inicio, '+' || ((julianday(?) - julianday(fecha_inicio)) / 7 + 1) * 7 || ' days') < ?)
             OR (frecuencia = 'mensual' AND date(fecha_inicio, '+' || (strftime('%m', ?) - strftime('%m', fecha_inicio) + (strftime('%Y', ?) - strftime('%Y', fecha_inicio)) * 12) || ' months') < ?)
         )"
    )
    .bind(hoy.format("%Y-%m-%d").to_string())
    .bind(hoy.format("%Y-%m-%d").to_string())
    .bind(hoy.format("%Y-%m-%d").to_string())
    .bind(hoy.format("%Y-%m-%d").to_string())
    .bind(hoy.format("%Y-%m-%d").to_string())
    .bind(hoy.format("%Y-%m-%d").to_string())
    .execute(&*state)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query(
        "UPDATE gastos_recurrentes SET estado_pago = 'proximo_vencer', actualizado_en = datetime('now','localtime')
         WHERE estado_pago = 'pendiente'
         AND fecha_fin IS NULL
         AND (
             (frecuencia = 'semanal' AND date(fecha_inicio, '+' || ((julianday(?) - julianday(fecha_inicio)) / 7 + 1) * 7 || ' days') BETWEEN ? AND ?)
             OR (frecuencia = 'mensual' AND date(fecha_inicio, '+' || (strftime('%m', ?) - strftime('%m', fecha_inicio) + (strftime('%Y', ?) - strftime('%Y', fecha_inicio)) * 12) || ' months') BETWEEN ? AND ?)
         )"
    )
    .bind(hoy.format("%Y-%m-%d").to_string())
    .bind(hoy.format("%Y-%m-%d").to_string())
    .bind(en_3_dias.format("%Y-%m-%d").to_string())
    .bind(hoy.format("%Y-%m-%d").to_string())
    .bind(hoy.format("%Y-%m-%d").to_string())
    .bind(hoy.format("%Y-%m-%d").to_string())
    .bind(en_3_dias.format("%Y-%m-%d").to_string())
    .execute(&*state)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}