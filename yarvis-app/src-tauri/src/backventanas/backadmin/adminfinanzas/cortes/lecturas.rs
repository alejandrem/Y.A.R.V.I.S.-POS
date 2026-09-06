// ============================================================
// lecturas — Consulta de cortes y su detalle completo.
// ============================================================

use super::movimientos::get_movimientos_corte;
use crate::backventanas::auth::AuthState;
use crate::backventanas::backadmin::adminfinanzas::models::*;
use crate::dinero::a_pesos;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::SqlitePool;

/// Lee una columna monetaria (INTEGER en centavos) y la devuelve en pesos.
pub(super) fn decode_dinero(row: &sqlx::sqlite::SqliteRow, col: &str) -> f64 {
    row.try_get::<i64, _>(col)
        .map(a_pesos)
        .unwrap_or(0.0)
}

#[tauri::command]
pub async fn get_cortes_caja(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    filtros: FiltrosCortes,
) -> Result<Vec<CorteCaja>, String> {
    auth.require_admin()?;
    let mut query = String::from(
        r#"SELECT c.*, u.nombre as usuario_nombre 
           FROM cortes_caja c
           LEFT JOIN usuarios u ON c.usuario_id = u.id
           WHERE 1=1"#,
    );
    let mut params: Vec<String> = vec![];

    if let Some(cajero_id) = filtros.cajero_id {
        query.push_str(" AND c.usuario_id = ?");
        params.push(cajero_id.to_string());
    }
    if let Some(fecha_inicio) = filtros.fecha_inicio {
        query.push_str(" AND date(c.fecha_apertura) >= ?");
        params.push(fecha_inicio);
    }
    if let Some(fecha_fin) = filtros.fecha_fin {
        query.push_str(" AND date(c.fecha_apertura) <= ?");
        params.push(fecha_fin);
    }
    if let Some(turno) = filtros.turno {
        query.push_str(" AND c.turno = ?");
        params.push(turno);
    }
    if let Some(tipo_corte) = filtros.tipo_corte {
        query.push_str(" AND c.tipo_corte = ?");
        params.push(tipo_corte);
    }
    if let Some(estado) = filtros.estado {
        query.push_str(" AND c.estado = ?");
        params.push(estado);
    }

    query.push_str(" ORDER BY c.fecha_apertura DESC LIMIT 100");

    let mut q = sqlx::query(&query);
    for param in params {
        q = q.bind(param);
    }

    let rows = q.fetch_all(&*state).await.map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|row| CorteCaja {
            id: row.get("id"),
            fecha_apertura: row.get("fecha_apertura"),
            fecha_cierre: row.try_get("fecha_cierre").ok(),
            monto_inicial: decode_dinero(&row, "monto_inicial"),
            total_ventas: decode_dinero(&row, "total_ventas"),
            total_efectivo: decode_dinero(&row, "total_efectivo"),
            total_tarjeta: decode_dinero(&row, "total_tarjeta"),
            total_transferencia: decode_dinero(&row, "total_transferencia"),
            entradas_manuales: decode_dinero(&row, "entradas_manuales"),
            retiros_manuales: decode_dinero(&row, "retiros_manuales"),
            diferencia: decode_dinero(&row, "diferencia"),
            usuario_id: row.get("usuario_id"),
            usuario_nombre: row.get("usuario_nombre"),
            estado: row.get("estado"),
            tipo_corte: row.get("tipo_corte"),
            turno: row.try_get("turno").ok(),
            observaciones: row.try_get("observaciones").ok(),
        })
        .collect())
}

#[tauri::command]
pub async fn get_corte_detalle(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    corte_id: i64,
) -> Result<CorteDetalle, String> {
    auth.require_admin()?;
    let corte_row = sqlx::query(
        r#"SELECT c.*, u.nombre as usuario_nombre 
           FROM cortes_caja c
           LEFT JOIN usuarios u ON c.usuario_id = u.id
           WHERE c.id = ?"#,
    )
    .bind(corte_id)
    .fetch_optional(&*state)
    .await
    .map_err(|e| e.to_string())?;

    let corte = match corte_row {
        Some(row) => CorteCaja {
            id: row.get("id"),
            fecha_apertura: row.get("fecha_apertura"),
            fecha_cierre: row.try_get("fecha_cierre").ok(),
            monto_inicial: decode_dinero(&row, "monto_inicial"),
            total_ventas: decode_dinero(&row, "total_ventas"),
            total_efectivo: decode_dinero(&row, "total_efectivo"),
            total_tarjeta: decode_dinero(&row, "total_tarjeta"),
            total_transferencia: decode_dinero(&row, "total_transferencia"),
            entradas_manuales: decode_dinero(&row, "entradas_manuales"),
            retiros_manuales: decode_dinero(&row, "retiros_manuales"),
            diferencia: decode_dinero(&row, "diferencia"),
            usuario_id: row.get("usuario_id"),
            usuario_nombre: row.get("usuario_nombre"),
            estado: row.get("estado"),
            tipo_corte: row.get("tipo_corte"),
            turno: row.try_get("turno").ok(),
            observaciones: row.try_get("observaciones").ok(),
        },
        None => return Err("Corte no encontrado".into()),
    };

    let movimientos = get_movimientos_corte(state.clone(), auth, corte_id).await?;

    let ventas_por_metodo = sqlx::query(
        "SELECT metodo_pago, COALESCE(SUM(total), 0) as total, COUNT(*) as count 
         FROM ventas 
         WHERE fecha BETWEEN ? AND ? AND estado = 'completada'
         GROUP BY metodo_pago",
    )
    .bind(&corte.fecha_apertura)
    .bind(
        corte
            .fecha_cierre
            .as_ref()
            .unwrap_or(&chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
    )
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    Ok(CorteDetalle {
        corte,
        movimientos,
        ventas_por_metodo: ventas_por_metodo
            .into_iter()
            .map(|row| {
                (
                    row.get::<String, _>("metodo_pago"),
                    decode_dinero(&row, "total"),
                    row.get::<i64, _>("count"),
                )
            })
            .collect(),
    })
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CorteDetalle {
    pub corte: CorteCaja,
    pub movimientos: Vec<MovimientoCaja>,
    pub ventas_por_metodo: Vec<(String, f64, i64)>,
}
