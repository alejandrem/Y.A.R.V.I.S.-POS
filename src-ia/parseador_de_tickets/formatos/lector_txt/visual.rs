// ============================================================
// visual — Catálogo completo: detección de formato + tabla visual.
// ============================================================
//
// `parsear_catalogo_visual` mira la primera línea: si huele a CSV
// (separador `,` o `;`) delega al lector CSV; si no, lee la tabla visual
// (categorías en mayúsculas + productos por renglón).

use super::linea::parsear_linea_catalogo;
use super::patrones::PATRON_LINEA_HEADER;
use super::super::lector_csv::{detectar_separador_csv, parsear_csv};
use super::super::ProductoCatalogo;
use crate::cerebro::filtrador::es_categoria;

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
