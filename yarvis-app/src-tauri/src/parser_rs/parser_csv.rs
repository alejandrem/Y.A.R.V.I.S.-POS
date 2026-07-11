// Parser CSV en Rust (local, sin sidecar).
// Lee archivos .csv y .tsv directamente.
use std::fs;
use crate::models::InventoryItem;
use super::utils::sanitize_path;

// ============================================================
// Parser CSV
// ============================================================

/// Detecta si una línea es CSV y retorna el separador
fn detectar_separador(line: &str) -> Option<char> {
    let comas = line.matches(',').count();
    let puntos_coma = line.matches(';').count();

    if comas >= 2 && comas > puntos_coma {
        Some(',')
    } else if puntos_coma >= 2 && puntos_coma > comas {
        Some(';')
    } else {
        None
    }
}

#[tauri::command]
pub fn parsear_catalogo_csv(path: String) -> Result<Vec<InventoryItem>, String> {
    let safe_path = sanitize_path(&path)?;
    let content = fs::read_to_string(safe_path).map_err(|e| e.to_string())?;
    let mut items = Vec::new();

    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return Ok(items);
    }

    // Detectar separador
    let separador = detectar_separador(lines[0])
        .ok_or("No se pudo detectar separador CSV (use , o ;)")?;

    // Detectar header
    let primera_linea = lines[0].to_lowercase();
    let tiene_header = ["nombre", "producto", "precio", "costo", "venta", "categoria"]
        .iter()
        .any(|p| primera_linea.contains(p));

    let data_start = if tiene_header { 1 } else { 0 };

    for line in &lines[data_start..] {
        let line_trimmed = line.trim();
        if line_trimmed.is_empty() {
            continue;
        }

        let parts: Vec<&str> = if separador == ',' {
            line_trimmed.split(',').collect()
        } else {
            line_trimmed.split(';').collect()
        };

        if parts.len() < 2 {
            continue;
        }

        // Buscar nombre (primera columna no numérica)
        let mut nombre = String::new();
        let mut precio_venta = 0.0;
        let mut precio_costo = 0.0;
        let mut numeric_vals: Vec<(usize, f64)> = Vec::new();

        for (i, part) in parts.iter().enumerate() {
            let clean = part.replace('$', "").replace(',', "").trim().to_string();
            if let Ok(val) = clean.parse::<f64>() {
                numeric_vals.push((i, val));
            } else if nombre.is_empty() && !clean.is_empty() {
                nombre = clean.to_uppercase();
            }
        }

        if nombre.is_empty() || numeric_vals.is_empty() {
            continue;
        }

        // Asignar precios
        if numeric_vals.len() >= 2 {
            let (_, val1) = numeric_vals[0];
            let (_, val2) = numeric_vals[1];
            if val1 > val2 {
                precio_venta = val1;
                precio_costo = val2;
            } else {
                precio_venta = val2;
                precio_costo = val1;
            }
        } else if numeric_vals.len() == 1 {
            precio_venta = numeric_vals[0].1;
        }

        if precio_venta > 0.0 {
            items.push(InventoryItem {
                id: None,
                nombre,
                descripcion: None,
                precio_costo,
                precio_venta,
                stock: 0.0,
                stock_minimo: 5.0,
                vendido: 0.0,
                codigo_barras: None,
                categoria: None,
            });
        }
    }

    Ok(items)
}
