use crate::backventanas::auth::AuthState;
use crate::backventanas::backadmin::adminfinanzas::models::*;
use chrono::{Datelike, NaiveDate};
use sqlx::Row;
use sqlx::SqlitePool;

fn decode_f64(row: &sqlx::sqlite::SqliteRow, col: &str) -> f64 {
    row.try_get::<f64, _>(col)
        .or_else(|_| row.try_get::<i64, _>(col).map(|v| v as f64))
        .unwrap_or(0.0)
}

async fn calcular_costo_ventas(
    pool: &SqlitePool,
    fecha_inicio: &str,
    fecha_fin: &str,
) -> Result<f64, String> {
    let row = sqlx::query(
        r#"SELECT COALESCE(SUM(dv.cantidad * p.precio_costo), 0) as cogs
           FROM detalle_ventas dv
           JOIN ventas v ON dv.venta_id = v.id
           JOIN productos p ON dv.producto_id = p.id
           WHERE date(v.fecha) BETWEEN ? AND ? AND v.estado = 'completada'"#,
    )
    .bind(fecha_inicio)
    .bind(fecha_fin)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(decode_f64(&row, "cogs"))
}

async fn calcular_gastos_operativos(
    pool: &SqlitePool,
    fecha_inicio: &str,
    fecha_fin: &str,
) -> Result<f64, String> {
    let row = sqlx::query(
        r#"SELECT COALESCE(SUM(pg.monto_pagado), 0) as total_gastos
           FROM pagos_gastos pg
           WHERE date(pg.fecha_pago) BETWEEN ? AND ?"#,
    )
    .bind(fecha_inicio)
    .bind(fecha_fin)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(decode_f64(&row, "total_gastos"))
}

async fn calcular_impuestos_comisiones(
    pool: &SqlitePool,
    fecha_inicio: &str,
    fecha_fin: &str,
) -> Result<f64, String> {
    // IVA de las ventas (16% en México) - simplificado
    let row = sqlx::query(
        r#"SELECT COALESCE(SUM(iva), 0) as total_iva
           FROM ventas 
           WHERE date(fecha) BETWEEN ? AND ? AND estado = 'completada'"#,
    )
    .bind(fecha_inicio)
    .bind(fecha_fin)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(decode_f64(&row, "total_iva"))
}

async fn calcular_ventas_totales(
    pool: &SqlitePool,
    fecha_inicio: &str,
    fecha_fin: &str,
) -> Result<f64, String> {
    let row = sqlx::query(
        "SELECT COALESCE(SUM(total), 0) as total FROM ventas WHERE date(fecha) BETWEEN ? AND ? AND estado = 'completada'"
    )
    .bind(fecha_inicio)
    .bind(fecha_fin)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(decode_f64(&row, "total"))
}

async fn calcular_ventas_por_metodo(
    pool: &SqlitePool,
    fecha_inicio: &str,
    fecha_fin: &str,
) -> Result<(f64, f64, f64), String> {
    let rows = sqlx::query(
        "SELECT metodo_pago, COALESCE(SUM(total), 0) as total FROM ventas WHERE date(fecha) BETWEEN ? AND ? AND estado = 'completada' GROUP BY metodo_pago"
    )
    .bind(fecha_inicio)
    .bind(fecha_fin)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut efectivo = 0.0;
    let mut tarjeta = 0.0;
    let mut transferencia = 0.0;

    for row in rows {
        let metodo: String = row.get("metodo_pago");
        let total = decode_f64(&row, "total");
        match metodo.as_str() {
            "efectivo" => efectivo = total,
            "tarjeta" => tarjeta = total,
            "transferencia" => transferencia = total,
            _ => {}
        }
    }
    Ok((efectivo, tarjeta, transferencia))
}

/// Fuente única para el cálculo de utilidad neta: lee las tablas
/// transaccionales y no depende de `resumen_financiero_diario`.
pub(crate) async fn calcular_utilidad_neta_periodo(
    pool: &SqlitePool,
    fecha_inicio: &str,
    fecha_fin: &str,
) -> Result<f64, String> {
    let ventas_totales = calcular_ventas_totales(pool, fecha_inicio, fecha_fin).await?;
    let costo_ventas = calcular_costo_ventas(pool, fecha_inicio, fecha_fin).await?;
    let gastos_operativos = calcular_gastos_operativos(pool, fecha_inicio, fecha_fin).await?;
    let impuestos_comisiones = calcular_impuestos_comisiones(pool, fecha_inicio, fecha_fin).await?;

    Ok(ventas_totales - costo_ventas - gastos_operativos - impuestos_comisiones)
}

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
    let dias_periodo = (NaiveDate::parse_from_str(&fecha_fin, "%Y-%m-%d").unwrap()
        - NaiveDate::parse_from_str(&fecha_inicio, "%Y-%m-%d").unwrap())
    .num_days() as f64;
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
    let diferencia_caja_total = decode_f64(&row, "diff");

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
    .bind(ventas_totales)
    .bind(ventas_efectivo)
    .bind(ventas_tarjeta)
    .bind(ventas_transferencia)
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

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    use sqlx::Executor;

    #[tokio::test]
    async fn utilidad_viva_no_depende_de_resumen_materializado() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();

        pool.execute(
            "CREATE TABLE ventas (id INTEGER PRIMARY KEY, fecha TEXT, total REAL, iva REAL, estado TEXT)",
        )
        .await
        .unwrap();
        pool.execute(
            "CREATE TABLE detalle_ventas (venta_id INTEGER, producto_id INTEGER, cantidad REAL)",
        )
        .await
        .unwrap();
        pool.execute("CREATE TABLE productos (id INTEGER PRIMARY KEY, precio_costo REAL)")
            .await
            .unwrap();
        pool.execute("CREATE TABLE pagos_gastos (fecha_pago TEXT, monto_pagado REAL)")
            .await
            .unwrap();

        sqlx::query("INSERT INTO ventas VALUES (1, '2026-08-10 12:00:00', 100, 16, 'completada')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO productos VALUES (1, 60)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO detalle_ventas VALUES (1, 1, 1)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO pagos_gastos VALUES ('2026-08-10', 50)")
            .execute(&pool)
            .await
            .unwrap();

        // 100 - 60 - 50 - 16 = -26. No se crea resumen_financiero_diario:
        // el cálculo debe depender de las tablas transaccionales vivas.
        let utilidad = calcular_utilidad_neta_periodo(&pool, "2026-08-01", "2026-08-31")
            .await
            .unwrap();
        assert!((utilidad + 26.0).abs() < f64::EPSILON);
    }
}
