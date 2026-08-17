//! analizador.rs — Port de `yarvis-IA/parseador_de_tickets/cerebro/analizador.py`
//!
//! Funciones PURAS de análisis de tickets (regex, sin modelos):
//! fecha/hora, método de pago, detección de línea útil y parseo con mapeo.
//! Conexión con filtrador: `parsear_linea` llama a `filtrador::limpiar_producto`.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

use super::filtrador::limpiar_producto;

// ---------------------------------------------------------------------------
// Meses en español (mapa fecha → número)
// ---------------------------------------------------------------------------

const MESES_ES: &[(&str, &str)] = &[
    ("enero", "01"),
    ("febrero", "02"),
    ("marzo", "03"),
    ("abril", "04"),
    ("mayo", "05"),
    ("junio", "06"),
    ("julio", "07"),
    ("agosto", "08"),
    ("septiembre", "09"),
    ("octubre", "10"),
    ("noviembre", "11"),
    ("diciembre", "12"),
    ("jan", "01"),
    ("feb", "02"),
    ("mar", "03"),
    ("apr", "04"),
    ("may", "05"),
    ("jun", "06"),
    ("jul", "07"),
    ("aug", "08"),
    ("sep", "09"),
    ("oct", "10"),
    ("nov", "11"),
    ("dec", "12"),
];

fn mes_numero(nombre: &str) -> &str {
    for (k, v) in MESES_ES {
        if *k == nombre {
            return v;
        }
    }
    "01"
}

fn pad2(n: i32) -> String {
    format!("{:02}", n)
}

// ---------------------------------------------------------------------------
// Fecha y hora (regex, fallback si el LLM no detecta)
// ---------------------------------------------------------------------------

static RE_FECHA_ISO: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\b(\d{4})-(\d{2})-(\d{2})\b").expect("regex fecha ISO"));
static RE_FECHA_SLASH: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(\d{1,2})/(\d{1,2})/(\d{2,4})\b").expect("regex fecha slash")
});
static RE_FECHA_GUION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(\d{1,2})-(\d{1,2})-(\d{4})\b").expect("regex fecha guion")
});
static RE_FECHA_MES: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\b(\d{1,2})\s+(?:de\s+)?(enero|febrero|marzo|abril|mayo|junio|julio|agosto|septiembre|octubre|noviembre|diciembre)\s+(?:de\s+)?(\d{4})\b",
    )
    .expect("regex fecha con mes")
});
static RE_HORA: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(\d{1,2}):(\d{2})(?::\d{2})?\s*(AM|PM|am|pm)?\b").expect("regex hora")
});

/// Busca fecha y hora en el texto del ticket usando regex.
/// Retorna `(fecha_iso "YYYY-MM-DD" o None, hora "HH:MM" o None)`.
pub fn extraer_fecha_hora_regex(texto: &str) -> (Option<String>, Option<String>) {
    let mut fecha: Option<String> = None;
    let mut hora: Option<String> = None;

    for linea in texto.lines() {
        if fecha.is_none() {
            // Patrones de fecha ordenados de más específico a menos.
            if let Some(c) = RE_FECHA_ISO.captures(linea) {
                fecha = Some(format!("{}-{}-{}", &c[1], &c[2], &c[3]));
            } else if let Some(c) = RE_FECHA_SLASH.captures(linea) {
                // DD/MM/YYYY o DD/MM/YY
                let anio = if c[3].len() == 4 {
                    c[3].to_string()
                } else {
                    format!("20{}", &c[3])
                };
                let mes = pad2(c[2].parse().unwrap_or(0));
                let dia = pad2(c[1].parse().unwrap_or(0));
                fecha = Some(format!("{}-{}-{}", anio, mes, dia));
            } else if let Some(c) = RE_FECHA_GUION.captures(linea) {
                // DD-MM-YYYY
                let anio = &c[3];
                let mes = pad2(c[2].parse().unwrap_or(0));
                let dia = pad2(c[1].parse().unwrap_or(0));
                fecha = Some(format!("{}-{}-{}", anio, mes, dia));
            } else if let Some(c) = RE_FECHA_MES.captures(linea) {
                // "15 de marzo de 2024" o "15 marzo 2024"
                let anio = &c[3];
                let mes_nombre = c[2].to_lowercase();
                let mes = mes_numero(&mes_nombre);
                let dia = pad2(c[1].parse().unwrap_or(0));
                fecha = Some(format!("{}-{}-{}", anio, mes, dia));
            }
        }

        if hora.is_none() {
            if let Some(c) = RE_HORA.captures(linea) {
                let mut h: i32 = c[1].parse().unwrap_or(0);
                let mins = &c[2];
                if let Some(ampm) = c.get(3) {
                    match ampm.as_str().to_uppercase().as_str() {
                        "PM" if h < 12 => h += 12,
                        "AM" if h == 12 => h = 0,
                        _ => {}
                    }
                }
                hora = Some(format!("{:02}:{}", h, mins));
            }
        }

        if fecha.is_some() && hora.is_some() {
            break;
        }
    }

    (fecha, hora)
}

// ---------------------------------------------------------------------------
// Método de pago (regex sobre las últimas 25 líneas)
// ---------------------------------------------------------------------------

const PATRONES_PAGO: &[(&str, &str)] = &[
    (
        r"(?:FORMA\s+DE\s+PAGO|METODO\s+DE\s+PAGO|PAGO\s+CON|PAGO:?)\s*:?\s*(\d+\s*[-:]?\s*)?(EFECTIVO)",
        "efectivo",
    ),
    (
        r"(?:FORMA\s+DE\s+PAGO|METODO\s+DE\s+PAGO|PAGO\s+CON|PAGO:?)\s*:?\s*(\d+\s*[-:]?\s*)?(TARJETA\s+DEBITO)",
        "tarjeta",
    ),
    (
        r"(?:FORMA\s+DE\s+PAGO|METODO\s+DE\s+PAGO|PAGO\s+CON|PAGO:?)\s*:?\s*(\d+\s*[-:]?\s*)?(TARJETA\s+CREDITO)",
        "tarjeta",
    ),
    (
        r"(?:FORMA\s+DE\s+PAGO|METODO\s+DE\s+PAGO|PAGO\s+CON|PAGO:?)\s*:?\s*(\d+\s*[-:]?\s*)?(DEBITO)",
        "tarjeta",
    ),
    (
        r"(?:FORMA\s+DE\s+PAGO|METODO\s+DE\s+PAGO|PAGO\s+CON|PAGO:?)\s*:?\s*(\d+\s*[-:]?\s*)?(CREDITO)",
        "tarjeta",
    ),
    (
        r"(?:FORMA\s+DE\s+PAGO|METODO\s+DE\s+PAGO|PAGO\s+CON|PAGO:?)\s*:?\s*(\d+\s*[-:]?\s*)?(TRANSFERENCIA)",
        "transferencia",
    ),
    (
        r"(?:FORMA\s+DE\s+PAGO|METODO\s+DE\s+PAGO|PAGO\s+CON|PAGO:?)\s*:?\s*(\d+\s*[-:]?\s*)?(CHEQUE)",
        "cheque",
    ),
    (r"^EFECTIVO[\s\.]+(?:\$|[\d])", "efectivo"),
    (r"^TARJETA\s+DEBITO[\s\.]+(?:\$|[\d])", "tarjeta"),
    (r"^TARJETA\s+CREDITO[\s\.]+(?:\$|[\d])", "tarjeta"),
    (r"^DEBITO[\s\.]+(?:\$|[\d])", "tarjeta"),
    (r"^CREDITO[\s\.]+(?:\$|[\d])", "tarjeta"),
    (r"^TRANSFERENCIA[\s\.]+(?:\$|[\d])", "transferencia"),
    (r"^CHEQUE[\s\.]+(?:\$|[\d])", "cheque"),
    (r"^EFECTIVO\s*:", "efectivo"),
    (r"^TARJETA\s+DEBITO\s*:", "tarjeta"),
    (r"^TARJETA\s+CREDITO\s*:", "tarjeta"),
    (r"^DEBITO\s*:", "tarjeta"),
    (r"^CREDITO\s*:", "tarjeta"),
    (r"^TRANSFERENCIA\s*:", "transferencia"),
    (r"^CHEQUE\s*:", "cheque"),
];

/// Busca en las últimas 25 líneas del ticket el método de pago.
/// Retorna "efectivo" por defecto.
pub fn extraer_metodo_pago(texto: &str) -> String {
    static REGS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
        PATRONES_PAGO
            .iter()
            .map(|(p, m)| (Regex::new(p).expect("regex pago"), *m))
            .collect()
    });

    let lineas: Vec<&str> = texto.lines().collect();
    let start = lineas.len().saturating_sub(25);

    for linea in &lineas[start..] {
        let linea_upper = linea.trim().to_uppercase();
        for (re, metodo) in REGS.iter() {
            if re.is_match(&linea_upper) {
                return metodo.to_string();
            }
        }
    }

    "efectivo".to_string()
}

// ---------------------------------------------------------------------------
// Parseo con mapeo de columnas
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapeoColumnas {
    pub cantidad: Option<i32>,
    pub producto: Option<Vec<i32>>,
    #[serde(rename = "precio_unitario")]
    pub precio_unitario: Option<i32>,
    pub total: Option<i32>,
    pub descuento: Option<i32>,
}

/// Resultado de parsear UNA línea de ticket (espejo del dict de Python).
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Item {
    pub producto: String,
    pub cantidad: f64,
    pub precio_unitario: f64,
    pub total: f64,
    pub descuento: Option<f64>,
}

/// Resuelve un índice de columna (admite negativos: -1 = última columna).
pub fn resolver_indice(col: Option<i32>, total_cols: usize) -> Option<usize> {
    let col = col?;
    if col < 0 {
        let res = total_cols as i64 + col as i64;
        if res < 0 {
            None
        } else {
            Some(res as usize)
        }
    } else {
        Some(col as usize)
    }
}

/// Limpia un string de precio: "$1,234.56" -> 1234.56
/// Guardia anti-explosión: valores no finitos (inf/nan) o con magnitud > 1e12
/// (más de un millón de millones = parseo inválido) se devuelven como 0.0.
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

    // Nivel 1: frases que NUNCA aparecen en un nombre de producto.
    for frase in FRASES_NIVEL1 {
        if linea_lower.contains(frase) {
            return false;
        }
    }

    // Nivel 3: cabeceras de total/pago que arrancan la línea.
    if nums < 3 && !tokens.is_empty() {
        let primer = tokens[0].trim_end_matches([':', '.']);
        if PRIMERAS_CABECERAS.contains(&primer) {
            return false;
        }
    }

    // Línea que termina en número grande (folio/número de ticket).
    if RE_NUM_FINAL.is_match(&linea_lower) {
        return false;
    }

    // Nivel 2: word-boundary, solo si NO hay columnas numéricas.
    if nums == 0 && RE_PALABRAS_NIVEL2.is_match(&linea_lower) {
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

// ---------------------------------------------------------------------------
// Tests (espejo de test_analizador.py + porciones verificadas contra Python)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn mapeo(cantidad: i32, producto: i32, precio: i32, total: i32) -> MapeoColumnas {
        MapeoColumnas {
            cantidad: Some(cantidad),
            producto: Some(vec![producto]),
            precio_unitario: Some(precio),
            total: Some(total),
            descuento: None,
        }
    }

    // ---------- es_linea_util (test_analizador.py) ----------

    #[test]
    fn productos_legitimos_no_se_descartan() {
        let productos = [
            "GATORADE TOTAL 600ML $35.00",
            "CAJA DE MADERA 30X30 $120.00",
            "CAJA DE 24 CERVEZAS $540.00",
            "CANTIMPLORA LITRO $45.00",
            "PRECIOS JUSTOS COMBO $99.00",
            "OLIVA EXTRA VIRGEN 500ML $185.00",
            "COLONIA 900 PERFUME $150.00",
            "COCA-COLA 600ML $25.00",
            "TOTALMAX $18.00",
            "PAN WHITE 680GR $22.00",
            "DIVA PLATINUM $65.00",
        ];
        for p in productos {
            assert!(es_linea_util(p), "Producto perdido: {p}");
        }
    }

    #[test]
    fn cabeceras_se_descartan() {
        let cabeceras = [
            "TOTAL ---- $1,234.56",
            "EFECTIVO $500.00",
            "IVA 16%",
            "SUBTOTAL $1,064.28",
            "GRACIAS POR SU COMPRA",
            "METODO DE PAGO: TARJETA",
            "CFDI: 4D8F2A1",
            "CAJA: 3",
        ];
        for c in cabeceras {
            assert!(!es_linea_util(c), "Cabecera como producto: {c}");
        }
    }

    #[test]
    fn linea_vacia_no_es_util() {
        assert!(!es_linea_util(""));
        assert!(!es_linea_util("   "));
    }

    #[test]
    fn multiples_productos_por_linea() {
        assert!(es_linea_util("2 TAZAS $60.00 $120.00"));
        assert!(es_linea_util("Coca-Cola 600ML $25 $18"));
    }

    #[test]
    fn cabeceras_con_dos_puntos_se_descartan() {
        for c in [
            "TOTAL: $1,234.56",
            "CAJA: 3",
            "FECHA: 12/05/2026",
            "ATENDIO: MARIA",
            "METODO DE PAGO: EFECTIVO",
        ] {
            assert!(!es_linea_util(c), "{c}");
        }
    }

    #[test]
    fn productos_ambiguos_con_numeros_no_se_descartan() {
        for p in [
            "CAJA DE 24 CERVEZAS MODELO $540.00",
            "BEBIDA CAJA TETRA 1L $19.00",
            "ABARROTES VARIOS $5.00",
        ] {
            assert!(es_linea_util(p), "{p}");
        }
    }

    #[test]
    fn linea_solo_separadores() {
        assert!(!es_linea_util("----------------"));
        assert!(!es_linea_util("===================="));
        assert!(!es_linea_util("~~~~~~~"));
    }

    #[test]
    fn saludo_breve_no_crashea() {
        let res = es_linea_util("HOLA");
        assert!(res);
    }

    // ---------- parsear_linea (verificado contra Python) ----------

    #[test]
    fn parsea_linea_tipica() {
        let item = parsear_linea("2 COCA 25.00 50.00", &mapeo(0, 1, 2, 3), 4).unwrap();
        assert_eq!(item.producto, "COCA");
        assert_eq!(item.cantidad, 2.0);
        assert_eq!(item.precio_unitario, 25.0);
        assert_eq!(item.total, 50.0);
        assert_eq!(item.descuento, None);
    }

    #[test]
    fn parsea_linea_con_indice_negativo() {
        let m = MapeoColumnas {
            cantidad: Some(1),
            producto: Some(vec![0]),
            precio_unitario: Some(2),
            total: Some(-1),
            descuento: None,
        };
        let item = parsear_linea("COCA-COLA 2 25.00 50.00", &m, 4).unwrap();
        assert_eq!(item.producto, "COCA-COLA");
        assert_eq!(item.cantidad, 2.0);
        assert_eq!(item.total, 50.0);
    }

    #[test]
    fn parsea_rango_de_producto() {
        let m = MapeoColumnas {
            cantidad: None,
            producto: Some(vec![0, 1, 2, 3, 4]),
            precio_unitario: None,
            total: Some(-1),
            descuento: None,
        };
        let item =
            parsear_linea("CAJA DE 24 CERVEZAS MODELO $540.00", &m, 5).unwrap();
        assert_eq!(item.producto, "CAJA DE 24 CERVEZAS MODELO");
        assert_eq!(item.total, 540.0);
    }

    #[test]
    fn parsea_con_descuento() {
        // 3 COCA $25.00 $75.00 $5.00  (última columna = descuento)
        let m = MapeoColumnas {
            cantidad: Some(0),
            producto: Some(vec![1]),
            precio_unitario: Some(2),
            total: Some(3),
            descuento: Some(4),
        };
        let item = parsear_linea("3 COCA 25.00 75.00 5.00", &m, 5).unwrap();
        assert_eq!(item.descuento, Some(5.0));
    }

    #[test]
    fn linea_no_util_devuelve_none() {
        assert!(parsear_linea("GRACIAS POR SU COMPRA", &mapeo(0, 1, 2, 3), 4).is_none());
        assert!(parsear_linea("", &mapeo(0, 1, 2, 3), 4).is_none());
    }

    // ---------- extraer_fecha_hora_regex (verificado contra Python) ----------

    #[test]
    fn fecha_dd_mm_yyyy_y_hora() {
        let (f, h) = extraer_fecha_hora_regex("Fecha: 15/03/2024\nCompra: 14:32\n");
        assert_eq!(f.as_deref(), Some("2024-03-15"));
        assert_eq!(h.as_deref(), Some("14:32"));
    }

    #[test]
    fn fecha_iso_y_hora_pm() {
        let (f, h) = extraer_fecha_hora_regex("2024-03-15\nHora: 2:32 PM\n");
        assert_eq!(f.as_deref(), Some("2024-03-15"));
        assert_eq!(h.as_deref(), Some("14:32"));
    }

    #[test]
    fn fecha_con_mes_en_texto() {
        let (f, h) = extraer_fecha_hora_regex("15 de marzo de 2024\n14:32:05\n");
        assert_eq!(f.as_deref(), Some("2024-03-15"));
        assert_eq!(h.as_deref(), Some("14:32"));
    }

    #[test]
    fn fecha_generica_sin_encabezado() {
        let (f, _) = extraer_fecha_hora_regex("Compra\n2024-03-15\n");
        assert_eq!(f.as_deref(), Some("2024-03-15"));
    }

    // ---------- extraer_metodo_pago (verificado contra Python) ----------

    #[test]
    fn pago_tarjeta_debito() {
        let texto = "TOTAL $100.00\nMETODO DE PAGO: TARJETA DEBITO\n";
        assert_eq!(extraer_metodo_pago(texto), "tarjeta");
    }

    #[test]
    fn pago_tarjeta_sola_cae_a_efectivo() {
        let texto = "TOTAL $100.00\nMETODO DE PAGO: TARJETA\n";
        assert_eq!(extraer_metodo_pago(texto), "efectivo");
    }

    #[test]
    fn pago_efectivo_linea_con_monto() {
        let texto = "EFECTIVO........... $1,234.56\n";
        assert_eq!(extraer_metodo_pago(texto), "efectivo");
    }

    #[test]
    fn pago_transferencia() {
        let texto = "TOTAL $500.00\nFORMA DE PAGO: TRANSFERENCIA\n";
        assert_eq!(extraer_metodo_pago(texto), "transferencia");
    }

    #[test]
    fn pago_solo_ultimas_25_lineas() {
        let mut texto = String::from("TRANSFERENCIA $500\n");
        for _ in 0..30 {
            texto.push_str("ITEM DE PRUEBA $1.00\n");
        }
        // Método de pago fuera de las últimas 25 líneas → no se detecta.
        assert_eq!(extraer_metodo_pago(&texto), "efectivo");
    }
}