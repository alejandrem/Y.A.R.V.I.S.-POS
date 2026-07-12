use sqlx::SqlitePool;
use std::sync::Arc;
use sha2::{Sha256, Digest};
use crate::models::InventoryItem;
use crate::sidecar::{AiSidecar, AiStatus};
use crate::backventanas::db::db::DbPath;

/// Calcula SHA256 del contenido del catálogo
fn calcular_hash_catalogo(contenido: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(contenido.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Verifica si un catálogo ya fue importado (por hash)
async fn catalogo_ya_importado(pool: &SqlitePool, hash: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM catalogos_importados WHERE hash = ?"
    )
    .bind(hash)
    .fetch_one(pool)
    .await?;
    Ok(result > 0)
}

/// Registra un catálogo como importado
async fn registrar_catalogo_importado(
    pool: &SqlitePool,
    hash: &str,
    ruta: &str,
    total_productos: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO catalogos_importados (hash, ruta_archivo, total_productos) VALUES (?, ?, ?)"
    )
    .bind(hash)
    .bind(ruta)
    .bind(total_productos)
    .execute(pool)
    .await?;
    Ok(())
}

/// Cuenta cuántos productos con el mismo nombre ya existen en la DB
async fn contar_productos_por_nombre(pool: &SqlitePool, nombre: &str) -> Result<i64, sqlx::Error> {
    let result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM productos WHERE nombre = ?"
    )
    .bind(nombre)
    .fetch_one(pool)
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
) {
    // Verificar que el sidecar esté listo antes de intentar
    sidecar.check_process_alive();
    if sidecar.get_status() != AiStatus::Ready {
        return;
    }

    let base_url = match sidecar.base_url() {
        Some(url) => url,
        None => return,
    };

    let texto = build_embedding_text(item);

    // Llamar a Python: POST /generar_embedding
    let payload = serde_json::json!({ "texto": texto });

    let response = match sidecar.http_client
        .post(format!("{}/generar_embedding", base_url))
        .json(&payload)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
    {
        Ok(r) => r,
        Err(_e) => {
            static EMBEDDING_WARNED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
            if !EMBEDDING_WARNED.swap(true, std::sync::atomic::Ordering::Relaxed) {
                println!("[YARVIS-INVENTORY] Embeddings no disponibles (sidecar no listo). El sistema funciona sin embeddings.");
            }
            return;
        }
    };

    if !response.status().is_success() {
        static EMBEDDING_ERR_WARNED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if !EMBEDDING_ERR_WARNED.swap(true, std::sync::atomic::Ordering::Relaxed) {
            println!(
                "[YARVIS-INVENTORY] Python retornó error en /generar_embedding: {}",
                response.status()
            );
        }
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
    let rows = sqlx::query_as::<_, (Option<i32>, String, Option<String>, f64, f64, f64, f64, f64, Option<String>, Option<String>)>(
        "SELECT id, nombre, descripcion, precio_costo, precio_venta, stock, stock_minimo, vendido, codigo_barras, categoria FROM productos"
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
        vendido: row.7,
        codigo_barras: row.8,
        categoria: row.9,
    }).collect();

    Ok(items)
}

#[tauri::command]
pub async fn add_inventory_item(
    state: tauri::State<'_, SqlitePool>,
    item: InventoryItem,
    sidecar: tauri::State<'_, Arc<AiSidecar>>,
) -> Result<i32, String> {
    let result = sqlx::query("INSERT INTO productos (nombre, descripcion, precio_costo, precio_venta, stock, stock_minimo, vendido, codigo_barras, categoria) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&item.nombre)
        .bind(&item.descripcion)
        .bind(item.precio_costo)
        .bind(item.precio_venta)
        .bind(item.stock)
        .bind(item.stock_minimo)
        .bind(item.vendido)
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
        generate_and_store_embedding(&pool, &sidecar_arc, &item_clone).await;
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
        sqlx::query("UPDATE productos SET nombre = ?, descripcion = ?, precio_costo = ?, precio_venta = ?, stock = ?, stock_minimo = ?, vendido = ?, codigo_barras = ?, categoria = ? WHERE id = ?")
            .bind(&item.nombre)
            .bind(&item.descripcion)
            .bind(item.precio_costo)
            .bind(item.precio_venta)
            .bind(item.stock)
            .bind(item.stock_minimo)
            .bind(item.vendido)
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
            generate_and_store_embedding(&pool, &sidecar_arc, &item_clone).await;
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
    ruta_archivo: Option<String>,
    contenido_archivo: Option<String>,
) -> Result<String, String> {
    // 1. Verificar si el catálogo ya fue importado (por hash)
    if let Some(ref contenido) = contenido_archivo {
        let hash = calcular_hash_catalogo(contenido);
        if catalogo_ya_importado(&*state, &hash).await.map_err(|e| e.to_string())? {
            return Err("Este catálogo ya fue importado anteriormente. No se permiten duplicados.".to_string());
        }
    }

    // 2. Importar productos con deduplicación (máximo 2 con mismo nombre)
    let mut count = 0;
    let mut omitidos = 0;

    for item in items {
        // Verificar cuántos productos con este nombre ya existen
        let existentes = contar_productos_por_nombre(&*state, &item.nombre)
            .await
            .map_err(|e| e.to_string())?;

        if existentes >= 2 {
            // Ya hay 2 o más productos con este nombre → omitir
            omitidos += 1;
            continue;
        }

        // Insertar producto
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

        if let Ok(_r) = result {
            // Generar embedding en background
            let pool = (*state).clone();
            let sidecar_arc = (*sidecar).clone();
            let item_clone = item.clone();
            tokio::spawn(async move {
                generate_and_store_embedding(&pool, &sidecar_arc, &item_clone).await;
            });
            count += 1;
        }
    }

    // 3. Registrar el catálogo como importado
    if let Some(ref contenido) = contenido_archivo {
        let hash = calcular_hash_catalogo(contenido);
        let ruta = ruta_archivo.unwrap_or_default();
        if let Err(e) = registrar_catalogo_importado(&*state, &hash, &ruta, count).await {
            println!("[YARVIS-INVENTORY] Error registrando catálogo: {}", e);
        }
    }

    // 4. Retornar resultado con estadísticas
    let mensaje = if omitidos > 0 {
        format!(
            "Catálogo importado: {} productos insertados, {} omitidos por duplicados (máximo 2 con mismo nombre)",
            count, omitidos
        )
    } else {
        format!("Catálogo importado: {} productos con embeddings", count)
    };

    Ok(mensaje)
}

// ============================================================
// BÚSQUEDA SEMÁNTICA: /buscar_similar via Python
// ============================================================

#[tauri::command]
pub async fn buscar_producto_similar(
    sidecar: tauri::State<'_, Arc<AiSidecar>>,
    db_path_state: tauri::State<'_, DbPath>,
    query: String,
    top_k: Option<u32>,
    categoria: Option<String>,
) -> Result<Vec<crate::models::SimilarResult>, String> {
    sidecar.check_process_alive();
    if sidecar.get_status() != AiStatus::Ready {
        return Err("Motor de IA no está listo".to_string());
    }

    let base_url = sidecar.base_url()
        .ok_or("Motor de IA no disponible".to_string())?;

    let payload = crate::models::SimilarSearchRequest {
        query,
        db_path: db_path_state.0.clone(),
        top_k: top_k.unwrap_or(5),
        categoria,
    };

    let response = sidecar.http_client
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

#[tauri::command]
pub async fn get_catalogos_importados(
    state: tauri::State<'_, SqlitePool>,
) -> Result<Vec<CatalogoImportado>, String> {
    let rows = sqlx::query_as::<_, (i64, String, String, String, i64)>(
        "SELECT id, hash, ruta_archivo, fecha_importacion, total_productos FROM catalogos_importados ORDER BY fecha_importacion DESC"
    )
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    let catalogos = rows.into_iter().map(|row| CatalogoImportado {
        id: row.0,
        hash: row.1,
        ruta_archivo: row.2,
        fecha_importacion: row.3,
        total_productos: row.4,
    }).collect();

    Ok(catalogos)
}

#[tauri::command]
pub async fn get_productos_por_catalogo(
    state: tauri::State<'_, SqlitePool>,
    _catalogo_id: i64,
) -> Result<Vec<InventoryItem>, String> {
    // Por ahora retornamos todos los productos recientes
    let rows = sqlx::query_as::<_, (Option<i32>, String, Option<String>, f64, f64, f64, f64, f64, Option<String>, Option<String>)>(
        "SELECT id, nombre, descripcion, precio_costo, precio_venta, stock, stock_minimo, vendido, codigo_barras, categoria FROM productos ORDER BY creado_en DESC LIMIT 100"
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
        vendido: row.7,
        codigo_barras: row.8,
        categoria: row.9,
    }).collect();

    Ok(items)
}
