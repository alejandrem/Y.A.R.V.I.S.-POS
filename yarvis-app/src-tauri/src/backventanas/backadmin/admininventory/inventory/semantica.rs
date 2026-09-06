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

    // rusqlite es bloqueante -> spawn_blocking para no congelar el runtime Tauri
    let result = tokio::task::spawn_blocking(move || {
        use rusqlite::Connection;
        let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
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
            // Buscar en knowledge_base y joinear con productos por nombre
            let mut stmt = conn
                .prepare("SELECT id, contenido, categoria, embedding FROM knowledge_base WHERE embedding IS NOT NULL")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, Vec<u8>>(3)?,
                    ))
                })
                .map_err(|e| e.to_string())?;

            // Mapa nombre -> id de productos para resolver id real
            let mut prod_map: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
            if let Ok(mut s2) = conn.prepare("SELECT id, nombre FROM productos") {
                if let Ok(r2) = s2.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))) {
                    for r in r2.flatten() {
                        prod_map.insert(src_ia::embeddings::normalizar(&r.1), r.0);
                    }
                }
            }

            for row in rows.flatten() {
                let (_kb_id, contenido, cat, blob) = row;
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
                // contenido es "nombre | ..." -> extraer nombre. Si no hay match exacto en productos (ej. nombre editado),
                // NO usar fallback _kb_id (id de knowledge_base ≠ id de productos) — mejor descartar y dejar que el fallback de productos lo resuelva.
                let nombre = contenido.split('|').next().unwrap_or(&contenido).trim();
                let Some(pid) = prod_map.get(&src_ia::embeddings::normalizar(nombre)).copied() else {
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
        use rusqlite::Connection;
        let mut conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
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
