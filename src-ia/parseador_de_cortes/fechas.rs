// ============================================================
// fechas — Fecha del corte a ISO local "YYYY-MM-DD HH:MM:SS".
//
// Formatos vistos: `01/01/2025 11:25:57 a. m.` (12h con marcador) y
// `31/03/2026 09:35:08 p. m.`. También se acepta 24h sin marcador.
// ============================================================

use regex::Regex;
use std::sync::LazyLock;

static RE_HORA: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^\s*(\d{1,2}):(\d{2})(?::(\d{2}))?\s*([ap])\.?\s*m\.?\s*$").expect("regex hora corte")
});

/// Convierte fecha dd/mm/yyyy + hora a ISO. None si es ilegible.
pub fn a_iso(fecha: &str, hora: &str) -> Option<String> {
    let p: Vec<&str> = fecha.split('/').collect();
    if p.len() != 3 {
        return None;
    }
    let (d, m, y) = (p[0].parse::<u32>().ok()?, p[1].parse::<u32>().ok()?, p[2].parse::<u32>().ok()?);
    if !(1..=31).contains(&d) || !(1..=12).contains(&m) || y < 1900 || y > 2200 {
        return None;
    }

    // Sin marcador am/pm se asume 24h tal cual.
    let h24 = match RE_HORA.captures(hora) {
        None => {
            let q: Vec<&str> = hora.trim().split(':').collect();
            if q.len() < 2 {
                return None;
            }
            (q[0].parse::<u32>().ok()?, q[1].parse::<u32>().ok()?, q.get(2).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0))
        }
        Some(c) => {
            let mut h = c[1].parse::<u32>().ok()?;
            let min = c[2].parse::<u32>().ok()?;
            let seg = c.get(3).map(|v| v.as_str().parse::<u32>().unwrap_or(0)).unwrap_or(0);
            if h < 1 || h > 12 || min > 59 || seg > 59 {
                return None;
            }
            let pm = c[4].eq_ignore_ascii_case("p");
            if pm && h < 12 {
                h += 12;
            } else if !pm && h == 12 {
                h = 0;
            }
            (h, min, seg)
        }
    };
    if h24.0 > 23 || h24.1 > 59 || h24.2 > 59 {
        return None;
    }

    Some(format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", y, m, d, h24.0, h24.1, h24.2))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manana_y_tarde_del_dataset() {
        assert_eq!(a_iso("01/01/2025", "11:25:57 a. m."), Some("2025-01-01 11:25:57".into()));
        assert_eq!(a_iso("31/03/2026", "09:35:08 p. m."), Some("2026-03-31 21:35:08".into()));
    }

    #[test]
    fn bordes_doce() {
        assert_eq!(a_iso("01/01/2025", "12:00:00 a. m."), Some("2025-01-01 00:00:00".into()));
        assert_eq!(a_iso("01/01/2025", "12:00:00 p. m."), Some("2025-01-01 12:00:00".into()));
    }

    #[test]
    fn veinticuatro_horas_sin_marcador() {
        assert_eq!(a_iso("05/06/2024", "21:05:08"), Some("2024-06-05 21:05:08".into()));
    }

    #[test]
    fn basura_devuelve_none() {
        assert_eq!(a_iso("2025-01-01", "11:25"), None);
        assert_eq!(a_iso("01/01/2025", "25:00"), None);
        assert_eq!(a_iso("99/99/9999", "10:00 a. m."), None);
    }
}
