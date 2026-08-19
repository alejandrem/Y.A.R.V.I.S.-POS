//! lector_txt.rs — Port de `yarvis-IA/parseador_de_tickets/formatos/lector_txt.py`
//!
//! Parser de catálogos en formato visual (texto plano .txt):
//!   - Producto -- $VENTA $COSTO
//!   - Producto - $VENTA - $COSTO
//!   - Producto = $VENTA $COSTO
//!   - Producto VENTA COSTO (sin $)
//!   - Múltiples productos por línea con |
//!   - Cantidad al inicio: 10Producto $10 $5
//!   - Formato de tabla: Producto  CANT  $VTA  $CST (sin separador)
//!
//! Nota regex: Python usaba lookbehind `(?<![-=*~>])` para que el nombre no
//! robe un separador real ("Coca-Cola 600ML -- $25 $18" cae en _PATRON_PRODUCTO).
//! El crate `regex` de Rust NO soporta lookaround, así que se reescribe como
//! clase negada final: `(.+?[^-=*~>])`. Semántica equivalente, verificada
//! contra los mismos casos de Python (test_lector_txt.py).

use regex::Regex;
use std::sync::LazyLock;

use super::lector_csv::{detectar_separador_csv, parsear_csv};
use super::ProductoCatalogo;
use crate::cerebro::analizador_tickets::PRECIO_MAXIMO;
use crate::cerebro::filtrador::{es_categoria, limpiar_producto};

// ---------------------------------------------------------------------------
// Patrones
// ---------------------------------------------------------------------------

// Patrón flexible: nombre + separador + precio1 + precio2
static PATRON_PRODUCTO: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"([A-Za-z0-9áéíóúñüÁÉÍÓÚÑÜ\s\.\-'"°®™]+?)\s*[-=*~>]+\s*\$?\s*([\d,]+(?:\.\d+)?)(?:\s*[-=*~>]*\s*\$?\s*([\d,]+(?:\.\d+)?))?"#,
    )
    .expect("regex producto")
});

// Patrón para detectar cantidad al inicio de la línea (ej: "10Producto $10 $5")
static PATRON_CANTIDAD_INICIO: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"^(\d+)\s*([A-Za-z0-9áéíóúñüÁÉÍÓÚÑÜ\s\.\-'"°®™]+?)\s*[-=*~>]+\s*\$?\s*([\d,]+(?:\.\d+)?)?(?:\s+\$?\s*([\d,]+(?:\.\d+)?)?)?"#,
    )
    .expect("regex cantidad inicio")
});

// Patrones SIN separador (solo espacios): Nombre  CANT  $VTA  $CST
// El lookbehind original (?<![-=*~>]) se reescribe como `(.+?[^-=*~>])` para
// que el nombre NO termine en un separador real (bug 8: SIN_SEP ganaba por
// precedencia sobre _PATRON_PRODUCTO y se comía el "--").
// Patrón 1: Con cantidad y con $ en precios
static PATRON_SIN_SEP_CANT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(.+?[^-=*~>])\s+(\d+)\s+\$([\d,]+(?:\.\d+)?)\s+\$([\d,]+(?:\.\d+)?)")
        .expect("regex sin separador con cantidad")
});
// Patrón 2: Sin cantidad, con $ en precios
static PATRON_SIN_SEP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(.+?[^-=*~>])\s+\$([\d,]+(?:\.\d+)?)\s+\$([\d,]+(?:\.\d+)?)")
        .expect("regex sin separador")
});
// Patrón 3: Con cantidad, sin $ en precios
static PATRON_SIN_SEP_CANT_SINDOL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(.+?[^-=*~>])\s+(\d+)\s+([\d,]+(?:\.\d+)?)\s+([\d,]+(?:\.\d+)?)")
        .expect("regex sin separador con cantidad sin dolar")
});
// Patrón 4: Sin cantidad, sin $
static PATRON_SIN_SEP_SINDOL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(.+?[^-=*~>])\s+([\d,]+(?:\.\d+)?)\s+([\d,]+(?:\.\d+)?)")
        .expect("regex sin separador sin dolar")
});

static PATRON_LINEA_HEADER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[\s─]+$|PRODUCTO.*CANT.*VTA|^\s*$").expect("regex linea header")
});

static CANTIDAD_FINAL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(.*?)\s+(\d{1,3})$").expect("regex cantidad final"));

const UNIDADES: &[&str] = &["ml", "l", "kg", "g", "gr", "oz", "lb"];

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
fn extraer_nombre_cantidad(texto_limpio: &str) -> (String, i64) {
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

// ---------------------------------------------------------------------------
// Parseo de líneas de catálogo
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Catálogo completo
// ---------------------------------------------------------------------------

/// Parsea un catálogo en formato de tabla visual (categorías + productos).
fn parsear_visual(texto: &str) -> Vec<ProductoCatalogo> {
    let mut productos = Vec::new();
    let mut categoria_actual = "SIN CATEGORÍA".to_string();

    for linea in texto.lines() {
        let linea_limpia = linea.trim();

        if linea_limpia.is_empty() || PATRON_LINEA_HEADER.is_match(linea_limpia) {
            continue;
        }

        if es_categoria(linea_limpia) {
            categoria_actual = linea_limpia.trim_end_matches(':').trim().to_string();
            continue;
        }

        for p in parsear_linea_catalogo(linea_limpia, &categoria_actual) {
            productos.push(p);
        }
    }

    productos
}

/// Parsea un catálogo detectando automáticamente el formato:
/// - CSV (con , o ; como separador)
/// - Formato visual (con --, -, =, > como separador)
pub fn parsear_catalogo_visual(texto: &str) -> Vec<ProductoCatalogo> {
    let texto = texto.trim();
    if texto.is_empty() {
        return Vec::new();
    }

    let Some(primera_linea) = texto.lines().next() else {
        return Vec::new();
    };

    if detectar_separador_csv(primera_linea).is_some() {
        return parsear_csv(texto);
    }

    parsear_visual(texto)
}

// ---------------------------------------------------------------------------
// Tests (espejo de test_lector_txt.py, valores verificados contra Python)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_una(linea: &str) -> ProductoCatalogo {
        let res = parsear_linea_catalogo(linea, "");
        assert!(res.len() == 1, "esperaba 1 producto, dio {}", res.len());
        res[0].clone()
    }

    // ---------- Bug 8: separador no contamina el nombre ----------

    #[test]
    fn bug8_separador_no_contamina_nombre() {
        for (linea, nombre, venta) in [
            ("Coca-Cola 600ML -- $25 $18", "COCA-COLA 600ML", 25.0),
            ("AGUA 1500 -- $20 $16", "AGUA 1500", 20.0),
            ("PAN BLANCO 12 -- $15 $10", "PAN BLANCO 12", 15.0),
            ("TOTAL -- $1,234.56 $1,000", "TOTAL", 1234.56),
            ("Producto -- $10 $5", "PRODUCTO", 10.0),
            ("Producto - $10 - $5", "PRODUCTO", 10.0),
            ("Producto = $10 $5", "PRODUCTO", 10.0),
        ] {
            let p = parse_una(linea);
            assert_eq!(p.nombre, nombre, "{linea}");
            assert_eq!(p.precio_venta, venta, "{linea}");
        }
    }

    // ---------- Tablas SIN separador: nombre conserva volúmenes ----------

    #[test]
    fn tablas_sin_separador_con_volumen() {
        for (linea, nombre, stock) in [
            ("Coca-Cola 600ML $25 $18", "COCA-COLA 600ML", 0),
            ("Coca-Cola 600ML 12 $29 $23", "COCA-COLA 600ML", 12),
            ("Coca-Cola 600ML 12 29 23", "COCA-COLA 600ML", 12),
            ("Sabritas 16 12", "SABRITAS", 0),
            ("Sabritas 60 16 12", "SABRITAS", 60),
        ] {
            let p = parse_una(linea);
            assert_eq!(p.nombre, nombre, "{linea}");
            assert_eq!(p.stock, stock, "{linea}");
        }
    }

    #[test]
    fn cantidad_al_inicio() {
        let p = parse_una("10Producto - $10 $5");
        assert_eq!(p.nombre, "PRODUCTO");
        assert_eq!(p.stock, 10);
    }

    // ---------- A4: volúmenes que NO deben separarse como cantidad ----------

    #[test]
    fn extraer_nombre_cantidad_no_come_volumen() {
        for (texto, esperado) in [
            ("COCA-COLA 600ML 2880", ("COCA-COLA 600ML 2880".to_string(), 0)),
            ("COCA-COLA 600", ("COCA-COLA".to_string(), 600)),
            ("Coca-Cola 600 ml 12", ("Coca-Cola 600 ml 12".to_string(), 0)),
            ("Agua 1500", ("Agua 1500".to_string(), 0)),
        ] {
            assert_eq!(extraer_nombre_cantidad(texto), esperado, "{texto}");
        }
    }

    #[test]
    fn extraer_nombre_cantidad_si_es_pieza_pequena() {
        assert_eq!(extraer_nombre_cantidad("Producto 12"), ("Producto".to_string(), 12));
    }

    // ---------- End-to-end con categorías y múltiples productos ----------

    #[test]
    fn catalogo_end_to_end() {
        let texto = "BEBIDAS
Coca-Cola 600ML -- $25 $18 | AGUA 1500 -- $20 $16
ABARROTES
PAN BLANCO 12 -- $15 $10 | SABRITAS 16 12
";
        let productos = parsear_catalogo_visual(texto);
        let nombres: Vec<&str> = productos.iter().map(|p| p.nombre.as_str()).collect();
        assert_eq!(
            nombres,
            vec!["COCA-COLA 600ML", "AGUA 1500", "PAN BLANCO 12", "SABRITAS"]
        );
        let categorias: Vec<&str> = productos.iter().map(|p| p.categoria.as_str()).collect();
        assert_eq!(categorias, vec!["BEBIDAS", "BEBIDAS", "ABARROTES", "ABARROTES"]);
        for p in &productos {
            assert!(!p.nombre.ends_with("--"), "nombre contaminado: {}", p.nombre);
        }
    }
}