// ============================================================
// catalogo — Parseo de catálogos visuales (nativo, sin IA).
// ============================================================

use super::super::utils::sanitize_path;
use crate::backventanas::auth::AuthState;
use std::fs;

#[tauri::command]
pub fn parsear_catalogo_visual(
    auth: tauri::State<'_, AuthState>,
    path: String,
) -> Result<serde_json::Value, String> {
    auth.require_admin()?;
    let safe_path = sanitize_path(&path)?;
    let content = fs::read_to_string(safe_path).map_err(|e| e.to_string())?;

    let productos = src_ia::formatos::lector_txt::parsear_catalogo_visual(&content);
    if productos.is_empty() {
        return Err("No se encontraron productos en el catálogo".to_string());
    }

    let mut categorias: Vec<String> = productos
        .iter()
        .map(|p| p.categoria.clone())
        .filter(|c| !c.is_empty())
        .collect();
    categorias.sort();
    categorias.dedup();

    Ok(serde_json::json!({
        "status": "ok",
        "productos": productos,
        "total": productos.len(),
        "categorias": categorias,
    }))
}
