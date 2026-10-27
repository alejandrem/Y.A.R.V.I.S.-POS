// Devoluciones de ticket completo: consulta operator-scoped y transacción
// que cambia el estado de la venta, repone inventario y registra el egreso.
use crate::backventanas::auth::AuthState;
use crate::dinero::a_pesos;
use chrono::NaiveDate;
use serde::Serialize;
use sqlx::SqlitePool;

const LIMITE_BUSQUEDA: i64 = 100;

#[derive(Debug, Clone, Serialize)]
pub struct DevolucionRegistrada {
    pub id: i64,
    pub venta_id: i64,
    pub folio_ticket: String,
    pub fecha_ticket: String,
    pub creado_en: String,
    pub monto: f64,
    pub empleado_nombre: String,
    pub estado_movimiento: String,
}

#[tauri::command]
pub async fn get_mis_devoluciones(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
) -> Result<Vec<DevolucionRegistrada>, String> {
    let sesion = auth.require_operator()?;
    listar_devoluciones_impl(&state, Some(sesion.user_id)).await
}

#[tauri::command]
pub async fn get_devoluciones_empleado(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    empleado_id: i64,
) -> Result<Vec<DevolucionRegistrada>, String> {
    auth.require_admin()?;
    listar_devoluciones_impl(&state, Some(empleado_id)).await
}

async fn listar_devoluciones_impl(
    pool: &SqlitePool,
    empleado_id: Option<i64>,
) -> Result<Vec<DevolucionRegistrada>, String> {
    let filas = sqlx::query_as::<_, (i64, i64, Option<String>, String, String, i64, String, String)>(
        r#"SELECT d.id, d.venta_id, v.folio_ticket,
                  strftime('%Y-%m-%d %H:%M:%S', v.fecha),
                  strftime('%Y-%m-%d %H:%M:%S', d.creado_en), d.monto,
                  COALESCE(u.nombre, 'Empleado'), d.estado_movimiento
           FROM devoluciones d
           JOIN ventas v ON v.id = d.venta_id
           LEFT JOIN usuarios u ON u.id = d.empleado_id
           WHERE (?1 IS NULL OR d.empleado_id = ?1)
           ORDER BY d.creado_en DESC, d.id DESC"#,
    )
    .bind(empleado_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(filas.into_iter().map(|f| DevolucionRegistrada {
        id: f.0,
        venta_id: f.1,
        folio_ticket: f.2.unwrap_or_else(|| format!("TICKET-{:04}", f.1)),
        fecha_ticket: f.3,
        creado_en: f.4,
        monto: a_pesos(f.5),
        empleado_nombre: f.6,
        estado_movimiento: f.7,
    }).collect())
}

#[derive(Debug, Clone, Serialize)]
pub struct TicketDevolucion {
    pub id: i64,
    pub folio_ticket: Option<String>,
    pub fecha: String,
    pub total: f64,
    pub metodo_pago: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ItemTicketDevolucion {
    pub producto_nombre: String,
    pub cantidad: f64,
    pub precio_unitario: f64,
    pub subtotal: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DetalleTicketDevolucion {
    pub id: i64,
    pub folio_ticket: Option<String>,
    pub fecha: String,
    pub total: f64,
    pub metodo_pago: String,
    pub items: Vec<ItemTicketDevolucion>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResultadoDevolucion {
    pub devolucion_id: i64,
    pub venta_id: i64,
    pub folio_ticket: String,
    pub monto: f64,
    pub egreso_registrado: bool,
}

fn fecha_de_busqueda(termino: &str) -> Option<String> {
    ["%m/%d/%y", "%m/%d/%Y", "%Y-%m-%d"]
        .iter()
        .find_map(|formato| NaiveDate::parse_from_str(termino.trim(), formato).ok())
        .map(|fecha| fecha.format("%Y-%m-%d").to_string())
}

fn id_de_folio(termino: &str) -> Option<i64> {
    let limpio = termino.trim();
    let sin_prefijo = limpio
        .strip_prefix("TICKET-")
        .or_else(|| limpio.strip_prefix("ticket-"))
        .or_else(|| limpio.strip_prefix('#'))
        .unwrap_or(limpio);
    sin_prefijo.parse::<i64>().ok()
}

#[tauri::command]
pub async fn buscar_tickets_devolucion(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    termino: String,
) -> Result<Vec<TicketDevolucion>, String> {
    auth.require_operator()?;
    buscar_tickets_devolucion_impl(&state, &termino).await
}

pub async fn buscar_tickets_devolucion_impl(
    pool: &SqlitePool,
    termino: &str,
) -> Result<Vec<TicketDevolucion>, String> {
    let termino = termino.trim();
    let recientes = termino.is_empty();
    let fecha = fecha_de_busqueda(termino);
    let patron = format!("%{termino}%");
    let id = id_de_folio(termino);
    let filas = sqlx::query_as::<_, (i64, Option<String>, String, i64, String)>(
        r#"SELECT id, folio_ticket, strftime('%Y-%m-%d %H:%M:%S', fecha), total, metodo_pago
           FROM ventas
           WHERE estado = 'completada'
             AND ((?1 = 1 AND 1 = 1)
               OR (?1 = 0 AND ?2 IS NOT NULL AND date(fecha) = ?2)
               OR (?1 = 0 AND ?2 IS NULL AND (
                    COALESCE(folio_ticket, '') LIKE ?3 COLLATE NOCASE
                    OR CAST(id AS TEXT) LIKE ?3
                    OR ('TICKET-' || printf('%04d', id)) LIKE ?3 COLLATE NOCASE
                    OR (?4 IS NOT NULL AND id = ?4)
               )))
           ORDER BY fecha DESC, id DESC LIMIT ?5"#,
    )
    .bind(recientes)
    .bind(fecha)
    .bind(patron)
    .bind(id)
    .bind(LIMITE_BUSQUEDA)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(filas
        .into_iter()
        .map(|(id, folio_ticket, fecha, total, metodo_pago)| TicketDevolucion {
            id,
            folio_ticket,
            fecha,
            total: a_pesos(total),
            metodo_pago,
        })
        .collect())
}

#[tauri::command]
pub async fn get_ticket_devolucion(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    venta_id: i64,
) -> Result<DetalleTicketDevolucion, String> {
    auth.require_operator()?;
    get_ticket_devolucion_impl(&state, venta_id).await
}

pub async fn get_ticket_devolucion_impl(
    pool: &SqlitePool,
    venta_id: i64,
) -> Result<DetalleTicketDevolucion, String> {
    let cabecera = sqlx::query_as::<_, (i64, Option<String>, String, i64, String)>(
        "SELECT id, folio_ticket, strftime('%Y-%m-%d %H:%M:%S', fecha), total, metodo_pago FROM ventas WHERE id = ? AND estado = 'completada'",
    )
    .bind(venta_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "El ticket no existe o ya fue devuelto".to_string())?;

    let filas = sqlx::query_as::<_, (String, f64, i64, i64)>(
        "SELECT producto_nombre, cantidad, precio_unitario, subtotal FROM detalle_ventas WHERE venta_id = ? ORDER BY id ASC",
    )
    .bind(venta_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(DetalleTicketDevolucion {
        id: cabecera.0,
        folio_ticket: cabecera.1,
        fecha: cabecera.2,
        total: a_pesos(cabecera.3),
        metodo_pago: cabecera.4,
        items: filas
            .into_iter()
            .map(|(producto_nombre, cantidad, precio_unitario, subtotal)| ItemTicketDevolucion {
                producto_nombre,
                cantidad,
                precio_unitario: a_pesos(precio_unitario),
                subtotal: a_pesos(subtotal),
            })
            .collect(),
    })
}

#[tauri::command]
pub async fn devolver_ticket_completo(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    venta_id: i64,
) -> Result<ResultadoDevolucion, String> {
    let sesion = auth.require_operator()?;
    devolver_ticket_completo_impl(&state, venta_id, sesion.user_id).await
}

pub async fn devolver_ticket_completo_impl(
    pool: &SqlitePool,
    venta_id: i64,
    empleado_id: i64,
) -> Result<ResultadoDevolucion, String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let venta = sqlx::query_as::<_, (Option<String>, i64)>(
        "SELECT folio_ticket, total FROM ventas WHERE id = ? AND estado = 'completada'",
    )
    .bind(venta_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "El ticket no existe o ya fue devuelto".to_string())?;

    // Cambio condicional: ante dos intentos concurrentes solo uno puede ganar.
    let cambio = sqlx::query("UPDATE ventas SET estado = 'devuelta' WHERE id = ? AND estado = 'completada'")
        .bind(venta_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    if cambio.rows_affected() != 1 {
        return Err("El ticket ya fue devuelto. Actualiza el historial e intenta con otro ticket.".into());
    }

    // Regresa al inventario todo lo vendido que estaba vinculado a producto.
    sqlx::query(
        r#"UPDATE productos
           SET stock = stock + (
                 SELECT COALESCE(SUM(d.cantidad), 0) FROM detalle_ventas d
                 WHERE d.venta_id = ? AND d.producto_id = productos.id
               ),
               vendido = MAX(0, vendido - (
                 SELECT COALESCE(SUM(d.cantidad), 0) FROM detalle_ventas d
                 WHERE d.venta_id = ? AND d.producto_id = productos.id
               ))
           WHERE id IN (SELECT producto_id FROM detalle_ventas WHERE venta_id = ? AND producto_id IS NOT NULL)"#,
    )
    .bind(venta_id)
    .bind(venta_id)
    .bind(venta_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    let folio = venta
        .0
        .unwrap_or_else(|| format!("TICKET-{:04}", venta_id));
    let devolucion_id = sqlx::query(
        "INSERT INTO devoluciones (venta_id, empleado_id, monto, metodo_pago) VALUES (?, ?, ?, 'efectivo')",
    )
    .bind(venta_id)
    .bind(empleado_id)
    .bind(venta.1)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?
    .last_insert_rowid();

    let corte_abierto = sqlx::query_scalar::<_, i64>(
        "SELECT id FROM cortes_caja WHERE usuario_id = ? AND estado = 'abierto' ORDER BY fecha_apertura DESC LIMIT 1",
    )
    .bind(empleado_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    let egreso_registrado = if let Some(corte_id) = corte_abierto {
        let concepto = format!("Devolución {folio}");
        let movimiento_id = sqlx::query(
            "INSERT INTO movimientos_caja (corte_id, tipo, concepto, monto, metodo_pago, referencia_id) VALUES (?, 'retiro', ?, ?, 'efectivo', ?)",
        )
        .bind(corte_id)
        .bind(concepto)
        .bind(venta.1)
        .bind(devolucion_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?
        .last_insert_rowid();
        sqlx::query("UPDATE devoluciones SET corte_id = ?, movimiento_id = ?, estado_movimiento = 'registrado' WHERE id = ?")
            .bind(corte_id)
            .bind(movimiento_id)
            .bind(devolucion_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
        true
    } else {
        false
    };

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(ResultadoDevolucion {
        devolucion_id,
        venta_id,
        folio_ticket: folio,
        monto: a_pesos(venta.1),
        egreso_registrado,
    })
}
