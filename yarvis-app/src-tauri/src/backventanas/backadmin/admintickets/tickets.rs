use crate::backventanas::auth::AuthState;
use crate::backventanas::db::db::DbPath;
use crate::dinero::a_centavos;
use crate::models::{CorteDb, TicketDb, TicketItem};
use sqlx::SqlitePool;
use std::path::PathBuf;

#[tauri::command]
pub async fn get_tickets(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
) -> Result<Vec<TicketDb>, String> {
    auth.require_admin()?;
    let rows = sqlx::query_as::<_, (i32, Option<String>, String, i64, String)>(
        "SELECT id, folio_ticket, strftime('%Y-%m-%d %H:%M:%S', fecha) as fecha, total, metodo_pago FROM ventas ORDER BY fecha DESC LIMIT 500"
    )
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    let tickets = rows
        .into_iter()
        .map(|row| TicketDb {
            id: row.0,
            folio_ticket: row.1,
            fecha: row.2,
            total: crate::dinero::a_pesos(row.3),
            metodo_pago: row.4,
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
    let rows = sqlx::query_as::<_, (i32, String, i64, i64)>(
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
            total_ventas: crate::dinero::a_pesos(row.2),
            total_efectivo: crate::dinero::a_pesos(row.3),
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
    guardar_ticket_parseado_impl(&*state, items, total, fecha, hora, metodo_pago).await
}

/// Núcleo de importación de ticket, testeable sin runtime de Tauri.
pub async fn guardar_ticket_parseado_impl(
    pool: &SqlitePool,
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

    // TRANSACCIÓN todo-o-nada: la venta importada, sus detalles y los
    // ajustes de inventario se escriben como una sola unidad. Si cualquier
    // paso falla, SQLite revierte TODO (antes quedaba venta sin items o
    // stock descuadrado a medias).
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let result = if let Some(ref f_iso) = fecha_iso {
        sqlx::query("INSERT INTO ventas (total, subtotal, cajero, metodo_pago, fecha) VALUES (?, ?, ?, ?, ?)")
            .bind(a_centavos(total))
            .bind(a_centavos(total))
            .bind("IMPORTADOR")
            .bind(&metodo_pago)
            .bind(f_iso)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?
    } else {
        sqlx::query("INSERT INTO ventas (total, subtotal, cajero, metodo_pago) VALUES (?, ?, ?, ?)")
            .bind(a_centavos(total))
            .bind(a_centavos(total))
            .bind("IMPORTADOR")
            .bind(&metodo_pago)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?
    };

    let venta_id = result.last_insert_rowid();
    let total_items = items.len();

    // Items cuyo nombre no coincide con ningún producto del inventario.
    // Es un caso legítimo al importar histórico (el producto ya no existe),
    // pero debe ser VISIBLE en el resultado, nunca silenciado.
    let mut sin_vincular: usize = 0;

    for item in items {
        // Insertar en detalle_ventas
        sqlx::query("INSERT INTO detalle_ventas (venta_id, producto_id, producto_nombre, cantidad, precio_unitario, subtotal) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(venta_id)
            .bind(None::<i32>)
            .bind(&item.producto)
            .bind(item.cantidad)
            .bind(a_centavos(item.precio))
            .bind(a_centavos(item.total))
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

        // Ajustar stock y vendido en UN solo UPDATE. Si el nombre no hace
        // match con el inventario (0 filas afectadas) se contabiliza para
        // reportarlo; ya no se traga el error ni se pierde a medias.
        let actualizados = sqlx::query(
            "UPDATE productos SET stock = stock - ?, vendido = vendido + ? WHERE LOWER(nombre) = LOWER(?)",
        )
        .bind(item.cantidad)
        .bind(item.cantidad)
        .bind(&item.producto)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        if actualizados.rows_affected() == 0 {
            sin_vincular += 1;
        }
    }

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(if sin_vincular > 0 {
        format!(
            "Ticket importado ({}/{} items vinculados al inventario; {} sin coincidencia por nombre)",
            total_items - sin_vincular,
            total_items,
            sin_vincular
        )
    } else {
        "Ticket importado correctamente".into()
    })
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
