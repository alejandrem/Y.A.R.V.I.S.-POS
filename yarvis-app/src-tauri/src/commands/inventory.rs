use sqlx::SqlitePool;
use std::sync::Arc;
use crate::models::InventoryItem;
use crate::sidecar::AiSidecar;

/// Texto descriptivo del producto para generar embedding.
/// Combina nombre + descripción + categoría en un solo string.
fn build_embedding_text(item: &InventoryItem) -> String {
    let mut parts = vec![item.nombre.clone()];
    if let Some(ref desc) = item.descripcion {
        if !desc.is_empty() {
            parts.push(desc.clone());
        }
    }
    if let Some(ref cat) = item.categoria {
        if !cat.is_empty() {
            parts.push(cat.clone());
        }
    }
    parts.join(" ")
}

/// Llama a Python para generar un embedding y lo guarda en knowledge_base.
/// Si Python no está disponible, falla silenciosamente (modo clásico).
async fn generate_and_store_embedding(
    pool: &SqlitePool,
    sidecar: &Arc<AiSidecar>,
    item: &InventoryItem,
    _producto_id: i32,
) {
    let base_url = match sidecar.base_url() {
        Some(url) => url,
        None => return, // Motor de IA no disponible, skip
    };

    let texto = build_embedding_text(item);

    // Llamar a Python: POST /generar_embedding
    let client = reqwest::Client::new();
    let payload = serde_json::json!({ "texto": texto });

    let response = match client
        .post(format!("{}/generar_embedding", base_url))
        .json(&payload)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            println!("[YARVIS-INVENTORY] Error llamando a /generar_embedding: {}", e);
            return;
        }
    };

    if !response.status().is_success() {
        println!(
            "[YARVIS-INVENTORY] Python retornó error en /generar_embedding: {}",
            response.status()
        );
        return;
    }

    // Decodificar la respuesta
    let resp: crate::models::EmbeddingResponse = match response.json().await {
        Ok(r) => r,
        Err(e) => {
            println!("[YARVIS-INVENTORY] Error decodificando respuesta: {}", e);
            return;
        }
    };

    // Decodificar base64 → bytes
    use base64::Engine;
    let blob = match base64::engine::general_purpose::STANDARD.decode(&resp.blob_b64) {
        Ok(b) => b,
        Err(e) => {
            println!("[YARVIS-INVENTORY] Error decodificando base64: {}", e);
            return;
        }
    };

    // Insertar en knowledge_base
    let contenido = format!("{} | ${:.2} | stock: {:.0}", item.nombre, item.precio_venta, item.stock);
    let categoria = item.categoria.clone().unwrap_or_else(|| "producto".to_string());

    let result = sqlx::query(
        "INSERT INTO knowledge_base (contenido, categoria, embedding) VALUES (?, ?, ?)"
    )
    .bind(&contenido)
    .bind(&categoria)
    .bind(&blob)
    .execute(pool)
    .await;

    match result {
        Ok(_) => println!(
            "[YARVIS-INVENTORY] Embedding guardado para '{}' ({} dims)",
            item.nombre, resp.dimensions
        ),
        Err(e) => println!("[YARVIS-INVENTORY] Error guardando embedding: {}", e),
    }
}

// ============================================================
// COMMANDS
// ============================================================

#[tauri::command]
pub async fn get_inventory(state: tauri::State<'_, SqlitePool>) -> Result<Vec<InventoryItem>, String> {
    let rows = sqlx::query_as::<_, (Option<i32>, String, Option<String>, f64, f64, f64, f64, Option<String>, Option<String>)>(
        "SELECT id, nombre, descripcion, precio_costo, precio_venta, stock, stock_minimo, codigo_barras, categoria FROM productos"
    )
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    let items = rows.into_iter().map(|row| InventoryItem {
        id: row.0,
        nombre: row.1,
        descripcion: row.2,
        precio_costo: row.3,
        precio_venta: row.4,
        stock: row.5,
        stock_minimo: row.6,
        codigo_barras: row.7,
        categoria: row.8,
    }).collect();

    Ok(items)
}

#[tauri::command]
pub async fn add_inventory_item(
    state: tauri::State<'_, SqlitePool>,
    item: InventoryItem,
    sidecar: tauri::State<'_, Arc<AiSidecar>>,
) -> Result<i32, String> {
    let result = sqlx::query("INSERT INTO productos (nombre, descripcion, precio_costo, precio_venta, stock, stock_minimo, codigo_barras, categoria) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&item.nombre)
        .bind(&item.descripcion)
        .bind(item.precio_costo)
        .bind(item.precio_venta)
        .bind(item.stock)
        .bind(item.stock_minimo)
        .bind(&item.codigo_barras)
        .bind(&item.categoria)
        .execute(&*state)
        .await
        .map_err(|e| e.to_string())?;

    let producto_id = result.last_insert_rowid() as i32;

    // Generar embedding en background (no bloquea la respuesta)
    let pool = (*state).clone();
    let sidecar_arc = (*sidecar).clone();
    let item_clone = item.clone();
    tokio::spawn(async move {
        generate_and_store_embedding(&pool, &sidecar_arc, &item_clone, producto_id).await;
    });

    Ok(producto_id)
}

#[tauri::command]
pub async fn update_inventory_item(
    state: tauri::State<'_, SqlitePool>,
    item: InventoryItem,
    sidecar: tauri::State<'_, Arc<AiSidecar>>,
) -> Result<(), String> {
    if let Some(id) = item.id {
        sqlx::query("UPDATE productos SET nombre = ?, descripcion = ?, precio_costo = ?, precio_venta = ?, stock = ?, stock_minimo = ?, codigo_barras = ?, categoria = ? WHERE id = ?")
            .bind(&item.nombre)
            .bind(&item.descripcion)
            .bind(item.precio_costo)
            .bind(item.precio_venta)
            .bind(item.stock)
            .bind(item.stock_minimo)
            .bind(&item.codigo_barras)
            .bind(&item.categoria)
            .bind(id)
            .execute(&*state)
            .await
            .map_err(|e| e.to_string())?;

        // Regenerar embedding en background
        let pool = (*state).clone();
        let sidecar_arc = (*sidecar).clone();
        let item_clone = item.clone();
        tokio::spawn(async move {
            generate_and_store_embedding(&pool, &sidecar_arc, &item_clone, id).await;
        });

        Ok(())
    } else {
        Err("ID de producto no proporcionado".into())
    }
}

#[tauri::command]
pub async fn delete_inventory_item(state: tauri::State<'_, SqlitePool>, id: i32) -> Result<(), String> {
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
    items: Vec<InventoryItem>,
    sidecar: tauri::State<'_, Arc<AiSidecar>>,
) -> Result<String, String> {
    let mut count = 0;
    for item in items {
        let result = sqlx::query("INSERT INTO productos (nombre, descripcion, precio_costo, precio_venta, stock, stock_minimo, codigo_barras, categoria) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(&item.nombre)
            .bind(&item.descripcion)
            .bind(item.precio_costo)
            .bind(item.precio_venta)
            .bind(item.stock)
            .bind(item.stock_minimo)
            .bind(&item.codigo_barras)
            .bind(&item.categoria)
            .execute(&*state)
            .await;

        if let Ok(r) = result {
            let producto_id = r.last_insert_rowid() as i32;
            // Generar embedding en background
            let pool = (*state).clone();
            let sidecar_arc = (*sidecar).clone();
            let item_clone = item.clone();
            tokio::spawn(async move {
                generate_and_store_embedding(&pool, &sidecar_arc, &item_clone, producto_id).await;
            });
            count += 1;
        }
    }
    Ok(format!("Catálogo importado: {} productos con embeddings", count))
}

// ============================================================
// BÚSQUEDA SEMÁNTICA: /buscar_similar via Python
// ============================================================

#[tauri::command]
pub async fn buscar_producto_similar(
    state: tauri::State<'_, SqlitePool>,
    sidecar: tauri::State<'_, Arc<AiSidecar>>,
    query: String,
    top_k: Option<u32>,
    categoria: Option<String>,
) -> Result<Vec<crate::models::SimilarResult>, String> {
    let base_url = sidecar.base_url()
        .ok_or("Motor de IA no disponible".to_string())?;

    // Obtener la ruta de la DB
    let db_path = get_db_path(&state).await?;

    let payload = crate::models::SimilarSearchRequest {
        query,
        db_path,
        top_k: top_k.unwrap_or(5),
        categoria,
    };

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/buscar_similar", base_url))
        .json(&payload)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Error de conexión con Python: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Python retornó error: {}", response.status()));
    }

    let resp: crate::models::SimilarSearchResponse = response.json().await
        .map_err(|e| format!("Error decodificando respuesta: {}", e))?;

    Ok(resp.results)
}

/// Helper para obtener la ruta de la DB desde el pool
async fn get_db_path(pool: &SqlitePool) -> Result<String, String> {
    let row: (String,) = sqlx::query_as::<_, (String,)>("SELECT file FROM pragma_database_list WHERE name='main'")
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("No se pudo obtener la ruta de la DB".to_string())?;
    Ok(row.0)
}
