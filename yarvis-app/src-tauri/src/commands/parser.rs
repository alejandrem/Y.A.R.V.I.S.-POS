use std::fs;
use crate::models::{TicketItem, InventoryItem};

#[tauri::command]
pub fn leer_archivo_raw(path: String) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| e.to_string())
}

// FIX Bug 4: Parser que respeta campos entre comillas en CSVs
fn split_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in line.chars() {
        if ch == '"' {
            in_quotes = !in_quotes;
        } else if (ch == ',' || ch == ';' || ch == '\t') && !in_quotes {
            if !current.trim().is_empty() || !fields.is_empty() {
                fields.push(current.trim().to_string());
            }
            current.clear();
        } else {
            current.push(ch);
        }
    }

    if !current.is_empty() {
        fields.push(current.trim().to_string());
    }

    fields
}

#[tauri::command]
pub fn parsear_catalogo(path: String) -> Result<Vec<InventoryItem>, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut items = Vec::new();

    for line in content.lines() {
        let line_trimmed = line.trim();
        if line_trimmed.is_empty() {
            continue;
        }

        let parts = split_csv_line(line_trimmed);
        if parts.len() >= 3 {
            let nombre = parts[0].to_uppercase();
            let costo = parts[1].parse::<f64>().unwrap_or(0.0);
            let venta = parts[2].parse::<f64>().unwrap_or(0.0);
            let stock = if parts.len() > 3 {
                parts[3].parse::<f64>().unwrap_or(0.0)
            } else {
                0.0
            };

            if !nombre.is_empty() && venta > 0.0 {
                items.push(InventoryItem {
                    id: None,
                    nombre,
                    descripcion: None,
                    precio_costo: costo,
                    precio_venta: venta,
                    stock,
                    stock_minimo: 5.0,
                    codigo_barras: None,
                    categoria: None,
                });
            }
        }
    }
    Ok(items)
}

#[tauri::command]
pub fn parsear_ticket(path: String) -> Result<Vec<TicketItem>, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut items = Vec::new();

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            if let Ok(cantidad) = parts[0].parse::<f64>() {
                if cantidad > 0.0 {
                    let total_str = parts.last().unwrap().replace('$', "").replace(',', "");
                    let precio_str = parts[parts.len()-2].replace('$', "").replace(',', "");

                    let total = total_str.parse::<f64>().unwrap_or(0.0);
                    let precio = precio_str.parse::<f64>().unwrap_or(0.0);

                    let producto = parts[1..parts.len()-2].join(" ");

                    if !producto.is_empty() && total > 0.0 {
                        items.push(TicketItem {
                            producto: producto.to_uppercase(),
                            cantidad,
                            precio,
                            total
                        });
                    }
                }
            }
        }
    }
    Ok(items)
}
