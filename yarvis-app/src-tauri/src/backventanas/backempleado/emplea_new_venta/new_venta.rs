use sqlx::SqlitePool;
use crate::models::{VentaRequest, VentaResponse, TiendaInfo};

#[tauri::command]
pub async fn completar_venta(
    state: tauri::State<'_, SqlitePool>,
    venta: VentaRequest,
) -> Result<VentaResponse, String> {
    if venta.items.is_empty() {
        return Err("No hay productos en la venta".into());
    }

    let pagado = venta.monto_efectivo + venta.monto_tarjeta + venta.monto_transferencia;
    if pagado + 0.01 < venta.total {
        return Err("El monto pagado es menor al total".into());
    }

    let metodo_pago = if venta.monto_efectivo > 0.0 && venta.monto_tarjeta > 0.0 && venta.monto_transferencia > 0.0 {
        "mixto"
    } else if venta.monto_efectivo > 0.0 && venta.monto_tarjeta > 0.0 {
        "efectivo/tarjeta"
    } else if venta.monto_efectivo > 0.0 && venta.monto_transferencia > 0.0 {
        "efectivo/transferencia"
    } else if venta.monto_tarjeta > 0.0 && venta.monto_transferencia > 0.0 {
        "tarjeta/transferencia"
    } else if venta.monto_tarjeta > 0.0 {
        "tarjeta"
    } else if venta.monto_transferencia > 0.0 {
        "transferencia"
    } else {
        "efectivo"
    };

    let result = sqlx::query(
        "INSERT INTO ventas (total, subtotal, descuento, metodo_pago, cajero, cliente_id, monto_efectivo, monto_tarjeta, monto_transferencia) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(venta.total)
    .bind(venta.subtotal)
    .bind(venta.descuento)
    .bind(metodo_pago)
    .bind(&venta.cajero)
    .bind(venta.cliente_id)
    .bind(venta.monto_efectivo)
    .bind(venta.monto_tarjeta)
    .bind(venta.monto_transferencia)
    .execute(&*state)
    .await
    .map_err(|e| e.to_string())?;

    let venta_id = result.last_insert_rowid();

    for item in &venta.items {
        sqlx::query(
            "INSERT INTO detalle_ventas (venta_id, producto_id, producto_nombre, cantidad, precio_unitario, subtotal) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(venta_id)
        .bind(item.id)
        .bind(&item.nombre)
        .bind(item.cantidad)
        .bind(item.precio_venta)
        .bind(item.precio_venta * item.cantidad)
        .execute(&*state)
        .await
        .map_err(|e| e.to_string())?;

        if let Some(producto_id) = item.id {
            let _ = sqlx::query("UPDATE productos SET stock = stock - ?, vendido = vendido + ? WHERE id = ?")
                .bind(item.cantidad)
                .bind(item.cantidad)
                .bind(producto_id)
                .execute(&*state)
                .await;
        }
    }

    Ok(VentaResponse {
        venta_id,
        ticket_number: venta_id,
        mensaje: "Venta completada correctamente".into(),
    })
}

#[tauri::command]
pub async fn get_next_ticket_number(
    state: tauri::State<'_, SqlitePool>,
) -> Result<i64, String> {
    let row: (Option<i64>,) = sqlx::query_as("SELECT MAX(id) FROM ventas")
        .fetch_one(&*state)
        .await
        .map_err(|e| e.to_string())?;

    Ok(row.0.unwrap_or(0) + 1)
}

#[tauri::command]
pub async fn get_tienda_info(
    state: tauri::State<'_, SqlitePool>,
) -> Result<TiendaInfo, String> {
    let row: (Option<String>, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT tienda, ubicacion, cp FROM usuarios WHERE rol = 'admin' LIMIT 1"
    )
    .fetch_one(&*state)
    .await
    .map_err(|e| e.to_string())?;

    Ok(TiendaInfo {
        nombre: row.0,
        ubicacion: row.1,
        cp: row.2,
    })
}
