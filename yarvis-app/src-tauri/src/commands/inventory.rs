use sqlx::SqlitePool;
use crate::models::InventoryItem;

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
pub async fn add_inventory_item(state: tauri::State<'_, SqlitePool>, item: InventoryItem) -> Result<i32, String> {
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

    Ok(result.last_insert_rowid() as i32)
}

#[tauri::command]
pub async fn update_inventory_item(state: tauri::State<'_, SqlitePool>, item: InventoryItem) -> Result<(), String> {
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
pub async fn importar_catalogo(state: tauri::State<'_, SqlitePool>, items: Vec<InventoryItem>) -> Result<String, String> {
    for item in items {
        sqlx::query("INSERT INTO productos (nombre, descripcion, precio_costo, precio_venta, stock, stock_minimo, codigo_barras, categoria) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
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
    }
    Ok("Catalogo importado correctamente".into())
}
