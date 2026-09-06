// ============================================================
// historial — Catálogos importados y productos por catálogo.
// ============================================================
//
// Lecturas del historial de importaciones para la UI (qué tablas maestras
// ya entraron y qué trajo cada una, top 100 recientes).

use super::catalogo::CatalogoImportado;
use crate::backventanas::auth::AuthState;
use crate::models::InventoryItem;
use sqlx::SqlitePool;

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
    catalogo_id: i64,
) -> Result<Vec<InventoryItem>, String> {
    auth.require_admin()?;
    let rows = if catalogo_id > 0 {
        sqlx::query_as::<_, (Option<i32>, String, Option<String>, i64, i64, f64, f64, f64, Option<String>, Option<String>)>(
            "SELECT id, nombre, descripcion, precio_costo, precio_venta, stock, stock_minimo, vendido, codigo_barras, categoria FROM productos WHERE catalogo_id = ? ORDER BY creado_en DESC LIMIT 100"
        )
        .bind(catalogo_id)
        .fetch_all(&*state)
        .await
        .map_err(|e| e.to_string())?
    } else {
        sqlx::query_as::<_, (Option<i32>, String, Option<String>, i64, i64, f64, f64, f64, Option<String>, Option<String>)>(
            "SELECT id, nombre, descripcion, precio_costo, precio_venta, stock, stock_minimo, vendido, codigo_barras, categoria FROM productos ORDER BY creado_en DESC LIMIT 100"
        )
        .fetch_all(&*state)
        .await
        .map_err(|e| e.to_string())?
    };

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
