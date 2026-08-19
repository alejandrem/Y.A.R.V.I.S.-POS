use regex::Regex;
use std::sync::LazyLock;

use crate::cerebro::filtrador::limpiar_producto;
use super::esquema::{Item, MapeoColumnas, resolver_indice};

pub const PRECIO_MAXIMO: f64 = 1e12;

pub fn limpiar_precio(texto: &str) -> f64 {
    if texto.is_empty() {
        return 0.0;
    }
    let limpio: String = texto
        .chars()
        .filter(|c| !matches!(c, '$' | ',' | ' '))
        .collect();
    let valor = limpio.parse::<f64>().unwrap_or(0.0);
    if valor.is_finite() && valor.abs() <= PRECIO_MAXIMO {
        valor
    } else {
        0.0
    }
}

static RE_TOKEN_NUM: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\$?[\d,]+(?:\.\d+)?$").expect("regex token numerico"));

/// True si el token parece número/precio (patrón `^\$?[\d,]+(?:\.\d+)?$`).
fn es_token_numero(token: &str) -> bool {
    let sin_dolar = token.replacen('$', "", 1);
    RE_TOKEN_NUM.is_match(&sin_dolar)
}

/// Unidades de medida/empaque que acompañan a productos: 600ML, 32GB, 1.5L,
/// 680GR, 30X30, 2X1, 1.5M, 15MM, 1KG, 250MB…
static RE_TOKEN_MEDIDA: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)^\d+(?:[.,]\d+)?(?:ml|lt|l|lb|gr|g|kg|oz|cc|gb|mb|mm|cm|m|pz|pzas|pza|und|un|ud|x)(?:x\d+(?:[.,]\d+)?)*$",
    )
    .expect("regex token medida")
});

/// True si el token parece una medida/empaque (600ML, 32GB, 30X30, 1.5L).
fn es_token_medida(token: &str) -> bool {
    RE_TOKEN_MEDIDA.is_match(token.trim())
}

const FRASES_NIVEL1: &[&str] = &[
    "factura",
    "cfdi",
    "gracias",
    "vuelva",
    "bienvenido",
    "metodo de pago",
    "forma de pago",
    "razon social",
    "regimen fiscal",
    "total a pagar",
    "efectivo recibido",
    "dinero recibido",
    "ticket #",
    "www",
    "pagina",
    "c.p.",
    "cp:",
    "telefono:",
    "direccion:",
    "correo:",
    "email:",
    "total:",
    "subtotal:",
    "iva:",
    "efectivo:",
    "tarjeta:",
    "cambio:",
    "pago:",
    "caja:",
    "nombre:",
    "fecha:",
    "hora:",
    "folio:",
    "serie:",
    "av.",
    "atendio:",
    "despacho:",
    "cliente:",
    "domicilio:",
];

const PRIMERAS_CABECERAS: &[&str] = &[
    "total", "subtotal", "iva", "efectivo", "tarjeta", "cambio", "credito", "debito", "pago",
    "cuota", "extra",
];

static RE_PALABRAS_NIVEL2: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"\b(?:ticket|caja|cajero|turno|rfc|cambio|nombre|articulo|producto|cant|precio|empresa|calle|ciudad|estado|colonia|direccion|telefono|tel|descripcion|serie|folio|copias|correo|miscelanea|total|efectivo|tarjeta|iva|av)\b",
    )
    .expect("regex nivel 2")
});

static RE_NUM_FINAL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"#?\d{4,}$").expect("regex numero final"));

/// Detecta si una línea es dato de producto o encabezado/separador.
/// Mismo algoritmo de 3 niveles que Python (bugs A3 resueltos).
pub fn es_linea_util(linea: &str) -> bool {
    let linea_lower = linea.trim().to_lowercase();
    if linea_lower.is_empty() {
        return false;
    }

    // Línea compuesta solo de separadores: "-----", "====", "~~~~~"
    let sin_espacios: String = linea_lower.replace(' ', "");
    if sin_espacios.chars().all(|c| "-=_*~.:".contains(c)) {
        return false;
    }

    let tokens: Vec<&str> = linea_lower.split_whitespace().collect();
    let nums = tokens.iter().filter(|t| es_token_numero(t)).count();
    let medidas = tokens.iter().filter(|t| es_token_medida(t)).count();

    // Nivel 1: frases que NUNCA aparecen en un nombre de producto.
    for frase in FRASES_NIVEL1 {
        if linea_lower.contains(frase) {
            return false;
        }
    }

    // Nivel 3: cabeceras de total/pago que arrancan la línea. Las unidades
    // de medida (600ML, 32GB, 30X30…) cuentan como dato de producto, así un
    // "TARJETA MEMORIA 32GB $250 $250" no se confunde con un encabezado.
    if nums + medidas < 3 && !tokens.is_empty() {
        let primer = tokens[0].trim_end_matches([':', '.']);
        if PRIMERAS_CABECERAS.contains(&primer) {
            return false;
        }
    }

    // Línea que termina en número grande (folio/número de ticket).
    if RE_NUM_FINAL.is_match(&linea_lower) {
        return false;
    }

    // Nivel 2: word-boundary, solo si NO hay columnas numéricas ni medidas.
    if nums == 0 && medidas == 0 && RE_PALABRAS_NIVEL2.is_match(&linea_lower) {
        return false;
    }

    true
}

static RE_DOLAR_ESPACIO: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\$\s+(\d)").expect("regex dolar espacio"));

/// Une "$" con el número siguiente: "$ 25" -> "$25".
fn preprocesar_linea(linea: &str) -> String {
    RE_DOLAR_ESPACIO.replace_all(linea, "$$1").into_owned()
}

fn round(x: f64, escala: u32) -> f64 {
    let f = 10f64.powi(escala as i32);
    (x * f).round() / f
}

fn col_valor<'a>(cols: &'a [&str], idx: Option<usize>) -> &'a str {
    match idx {
        Some(i) if i < cols.len() => cols[i],
        _ => "",
    }
}

/// Parsea UNA línea del ticket usando el mapeo del usuario.
/// Retorna `None` si la línea no es útil o no cumple el mapeo.
pub fn parsear_linea(linea: &str, mapeo: &MapeoColumnas, _total_cols: usize) -> Option<Item> {
    let linea = linea.trim();
    if !es_linea_util(linea) {
        return None;
    }

    let linea = preprocesar_linea(linea);
    let cols: Vec<&str> = linea.split_whitespace().collect();
    if cols.len() < 2 {
        return None;
    }
    let line_cols = cols.len();

    let idx_cant = resolver_indice(mapeo.cantidad, line_cols);
    let idx_precio = resolver_indice(mapeo.precio_unitario, line_cols);
    let idx_total = resolver_indice(mapeo.total, line_cols);
    let idx_desc = resolver_indice(mapeo.descuento, line_cols);

    let mut producto = String::new();
    if let Some(prod_rango) = &mapeo.producto {
        if prod_rango.len() >= 2 {
            let ini = resolver_indice(prod_rango.first().copied(), line_cols);
            let fin = resolver_indice(prod_rango.last().copied(), line_cols);
            if let (Some(ini), Some(fin)) = (ini, fin) {
                let start = std::cmp::min(ini, fin);
                let end = std::cmp::max(ini, fin).min(cols.len() - 1);
                if start <= end {
                    producto = cols[start..=end].join(" ");
                }
            }
        } else if prod_rango.len() == 1 {
            if let Some(idx) = resolver_indice(prod_rango.first().copied(), line_cols) {
                if idx < cols.len() {
                    producto = cols[idx].to_string();
                }
            }
        }
    }

    let cantidad = limpiar_precio(col_valor(&cols, idx_cant));
    let precio = limpiar_precio(col_valor(&cols, idx_precio));
    let total = limpiar_precio(col_valor(&cols, idx_total));
    let descuento = limpiar_precio(col_valor(&cols, idx_desc));

    if producto.is_empty() || (cantidad == 0.0 && total == 0.0) {
        return None;
    }

    Some(Item {
        producto: limpiar_producto(&producto),
        cantidad: round(cantidad, 3),
        precio_unitario: round(precio, 2),
        total: round(total, 2),
        descuento: if descuento > 0.0 {
            Some(round(descuento, 2))
        } else {
            None
        },
    })
}
