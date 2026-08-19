//! lector_csv.rs — Port de `yarvis-IA/parseador_de_tickets/formatos/lector_csv.py`
//!
//! Parser de catálogos en formato CSV (.csv, .tsv). Detecta separadores
//! automáticamente (, o ;), headers por palabra clave, stock/categoría por
//! nombre de columna y precio mayor → venta, menor → costo.

use super::ProductoCatalogo;
use crate::cerebro::analizador_tickets::PRECIO_MAXIMO;
use crate::cerebro::filtrador::limpiar_producto;

const PALABRAS_HEADER: &[&str] = &[
    "nombre",
    "producto",
    "precio",
    "costo",
    "venta",
    "categoria",
    "stock",
    "cantidad",
    "existencia",
];

const PALABRAS_STOCK: &[&str] = &["stock", "existencia", "cantidad", "inventario", "qty", "quantity"];

const PALABRAS_CATEGORIA: &[&str] = &[
    "categoría",
    "categoria",
    "tipo",
    "seccion",
    "sección",
    "departamento",
    "category",
    "type",
];

const SIN_CATEGORIA: &str = "SIN CATEGORÍA";

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

/// Detecta si una línea es CSV y retorna el separador (`,` o `;`).
pub fn detectar_separador_csv(linea: &str) -> Option<char> {
    let comas = linea.matches(',').count();
    let puntos_coma = linea.matches(';').count();

    if comas >= 1 && comas > puntos_coma {
        Some(',')
    } else if puntos_coma >= 1 && puntos_coma > comas {
        Some(';')
    } else {
        None
    }
}

/// Parsea un catálogo en formato CSV (mismo orden de columnas que Python).
pub fn parsear_csv(texto: &str) -> Vec<ProductoCatalogo> {
    let lineas: Vec<&str> = texto.trim().lines().collect();
    if lineas.is_empty() {
        return Vec::new();
    }

    let Some(separador) = detectar_separador_csv(lineas[0]) else {
        return Vec::new();
    };

    let primera_linea = lineas[0].to_lowercase();
    let tiene_header = PALABRAS_HEADER.iter().any(|p| primera_linea.contains(p));

    let headers: Vec<String> = if tiene_header {
        lineas[0]
            .split(separador)
            .map(|h| h.trim().to_lowercase())
            .collect()
    } else {
        Vec::new()
    };

    let mut col_stock: Option<usize> = None;
    for (i, h) in headers.iter().enumerate() {
        if PALABRAS_STOCK.iter().any(|k| h.contains(k)) {
            col_stock = Some(i);
            break;
        }
    }

    let mut col_categoria: Option<usize> = None;
    for (i, h) in headers.iter().enumerate() {
        if PALABRAS_CATEGORIA.iter().any(|k| h.contains(k)) {
            col_categoria = Some(i);
            break;
        }
    }

    let mut productos = Vec::new();

    for (i, linea) in lineas.iter().enumerate() {
        let linea = linea.trim();
        if linea.is_empty() {
            continue;
        }
        if tiene_header && i == 0 {
            continue;
        }

        let partes: Vec<String> = linea
            .split(separador)
            .map(|p| p.trim().trim_matches('"').trim_matches('\'').to_string())
            .collect();
        if partes.len() < 2 {
            continue;
        }

        // Clasificar columnas numéricas y de texto.
        let mut numeric_cols: Vec<(usize, f64)> = Vec::new();
        let mut text_cols: Vec<(usize, String)> = Vec::new();
        for (j, p) in partes.iter().enumerate() {
            let p_clean: String = p.chars().filter(|c| !matches!(c, '$' | ',')).collect();
            let p_clean = p_clean.trim();
            match p_clean.parse::<f64>() {
                Ok(val) if val.is_finite() && val.abs() <= PRECIO_MAXIMO => {
                    numeric_cols.push((j, val))
                }
                // inf/nan o magnitudes absurdas → se ignoran como número.
                Ok(_) => {}
                Err(_) => {
                    if !p_clean.is_empty() {
                        text_cols.push((j, p_clean.to_string()));
                    }
                }
            }
        }

        // La primera columna de texto (no numérica) es el nombre.
        let mut nombre: Option<String> = None;
        for (_, p) in &text_cols {
            let sin_puntos = p.replace('.', "");
            let solo_digitos = !sin_puntos.is_empty() && sin_puntos.chars().all(|c| c.is_ascii_digit());
            if !solo_digitos {
                nombre = Some(p.clone());
                break;
            }
        }
        let mut nombre = match nombre {
            Some(n) if !n.is_empty() => n,
            _ => continue,
        };

        let mut categoria: Option<String> = None;

        // Si solo hay 2 columnas de texto y no hay precios, la primera puede
        // ser categoría y la segunda producto.
        if numeric_cols.is_empty() && text_cols.len() == 2 {
            let first_text = &text_cols[0].1;
            let second_text = &text_cols[1].1;
            if second_text.chars().count() > first_text.chars().count() {
                nombre = second_text.clone();
                categoria = Some(first_text.to_uppercase());
            } else {
                nombre = first_text.clone();
                categoria = Some(second_text.to_uppercase());
            }
        }

        // Asignar precios (el mayor es venta, el menor costo).
        let mut precio_venta = 0.0f64;
        let mut precio_costo = 0.0f64;
        if numeric_cols.len() >= 2 {
            let (_, val1) = numeric_cols[0];
            let (_, val2) = numeric_cols[1];
            if val1 > val2 {
                precio_venta = val1;
                precio_costo = val2;
            } else {
                precio_venta = val2;
                precio_costo = val1;
            }
            // Buscar categoría en la última columna de texto.
            for (_, p) in text_cols.iter().rev() {
                if *p != nombre && p.chars().count() < 30 {
                    categoria = Some(p.to_uppercase());
                    break;
                }
            }
        } else if numeric_cols.len() == 1 {
            let (_, val) = numeric_cols[0];
            precio_venta = val;
        }

        // Stock desde la columna detectada por header.
        let mut stock = 0i64;
        if let Some(cs) = col_stock {
            if cs < partes.len() {
                let stock_str = partes[cs].replace(',', "");
                stock = stock_str
                    .trim()
                    .parse::<f64>()
                    .map(|v| v as i64)
                    .unwrap_or(0);
            }
        }

        // Categoría explícita por header.
        if let Some(cc) = col_categoria {
            if cc < partes.len() && !partes[cc].trim().is_empty() {
                categoria = Some(partes[cc].trim().to_uppercase());
            }
        }

        // Fallback: categoría en cualquier columna de texto corta.
        if categoria.is_none() || categoria.as_deref() == Some(SIN_CATEGORIA) {
            for (_, p) in &text_cols {
                if *p != nombre && p.chars().count() < 30 {
                    categoria = Some(p.to_uppercase());
                    break;
                }
            }
        }

        // Fallback final con solo 2 columnas de texto.
        if (categoria.is_none() || categoria.as_deref() == Some(SIN_CATEGORIA)) && text_cols.len() == 2 {
            let first_text = &text_cols[0].1;
            let second_text = &text_cols[1].1;
            if *first_text != nombre {
                categoria = Some(first_text.to_uppercase());
            } else if *second_text != nombre {
                categoria = Some(second_text.to_uppercase());
            }
        }

        let categoria_final = {
            let c = categoria.unwrap_or_default();
            if c.is_empty() {
                SIN_CATEGORIA.to_string()
            } else {
                c
            }
        };

        if precio_venta > 0.0 {
            productos.push(ProductoCatalogo {
                nombre: limpiar_producto(&nombre),
                precio_costo: round2(precio_costo),
                precio_venta: round2(precio_venta),
                stock,
                categoria: categoria_final,
            });
        } else {
            productos.push(ProductoCatalogo {
                nombre: limpiar_producto(&nombre),
                precio_costo: 0.0,
                precio_venta: 0.0,
                stock,
                categoria: categoria_final,
            });
        }
    }

    productos
}

// ---------------------------------------------------------------------------
// Tests (verificados contra el comportamiento de Python)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detecta_separador() {
        assert_eq!(detectar_separador_csv("a,b,c"), Some(','));
        assert_eq!(detectar_separador_csv("a;b;c"), Some(';'));
        assert_eq!(detectar_separador_csv("a;b,c"), None);
        assert_eq!(detectar_separador_csv("solo texto"), None);
        assert_eq!(detectar_separador_csv(""), None);
    }

    #[test]
    fn csv_con_header_identifica_stock_y_categoria() {
        let texto = "nombre,costo,venta,categoria,stock\nCOCA,10,20,BEBIDAS,5\n";
        let res = parsear_csv(texto);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].nombre, "COCA");
        assert_eq!(res[0].precio_costo, 10.0);
        assert_eq!(res[0].precio_venta, 20.0);
        assert_eq!(res[0].stock, 5);
        assert_eq!(res[0].categoria, "BEBIDAS");
    }

    #[test]
    fn csv_detecta_precio_mayor_como_venta() {
        // Sin header: nombre,venta,costo en orden inverso.
        let texto = "TAZAS,60,40\n";
        let res = parsear_csv(texto);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].nombre, "TAZAS");
        assert_eq!(res[0].precio_venta, 60.0);
        assert_eq!(res[0].precio_costo, 40.0);
        assert_eq!(res[0].categoria, "SIN CATEGORÍA");
    }

    #[test]
    fn csv_con_punto_y_coma() {
        let texto = "producto;precio;existencia\nFANTA 500ML;18;24\n";
        let res = parsear_csv(texto);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].nombre, "FANTA 500ML");
        // Verificado contra Python: el MAYOR de los dos números es la venta (24).
        assert_eq!(res[0].precio_venta, 24.0);
        assert_eq!(res[0].precio_costo, 18.0);
        assert_eq!(res[0].stock, 24);
    }

    #[test]
    fn csv_sin_precios_agrega_con_cero() {
        let texto = "nombre\nALGO SOLO NOMBRE\n";
        // 1 sola columna → se salta (necesita ≥ 2).
        assert!(parsear_csv(texto).is_empty());
    }

    #[test]
    fn csv_vacio_o_sin_separador_devuelve_vacio() {
        assert!(parsear_csv("").is_empty());
        assert!(parsear_csv("solo una linea sin comas").is_empty());
    }
}