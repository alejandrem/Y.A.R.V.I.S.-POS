use rusqlite::Connection;
use std::collections::HashMap;

use super::similitud::{blob_a_embedding, normalizar};

#[derive(Debug, Clone)]
pub struct ProductoInventario {
    pub id: i64,
    pub nombre: String,
    pub nombre_norm: String,
    pub precio_venta: f64,
    pub embedding: Option<Vec<f32>>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProductoDb {
    pub id: i64,
    pub nombre: String,
    pub precio_venta: f64,
}

pub fn cargar_inventario(db_path: &str) -> Vec<ProductoInventario> {
    let mut productos = Vec::new();
    let Ok(conn) = Connection::open(db_path) else {
        return productos;
    };

    // Mapa nombre-normalizado → embedding, desde knowledge_base.
    let mut embeddings_por_nombre: HashMap<String, Vec<f32>> = HashMap::new();
    if let Ok(mut stmt) =
        conn.prepare("SELECT contenido, embedding FROM knowledge_base WHERE embedding IS NOT NULL")
    {
        let rows = stmt.query_map([], |row| {
            let contenido: String = row.get(0)?;
            let blob: Vec<u8> = row.get(1)?;
            Ok((contenido, blob))
        });
        if let Ok(rows) = rows {
            for r in rows.flatten() {
                let (contenido, blob) = r;
                if let Some(nombre_kb) = contenido.split('|').next() {
                    embeddings_por_nombre.insert(normalizar(nombre_kb), blob_a_embedding(&blob));
                }
            }
        }
    }

    if let Ok(mut stmt) = conn.prepare("SELECT id, nombre, precio_venta FROM productos") {
        let rows = stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let nombre: String = row.get(1)?;
            // La DB guarda centavos enteros desde la migración f64→centavos;
            // aquí se expone en PESOS porque los precios de los items del
            // ticket (y el resto del vinculador) viven en pesos.
            let precio_centavos: f64 = row.get(2)?;
            let precio_venta = precio_centavos / 100.0;
            Ok((id, nombre, precio_venta))
        });
        if let Ok(rows) = rows {
            for r in rows.flatten() {
                let (id, nombre, precio_venta) = r;
                let nombre_norm = normalizar(&nombre);
                let embedding = embeddings_por_nombre.get(&nombre_norm).cloned();
                productos.push(ProductoInventario {
                    id,
                    nombre,
                    nombre_norm,
                    precio_venta,
                    embedding,
                });
            }
        }
    }

    productos
}
