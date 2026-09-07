// ============================================================
// componentes — Piezas de cálculo reutilizables (pesos, costos,
// gastos, impuestos, ventas). Fuente única de la utilidad neta.
// ============================================================

use crate::dinero::{a_pesos, centavos_f64_a_i64};
use sqlx::{Row, SqlitePool};

/// Lee una columna monetaria y la devuelve EN PESOS. Las columnas INTEGER
/// vienen en centavos (i64); los agregados que mezclan cantidad REAL ×
/// precio en centavos llegan como f64 en unidades de centavos y se
/// redondean al centavo exacto antes de convertir.
pub(crate) fn decode_dinero(row: &sqlx::sqlite::SqliteRow, col: &str) -> f64 {
    match row.try_get::<i64, _>(col) {
        Ok(v) => a_pesos(v),
        Err(_) => row
            .try_get::<f64, _>(col)
            .map(|v| a_pesos(centavos_f64_a_i64(v)))
            .unwrap_or(0.0),
    }
}

pub(crate) async fn calcular_costo_ventas(
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
    Ok(decode_dinero(&row, "cogs"))
}

pub(crate) async fn calcular_gastos_operativos(
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
    Ok(decode_dinero(&row, "total_gastos"))
}

pub(crate) async fn calcular_impuestos_comisiones(
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
    Ok(decode_dinero(&row, "total_iva"))
}

pub(crate) async fn calcular_ventas_totales(
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
    Ok(decode_dinero(&row, "total"))
}

pub(crate) async fn calcular_ventas_por_metodo(
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
        let total = decode_dinero(&row, "total");
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
