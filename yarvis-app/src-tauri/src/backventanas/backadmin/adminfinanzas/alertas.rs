use crate::backventanas::auth::AuthState;
use crate::backventanas::backadmin::adminfinanzas::fechas::calcular_proxima_fecha;
use crate::backventanas::backadmin::adminfinanzas::metricas::{
    calcular_utilidad_neta_periodo, sincronizar_resumen_mes_actual,
};
use crate::backventanas::backadmin::adminfinanzas::models::*;
use chrono::{Datelike, Duration};
use sqlx::Row;
use sqlx::SqlitePool;

/// Lee una columna monetaria (INTEGER en centavos) y la devuelve en pesos.
/// Nota: el ratio ABS(diferencia)/total_ventas del WHERE es insensible a
/// la escala (ambos lados quedan en centavos), por eso no se toca.
fn decode_dinero(row: &sqlx::sqlite::SqliteRow, col: &str) -> f64 {
    row.try_get::<i64, _>(col)
        .map(crate::dinero::a_pesos)
        .unwrap_or(0.0)
}

#[tauri::command]
pub async fn get_alertas(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    solo_no_leidas: bool,
) -> Result<Vec<AlertaFinanciera>, String> {
    auth.require_admin()?;
    let mut query = String::from("SELECT * FROM alertas_financieras WHERE 1=1");
    if solo_no_leidas {
        query.push_str(" AND leida = 0");
    }
    query.push_str(
        r#" ORDER BY
        CASE severidad WHEN 'rojo' THEN 0 WHEN 'amarillo' THEN 1 WHEN 'verde' THEN 2 ELSE 3 END,
        creada_en DESC
        LIMIT 50"#,
    );

    let rows = sqlx::query(&query)
        .fetch_all(&*state)
        .await
        .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|row| AlertaFinanciera {
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
        })
        .collect())
}

#[tauri::command]
pub async fn marcar_alerta_leida(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    id: i64,
) -> Result<(), String> {
    auth.require_admin()?;
    sqlx::query("UPDATE alertas_financieras SET leida = 1 WHERE id = ?")
        .bind(id)
        .execute(&*state)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn generar_alertas_automaticas(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
) -> Result<Vec<AlertaFinanciera>, String> {
    auth.require_admin()?;
    generar_alertas_automaticas_impl(&*state).await
}

pub async fn generar_alertas_automaticas_impl(
    state: &SqlitePool,
) -> Result<Vec<AlertaFinanciera>, String> {
    let hoy = chrono::Local::now().date_naive();
    let hace_30_dias = hoy - Duration::days(30);

    // `resumen_financiero_diario` es una caché, no la fuente de verdad.
    // Se refresca para que reportes que la consuman no queden obsoletos;
    // las alertas calculan su importe directamente desde las tablas vivas.
    sincronizar_resumen_mes_actual(state).await?;

    let mut nuevas_alertas = Vec::new();

    // 1. Alertas de gastos por vencer (3 días)
    let gastos_por_vencer = sqlx::query(
        r#"SELECT * FROM gastos_recurrentes 
           WHERE estado_pago IN ('pendiente', 'proximo_vencer')
           AND (fecha_fin IS NULL OR fecha_fin >= ?)
           AND id NOT IN (
               SELECT entidad_id FROM alertas_financieras 
               WHERE tipo = 'gasto_vencimiento' AND entidad_tipo = 'gasto' AND leida = 0
           )"#,
    )
    .bind(hoy.format("%Y-%m-%d").to_string())
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    for gasto in gastos_por_vencer {
        let gasto_id: i64 = gasto.get("id");
        let nombre: String = gasto.get("nombre");
        let monto_proyectado: f64 = decode_dinero(&gasto, "monto_proyectado");
        let fecha_inicio: String = gasto.get("fecha_inicio");
        let frecuencia: String = gasto.get("frecuencia");
        let dia_pago: Option<i32> = gasto.try_get("dia_pago").ok();
        let intervalo_dias: Option<i32> = gasto.try_get("intervalo_dias").ok();

        // Próxima ocurrencia calculada (día 0 = vence hoy).
        let proxima =
            calcular_proxima_fecha(&fecha_inicio, &frecuencia, dia_pago, intervalo_dias, hoy);

        if let Some(prox_fecha) = proxima {
            let dias = (prox_fecha - hoy).num_days();

            let (severidad, titulo) = if dias <= 0 {
                ("rojo".to_string(), "GASTO VENCIDO".to_string())
            } else if dias <= 3 {
                ("rojo".to_string(), "GASTO PRÓXIMO A VENCER".to_string())
            } else if dias <= 7 {
                ("amarillo".to_string(), "GASTO EN 7 DÍAS".to_string())
            } else {
                continue;
            };

            let mensaje = format!(
                "El gasto '{}' de ${:.2} vence el {} ({} días)",
                nombre,
                monto_proyectado,
                prox_fecha.format("%d/%m/%Y"),
                dias
            );

            let alerta_id = sqlx::query(
                r#"INSERT INTO alertas_financieras (tipo, severidad, titulo, mensaje, entidad_id, entidad_tipo, fecha_vencimiento)
                   VALUES ('gasto_vencimiento', ?, ?, ?, ?, 'gasto', ?)"#
            )
            .bind(&severidad)
            .bind(&titulo)
            .bind(&mensaje)
            .bind(gasto_id)
            .bind(prox_fecha.format("%Y-%m-%d").to_string())
            .execute(&*state)
            .await
            .map_err(|e| e.to_string())?
            .last_insert_rowid();

            nuevas_alertas.push(AlertaFinanciera {
                id: alerta_id,
                tipo: "gasto_vencimiento".to_string(),
                severidad,
                titulo,
                mensaje,
                entidad_id: Some(gasto_id),
                entidad_tipo: Some("gasto".to_string()),
                fecha_vencimiento: Some(prox_fecha.format("%Y-%m-%d").to_string()),
                leida: false,
                creada_en: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            });
        }
    }

    // 2. Alertas de cortes pendientes (cortes abiertos > 24h)
    let cortes_abiertos = sqlx::query(
        r#"SELECT c.id, c.fecha_apertura, u.nombre 
           FROM cortes_caja c
           JOIN usuarios u ON c.usuario_id = u.id
           WHERE c.estado = 'abierto' 
           AND datetime(c.fecha_apertura) < datetime('now', '-24 hours')
           AND c.id NOT IN (
               SELECT entidad_id FROM alertas_financieras 
               WHERE tipo = 'corte_pendiente' AND entidad_tipo = 'corte' AND leida = 0
           )"#,
    )
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    for corte in cortes_abiertos {
        let corte_id: i64 = corte.get("id");
        let cajero: String = corte.get("nombre");
        let fecha_apertura: String = corte.get("fecha_apertura");

        // Fecha de apertura VALIDADA: si viene malformada (DB corrupta o
        // formato inesperado) se salta este corte con registro, no crashea
        // el job entero de alertas.
        let apertura_parseada = match chrono::DateTime::parse_from_str(
            &fecha_apertura,
            "%Y-%m-%d %H:%M:%S",
        ) {
            Ok(f) => f.with_timezone(&chrono::Local),
            Err(e) => {
                tracing::warn!("[ALERTAS] fecha_apertura inválida en corte {corte_id} ('{fecha_apertura}'): {e}");
                continue;
            }
        };

        let horas_abierto = chrono::Local::now()
            .signed_duration_since(apertura_parseada)
            .num_hours();

        let alerta_id = sqlx::query(
            r#"INSERT INTO alertas_financieras (tipo, severidad, titulo, mensaje, entidad_id, entidad_tipo)
               VALUES ('corte_pendiente', 'amarillo', 'CORTE SIN CERRAR', ?, ?, 'corte')"#
        )
        .bind(format!("El corte #{} de {} lleva {} horas abierto sin cerrar", corte_id, cajero, horas_abierto))
        .bind(corte_id)
        .execute(&*state)
        .await
        .map_err(|e| e.to_string())?
        .last_insert_rowid();

        nuevas_alertas.push(AlertaFinanciera {
            id: alerta_id,
            tipo: "corte_pendiente".to_string(),
            severidad: "amarillo".to_string(),
            titulo: "CORTE SIN CERRAR".to_string(),
            mensaje: format!(
                "El corte #{} de {} lleva {} horas abierto",
                corte_id, cajero, horas_abierto
            ),
            entidad_id: Some(corte_id),
            entidad_tipo: Some("corte".to_string()),
            fecha_vencimiento: None,
            leida: false,
            creada_en: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        });
    }

    // 3. Alertas de diferencias de caja altas (> 5% de ventas)
    let diferencias_altas = sqlx::query(
        r#"SELECT c.id, c.total_ventas, c.diferencia, u.nombre, c.fecha_cierre
           FROM cortes_caja c
           JOIN usuarios u ON c.usuario_id = u.id
           WHERE c.tipo_corte = 'Z' 
           AND c.estado = 'cerrado'
           AND c.total_ventas > 0
           AND ABS(c.diferencia) / c.total_ventas > 0.05
           AND date(c.fecha_cierre) >= ?
           AND c.id NOT IN (
               SELECT entidad_id FROM alertas_financieras 
               WHERE tipo = 'diferencia_caja' AND entidad_tipo = 'corte' AND leida = 0
           )"#,
    )
    .bind(hace_30_dias.format("%Y-%m-%d").to_string())
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    for corte in diferencias_altas {
        let corte_id: i64 = corte.get("id");
        let cajero: String = corte.get("nombre");
        let total_ventas: f64 = decode_dinero(&corte, "total_ventas");
        let diferencia: f64 = decode_dinero(&corte, "diferencia");
        let fecha_cierre: String = corte.get("fecha_cierre");
        let pct = (diferencia.abs() / total_ventas) * 100.0;

        let severidad = if pct > 10.0 { "rojo" } else { "amarillo" };
        let titulo = if diferencia > 0.0 {
            "SOBRANTE EN CAJA"
        } else {
            "FALTANTE EN CAJA"
        };

        let alerta_id = sqlx::query(
            r#"INSERT INTO alertas_financieras (tipo, severidad, titulo, mensaje, entidad_id, entidad_tipo)
               VALUES ('diferencia_caja', ?, ?, ?, ?, 'corte')"#
        )
        .bind(severidad)
        .bind(titulo)
        .bind(format!("Corte #{} de {}: diferencia de ${:.2} ({:.1}% de ventas) el {}", corte_id, cajero, diferencia.abs(), pct, fecha_cierre))
        .bind(corte_id)
        .execute(&*state)
        .await
        .map_err(|e| e.to_string())?
        .last_insert_rowid();

        nuevas_alertas.push(AlertaFinanciera {
            id: alerta_id,
            tipo: "diferencia_caja".to_string(),
            severidad: severidad.to_string(),
            titulo: titulo.to_string(),
            mensaje: format!(
                "Corte #{}: diferencia de ${:.2} ({:.1}%)",
                corte_id,
                diferencia.abs(),
                pct
            ),
            entidad_id: Some(corte_id),
            entidad_tipo: Some("corte".to_string()),
            fecha_vencimiento: None,
            leida: false,
            creada_en: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        });
    }

    // 4. Alerta de utilidad neta negativa en el mes
    let inicio_mes = chrono::NaiveDate::from_ymd_opt(hoy.year(), hoy.month(), 1)
        .ok_or_else(|| "No se pudo calcular el inicio del mes actual".to_string())?;
    let utilidad_total = calcular_utilidad_neta_periodo(
        state,
        &inicio_mes.format("%Y-%m-%d").to_string(),
        &hoy.format("%Y-%m-%d").to_string(),
    )
    .await?;

    if utilidad_total < 0.0 {
        // Verificar si ya existe alerta este mes
        let existe = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM alertas_financieras WHERE tipo = 'utilidad_negativa' AND date(creada_en) >= date('now', 'start of month')"
        )
        .fetch_one(&*state)
        .await
        .unwrap_or(0);

        if existe == 0 {
            let alerta_id = sqlx::query(
                r#"INSERT INTO alertas_financieras (tipo, severidad, titulo, mensaje, entidad_tipo)
                   VALUES ('utilidad_negativa', 'rojo', 'UTILIDAD NETA NEGATIVA', ?, 'sistema')"#,
            )
            .bind(format!(
                "La utilidad neta del mes actual es de ${:.2}. Revisar gastos e ingresos.",
                utilidad_total
            ))
            .execute(&*state)
            .await
            .map_err(|e| e.to_string())?
            .last_insert_rowid();

            nuevas_alertas.push(AlertaFinanciera {
                id: alerta_id,
                tipo: "utilidad_negativa".to_string(),
                severidad: "rojo".to_string(),
                titulo: "UTILIDAD NETA NEGATIVA".to_string(),
                mensaje: format!("Utilidad neta del mes: ${:.2}", utilidad_total),
                entidad_id: None,
                entidad_tipo: Some("sistema".to_string()),
                fecha_vencimiento: None,
                leida: false,
                creada_en: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            });
        }
    }

    Ok(nuevas_alertas)
}

// Background job para ejecutar alertas automáticas cada hora.
// Se arranca en lib.rs (setup) con el pool de la DB.
pub fn iniciar_job_alertas(pool: SqlitePool) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3600)); // 1 hora
        loop {
            interval.tick().await;
            let _ = generar_alertas_automaticas_impl(&pool).await;
            let _ = crate::backventanas::backadmin::adminfinanzas::gastos::actualizar_estados_gastos_impl(&pool).await;
        }
    });
}
