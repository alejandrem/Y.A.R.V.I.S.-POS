// Parser Excel en Rust (nativo, sin sidecar Python).
// El port vive en `src-ia::formatos::lector_excel` (calamine), espejo de
// `lector_excel.py` de la migración Python → Rust.
use std::collections::BTreeSet;

/// Parsea catálogo Excel (.xlsx/.xls) - recibe bytes del archivo.
#[tauri::command]
pub fn parsear_excel(archivo: Vec<u8>) -> Result<serde_json::Value, String> {
    let productos = src_ia::formatos::lector_excel::parsear_excel(&archivo)?;
    if productos.is_empty() {
        return Err("No se encontraron productos en el archivo Excel".to_string());
    }

    let categorias: Vec<String> = productos
        .iter()
        .map(|p| p.categoria.clone())
        .filter(|c| !c.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    Ok(serde_json::json!({
        "status": "ok",
        "productos": productos,
        "total": productos.len(),
        "categorias": categorias,
    }))
}