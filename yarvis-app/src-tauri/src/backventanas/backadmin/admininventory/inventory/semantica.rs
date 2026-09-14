// ============================================================
// semantica — Búsqueda por similitud y re-indexado de embeddings.
// ============================================================
//
// Motor propio (src-ia/embeddings: HashEmbedder 384d por trigramas, sin
// red neuronal ni servicios externos). `buscar_producto_similar` prefiere
// knowledge_base y cae a cálculo al vuelo sobre productos; `backfill`
// puebla el índice (construye todo ANTES de borrar, BUG-01).
//
// Todo lo bloqueante (rusqlite) corre en spawn_blocking para no congelar
// el runtime Tauri.
// ============================================================

use crate::backventanas::auth::AuthState;
use crate::backventanas::db::db::DbPath;
use src_ia::embeddings::{cosine_similarity, embedding_a_blob, HashEmbedder, Embedder};

#[tauri::command]
pub async fn buscar_producto_similar(
    db_path_state: tauri::State<'_, DbPath>,
    auth: tauri::State<'_, AuthState>,
    query: String,
    top_k: Option<u32>,
    categoria: Option<String>,
) -> Result<Vec<crate::models::SimilarResult>, String> {
    auth.require_operator()?;
    let q = query.trim().to_string();
    if q.is_empty() {
        return Ok(vec![]);
    }
    let k = top_k.unwrap_or(5).clamp(1, 20) as usize;
    let cat_filter = categoria.clone();
    let db_path = db_path_state.0.clone();

    // rusqlite es bloqueante -> spawn_blocking para no congelar el runtime Tauri.
    // El open sale de `src_ia::sqlite` (busy_timeout + WAL): sin eso, un cobro
    // por sqlx en el mismo instante truena con `database is locked` (issue #2).
    let result = tokio::task::spawn_blocking(move || {
        let conn = src_ia::sqlite::abrir_db(&db_path).map_err(|e| e.to_string())?;
        let embedder = HashEmbedder;
        let q_emb = embedder
            .texto_a_embedding(&q)
            .ok_or_else(|| "Query vacía tras normalizar".to_string())?;

        // Intentar usar knowledge_base si ya tiene embeddings (backfill previo).
        // Si está vacía, hacemos fallback calculando al vuelo sobre productos.
        let kb_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM knowledge_base WHERE embedding IS NOT NULL",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let mut candidatos: Vec<(i64, String, String, f64)> = Vec::new();

        if kb_count > 0 {
            // IDs reales de productos: validan que producto_id siga existiendo.
            // Si el producto se borró, su fila del índice queda huérfana y se
            // omite (el próximo backfill la limpia). Ver issue #3.
            let mut ids_productos: std::collections::HashSet<i64> =
                std::collections::HashSet::new();
            if let Ok(mut s_ids) = conn.prepare("SELECT id FROM productos") {
                if let Ok(r_ids) = s_ids.query_map([], |r| r.get::<_, i64>(0)) {
                    ids_productos.extend(r_ids.flatten());
                }
            }

            // Mapa legacy nombre -> id: SOLO para filas viejas sin producto_id
            // (previas a la migración 0006). Ver issue #3.
            let mut prod_map: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
            if let Ok(mut s2) = conn.prepare("SELECT id, nombre FROM productos") {
                if let Ok(r2) = s2.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))) {
                    for r in r2.flatten() {
                        prod_map.insert(src_ia::embeddings::normalizar(&r.1), r.0);
                    }
                }
            }

            // Buscar en knowledge_base resolviendo por producto_id (estable
            // ante renombres: el id no cambia aunque cambie el nombre).
            let mut stmt = conn
                .prepare("SELECT producto_id, contenido, categoria, embedding FROM knowledge_base WHERE embedding IS NOT NULL")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, Option<i64>>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Vec<u8>>(3)?,
                    ))
                })
                .map_err(|e| e.to_string())?;

            for row in rows.flatten() {
                let (prod_id, contenido, cat, blob) = row;
                if let Some(ref filtro) = cat_filter {
                    if !cat.eq_ignore_ascii_case(filtro) {
                        continue;
                    }
                }
                let emb = src_ia::embeddings::blob_a_embedding(&blob);
                let score = cosine_similarity(&q_emb, &emb);
                if score < 0.15 {
                    continue;
                }
                let Some(pid) = resolver_pid(prod_id, &ids_productos, &prod_map, &contenido) else {
                    continue;
                };
                candidatos.push((pid, contenido, cat, score));
            }
        }

        // Fallback: si knowledge_base vacía o sin resultados, calcular sobre productos directo
        if candidatos.is_empty() {
            let mut stmt = conn
                .prepare("SELECT id, nombre, COALESCE(categoria,'') FROM productos")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?)))
                .map_err(|e| e.to_string())?;
            for row in rows.flatten() {
                let (id, nombre, cat) = row;
                if let Some(ref filtro) = cat_filter {
                    if !cat.eq_ignore_ascii_case(filtro) {
                        continue;
                    }
                }
                let emb = match embedder.texto_a_embedding(&nombre) {
                    Some(v) => v,
                    None => continue,
                };
                let score = cosine_similarity(&q_emb, &emb);
                if score < 0.15 {
                    continue;
                }
                candidatos.push((id, nombre, cat, score));
            }
        }

        candidatos.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));
        candidatos.truncate(k);

        let out: Vec<crate::models::SimilarResult> = candidatos
            .into_iter()
            .map(|(id, contenido, categoria, score)| crate::models::SimilarResult {
                id,
                contenido,
                categoria,
                score: (score * 10000.0).round() / 10000.0,
            })
            .collect();
        Ok::<_, String>(out)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e: String| e)?;

    Ok(result)
}

// ============================================================
// Resolución de id de producto para una fila del índice.
// ============================================================

/// Resuelve el id real de producto para una fila de knowledge_base.
///
/// - `producto_id` válido (existe en productos) manda: es estable ante
///   renombres porque el id no cambia aunque cambie el nombre.
/// - `producto_id` de un producto ya borrado: se omite (fila huérfana;
///   el próximo backfill la limpia). No se adivina por nombre para no
///   devolver un producto distinto con nombre parecido.
/// - `producto_id` nulo (filas legacy previas a la migración 0006):
///   fallback al mapa por nombre, como antes.
///
/// Función pura para poder probarla sin abrir la DB. Ver issue #3.
fn resolver_pid(
    producto_id: Option<i64>,
    ids_productos: &std::collections::HashSet<i64>,
    prod_map: &std::collections::HashMap<String, i64>,
    contenido: &str,
) -> Option<i64> {
    match producto_id {
        Some(id) => ids_productos.contains(&id).then_some(id),
        None => {
            let nombre = contenido.split('|').next().unwrap_or(contenido).trim();
            prod_map
                .get(&src_ia::embeddings::normalizar(nombre))
                .copied()
        }
    }
}

// ============================================================
// BACKFILL: puebla knowledge_base con embeddings del catálogo
// ============================================================

#[tauri::command]
pub async fn backfill_embeddings(
    db_path_state: tauri::State<'_, DbPath>,
    auth: tauri::State<'_, AuthState>,
) -> Result<serde_json::Value, String> {
    auth.require_admin()?;
    let db_path = db_path_state.0.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = src_ia::sqlite::abrir_db(&db_path).map_err(|e| e.to_string())?;
        let embedder = HashEmbedder;

        let productos: Vec<(i64, String, String)> = {
            let mut stmt = conn
                .prepare("SELECT id, nombre, COALESCE(categoria,'general') FROM productos")
                .map_err(|e| e.to_string())?;
            let x = stmt
                .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
                .map_err(|e| e.to_string())?
                .flatten()
                .collect();
            x
        };

        if productos.is_empty() {
            return Err("No hay productos para indexar".to_string());
        }

        // Construir embeddings primero ANTES de borrar, para no dejar KB vacía si todo falla (BUG-01)
        let mut pending: Vec<(i64, String, String, Vec<u8>)> = Vec::with_capacity(productos.len());
        for (pid, nombre, categoria) in &productos {
            let emb = match embedder.texto_a_embedding(nombre) {
                Some(v) => v,
                None => continue,
            };
            let blob = embedding_a_blob(&emb);
            let contenido = format!("{} | categoria:{}", nombre, categoria);
            pending.push((*pid, contenido, categoria.clone(), blob));
        }

        if pending.is_empty() {
            return Err("No se generaron embeddings (textos vacíos tras normalizar). Índice no modificado.".to_string());
        }

        let tx = conn.transaction().map_err(|e| e.to_string())?;
        tx.execute("DELETE FROM knowledge_base", [])
            .map_err(|e| e.to_string())?;

        let mut inserted = 0usize;
        for (pid, contenido, categoria, blob) in pending {
            // Intentar con producto_id (migración 0006), fallback sin él para DBs viejas sin migrar
            let res = tx.execute(
                "INSERT INTO knowledge_base (contenido, categoria, embedding, producto_id) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![contenido, categoria, blob, pid],
            );
            if res.is_err() {
                tx.execute(
                    "INSERT INTO knowledge_base (contenido, categoria, embedding) VALUES (?1, ?2, ?3)",
                    rusqlite::params![contenido, categoria, blob],
                )
                .map_err(|e| e.to_string())?;
            }
            inserted += 1;
        }
        tx.commit().map_err(|e| e.to_string())?;

        Ok::<_, String>(serde_json::json!({
            "inserted": inserted,
            "total_productos": productos.len(),
            "dim": src_ia::embeddings::DIM,
            "motor": "hash-384-trigram"
        }))
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e: String| e)?;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::resolver_pid;
    use std::collections::{HashMap, HashSet};

    fn ids(ids: &[i64]) -> HashSet<i64> {
        ids.iter().copied().collect()
    }

    fn nombres(pares: &[(&str, i64)]) -> HashMap<String, i64> {
        pares
            .iter()
            .map(|(n, id)| (src_ia::embeddings::normalizar(n), *id))
            .collect()
    }

    #[test]
    fn producto_id_valido_manda_aunque_haya_renombre() {
        // El contenido aún dice el nombre viejo, pero el id es el bueno.
        let r = resolver_pid(
            Some(7),
            &ids(&[7]),
            &nombres(&[("Coca-Cola 600ml retornable", 9)]),
            "Coca 600 | categoria:refrescos",
        );
        assert_eq!(r, Some(7));
    }

    #[test]
    fn producto_id_de_producto_borrado_se_omite() {
        let r = resolver_pid(
            Some(7),
            &ids(&[9]),
            &nombres(&[("Coca 600", 9)]),
            "Coca 600 | categoria:refrescos",
        );
        assert_eq!(r, None, "huérfana: no adivinar por nombre");
    }

    #[test]
    fn fila_legacy_sin_producto_id_usa_nombre() {
        let r = resolver_pid(
            None,
            &ids(&[7]),
            &nombres(&[("Coca 600", 7)]),
            "Coca 600 | categoria:refrescos",
        );
        assert_eq!(r, Some(7));
    }

    #[test]
    fn fila_legacy_sin_match_se_omite() {
        let r = resolver_pid(None, &ids(&[7]), &nombres(&[("Pepsi", 7)]), "Coca 600 | x");
        assert_eq!(r, None);
    }
}
