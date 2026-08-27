use crate::backventanas::auth::AuthState;
use crate::backventanas::db::db::DbPath;
use crate::models::InventoryItem;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use src_ia::embeddings::{cosine_similarity, embedding_a_blob, HashEmbedder, Embedder};

/// Calcula SHA256 del contenido del catálogo
fn calcular_hash_catalogo(contenido: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(contenido.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Verifica si un catálogo ya fue importado (por hash).
/// Acepta pool o transacción para poder participar de escrituras atómicas.
async fn catalogo_ya_importado<'a, E>(
    executor: E,
    hash: &str,
) -> Result<bool, sqlx::Error>
where
    E: sqlx::Executor<'a, Database = sqlx::Sqlite>,
{
    let result =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM catalogos_importados WHERE hash = ?")
            .bind(hash)
            .fetch_one(executor)
            .await?;
    Ok(result > 0)
}

/// Registra un catálogo como importado (dentro de la transacción de importación).
async fn registrar_catalogo_importado<'a, E>(
    executor: E,
    hash: &str,
    ruta: &str,
    total_productos: i32,
) -> Result<(), sqlx::Error>
where
    E: sqlx::Executor<'a, Database = sqlx::Sqlite>,
{
    sqlx::query(
        "INSERT INTO catalogos_importados (hash, ruta_archivo, total_productos) VALUES (?, ?, ?)",
    )
    .bind(hash)
    .bind(ruta)
    .bind(total_productos)
    .execute(executor)
    .await?;
    Ok(())
}

/// Cuenta cuántos productos con el mismo nombre ya existen en la DB
async fn contar_productos_por_nombre<'a, E>(
    executor: E,
    nombre: &str,
) -> Result<i64, sqlx::Error>
where
    E: sqlx::Executor<'a, Database = sqlx::Sqlite>,
{
    let result = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM productos WHERE nombre = ?")
        .bind(nombre)
        .fetch_one(executor)
        .await?;
    Ok(result)
}

/// Struct para catálogos importados
#[derive(serde::Serialize)]
pub struct CatalogoImportado {
    pub id: i64,
    pub hash: String,
    pub ruta_archivo: String,
    pub fecha_importacion: String,
    pub total_productos: i64,
}

// ============================================================
// COMMANDS
// ============================================================

#[tauri::command]
pub async fn get_inventory(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
) -> Result<Vec<InventoryItem>, String> {
    auth.require_operator()?;
    let rows = sqlx::query_as::<_, (Option<i32>, String, Option<String>, i64, i64, f64, f64, f64, Option<String>, Option<String>)>(
        "SELECT id, nombre, descripcion, precio_costo, precio_venta, stock, stock_minimo, vendido, codigo_barras, categoria FROM productos"
    )
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    let items = rows
        .into_iter()
        .map(|row| InventoryItem {
            id: row.0,
            nombre: row.1,
            descripcion: row.2,
            precio_costo: crate::dinero::a_pesos(row.3),
            precio_venta: crate::dinero::a_pesos(row.4),
            stock: row.5,
            stock_minimo: row.6,
            vendido: row.7,
            codigo_barras: row.8,
            categoria: row.9,
        })
        .collect();

    Ok(items)
}

#[tauri::command]
/// Núcleo de alta de producto, testeable sin runtime de Tauri.
pub async fn add_inventory_item_impl(pool: &SqlitePool, item: &InventoryItem) -> Result<i32, String> {
    let result = sqlx::query("INSERT INTO productos (nombre, descripcion, precio_costo, precio_venta, stock, stock_minimo, vendido, codigo_barras, categoria) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&item.nombre)
        .bind(&item.descripcion)
        .bind(crate::dinero::a_centavos(item.precio_costo))
        .bind(crate::dinero::a_centavos(item.precio_venta))
        .bind(item.stock)
        .bind(item.stock_minimo)
        .bind(item.vendido)
        .bind(&item.codigo_barras)
        .bind(&item.categoria)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result.last_insert_rowid() as i32)
}

#[tauri::command]
pub async fn add_inventory_item(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    item: InventoryItem,
) -> Result<i32, String> {
    auth.require_admin()?;
    add_inventory_item_impl(&*state, &item).await
}

#[tauri::command]
/// Núcleo de edición de producto, testeable sin runtime de Tauri.
pub async fn update_inventory_item_impl(pool: &SqlitePool, item: &InventoryItem) -> Result<(), String> {
    if let Some(id) = item.id {
        sqlx::query("UPDATE productos SET nombre = ?, descripcion = ?, precio_costo = ?, precio_venta = ?, stock = ?, stock_minimo = ?, vendido = ?, codigo_barras = ?, categoria = ? WHERE id = ?")
            .bind(&item.nombre)
            .bind(&item.descripcion)
            .bind(crate::dinero::a_centavos(item.precio_costo))
            .bind(crate::dinero::a_centavos(item.precio_venta))
            .bind(item.stock)
            .bind(item.stock_minimo)
            .bind(item.vendido)
            .bind(&item.codigo_barras)
            .bind(&item.categoria)
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    } else {
        Err("ID de producto no proporcionado".into())
    }
}

#[tauri::command]
pub async fn update_inventory_item(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    item: InventoryItem,
) -> Result<(), String> {
    auth.require_admin()?;
    update_inventory_item_impl(&*state, &item).await
}

#[tauri::command]
pub async fn delete_inventory_item(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    id: i32,
) -> Result<(), String> {
    auth.require_admin()?;
    sqlx::query("DELETE FROM productos WHERE id = ?")
        .bind(id)
        .execute(&*state)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn importar_catalogo(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    items: Vec<InventoryItem>,
    ruta_archivo: Option<String>,
    contenido_archivo: Option<String>,
) -> Result<String, String> {
    auth.require_admin()?;
    // 1. Verificar si el catálogo ya fue importado (por hash)
    if let Some(ref contenido) = contenido_archivo {
        let hash = calcular_hash_catalogo(contenido);
        if catalogo_ya_importado(&*state, &hash)
            .await
            .map_err(|e| e.to_string())?
        {
            return Err(
                "Este catálogo ya fue importado anteriormente. No se permiten duplicados."
                    .to_string(),
            );
        }
    }

    // 2. Importar productos con deduplicación (máximo 2 con mismo nombre).
    // TRANSACCIÓN todo-o-nada: si CUALQUIER INSERT falla, se revierte la
    // importación completa. Antes los errores de INSERT se tragaban con
    // `if let Ok(_r)` y podían quedar catálogos a medias sin aviso alguno.
    let mut tx = state.begin().await.map_err(|e| e.to_string())?;

    let mut count = 0;
    let mut omitidos = 0;

    for item in items {
        // Verificar cuántos productos con este nombre ya existen
        let existentes = contar_productos_por_nombre(&mut *tx, &item.nombre)
            .await
            .map_err(|e| e.to_string())?;

        if existentes >= 2 {
            // Ya hay 2 o más productos con este nombre → omitir (regla de
            // negocio consciente, se reporta en el mensaje final)
            omitidos += 1;
            continue;
        }

        // Insertar producto — un fallo aquí aborta y revierte TODO
        sqlx::query("INSERT INTO productos (nombre, descripcion, precio_costo, precio_venta, stock, stock_minimo, codigo_barras, categoria) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(&item.nombre)
            .bind(&item.descripcion)
            .bind(crate::dinero::a_centavos(item.precio_costo))
            .bind(crate::dinero::a_centavos(item.precio_venta))
            .bind(item.stock)
            .bind(item.stock_minimo)
            .bind(&item.codigo_barras)
            .bind(&item.categoria)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("Error insertando '{}': {}", item.nombre, e))?;

        count += 1;
    }

    // 3. Registrar el catálogo como importado DENTRO de la misma transacción:
    // si el registro del hash fallara, los productos ya insertados también
    // se revierten (evita re-importaciones duplicadas ante un fallo a medias).
    if let Some(ref contenido) = contenido_archivo {
        let hash = calcular_hash_catalogo(contenido);
        let ruta = ruta_archivo.unwrap_or_default();
        registrar_catalogo_importado(&mut *tx, &hash, &ruta, count)
            .await
            .map_err(|e| e.to_string())?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;

    // 4. Retornar resultado con estadísticas
    let mensaje = if omitidos > 0 {
        format!(
            "Catálogo importado: {} productos insertados, {} omitidos por duplicados (máximo 2 con mismo nombre)",
            count, omitidos
        )
    } else {
        format!("Catálogo importado: {} productos", count)
    };

    Ok(mensaje)
}

// ============================================================
// BÚSQUEDA SEMÁNTICA: motor propio (src-ia/embeddings)
// ============================================================

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
                // contenido es "nombre | ..." -> extraer nombre
                let nombre = contenido.split('|').next().unwrap_or(&contenido).trim();
                let pid = prod_map
                    .get(&src_ia::embeddings::normalizar(nombre))
                    .copied()
                    .unwrap_or(_kb_id);
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

        let tx = conn.transaction().map_err(|e| e.to_string())?;
        // Limpiar index previo para evitar duplicados (contenido no es UNIQUE)
        tx.execute("DELETE FROM knowledge_base", [])
            .map_err(|e| e.to_string())?;

        let mut inserted = 0usize;
        for (_id, nombre, categoria) in &productos {
            let emb = match embedder.texto_a_embedding(nombre) {
                Some(v) => v,
                None => continue,
            };
            let blob = embedding_a_blob(&emb);
            let contenido = format!("{} | categoria:{}", nombre, categoria);
            tx.execute(
                "INSERT INTO knowledge_base (contenido, categoria, embedding) VALUES (?1, ?2, ?3)",
                rusqlite::params![contenido, categoria, blob],
            )
            .map_err(|e| e.to_string())?;
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

#[tauri::command]
pub async fn get_catalogos_importados(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
) -> Result<Vec<CatalogoImportado>, String> {
    auth.require_admin()?;
    let rows = sqlx::query_as::<_, (i64, String, String, String, i64)>(
        "SELECT id, hash, ruta_archivo, fecha_importacion, total_productos FROM catalogos_importados ORDER BY fecha_importacion DESC"
    )
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    let catalogos = rows
        .into_iter()
        .map(|row| CatalogoImportado {
            id: row.0,
            hash: row.1,
            ruta_archivo: row.2,
            fecha_importacion: row.3,
            total_productos: row.4,
        })
        .collect();

    Ok(catalogos)
}

#[tauri::command]
pub async fn get_productos_por_catalogo(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    _catalogo_id: i64,
) -> Result<Vec<InventoryItem>, String> {
    auth.require_admin()?;
    // Por ahora retornamos todos los productos recientes
    let rows = sqlx::query_as::<_, (Option<i32>, String, Option<String>, i64, i64, f64, f64, f64, Option<String>, Option<String>)>(
        "SELECT id, nombre, descripcion, precio_costo, precio_venta, stock, stock_minimo, vendido, codigo_barras, categoria FROM productos ORDER BY creado_en DESC LIMIT 100"
    )
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    let items = rows
        .into_iter()
        .map(|row| InventoryItem {
            id: row.0,
            nombre: row.1,
            descripcion: row.2,
            precio_costo: crate::dinero::a_pesos(row.3),
            precio_venta: crate::dinero::a_pesos(row.4),
            stock: row.5,
            stock_minimo: row.6,
            vendido: row.7,
            codigo_barras: row.8,
            categoria: row.9,
        })
        .collect();

    Ok(items)
}
