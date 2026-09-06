// ============================================================
// linea — Parseo de UNA línea de catálogo (7 patrones en orden).
//
// Una línea puede traer múltiples productos separados por `|`. Los
// patrones van del más específico al más laxo, en el mismo orden que
// Python: sin-separador-con-cantidad, sin-separador, cantidad al inicio,
// separador explícito y, al final, "todo es nombre".
// ============================================================

use super::patrones::{
    CANTIDAD_FINAL, PATRON_CANTIDAD_INICIO, PATRON_PRODUCTO, PATRON_SIN_SEP,
    PATRON_SIN_SEP_CANT, PATRON_SIN_SEP_CANT_SINDOL, PATRON_SIN_SEP_SINDOL, UNIDADES,
};
use super::super::ProductoCatalogo;
use crate::cerebro::analizador_tickets::PRECIO_MAXIMO;
use crate::cerebro::filtrador::{es_categoria, limpiar_producto};

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn parse_precio(s: &str) -> Option<f64> {
    let valor = s.replace(',', "").parse::<f64>().ok()?;
    if valor.is_finite() && valor.abs() <= PRECIO_MAXIMO {
        Some(valor)
    } else {
        None
    }
}

/// Extrae nombre y cantidad de un texto que termina en número (A4: no se come
/// volúmenes tipo "COCA-COLA 600ML": un número > 999 o con unidad no es pieza).
pub(crate) fn extraer_nombre_cantidad(texto_limpio: &str) -> (String, i64) {
    let limpio = texto_limpio.trim();
    if let Some(c) = CANTIDAD_FINAL.captures(limpio) {
        let nombre = c[1].trim();
        let num: i64 = c[2].parse().unwrap_or(0);
        let termina_en_unidad = UNIDADES.iter().any(|u| nombre.to_lowercase().ends_with(u));
        if !termina_en_unidad && 0 < num && num <= 999 {
            return (nombre.to_string(), num);
        }
    }
    (limpio.to_string(), 0)
}

/// Parsea una línea del catálogo que puede contener múltiples productos
/// separados por '|'. Orden de patrones 1:1 con Python.
pub fn parsear_linea_catalogo(linea: &str, categoria_actual: &str) -> Vec<ProductoCatalogo> {
    let mut productos = Vec::new();

    for segmento in linea.split('|') {
        let segmento = segmento.trim();
        if segmento.is_empty() {
            continue;
        }

        // 1. Patrón SIN separador: Nombre  CANT  $VTA  $CST (más específico, primero)
        if let Some(c) = PATRON_SIN_SEP_CANT.captures(segmento) {
            let texto_completo = c[1].trim();
            let cantidad: i64 = c[2].parse().unwrap_or(0);
            let (nombre, _) = extraer_nombre_cantidad(texto_completo);
            let (Some(venta), Some(costo)) = (parse_precio(&c[3]), parse_precio(&c[4])) else {
                continue;
            };
            if !nombre.is_empty() {
                productos.push(ProductoCatalogo {
                    nombre: limpiar_producto(&nombre),
                    precio_costo: round2(costo),
                    precio_venta: round2(venta),
                    stock: cantidad,
                    categoria: categoria_actual.to_string(),
                });
                continue;
            }
        }

        // 2. Patrón SIN separador, SIN cantidad: Nombre  $VTA  $CST
        if let Some(c) = PATRON_SIN_SEP.captures(segmento) {
            let nombre = c[1].trim();
            let (Some(venta), Some(costo)) = (parse_precio(&c[2]), parse_precio(&c[3])) else {
                continue;
            };
            if !nombre.is_empty() {
                productos.push(ProductoCatalogo {
                    nombre: limpiar_producto(nombre),
                    precio_costo: round2(costo),
                    precio_venta: round2(venta),
                    stock: 0,
                    categoria: categoria_actual.to_string(),
                });
                continue;
            }
        }

        // 3. Patrón SIN separador, con cantidad, sin $: Nombre  CANT  VTA  CST
        if let Some(c) = PATRON_SIN_SEP_CANT_SINDOL.captures(segmento) {
            let texto_completo = c[1].trim();
            let cantidad: i64 = c[2].parse().unwrap_or(0);
            let (nombre, _) = extraer_nombre_cantidad(texto_completo);
            let (Some(venta), Some(costo)) = (parse_precio(&c[3]), parse_precio(&c[4])) else {
                continue;
            };
            if !nombre.is_empty() {
                productos.push(ProductoCatalogo {
                    nombre: limpiar_producto(&nombre),
                    precio_costo: round2(costo),
                    precio_venta: round2(venta),
                    stock: cantidad,
                    categoria: categoria_actual.to_string(),
                });
                continue;
            }
        }

        // 4. Patrón SIN separador, SIN cantidad, sin $: Nombre  VTA  CST
        if let Some(c) = PATRON_SIN_SEP_SINDOL.captures(segmento) {
            let nombre = c[1].trim();
            let (Some(venta), Some(costo)) = (parse_precio(&c[2]), parse_precio(&c[3])) else {
                continue;
            };
            if !nombre.is_empty() {
                productos.push(ProductoCatalogo {
                    nombre: limpiar_producto(nombre),
                    precio_costo: round2(costo),
                    precio_venta: round2(venta),
                    stock: 0,
                    categoria: categoria_actual.to_string(),
                });
                continue;
            }
        }

        // 5. Intentar cantidad al inicio con separador (ej: "10Producto - $10 $5")
        if let Some(c) = PATRON_CANTIDAD_INICIO.captures(segmento) {
            let cantidad: i64 = c[1].parse().unwrap_or(0);
            let nombre = c[2]
                .trim()
                .trim_end_matches(['-', '=', '*', '~', '>'])
                .trim();
            let venta_str = c.get(3).map(|m| m.as_str()).unwrap_or("");
            let costo_str = c.get(4).map(|m| m.as_str()).unwrap_or("");

            let venta: f64 = if venta_str.is_empty() {
                0.0
            } else {
                let Some(v) = parse_precio(venta_str) else {
                    continue;
                };
                v
            };
            let costo: f64 = if costo_str.is_empty() {
                0.0
            } else {
                let Some(cv) = parse_precio(costo_str) else {
                    continue;
                };
                cv
            };

            if !nombre.is_empty() {
                productos.push(ProductoCatalogo {
                    nombre: limpiar_producto(nombre),
                    precio_costo: round2(costo),
                    precio_venta: round2(venta),
                    stock: cantidad,
                    categoria: categoria_actual.to_string(),
                });
                continue;
            }
        }

        // 6. Patrón con separador explícito (ej: "Producto -- $10 $5")
        if let Some(c) = PATRON_PRODUCTO.captures(segmento) {
            let nombre = c[1]
                .trim()
                .trim_end_matches(['-', '=', '*', '~', '>'])
                .trim();
            let venta_str = c.get(2).map(|m| m.as_str()).unwrap_or("");
            let costo_str = c.get(3).map(|m| m.as_str()).unwrap_or("");

            let venta: f64 = if venta_str.is_empty() {
                0.0
            } else {
                let Some(v) = parse_precio(venta_str) else {
                    continue;
                };
                v
            };
            let costo: f64 = if costo_str.is_empty() {
                0.0
            } else {
                let Some(cv) = parse_precio(costo_str) else {
                    continue;
                };
                cv
            };

            if !nombre.is_empty() {
                productos.push(ProductoCatalogo {
                    nombre: limpiar_producto(nombre),
                    precio_costo: round2(costo),
                    precio_venta: round2(venta),
                    stock: 0,
                    categoria: categoria_actual.to_string(),
                });
                continue;
            }
        }

        // 7. Sin ningún patrón: tratar toda la línea como nombre de producto
        let nombre = segmento.trim();
        if !nombre.is_empty() && !es_categoria(nombre) && nombre.chars().count() > 2 {
            let clean: String = nombre
                .chars()
                .filter(|c| !matches!(c, '$' | ',' | '.'))
                .collect();
            let clean = clean.trim();
            let solo_digitos = !clean.is_empty() && clean.chars().all(|c| c.is_ascii_digit());
            if !solo_digitos {
                productos.push(ProductoCatalogo {
                    nombre: limpiar_producto(nombre),
                    precio_costo: 0.0,
                    precio_venta: 0.0,
                    stock: 0,
                    categoria: categoria_actual.to_string(),
                });
            }
        }
    }

    productos
}
