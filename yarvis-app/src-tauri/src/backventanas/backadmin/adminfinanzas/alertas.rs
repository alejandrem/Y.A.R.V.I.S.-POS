use sqlx::SqlitePool;
use chrono::{NaiveDate, Duration};
use crate::backventanas::backadmin::adminfinanzas::models::*;

fn decode_f64(row: &sqlx::sqlite::SqliteRow, col: &str) -> f64 {
    row.try_get::<f64, _>(col)
        .or_else(|_| row.try_get::<i64, _>(col).map(|v| v as f64))
        .unwrap_or(0.0)
}

#[tauri::command]
pub async fn get_alertas(state: tauri::State<'_, SqlitePool>, solo_no_leidas: bool) -> Result<Vec<AlertaFinanciera>, String> {
    let mut query = String::from("SELECT * FROM alertas_financieras WHERE 1=1");
    if solo_no_leidas {
        query.push_str(" AND leida = 0");
    }
    query.push_str(" ORDER BY 
        CASE severidad WHEN 'rojo' THEN 0 WHEN 'amarillo' THEN 1 WHEN 'verde' THEN 2 END,
        creada_en DESC");

    let rows = sqlx::query(&query)
        .fetch_all(&*state)
        .await
        .map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|row| AlertaFinanciera {
        id: row.get("id"),
        tipo: row.get("tipo"),
        severidad: row.get("severidad"),
        titulo: row.get("titulo"),
        mensaje: row.get("mensaje"),
        entidad_id: row.try_get("entidad_id").ok(),
        entidad_tipo: row.try_get("entidad_tipo").ok(),
        fecha_vencimiento: row.try_get("fecha_vencimiento").ok(),
        leida: row.get::<i64, _>("leida") != 0,
        creada_en: row.get("creada_en"),
    }).collect())
}

#[tauri::command]
pub async fn marcar_alerta_leida(state: tauri::State<'_, SqlitePool>, id: i64) -> Result<(), String> {
    sqlx::query("UPDATE alertas_financieras SET leida = 1 WHERE id = ?")
        .bind(id)
        .execute(&*state)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn generar_alertas_automaticas(state: tauri::State<'_, SqlitePool>) -> Result<Vec<AlertaFinanciera>, String> {
    let mut nuevas_alertas = Vec::new();
    let hoy = chrono::Local::now().date_naive();
    let en_3_dias = hoy + Duration::days(3);
    let en_7_dias = hoy + Duration::days(7);

    // 1. Alertas de gastos próximos a vencer
    let gastos_proximos = sqlx::query(
        r#"SELECT id, nombre, tipo, monto_proyectado, proxima_fecha_pago
           FROM gastos_recurrentes 
           WHERE estado_pago IN ('pendiente', 'proximo_vencer')
           AND (fecha_fin IS NULL OR fecha_fin >= ?)"#
    )
    .bind(hoy.format("%Y-%m-%d").to_string())
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    for row in gastos_proximos {
        let gasto_id: i64 = row.get("id");
        let nombre: String = row.get("nombre");
        let tipo: String = row.get("tipo");
        let monto: f64 = decode_f64(&row, "monto_proyectado");
        let proxima_fecha: Option<String> = row.try_get("proxima_fecha_pago").ok();

        if let Some(fecha_str) = proxima_fecha {
            if let Ok(fecha_venc) = NaiveDate::parse_from_str(&fecha_str, "%Y-%m-%d") {
                let severidad = if fecha_venc <= hoy { "rojo" } else if fecha_venc <= en_3_dias { "rojo" } else if fecha_venc <= en_7_dias { "amarillo" } else { "verde" };
                let titulo = if fecha_venc <= hoy { "GASTO VENCIDO" } else if fecha_venc <= en_3_dias { "GASTO PRÓXIMO A VENCER" } else { "GASTO PRÓXIMO" };
                let mensaje = format!("El gasto '{}' ({}) de ${:.2} vence el {}", nombre, tipo, monto, fecha_venc.format("%d/%m/%Y"));

                // Verificar si ya existe alerta no leída para este gasto
                let existe = sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM alertas_financieras WHERE entidad_id = ? AND entidad_tipo = 'gasto' AND leida = 0"
                )
                .bind(gasto_id)
                .fetch_one(&*state)
                .await
                .unwrap_or(0);

                if existe == 0 {
                    let result = sqlx::query(
                        r#"INSERT INTO alertas_financieras (tipo, severidad, titulo, mensaje, entidad_id, entidad_tipo, fecha_vencimiento)
                           VALUES ('gasto_vencimiento', ?, ?, ?, ?, 'gasto', ?)"#
                    )
                    .bind(severidad)
                    .bind(titulo)
                    .bind(&mensaje)
                    .bind(gasto_id)
                    .bind(fecha_venc.format("%Y-%m-%d").to_string())
                    .execute(&*state)
                    .await
                    .map_err(|e| e.to_string())?;

                    nuevas_alertas.push(AlertaFinanciera {
                        id: result.last_insert_rowid(),
                        tipo: "gasto_vencimiento".to_string(),
                        severidad: severidad.to_string(),
                        titulo: titulo.to_string(),
                        mensaje,
                        entidad_id: Some(gasto_id),
                        entidad_tipo: Some("gasto".to_string()),
                        fecha_vencimiento: Some(fecha_venc.format("%Y-%m-%d").to_string()),
                        leida: false,
                        creada_en: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    });
                }
            }
        }
    }

    // 2. Alertas de cortes de caja pendientes (cortes abiertos > 12 horas)
    let cortes_abiertos = sqlx::query(
        "SELECT id, usuario_id, fecha_apertura FROM cortes_caja WHERE estado = 'abierto'"
    )
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    for row in cortes_abiertos {
        let corte_id: i64 = row.get("id");
        let usuario_id: i64 = row.get("usuario_id");
        let fecha_apertura: String = row.get("fecha_apertura");
        
        if let Ok(apertura) = chrono::DateTime::parse_from_str(&fecha_apertura, "%Y-%m-%d %H:%M:%S") {
            let horas_abierto = (chrono::Local::now() - apertura).num_hours();
            if horas_abierto > 12 {
                let severidad = if horas_abierto > 24 { "rojo" } else { "amarillo" };
                let titulo = "CORTE DE CAJA ABIERTO POR MUCHO TIEMPO";
                let mensaje = format!("El corte #{} lleva {} horas abierto sin cerrar", corte_id, horas_abierto);

                let existe = sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM alertas_financieras WHERE entidad_id = ? AND entidad_tipo = 'corte' AND leida = 0"
                )
                .bind(corte_id)
                .fetch_one(&*state)
                .await
                .unwrap_or(0);

                if existe == 0 {
                    let result = sqlx::query(
                        r#"INSERT INTO alertas_financieras (tipo, severidad, titulo, mensaje, entidad_id, entidad_tipo)
                           VALUES ('corte_pendiente', ?, ?, ?, ?, 'corte')"#
                    )
                    .bind(severidad)
                    .bind(titulo)
                    .bind(&mensaje)
                    .bind(corte_id)
                    .execute(&*state)
                    .await
                    .map_err(|e| e.to_string())?;

                    nuevas_alertas.push(AlertaFinanciera {
                        id: result.last_insert_rowid(),
                        tipo: "corte_pendiente".to_string(),
                        severidad: severidad.to_string(),
                        titulo: titulo.to_string(),
                        mensaje,
                        entidad_id: Some(corte_id),
                        entidad_tipo: Some("corte".to_string()),
                        fecha_vencimiento: None,
                        leida: false,
                        creada_en: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    });
                }
            }
        }
    }

    // 3. Alertas de diferencias de caja grandes
    let cortes_con_diferencia = sqlx::query(
        r#"SELECT c.id, c.diferencia, c.total_ventas, u.nombre as cajero
           FROM cortes_caja c
           LEFT JOIN usuarios u ON c.usuario_id = u.id
           WHERE c.estado = 'cerrado' AND c.tipo_corte = 'Z' 
           AND ABS(c.diferencia) > 50
           AND date(c.fecha_cierre) = date('now','localtime')"#
    )
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    for row in cortes_con_diferencia {
        let corte_id: i64 = row.get("id");
        let diferencia: f64 = decode_f64(&row, "diferencia");
        let cajero: String = row.get("cajero");
        let severidad = if diferencia.abs() > 200.0 { "rojo" } else { "amarillo" };
        let titulo = if diferencia < 0.0 { "FALTANTE EN CAJA" } else { "SOBRANTE EN CAJA" };
        let mensaje = format!("Corte #{} del cajero {}: {} de ${:.2}", corte_id, cajero, if diferencia < 0.0 { "Faltante" } else { "Sobrante" }, diferencia.abs());

        let existe = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM alertas_financieras WHERE entidad_id = ? AND entidad_tipo = 'corte' AND tipo = 'diferencia_caja' AND leida = 0"
        )
        .bind(corte_id)
        .fetch_one(&*state)
        .await
        .unwrap_or(0);

        if existe == 0 {
            let result = sqlx::query(
                r#"INSERT INTO alertas_financieras (tipo, severidad, titulo, mensaje, entidad_id, entidad_tipo)
                   VALUES ('diferencia_caja', ?, ?, ?, ?, 'corte')"#
            )
            .bind(severidad)
            .bind(titulo)
            .bind(&mensaje)
            .bind(corte_id)
            .execute(&*state)
            .await
            .map_err(|e| e.to_string())?;

            nuevas_alertas.push(AlertaFinanciera {
                id: result.last_insert_rowid(),
                tipo: "diferencia_caja".to_string(),
                severidad: severidad.to_string(),
                titulo: titulo.to_string(),
                mensaje,
                entidad_id: Some(corte_id),
                entidad_tipo: Some("corte".to_string()),
                fecha_vencimiento: None,
                leida: false,
                creada_en: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            });
        }
    }

    Ok(nuevas_alertas)
}

#[tauri::command]
pub async fn limpiar_alertas_antiguas(state: tauri::State<'_, SqlitePool>, dias: i32) -> Result<i64, String> {
    let fecha_limite = (chrono::Local::now().date_naive() - Duration::days(dias as i64)).format("%Y-%m-%d").to_string();
    let result = sqlx::query("DELETE FROM alertas_financieras WHERE leida = 1 AND date(creada_en) < ?")
        .bind(fecha_limite)
        .execute(&*state)
        .await
        .map_err(|e| e.to_string())?;
    Ok(result.rows_affected())
}