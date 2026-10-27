use std::collections::HashMap;

use crate::cerebro::analizador_tickets::Item;
use rusqlite::{params, Connection};

use super::super::items::{a_centavos, round2};

#[allow(clippy::too_many_arguments)]
pub fn insertar_venta_historica(
    conn: &Connection,
    items: &[Item],
    cajero: &str,
    fecha: Option<&str>,
    metodo: &str,
    folio: Option<&str>,
    subtotal: f64,
    iva: f64,
    total: f64,
    productos: &HashMap<String, Option<i64>>,
) -> Result<i64, rusqlite::Error> {
    let id = insertar_cabecera(conn, cajero, fecha, metodo, folio, subtotal, iva, total)?;
    for item in items {
        insertar_detalle_historico(conn, id, item, productos)?;
    }
    Ok(id)
}

fn insertar_cabecera(
    conn: &Connection,
    cajero: &str,
    fecha: Option<&str>,
    metodo: &str,
    folio: Option<&str>,
    subtotal: f64,
    iva: f64,
    total: f64,
) -> Result<i64, rusqlite::Error> {
    let descuento = (subtotal + iva - total).max(0.0);
    conn.execute(
        "INSERT INTO ventas (total, subtotal, iva, descuento, cajero, metodo_pago, estado, fecha, folio_ticket)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'completada', ?7, ?8)",
        params![
            a_centavos(total), a_centavos(subtotal), a_centavos(iva),
            a_centavos(descuento), cajero, metodo,
            fecha.map(str::to_owned).unwrap_or_else(super::ahora_iso_utc), folio
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

fn insertar_detalle_historico(
    conn: &Connection,
    venta_id: i64,
    item: &Item,
    productos: &HashMap<String, Option<i64>>,
) -> Result<(), rusqlite::Error> {
    let normalizado = crate::cerebro::vinculador_inventario::normalizar(&item.producto);
    let producto_id = productos.get(&normalizado).copied().flatten();
    let descuento = item.descuento.unwrap_or(0.0);
    let subtotal = round2(item.cantidad * item.precio_unitario - descuento);
    guardar_detalle(conn, venta_id, item, producto_id, descuento, subtotal)?;
    if let Some(id) = producto_id {
        conn.execute(
            "UPDATE productos SET vendido = vendido + ?1 WHERE id = ?2",
            params![item.cantidad, id],
        )?;
    }
    Ok(())
}

fn guardar_detalle(
    conn: &Connection,
    venta_id: i64,
    item: &Item,
    producto_id: Option<i64>,
    descuento: f64,
    subtotal: f64,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO detalle_ventas (venta_id, producto_id, producto_nombre, cantidad, precio_unitario, descuento, subtotal)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            venta_id, producto_id, item.producto, item.cantidad,
            a_centavos(item.precio_unitario), a_centavos(descuento), a_centavos(subtotal)
        ],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests;
