use crate::backventanas::auth::AuthState;
use crate::dinero::a_centavos;
use crate::models::{TiendaInfo, VentaRequest, VentaResponse};
use sqlx::SqlitePool;

/// Núcleo de cobro, testeable sin el runtime de Tauri: valida la venta,
/// persiste venta + items y descuenta stock. `cajero` es etiqueta de
/// display; `cajero_id` es la vinculación canónica con usuarios.id.
pub async fn completar_venta_impl(
    pool: &SqlitePool,
    venta: &VentaRequest,
    cajero: String,
    cajero_id: i64,
) -> Result<VentaResponse, String> {
    if venta.items.is_empty() {
        return Err("No hay productos en la venta".into());
    }

    let pagado = venta.monto_efectivo + venta.monto_tarjeta + venta.monto_transferencia;
    // Comparación EXACTA en centavos: con f64 hacía falta un epsilon (+0.01)
    // que a la vez era una ventana para cobros inconsistentes.
    if a_centavos(pagado) < a_centavos(venta.total) {
        return Err("El monto pagado es menor al total".into());
    }

    let metodo_pago = if venta.monto_efectivo > 0.0
        && venta.monto_tarjeta > 0.0
        && venta.monto_transferencia > 0.0
    {
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

    // Vinculación canónica por ID: la sesión ya resolvió quién cobra. El
    // nombre es etiqueta de display; el ID nunca queda huérfano aunque el
    // empleado sea renombrado después.
    //
    // TRANSACCIÓN todo-o-nada: venta + items + descuentos de stock se
    // escriben como una sola unidad. Si CUALQUIER paso falla (crash, luz,
    // producto fantasma), SQLite revierte TODO y no quedan datos a medias
    // (antes un fallo a mitad dejaba venta sin items o stock descuadrado).
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let result = sqlx::query(
        "INSERT INTO ventas (total, subtotal, descuento, metodo_pago, cajero, cajero_id, cliente_id, monto_efectivo, monto_tarjeta, monto_transferencia) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(a_centavos(venta.total))
    .bind(a_centavos(venta.subtotal))
    .bind(a_centavos(venta.descuento))
    .bind(metodo_pago)
    .bind(cajero)
    .bind(cajero_id)
    .bind(venta.cliente_id)
    .bind(a_centavos(venta.monto_efectivo))
    .bind(a_centavos(venta.monto_tarjeta))
    .bind(a_centavos(venta.monto_transferencia))
    .execute(&mut *tx)
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
        .bind(a_centavos(item.precio_venta))
        .bind(a_centavos(item.precio_venta * item.cantidad))
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

        if let Some(producto_id) = item.id {
            // Regla de negocio: jamás vender más stock del disponible. La
            // cláusula `stock >= ?` hace que el UPDATE afecte 0 filas si no
            // alcanza; `rows_affected() == 0` detecta ese caso y aborta la
            // transacción completa (la venta se revierte entera).
            let result = sqlx::query(
                "UPDATE productos SET stock = stock - ?, vendido = vendido + ? WHERE id = ? AND stock >= ?",
            )
            .bind(item.cantidad)
            .bind(item.cantidad)
            .bind(producto_id)
            .bind(item.cantidad)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

            if result.rows_affected() == 0 {
                return Err(format!(
                    "Stock insuficiente para '{}' (disponible menor a {})",
                    item.nombre, item.cantidad
                ));
            }
        }
    }

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(VentaResponse {
        venta_id,
        ticket_number: venta_id,
        mensaje: "Venta completada correctamente".into(),
    })
}


#[tauri::command]
pub async fn completar_venta(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    venta: VentaRequest,
) -> Result<VentaResponse, String> {
    let session = auth.require_operator()?;
    completar_venta_impl(&*state, &venta, session.name.clone(), session.user_id).await
}

#[tauri::command]
pub async fn get_next_ticket_number(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
) -> Result<i64, String> {
    auth.require_operator()?;
    let row: (Option<i64>,) = sqlx::query_as("SELECT MAX(id) FROM ventas")
        .fetch_one(&*state)
        .await
        .map_err(|e| e.to_string())?;

    Ok(row.0.unwrap_or(0) + 1)
}

#[tauri::command]
pub async fn get_tienda_info(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
) -> Result<TiendaInfo, String> {
    auth.require_operator()?;
    let row: (Option<String>, Option<String>, Option<String>) =
        sqlx::query_as("SELECT tienda, ubicacion, cp FROM usuarios WHERE rol = 'admin' LIMIT 1")
            .fetch_one(&*state)
            .await
            .map_err(|e| e.to_string())?;

    Ok(TiendaInfo {
        nombre: row.0,
        ubicacion: row.1,
        cp: row.2,
    })
}
