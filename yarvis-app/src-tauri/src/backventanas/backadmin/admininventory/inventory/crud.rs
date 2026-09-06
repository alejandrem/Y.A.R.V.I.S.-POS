// ============================================================
// crud — Altas, bajas y cambios de productos del inventario.
// ============================================================
//
// Los núcleos `*_impl` reciben el pool directo para ser testeables sin
// runtime de Tauri; los comandos solo validan rol y delegan.

use crate::backventanas::auth::AuthState;
use crate::models::InventoryItem;
use sqlx::SqlitePool;

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
