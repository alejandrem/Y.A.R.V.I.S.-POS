use regex::Regex;
use std::sync::LazyLock;

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

/// True si la línea contiene una fecha reconocible (ISO, DD/MM/YYYY,
/// DD-MM-YYYY o "15 de marzo de 2024"). Usada por el segmentador para
/// detectar el encabezado de un ticket nuevo.
pub fn tiene_fecha(linea: &str) -> bool {
    RE_FECHA_ISO.is_match(linea)
        || RE_FECHA_SLASH.is_match(linea)
        || RE_FECHA_GUION.is_match(linea)
        || RE_FECHA_MES.is_match(linea)
}
