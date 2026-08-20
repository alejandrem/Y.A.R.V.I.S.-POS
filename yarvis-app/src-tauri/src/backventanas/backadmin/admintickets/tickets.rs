use crate::backventanas::auth::AuthState;
use crate::backventanas::db::db::DbPath;
use crate::models::{CorteDb, TicketDb, TicketItem};
use sqlx::SqlitePool;
use std::path::PathBuf;

#[tauri::command]
pub async fn get_tickets(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
) -> Result<Vec<TicketDb>, String> {
    auth.require_admin()?;
    let rows = sqlx::query_as::<_, (i32, String, f64, String)>(
        "SELECT id, strftime('%Y-%m-%d %H:%M:%S', fecha) as fecha, total, metodo_pago FROM ventas ORDER BY fecha DESC LIMIT 500"
    )
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    let tickets = rows
        .into_iter()
        .map(|row| TicketDb {
            id: row.0,
            fecha: row.1,
            total: row.2,
            metodo_pago: row.3,
        })
        .collect();

    Ok(tickets)
}

#[tauri::command]
pub async fn get_cortes(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
) -> Result<Vec<CorteDb>, String> {
    auth.require_admin()?;
    let rows = sqlx::query_as::<_, (i32, String, f64, f64)>(
        "SELECT id, strftime('%Y-%m-%d %H:%M:%S', fecha_cierre) as fecha, total_ventas, total_efectivo FROM cortes_caja ORDER BY fecha_cierre DESC LIMIT 500"
    )
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    let cortes = rows
        .into_iter()
        .map(|row| CorteDb {
            id: row.0,
            fecha: row.1,
            total_ventas: row.2,
            total_efectivo: row.3,
        })
        .collect();

    Ok(cortes)
}

// FIX Bug 2: producto_id se guarda como NULL (None) en lugar de 0
// Asi no se viola la foreign key ni causa colision con productos reales
#[tauri::command]
pub async fn guardar_ticket_parseado(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    items: Vec<TicketItem>,
    total: f64,
    fecha: Option<String>,
    hora: Option<String>,
    metodo_pago: Option<String>,
) -> Result<String, String> {
    auth.require_admin()?;
    let metodo_pago = metodo_pago.unwrap_or_else(|| "efectivo".into());
    let fecha_iso = match (fecha, hora) {
        (Some(f), Some(h)) => {
            if !f.is_empty() && !h.is_empty() {
                Some(format!("{} {}:00", f, h))
            } else if !f.is_empty() {
                Some(format!("{} 00:00:00", f))
            } else {
                None
            }
        }
        (Some(f), None) => {
            if !f.is_empty() {
                Some(format!("{} 00:00:00", f))
            } else {
                None
            }
        }
        _ => None,
    };

    let result = if let Some(ref f_iso) = fecha_iso {
        sqlx::query("INSERT INTO ventas (total, subtotal, cajero, metodo_pago, fecha) VALUES (?, ?, ?, ?, ?)")
            .bind(total)
            .bind(total)
            .bind("IMPORTADOR")
            .bind(&metodo_pago)
            .bind(f_iso)
            .execute(&*state)
            .await
            .map_err(|e| e.to_string())?
    } else {
        sqlx::query("INSERT INTO ventas (total, subtotal, cajero, metodo_pago) VALUES (?, ?, ?, ?)")
            .bind(total)
            .bind(total)
            .bind("IMPORTADOR")
            .bind(&metodo_pago)
            .execute(&*state)
            .await
            .map_err(|e| e.to_string())?
    };

    let venta_id = result.last_insert_rowid();

    for item in items {
        // Insertar en detalle_ventas
        sqlx::query("INSERT INTO detalle_ventas (venta_id, producto_id, producto_nombre, cantidad, precio_unitario, subtotal) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(venta_id)
            .bind(None::<i32>)
            .bind(&item.producto)
            .bind(item.cantidad)
            .bind(item.precio)
            .bind(item.total)
            .execute(&*state)
            .await
            .map_err(|e| e.to_string())?;

        // Actualizar stock en productos (decrementar por la cantidad vendida)
        let _ =
            sqlx::query("UPDATE productos SET stock = stock - ? WHERE LOWER(nombre) = LOWER(?)")
                .bind(item.cantidad)
                .bind(&item.producto)
                .execute(&*state)
                .await;

        // Actualizar vendido en productos (incrementar por la cantidad vendida)
        let _ = sqlx::query(
            "UPDATE productos SET vendido = vendido + ? WHERE LOWER(nombre) = LOWER(?)",
        )
        .bind(item.cantidad)
        .bind(&item.producto)
        .execute(&*state)
        .await;
    }

    Ok("Ticket importado correctamente".into())
}

#[tauri::command]
pub async fn get_predictions(
    days: i32,
    db_path: tauri::State<'_, DbPath>,
    auth: tauri::State<'_, AuthState>,
) -> Result<serde_json::Value, String> {
    auth.require_admin()?;

    let horizonte = validar_horizonte_prediccion(days)?;
    let ruta_db = PathBuf::from(db_path.0.clone());
    let data = tokio::task::spawn_blocking(move || {
        src_ia::predicciones::predecir_ventas(&ruta_db, horizonte)
    })
    .await
    .map_err(|e| format!("Falló el cálculo de predicciones: {e}"))??;

    // El frontend existente consume un envoltorio `{ data: [...] }`.
    Ok(serde_json::json!({ "data": data }))
}

fn validar_horizonte_prediccion(days: i32) -> Result<usize, String> {
    if !(1..=365).contains(&days) {
        return Err("El horizonte de predicción debe estar entre 1 y 365 días".to_string());
    }
    Ok(days as usize)
}
