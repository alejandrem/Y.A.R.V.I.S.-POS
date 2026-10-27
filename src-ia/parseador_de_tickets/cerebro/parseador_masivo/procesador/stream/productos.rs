use std::collections::{HashMap, HashSet};

use super::super::super::almacen::resolver_producto_id;
use crate::cerebro::analizador_tickets::Item;
use crate::cerebro::parseador_masivo::resumen::ProductoNuevo;
use crate::cerebro::vinculador_inventario::ProductoInventario;
use crate::embeddings::Embedder;
use rusqlite::{params, Connection};

pub(super) fn asegurar_producto(
    conn: &Connection,
    item: &Item,
    clave: &str,
    vistos: &mut HashSet<String>,
    por_nombre: &mut HashMap<String, Option<i64>>,
    inventario: &[ProductoInventario],
) -> Result<Option<ProductoNuevo>, rusqlite::Error> {
    if vistos.contains(clave) {
        return Ok(None);
    }
    let nombre = crate::embeddings::normalizar(&item.producto);
    if let Some(id) = resolver_producto_id(&item.producto, por_nombre, inventario) {
        por_nombre.entry(nombre).or_insert(Some(id));
        vistos.insert(clave.to_string());
        return Ok(None);
    }
    if por_nombre.contains_key(&nombre) || !debe_crearse(item, inventario) {
        vistos.insert(clave.to_string());
        return Ok(None);
    }
    crear_producto(conn, item, clave, nombre, vistos, por_nombre)
}

fn debe_crearse(item: &Item, inventario: &[ProductoInventario]) -> bool {
    if item.producto.split_whitespace().count() >= 2 || inventario.is_empty() {
        return true;
    }
    similitud_maxima(&item.producto, inventario) <= 0.30
}

fn similitud_maxima(nombre: &str, inventario: &[ProductoInventario]) -> f64 {
    let embedder = crate::embeddings::HashEmbedder;
    let Some(query) = embedder.texto_a_embedding(nombre) else {
        return 0.0;
    };
    inventario
        .iter()
        .filter_map(|p| embedder.texto_a_embedding(&p.nombre))
        .map(|v| crate::embeddings::cosine_similarity(&query, &v))
        .fold(0.0, f64::max)
}

fn crear_producto(
    conn: &Connection,
    item: &Item,
    clave: &str,
    nombre: String,
    vistos: &mut HashSet<String>,
    por_nombre: &mut HashMap<String, Option<i64>>,
) -> Result<Option<ProductoNuevo>, rusqlite::Error> {
    conn.execute(
        "INSERT INTO productos (nombre, precio_venta, precio_costo, stock, stock_minimo, vendido, categoria)
         VALUES (?1, ?2, ?3, 0, 5, 0, '')",
        params![item.producto, super::super::super::items::a_centavos(item.precio_unitario), 0],
    )?;
    por_nombre.insert(nombre, Some(conn.last_insert_rowid()));
    vistos.insert(clave.to_string());
    Ok(Some(ProductoNuevo {
        nombre: item.producto.clone(),
        precio: item.precio_unitario,
    }))
}

#[cfg(test)]
mod tests;
