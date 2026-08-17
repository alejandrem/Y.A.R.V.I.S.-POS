// Parser CSV en Rust (nativo, sin sidecar).
// Delega el parseo al crate `src-ia` (port de lector_csv.py).
use std::fs;
use crate::models::InventoryItem;
use super::utils::sanitize_path;

#[tauri::command]
pub fn parsear_catalogo_csv(path: String) -> Result<Vec<InventoryItem>, String> {
    let safe_path = sanitize_path(&path)?;
    let content = fs::read_to_string(safe_path).map_err(|e| e.to_string())?;

    let productos = src_ia::formatos::lector_csv::parsear_csv(&content);
    let items = productos
        .into_iter()
        .map(|p| InventoryItem {
            id: None,
            nombre: p.nombre,
            descripcion: None,
            precio_costo: p.precio_costo,
            precio_venta: p.precio_venta,
            stock: p.stock as f64,
            stock_minimo: 5.0,
            vendido: 0.0,
            codigo_barras: None,
            categoria: Some(p.categoria),
        })
        .collect();

    Ok(items)
}
