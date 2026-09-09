// ============================================================
// vinculacion — Cruce de ARTICULOS del corte con el catálogo
// maestro (`productos`, el que ya se importó en el paso 01).
//
// Misma maquinaria que tickets (exacto normalizado no ambiguo →
// fuzzy HashEmbedder 0.55/0.52, sin inventar ante duplicados), pero
// con una diferencia documentada: los cortes son cierres de días
// PASADOS, así que NUNCA se descuenta stock (el stock es el presente
// y ya vivió esas ventas). Solo se acumula `vendido` y, si el
// producto no existe y es realmente nuevo, se crea con stock 0
// (igual que el flujo de tickets).
// ============================================================

use sqlx::SqlitePool;
use src_ia::embeddings::{cosine_similarity, normalizar, Embedder, HashEmbedder};
use std::collections::HashMap;

/// (id, nombre) del catálogo para fuzzy.
pub struct ProductoCatalogo {
    pub id: i64,
    pub nombre: String,
}

/// Mapa nombre normalizado → id (None si ambiguo: no se vincula).
pub async fn mapa_exactos(pool: &SqlitePool) -> Result<HashMap<String, Option<i64>>, String> {
    let filas: Vec<(i64, String)> = sqlx::query_as("SELECT id, nombre FROM productos")
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
    let mut acc: HashMap<String, Vec<i64>> = HashMap::new();
    for (id, nombre) in filas {
        acc.entry(normalizar(&nombre)).or_default().push(id);
    }
    Ok(acc
        .into_iter()
        .map(|(k, ids)| (k, if ids.len() == 1 { Some(ids[0]) } else { None }))
        .collect())
}

/// Catálogo mínimo para fuzzy.
pub async fn catalogo_fuzzy(pool: &SqlitePool) -> Result<Vec<ProductoCatalogo>, String> {
    let filas: Vec<(i64, String)> = sqlx::query_as("SELECT id, nombre FROM productos")
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(filas.into_iter().map(|(id, nombre)| ProductoCatalogo { id, nombre }).collect())
}

/// Resuelve un nombre de artículo a producto_id (None = sin vincular).
pub fn resolver_producto_corte(
    nombre: &str,
    exactos: &HashMap<String, Option<i64>>,
    catalogo: &[ProductoCatalogo],
) -> Option<i64> {
    let norm = normalizar(nombre);
    if let Some(Some(id)) = exactos.get(&norm) {
        return Some(*id);
    }
    if exactos.contains_key(&norm) {
        return None; // Ambiguo: no inventar.
    }
    if catalogo.is_empty() {
        return None;
    }
    let embedder = HashEmbedder;
    let q = embedder.texto_a_embedding(nombre)?;
    let mut mejor: Option<(i64, f64)> = None;
    let mut segundo = 0.0f64;
    for p in catalogo {
        let e = match embedder.texto_a_embedding(&p.nombre) {
            Some(v) => v,
            None => continue,
        };
        let s = cosine_similarity(&q, &e);
        if let Some((_, best)) = mejor {
            if s > best {
                segundo = best;
                mejor = Some((p.id, s));
            } else if s > segundo {
                segundo = s;
            }
        } else {
            mejor = Some((p.id, s));
        }
    }
    match mejor {
        Some((id, best)) if best >= 0.55 && segundo < 0.52 => Some(id),
        _ => None,
    }
}

/// ¿Es un producto realmente nuevo (se crea) o un truncado de algo
/// que ya existe? Espejo del flujo de tickets: un token suelto con
/// algo similar (>0.30) no se crea para no poblar fantasmas.
pub fn es_producto_nuevo(nombre: &str, catalogo: &[ProductoCatalogo]) -> bool {
    if nombre.split_whitespace().count() >= 2 || catalogo.is_empty() {
        return true;
    }
    let embedder = HashEmbedder;
    let mut max_score = 0.0f64;
    if let Some(q) = embedder.texto_a_embedding(nombre) {
        for p in catalogo {
            if let Some(e) = embedder.texto_a_embedding(&p.nombre) {
                let s = cosine_similarity(&q, &e);
                if s > max_score {
                    max_score = s;
                }
            }
        }
    }
    max_score <= 0.30
}
