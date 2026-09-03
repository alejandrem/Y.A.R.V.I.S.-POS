use regex::Regex;
use std::sync::LazyLock;

// ---------------------------------------------------------------------------
// Meses (mapa nombre → número). Incluye español completo, abreviaturas
// españolas e inglesas ("Mar 15 2024" llega de tickets de sistemas en inglés).
// ---------------------------------------------------------------------------

const MESES: &[(&str, &str)] = &[
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
    ("sept", "09"),
    ("oct", "10"),
    ("nov", "11"),
    ("dic", "12"),
    ("dec", "12"),
];

fn mes_numero(nombre: &str) -> Option<&'static str> {
    let nombre = nombre.to_lowercase();
    MESES
        .iter()
        .find(|(k, _)| *k == nombre)
        .map(|(_, v)| *v)
}

// ---------------------------------------------------------------------------
// Validación de fechas reales
// ---------------------------------------------------------------------------

fn es_bisiesto(anio: i32) -> bool {
    (anio % 4 == 0 && anio % 100 != 0) || anio % 400 == 0
}

fn dias_del_mes(mes: i32, anio: i32) -> i32 {
    match mes {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if es_bisiesto(anio) => 29,
        2 => 28,
        _ => 0,
    }
}

/// Año civil actual (mismo algoritmo civil_from_days de `almacen::ahora_iso_utc`,
/// sin dependencias). Sirve de techo: un ticket no puede venir del futuro.
fn anio_actual() -> i32 {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let dias = (secs / 86400) as i64;
    let z = dias + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let anio = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let mes = if mp < 10 { mp + 3 } else { mp - 9 };
    (if mes <= 2 { anio + 1 } else { anio }) as i32
}

/// Construye "YYYY-MM-DD" SOLO si la fecha es posible: mes 1-12, día dentro
/// del mes (bisiestos incluidos) y año en un rango de tickets reales
/// (1970 … año actual + 1). Antes de esto, "99/99/9999" entraba literal a
/// `ventas.fecha` como "9999-99-99".
fn fecha(dia: i32, mes: i32, anio: i32) -> Option<String> {
    let max_anio = anio_actual() + 1;
    if !(1900..=max_anio).contains(&anio) {
        return None;
    }
    if !(1..=12).contains(&mes) {
        return None;
    }
    if dia < 1 || dia > dias_del_mes(mes, anio) {
        return None;
    }
    Some(format!("{anio:04}-{mes:02}-{dia:02}"))
}

/// Años de 2 dígitos con pivote deslizante: se asumen 20XX salvo que eso caiga
/// en el futuro, en cuyo caso son 19XX. Con año actual 2026: "26"→2026,
/// "27"→2027, "98"→1998 (antes TODOS iban a 20XX: un ticket de 1998 salía 2098).
fn anio_desde_yy(yy: i32) -> i32 {
    let corto = 2000 + yy;
    if corto > anio_actual() + 1 {
        1900 + yy
    } else {
        corto
    }
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
/// Mes PRIMERO (formato inglés/abreviado): "Mar 15 2024", "Mar 15, 2024",
/// "15" no va primero porque choca con el formato español día-primero.
static RE_FECHA_MES_PRIMERO: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\b(enero|febrero|marzo|abril|mayo|junio|julio|agosto|septiembre|octubre|noviembre|diciembre|jan|feb|mar|apr|may|jun|jul|aug|sep|sept|oct|nov|dic|dec)\.?\s+(\d{1,2})\s*,?\s+(\d{4})\b",
    )
    .expect("regex fecha mes primero")
});
static RE_HORA: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(\d{1,2}):(\d{2})(?::(\d{2}))?\s*(AM|PM|am|pm)?\b").expect("regex hora")
});

/// Busca fecha y hora en el texto del ticket usando regex.
/// Retorna `(fecha_iso "YYYY-MM-DD" o None, hora "HH:MM" o None)`.
/// Fechas/horas IMPOSIBLES (99/99/9999, 31/02/2026, 75:99) se rechazan y se
/// sigue buscando en las líneas siguientes; nunca entran a la DB.
pub fn extraer_fecha_hora_regex(texto: &str) -> (Option<String>, Option<String>) {
    let mut fecha_encontrada: Option<String> = None;
    let mut hora: Option<String> = None;

    for linea in texto.lines() {
        if fecha_encontrada.is_none() {
            // Patrones de fecha ordenados de más específico a menos.
            if let Some(c) = RE_FECHA_ISO.captures(linea) {
                fecha_encontrada = fecha(
                    c[3].parse().unwrap_or(0),
                    c[2].parse().unwrap_or(0),
                    c[1].parse().unwrap_or(0),
                );
            } else if let Some(c) = RE_FECHA_SLASH.captures(linea) {
                // DD/MM/YYYY o DD/MM/YY (convención mexicana; MM/DD es
                // ambiguo y se documenta como no soportado).
                let anio: i32 = c[3].parse().unwrap_or(0);
                let anio = if c[3].len() == 4 {
                    anio
                } else {
                    anio_desde_yy(anio)
                };
                fecha_encontrada = fecha(
                    c[1].parse().unwrap_or(0),
                    c[2].parse().unwrap_or(0),
                    anio,
                );
            } else if let Some(c) = RE_FECHA_GUION.captures(linea) {
                // DD-MM-YYYY
                fecha_encontrada = fecha(
                    c[1].parse().unwrap_or(0),
                    c[2].parse().unwrap_or(0),
                    c[3].parse().unwrap_or(0),
                );
            } else if let Some(c) = RE_FECHA_MES.captures(linea) {
                // "15 de marzo de 2024" o "15 marzo 2024"
                if let Some(mes) = mes_numero(&c[2]) {
                    fecha_encontrada = fecha(
                        c[1].parse().unwrap_or(0),
                        mes.parse().unwrap_or(0),
                        c[3].parse().unwrap_or(0),
                    );
                }
            } else if let Some(c) = RE_FECHA_MES_PRIMERO.captures(linea) {
                // "Mar 15 2024" / "Mar 15, 2024" (formato inglés).
                if let Some(mes) = mes_numero(&c[1]) {
                    fecha_encontrada = fecha(
                        c[2].parse().unwrap_or(0),
                        mes.parse().unwrap_or(0),
                        c[3].parse().unwrap_or(0),
                    );
                }
            }
        }

        if hora.is_none() {
            for c in RE_HORA.captures_iter(linea) {
                let mut h: i32 = c[1].parse().unwrap_or(0);
                let mins: i32 = c[2].parse().unwrap_or(0);
                // Hora imposible ("75:99"): se ignora y se sigue buscando.
                if mins > 59 {
                    continue;
                }
                if let Some(ampm) = c.get(4) {
                    match ampm.as_str().to_uppercase().as_str() {
                        "PM" if h < 12 => h += 12,
                        "AM" if h == 12 => h = 0,
                        _ => {}
                    }
                }
                if h > 23 {
                    continue;
                }
                hora = Some(format!("{h:02}:{mins:02}"));
                break;
            }
        }

        if fecha_encontrada.is_some() && hora.is_some() {
            break;
        }
    }

    (fecha_encontrada, hora)
}

/// True si la línea contiene una fecha reconocible (ISO, DD/MM/YYYY,
/// DD-MM-YYYY o "15 de marzo de 2024"). Usada por el segmentador para
/// detectar el encabezado de un ticket nuevo. NO valida rangos (el
/// segmentador solo necesita saber si "huele" a fecha).
pub fn tiene_fecha(linea: &str) -> bool {
    RE_FECHA_ISO.is_match(linea)
        || RE_FECHA_SLASH.is_match(linea)
        || RE_FECHA_GUION.is_match(linea)
        || RE_FECHA_MES.is_match(linea)
        || RE_FECHA_MES_PRIMERO.is_match(linea)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fechas_imposibles_se_rechazan() {
        for s in [
            "99/99/9999",
            "31/02/2026",
            "29 de febrero de 2023",
            "2024-13-32",
            "00/00/2020",
            "15/03/1800",
        ] {
            let (f, _) = extraer_fecha_hora_regex(s);
            assert_eq!(f, None, "{s} no debió producir fecha");
        }
    }

    #[test]
    fn bisiestos_validos_pasan() {
        let (f, _) = extraer_fecha_hora_regex("29 de febrero de 2024");
        assert_eq!(f.as_deref(), Some("2024-02-29"));
        let (f, _) = extraer_fecha_hora_regex("29/02/2024");
        assert_eq!(f.as_deref(), Some("2024-02-29"));
    }

    #[test]
    fn anio_de_dos_digitos_con_pivote() {
        let actual = anio_actual();
        // Un año corto que NO cae en el futuro → 20XX.
        let reciente = format!("15/03/{:02}", actual % 100);
        let (f, _) = extraer_fecha_hora_regex(&reciente);
        assert_eq!(f, Some(format!("{actual}-03-15")));
        // Años claramente viejos no pueden ser futuro → 19XX.
        let (f, _) = extraer_fecha_hora_regex("15/03/98");
        assert_eq!(f.as_deref(), Some("1998-03-15"));
        let (f, _) = extraer_fecha_hora_regex("15/03/69");
        assert_eq!(f.as_deref(), Some("1969-03-15"));
    }

    #[test]
    fn fecha_mes_primero_ingles() {
        let (f, _) = extraer_fecha_hora_regex("Mar 15 2024");
        assert_eq!(f.as_deref(), Some("2024-03-15"));
        let (f, _) = extraer_fecha_hora_regex("Ventas\nDec 3, 2023\nTOTAL $1.00");
        assert_eq!(f.as_deref(), Some("2023-12-03"));
    }

    #[test]
    fn horas_imposibles_se_ignoran_y_se_sigue_buscando() {
        // La hora rota NO gana el slot: vale la siguiente línea.
        let (_, h) = extraer_fecha_hora_regex("75:99\n14:32");
        assert_eq!(h.as_deref(), Some("14:32"));
        let (_, h) = extraer_fecha_hora_regex("25:00");
        assert_eq!(h, None);
    }

    #[test]
    fn fecha_invalida_no_bloquea_a_la_siguiente_linea() {
        let (f, _) = extraer_fecha_hora_regex("31/02/2026\n12/05/2026");
        assert_eq!(f.as_deref(), Some("2026-05-12"));
    }
}
