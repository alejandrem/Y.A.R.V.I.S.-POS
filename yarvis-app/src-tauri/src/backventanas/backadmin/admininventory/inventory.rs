use sqlx::SqlitePool;
use sha2::{Sha256, Digest};
use crate::models::InventoryItem;
use crate::backventanas::db::db::DbPath;
use crate::backventanas::auth::AuthState;

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

// ============================================================
// COMMANDS
// ============================================================

#[tauri::command]
pub async fn get_inventory(state: tauri::State<'_, SqlitePool>, auth: tauri::State<'_, AuthState>) -> Result<Vec<InventoryItem>, String> {
    auth.require_operator()?;
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
    auth: tauri::State<'_, AuthState>,
    item: InventoryItem,
) -> Result<i32, String> {
    auth.require_admin()?;
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
    Ok(producto_id)
}

#[tauri::command]
pub async fn update_inventory_item(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    item: InventoryItem,
) -> Result<(), String> {
    auth.require_admin()?;
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

        Ok(())
    } else {
        Err("ID de producto no proporcionado".into())
    }
}

#[tauri::command]
pub async fn delete_inventory_item(state: tauri::State<'_, SqlitePool>, auth: tauri::State<'_, AuthState>, id: i32) -> Result<(), String> {
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
        format!("Catálogo importado: {} productos", count)
    };

    Ok(mensaje)
}

// ============================================================
// BÚSQUEDA SEMÁNTICA: requiere el motor de IA (desconectado)
// ============================================================

#[tauri::command]
pub async fn buscar_producto_similar(
    _db_path_state: tauri::State<'_, DbPath>,
    auth: tauri::State<'_, AuthState>,
    _query: String,
    _top_k: Option<u32>,
    _categoria: Option<String>,
) -> Result<Vec<crate::models::SimilarResult>, String> {
    auth.require_operator()?;
    Err("Motor de IA no disponible: los embeddings aún no están implementados de forma nativa.".to_string())
}

// ============================================================
// BACKFILL: requiere el motor de IA (desconectado)
// ============================================================

#[tauri::command]
pub async fn backfill_embeddings(
    _db_path_state: tauri::State<'_, DbPath>,
    auth: tauri::State<'_, AuthState>,
) -> Result<serde_json::Value, String> {
    auth.require_admin()?;
    Err("Motor de IA no disponible: los embeddings aún no están implementados de forma nativa.".to_string())
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
    auth: tauri::State<'_, AuthState>,
    _catalogo_id: i64,
) -> Result<Vec<InventoryItem>, String> {
    auth.require_admin()?;
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
