use std::sync::Arc;
use sqlx::SqlitePool;
use crate::models::{TicketDb, CorteDb, TicketItem};
use crate::sidecar::{AiSidecar, AiStatus};
use crate::backventanas::db::db::DbPath;

#[tauri::command]
pub async fn get_tickets(state: tauri::State<'_, SqlitePool>) -> Result<Vec<TicketDb>, String> {
    let rows = sqlx::query_as::<_, (i32, String, f64, String)>(
        "SELECT id, strftime('%Y-%m-%d %H:%M:%S', fecha) as fecha, total, metodo_pago FROM ventas ORDER BY fecha DESC LIMIT 500"
    )
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    let tickets = rows.into_iter().map(|row| TicketDb {
        id: row.0,
        fecha: row.1,
        total: row.2,
        metodo_pago: row.3,
    }).collect();

    Ok(tickets)
}

#[tauri::command]
pub async fn get_cortes(state: tauri::State<'_, SqlitePool>) -> Result<Vec<CorteDb>, String> {
    let rows = sqlx::query_as::<_, (i32, String, f64, f64)>(
        "SELECT id, strftime('%Y-%m-%d %H:%M:%S', fecha_cierre) as fecha, total_ventas, total_efectivo FROM cortes_caja ORDER BY fecha_cierre DESC LIMIT 500"
    )
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    let cortes = rows.into_iter().map(|row| CorteDb {
        id: row.0,
        fecha: row.1,
        total_ventas: row.2,
        total_efectivo: row.3,
    }).collect();

    Ok(cortes)
}

// FIX Bug 2: producto_id se guarda como NULL (None) en lugar de 0
// Asi no se viola la foreign key ni causa colision con productos reales
#[tauri::command]
pub async fn guardar_ticket_parseado(
    state: tauri::State<'_, SqlitePool>,
    items: Vec<TicketItem>,
    total: f64,
    fecha: Option<String>,
    hora: Option<String>,
    metodo_pago: Option<String>,
) -> Result<String, String> {
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
        let _ = sqlx::query("UPDATE productos SET stock = stock - ? WHERE LOWER(nombre) = LOWER(?)")
            .bind(item.cantidad)
            .bind(&item.producto)
            .execute(&*state)
            .await;

        // Actualizar vendido en productos (incrementar por la cantidad vendida)
        let _ = sqlx::query("UPDATE productos SET vendido = vendido + ? WHERE LOWER(nombre) = LOWER(?)")
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
    sidecar: tauri::State<'_, Arc<AiSidecar>>,
    db_path: tauri::State<'_, DbPath>,
) -> Result<serde_json::Value, String> {
    sidecar.check_process_alive();
    if sidecar.get_status() != AiStatus::Ready {
        return Err("Motor de IA no está listo".into());
    }

    let base_url = sidecar.base_url().ok_or("Sidecar sin puerto")?;
    let payload = serde_json::json!({ "db_path": db_path.0, "days": days });

    let resp = sidecar
        .http_client
        .post(format!("{}/recalcular_predicciones", base_url))
        .json(&payload)
        .timeout(std::time::Duration::from_secs(300))
        .send()
        .await
        .map_err(|e| format!("Error llamando a Prophet: {}", e))?;

    let result: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Error decodificando respuesta Prophet: {}", e))?;

    Ok(result)
}
