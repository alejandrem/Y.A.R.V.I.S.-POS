use crate::backventanas::auth::AuthState;
use crate::backventanas::backadmin::adminfinanzas::fechas::calcular_proxima_fecha;
use crate::backventanas::backadmin::adminfinanzas::models::*;
use chrono::NaiveDate;
use sqlx::Row;
use sqlx::SqlitePool;

fn decode_f64(row: &sqlx::sqlite::SqliteRow, col: &str) -> f64 {
    row.try_get::<f64, _>(col)
        .or_else(|_| row.try_get::<i64, _>(col).map(|v| v as f64))
        .unwrap_or(0.0)
}

fn map_row_to_gasto(row: sqlx::sqlite::SqliteRow) -> GastoRecurrente {
    let id: i64 = row.get("id");
    let fecha_inicio: String = row.get("fecha_inicio");
    let frecuencia: String = row.get("frecuencia");
    let dia_pago: Option<i32> = row.try_get("dia_pago").ok();
    let intervalo_dias: Option<i32> = row.try_get("intervalo_dias").ok();
    let hoy = chrono::Local::now().date_naive();

    let prox = calcular_proxima_fecha(&fecha_inicio, &frecuencia, dia_pago, intervalo_dias, hoy);

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
        proxima_fecha_pago: prox.map(|f| f.format("%Y-%m-%d").to_string()),
        dias_para_vencer: prox.map(|f| (f - hoy).num_days() as i32),
    }
}

#[tauri::command]
pub async fn get_gastos_recurrentes(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
) -> Result<Vec<GastoRecurrente>, String> {
    auth.require_admin()?;
    let rows = sqlx::query("SELECT * FROM gastos_recurrentes ORDER BY fecha_inicio ASC")
        .fetch_all(&*state)
        .await
        .map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(map_row_to_gasto).collect())
}

#[tauri::command]
pub async fn crear_gasto(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    gasto: CrearGastoRequest,
) -> Result<i64, String> {
    auth.require_admin()?;
    let result = sqlx::query(
        r#"INSERT INTO gastos_recurrentes (nombre, tipo, categoria, monto_proyectado, frecuencia, dia_pago, intervalo_dias, fecha_inicio, fecha_fin, folio_comprobante, notas)
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
    auth: tauri::State<'_, AuthState>,
    id: i64,
    gasto: CrearGastoRequest,
) -> Result<(), String> {
    auth.require_admin()?;
    sqlx::query(
        r#"UPDATE gastos_recurrentes SET
           nombre = ?, tipo = ?, categoria = ?, monto_proyectado = ?, frecuencia = ?, 
           dia_pago = ?, intervalo_dias = ?, fecha_inicio = ?, fecha_fin = ?, 
           folio_comprobante = ?, notas = ?, actualizado_en = datetime('now','localtime')
           WHERE id = ?"#,
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
pub async fn eliminar_gasto(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    id: i64,
) -> Result<(), String> {
    auth.require_admin()?;
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
    auth: tauri::State<'_, AuthState>,
    pago: RegistrarPagoRequest,
) -> Result<i64, String> {
    auth.require_admin()?;
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
    .execute(&*state)
    .await
    .map_err(|e| e.to_string())?;

    // Actualizar monto_real y estado del gasto
    let gasto_row =
        sqlx::query("SELECT monto_proyectado, monto_real FROM gastos_recurrentes WHERE id = ?")
            .bind(pago.gasto_id)
            .fetch_optional(&*state)
            .await
            .map_err(|e| e.to_string())?;

    if let Some(row) = gasto_row {
        let monto_proyectado: f64 = decode_f64(&row, "monto_proyectado");
        let monto_real_actual: f64 = decode_f64(&row, "monto_real");
        let nuevo_monto_real = monto_real_actual + pago.monto_pagado;

        let nuevo_estado = if nuevo_monto_real >= monto_proyectado {
            "pagado"
        } else {
            "pendiente"
        };

        sqlx::query("UPDATE gastos_recurrentes SET monto_real = ?, estado_pago = ?, actualizado_en = datetime('now','localtime') WHERE id = ?")
            .bind(nuevo_monto_real)
            .bind(nuevo_estado)
            .bind(pago.gasto_id)
            .execute(&*state)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(result.last_insert_rowid())
}

#[tauri::command]
pub async fn get_pagos_gasto(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    gasto_id: i64,
) -> Result<Vec<PagoGasto>, String> {
    auth.require_admin()?;
    let rows = sqlx::query_as::<_, (i64, i64, String, f64, Option<String>, Option<String>, Option<String>, Option<String>, String)>(
        "SELECT id, gasto_id, fecha_pago, monto_pagado, metodo_pago, folio_comprobante, comprobante_url, notas, creado_en FROM pagos_gastos WHERE gasto_id = ? ORDER BY fecha_pago DESC"
    )
    .bind(gasto_id)
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|row| PagoGasto {
            id: row.0,
            gasto_id: row.1,
            fecha_pago: row.2,
            monto_pagado: row.3,
            metodo_pago: row.4,
            folio_comprobante: row.5,
            comprobante_url: row.6,
            notas: row.7,
            creado_en: row.8,
        })
        .collect())
}

#[tauri::command]
pub async fn get_proximos_vencimientos(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    dias: i32,
) -> Result<Vec<GastoRecurrente>, String> {
    auth.require_admin()?;
    let hoy = chrono::Local::now().date_naive();

    let rows = sqlx::query(
        "SELECT * FROM gastos_recurrentes 
         WHERE estado_pago IN ('pendiente', 'proximo_vencer') 
         AND (fecha_fin IS NULL OR fecha_fin >= ?)
         ORDER BY fecha_inicio ASC",
    )
    .bind(hoy.format("%Y-%m-%d").to_string())
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    // La "próxima fecha" se calcula en Rust (fechas::calcular_proxima_fecha);
    // aquí solo filtramos por la ventana de días pedida por el frontend.
    let gastos: Vec<GastoRecurrente> = rows.into_iter().map(map_row_to_gasto).collect();

    Ok(gastos
        .into_iter()
        .filter(|g| g.dias_para_vencer.unwrap_or(i32::MAX) <= dias)
        .collect())
}

#[tauri::command]
pub async fn actualizar_estados_gastos(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
) -> Result<(), String> {
    auth.require_admin()?;
    actualizar_estados_gastos_impl(&*state).await
}

pub async fn actualizar_estados_gastos_impl(state: &SqlitePool) -> Result<(), String> {
    let hoy = chrono::Local::now().date_naive();

    let rows = sqlx::query(
        "SELECT id, fecha_inicio, frecuencia, dia_pago, intervalo_dias, estado_pago, fecha_fin FROM gastos_recurrentes"
    )
    .fetch_all(state)
    .await
    .map_err(|e| e.to_string())?;

    for row in rows {
        let id: i64 = row.get("id");
        let fecha_inicio: String = row.get("fecha_inicio");
        let frecuencia: String = row.get("frecuencia");
        let dia_pago: Option<i32> = row.try_get("dia_pago").ok();
        let intervalo_dias: Option<i32> = row.try_get("intervalo_dias").ok();
        let estado_actual: String = row.get("estado_pago");
        let fecha_fin: Option<String> = row.try_get("fecha_fin").ok();

        // Ya cubierto o fuera de vigencia → no tocar.
        if estado_actual == "pagado" {
            continue;
        }
        if let Some(fin) = &fecha_fin {
            if let Ok(fin) = NaiveDate::parse_from_str(fin, "%Y-%m-%d") {
                if fin < hoy {
                    continue;
                }
            }
        }

        // Calcular el estado desde la fecha computada (día 0 = vence hoy).
        let nuevo_estado =
            match calcular_proxima_fecha(&fecha_inicio, &frecuencia, dia_pago, intervalo_dias, hoy)
                .map(|f| (f - hoy).num_days())
            {
                Some(dias) if dias <= 0 => "vencido",
                Some(dias) if (1..=3).contains(&dias) => "proximo_vencer",
                _ => "pendiente",
            };

        if nuevo_estado != estado_actual {
            sqlx::query("UPDATE gastos_recurrentes SET estado_pago = ?, actualizado_en = datetime('now','localtime') WHERE id = ?")
                .bind(nuevo_estado)
                .bind(id)
                .execute(state)
                .await
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}
