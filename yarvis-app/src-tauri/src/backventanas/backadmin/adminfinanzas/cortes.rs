use crate::backventanas::auth::AuthState;
use crate::backventanas::backadmin::adminfinanzas::models::*;
use crate::dinero::{a_centavos, a_pesos};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::SqlitePool;

/// Lee una columna monetaria (INTEGER en centavos) y la devuelve en pesos.
fn decode_dinero(row: &sqlx::sqlite::SqliteRow, col: &str) -> f64 {
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

// Crear un corte de caja. El tipo de corte lo define el payload (`tipo_corte`);
// los comandos `crear_corte_x`/`crear_corte_z` lo fijan para que cada botón
// de la UI cree siempre su tipo, pero la lógica real vive en un solo sitio.
async fn crear_corte_impl(
    state: &SqlitePool,
    auth: &tauri::State<'_, AuthState>,
    datos: CrearCorteRequest,
) -> Result<i64, String> {
    let tipo = match datos.tipo_corte.as_str() {
        "X" => "X",
        _ => "Z",
    };
    let usuario_id = auth.require_admin()?.user_id;

    let result = sqlx::query(
        r#"INSERT INTO cortes_caja (monto_inicial, tipo_corte, turno, observaciones, usuario_id, estado)
           VALUES (?, ?, ?, ?, ?, 'abierto')"#
    )
    .bind(a_centavos(datos.monto_inicial))
    .bind(tipo)
    .bind(&datos.turno)
    .bind(&datos.observaciones)
    .bind(usuario_id)
    .execute(state)
    .await
    .map_err(|e| e.to_string())?;

    Ok(result.last_insert_rowid())
}

#[tauri::command]
pub async fn crear_corte_x(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    mut datos: CrearCorteRequest,
) -> Result<i64, String> {
    datos.tipo_corte = "X".to_string();
    crear_corte_impl(&*state, &auth, datos).await
}

#[tauri::command]
pub async fn crear_corte_z(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    mut datos: CrearCorteRequest,
) -> Result<i64, String> {
    datos.tipo_corte = "Z".to_string();
    crear_corte_impl(&*state, &auth, datos).await
}

#[tauri::command]
pub async fn cerrar_corte(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    corte_id: i64,
    total_ventas: f64,
    total_efectivo: f64,
    total_tarjeta: f64,
    total_transferencia: f64,
    entradas_manuales: f64,
    retiros_manuales: f64,
) -> Result<(), String> {
    auth.require_admin()?;
    // Aritmética de cierre EXACTA en centavos: la diferencia de caja es
    // dinero y no debe heredar ruido binario de los f64 del IPC.
    let total_ventas_c = a_centavos(total_ventas);
    let total_efectivo_c = a_centavos(total_efectivo);
    let total_tarjeta_c = a_centavos(total_tarjeta);
    let total_transferencia_c = a_centavos(total_transferencia);
    let entradas_c = a_centavos(entradas_manuales);
    let retiros_c = a_centavos(retiros_manuales);
    let total_calculado_c =
        total_efectivo_c + total_tarjeta_c + total_transferencia_c + entradas_c - retiros_c;
    let diferencia_c = total_calculado_c - total_ventas_c;

    sqlx::query(
        r#"UPDATE cortes_caja SET
           fecha_cierre = datetime('now','localtime'),
           total_ventas = ?,
           total_efectivo = ?,
           total_tarjeta = ?,
           total_transferencia = ?,
           entradas_manuales = ?,
           retiros_manuales = ?,
           diferencia = ?,
           estado = 'cerrado'
           WHERE id = ?"#,
    )
    .bind(total_ventas_c)
    .bind(total_efectivo_c)
    .bind(total_tarjeta_c)
    .bind(total_transferencia_c)
    .bind(entradas_c)
    .bind(retiros_c)
    .bind(diferencia_c)
    .bind(corte_id)
    .execute(&*state)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn agregar_movimiento_caja(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    mov: MovimientoCajaRequest,
) -> Result<i64, String> {
    auth.require_admin()?;
    let result = sqlx::query(
        r#"INSERT INTO movimientos_caja (corte_id, tipo, concepto, monto, metodo_pago, referencia_id)
           VALUES (?, ?, ?, ?, ?, ?)"#
    )
    .bind(mov.corte_id)
    .bind(&mov.tipo)
    .bind(&mov.concepto)
    .bind(a_centavos(mov.monto))
    .bind(&mov.metodo_pago)
    .bind(mov.referencia_id)
    .execute(&*state)
    .await
    .map_err(|e| e.to_string())?;

    Ok(result.last_insert_rowid())
}

#[tauri::command]
pub async fn get_movimientos_corte(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    corte_id: i64,
) -> Result<Vec<MovimientoCaja>, String> {
    auth.require_admin()?;
    let rows = sqlx::query_as::<_, (i64, i64, String, String, i64, Option<String>, Option<i64>, String)>(
        "SELECT id, corte_id, tipo, concepto, monto, metodo_pago, referencia_id, creado_en FROM movimientos_caja WHERE corte_id = ? ORDER BY creado_en ASC"
    )
    .bind(corte_id)
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|row| MovimientoCaja {
            id: row.0,
            corte_id: row.1,
            tipo: row.2,
            concepto: row.3,
            monto: a_pesos(row.4),
            metodo_pago: row.5,
            referencia_id: row.6,
            creado_en: row.7,
        })
        .collect())
}

#[tauri::command]
pub async fn get_cortes_por_cajero_fecha(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    cajero_id: i64,
    fecha_inicio: String,
    fecha_fin: String,
) -> Result<Vec<CorteCaja>, String> {
    auth.require_admin()?;
    let rows = sqlx::query(
        r#"SELECT c.*, u.nombre as usuario_nombre 
           FROM cortes_caja c
           LEFT JOIN usuarios u ON c.usuario_id = u.id
           WHERE c.usuario_id = ? AND date(c.fecha_apertura) BETWEEN ? AND ?
           ORDER BY c.fecha_apertura DESC"#,
    )
    .bind(cajero_id)
    .bind(fecha_inicio)
    .bind(fecha_fin)
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

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
