use sqlx::SqlitePool;
use crate::backventanas::backadmin::adminfinanzas::models::*;

fn decode_f64(row: &sqlx::sqlite::SqliteRow, col: &str) -> f64 {
    row.try_get::<f64, _>(col)
        .or_else(|_| row.try_get::<i64, _>(col).map(|v| v as f64))
        .unwrap_or(0.0)
}

#[tauri::command]
pub async fn get_cortes_caja(state: tauri::State<'_, SqlitePool>, filtros: FiltrosCortes) -> Result<Vec<CorteCaja>, String> {
    let mut query = String::from(
        r#"SELECT c.*, u.nombre as usuario_nombre 
           FROM cortes_caja c
           LEFT JOIN usuarios u ON c.usuario_id = u.id
           WHERE 1=1"#
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

    Ok(rows.into_iter().map(|row| CorteCaja {
        id: row.get("id"),
        fecha_apertura: row.get("fecha_apertura"),
        fecha_cierre: row.try_get("fecha_cierre").ok(),
        monto_inicial: decode_f64(&row, "monto_inicial"),
        total_ventas: decode_f64(&row, "total_ventas"),
        total_efectivo: decode_f64(&row, "total_efectivo"),
        total_tarjeta: decode_f64(&row, "total_tarjeta"),
        total_transferencia: decode_f64(&row, "total_transferencia"),
        entradas_manuales: decode_f64(&row, "entradas_manuales"),
        retiros_manuales: decode_f64(&row, "retiros_manuales"),
        diferencia: decode_f64(&row, "diferencia"),
        usuario_id: row.get("usuario_id"),
        usuario_nombre: row.get("usuario_nombre"),
        estado: row.get("estado"),
        tipo_corte: row.get("tipo_corte"),
        turno: row.try_get("turno").ok(),
        observaciones: row.try_get("observaciones").ok(),
    }).collect())
}

#[tauri::command]
pub async fn get_corte_detalle(state: tauri::State<'_, SqlitePool>, corte_id: i64) -> Result<CorteDetalle, String> {
    let corte_row = sqlx::query(
        r#"SELECT c.*, u.nombre as usuario_nombre 
           FROM cortes_caja c
           LEFT JOIN usuarios u ON c.usuario_id = u.id
           WHERE c.id = ?"#
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
            monto_inicial: decode_f64(&row, "monto_inicial"),
            total_ventas: decode_f64(&row, "total_ventas"),
            total_efectivo: decode_f64(&row, "total_efectivo"),
            total_tarjeta: decode_f64(&row, "total_tarjeta"),
            total_transferencia: decode_f64(&row, "total_transferencia"),
            entradas_manuales: decode_f64(&row, "entradas_manuales"),
            retiros_manuales: decode_f64(&row, "retiros_manuales"),
            diferencia: decode_f64(&row, "diferencia"),
            usuario_id: row.get("usuario_id"),
            usuario_nombre: row.get("usuario_nombre"),
            estado: row.get("estado"),
            tipo_corte: row.get("tipo_corte"),
            turno: row.try_get("turno").ok(),
            observaciones: row.try_get("observaciones").ok(),
        },
        None => return Err("Corte no encontrado".into()),
    };

    let movimientos = get_movimientos_corte(state.clone(), corte_id).await?;

    let ventas_por_metodo = sqlx::query(
        "SELECT metodo_pago, COALESCE(SUM(total), 0) as total, COUNT(*) as count 
         FROM ventas 
         WHERE fecha BETWEEN ? AND ? AND estado = 'completada'
         GROUP BY metodo_pago"
    )
    .bind(&corte.fecha_apertura)
    .bind(corte.fecha_cierre.as_ref().unwrap_or(&chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()))
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    Ok(CorteDetalle {
        corte,
        movimientos,
        ventas_por_metodo: ventas_por_metodo.into_iter().map(|row| {
            (row.get::<String, _>("metodo_pago"), decode_f64(&row, "total"), row.get::<i64, _>("count"))
        }).collect(),
    })
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CorteDetalle {
    pub corte: CorteCaja,
    pub movimientos: Vec<MovimientoCaja>,
    pub ventas_por_metodo: Vec<(String, f64, i64)>,
}

#[tauri::command]
pub async fn crear_corte_x(
    state: tauri::State<'_, SqlitePool>,
    datos: CrearCorteRequest,
) -> Result<i64, String> {
    let usuario_id = 1; // TODO: obtener del usuario autenticado
    
    let result = sqlx::query(
        r#"INSERT INTO cortes_caja (monto_inicial, tipo_corte, turno, observaciones, usuario_id, estado)
           VALUES (?, 'X', ?, ?, ?, 'abierto')"#
    )
    .bind(datos.monto_inicial)
    .bind(&datos.turno)
    .bind(&datos.observaciones)
    .bind(usuario_id)
    .execute(&*state)
    .await
    .map_err(|e| e.to_string())?;

    Ok(result.last_insert_rowid())
}

#[tauri::command]
pub async fn crear_corte_z(
    state: tauri::State<'_, SqlitePool>,
    datos: CrearCorteRequest,
) -> Result<i64, String> {
    let usuario_id = 1; // TODO: obtener del usuario autenticado
    
    let result = sqlx::query(
        r#"INSERT INTO cortes_caja (monto_inicial, tipo_corte, turno, observaciones, usuario_id, estado)
           VALUES (?, 'Z', ?, ?, ?, 'abierto')"#
    )
    .bind(datos.monto_inicial)
    .bind(&datos.turno)
    .bind(&datos.observaciones)
    .bind(usuario_id)
    .execute(&*state)
    .await
    .map_err(|e| e.to_string())?;

    Ok(result.last_insert_rowid())
}

#[tauri::command]
pub async fn cerrar_corte(
    state: tauri::State<'_, SqlitePool>,
    corte_id: i64,
    total_ventas: f64,
    total_efectivo: f64,
    total_tarjeta: f64,
    total_transferencia: f64,
    entradas_manuales: f64,
    retiros_manuales: f64,
) -> Result<(), String> {
    let total_calculado = total_efectivo + total_tarjeta + total_transferencia + entradas_manuales - retiros_manuales;
    let diferencia = total_calculado - total_ventas;

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
           WHERE id = ?"#
    )
    .bind(total_ventas)
    .bind(total_efectivo)
    .bind(total_tarjeta)
    .bind(total_transferencia)
    .bind(entradas_manuales)
    .bind(retiros_manuales)
    .bind(diferencia)
    .bind(corte_id)
    .execute(&*state)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn agregar_movimiento_caja(
    state: tauri::State<'_, SqlitePool>,
    mov: MovimientoCajaRequest,
) -> Result<i64, String> {
    let result = sqlx::query(
        r#"INSERT INTO movimientos_caja (corte_id, tipo, concepto, monto, metodo_pago, referencia_id)
           VALUES (?, ?, ?, ?, ?, ?)"#
    )
    .bind(mov.corte_id)
    .bind(&mov.tipo)
    .bind(&mov.concepto)
    .bind(mov.monto)
    .bind(&mov.metodo_pago)
    .bind(mov.referencia_id)
    .execute(&*state)
    .await
    .map_err(|e| e.to_string())?;

    Ok(result.last_insert_rowid())
}

#[tauri::command]
pub async fn get_movimientos_corte(state: tauri::State<'_, SqlitePool>, corte_id: i64) -> Result<Vec<MovimientoCaja>, String> {
    let rows = sqlx::query_as::<_, (i64, i64, String, String, f64, Option<String>, Option<i64>, String)>(
        "SELECT id, corte_id, tipo, concepto, monto, metodo_pago, referencia_id, creado_en FROM movimientos_caja WHERE corte_id = ? ORDER BY creado_en ASC"
    )
    .bind(corte_id)
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|row| MovimientoCaja {
        id: row.0,
        corte_id: row.1,
        tipo: row.2,
        concepto: row.3,
        monto: row.4,
        metodo_pago: row.5,
        referencia_id: row.6,
        creado_en: row.7,
    }).collect())
}

#[tauri::command]
pub async fn get_cortes_por_cajero_fecha(
    state: tauri::State<'_, SqlitePool>,
    cajero_id: i64,
    fecha_inicio: String,
    fecha_fin: String,
) -> Result<Vec<CorteCaja>, String> {
    let rows = sqlx::query(
        r#"SELECT c.*, u.nombre as usuario_nombre 
           FROM cortes_caja c
           LEFT JOIN usuarios u ON c.usuario_id = u.id
           WHERE c.usuario_id = ? AND date(c.fecha_apertura) BETWEEN ? AND ?
           ORDER BY c.fecha_apertura DESC"#
    )
    .bind(cajero_id)
    .bind(fecha_inicio)
    .bind(fecha_fin)
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|row| CorteCaja {
        id: row.get("id"),
        fecha_apertura: row.get("fecha_apertura"),
        fecha_cierre: row.try_get("fecha_cierre").ok(),
        monto_inicial: decode_f64(&row, "monto_inicial"),
        total_ventas: decode_f64(&row, "total_ventas"),
        total_efectivo: decode_f64(&row, "total_efectivo"),
        total_tarjeta: decode_f64(&row, "total_tarjeta"),
        total_transferencia: decode_f64(&row, "total_transferencia"),
        entradas_manuales: decode_f64(&row, "entradas_manuales"),
        retiros_manuales: decode_f64(&row, "retiros_manuales"),
        diferencia: decode_f64(&row, "diferencia"),
        usuario_id: row.get("usuario_id"),
        usuario_nombre: row.get("usuario_nombre"),
        estado: row.get("estado"),
        tipo_corte: row.get("tipo_corte"),
        turno: row.try_get("turno").ok(),
        observaciones: row.try_get("observaciones").ok(),
    }).collect())
}