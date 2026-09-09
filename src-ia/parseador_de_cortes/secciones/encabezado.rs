// ============================================================
// encabezado — Título, folio, empresa, estación, fecha y cajero.
//
// El título acepta variantes (`*** CORTE Z ...`, `CORTE DE CAJA X`):
// lo que manda es la letra X/Z. Si no hay título, no es corte
// (los tickets sueltos caen a `NoEsCorte` en el clasificador).
// ============================================================

use regex::Regex;
use std::sync::LazyLock;

use crate::parseador_de_cortes::tipos::ClaseArchivo;
use crate::parseador_de_cortes::valores::fechas::a_iso;

static RE_TITULO: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)CORTE\s+(?:DE\s+CAJA\s+)?([XZ])\b").expect("regex titulo corte")
});
static RE_MONEDA: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)MONEDA\s*:\s*([A-Z]{3})").expect("regex moneda"));
static RE_FOLIO: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?im)^\**\s*Corte\s+(?:de\s+caja\s+)?[XZ]\s+(\d+)").expect("regex folio corte")
});
static RE_ESTACION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^([A-Za-z0-9_]+)\s+(\d{2}/\d{2}/\d{4})\s+(\d{1,2}:\d{2}(?::\d{2})?\s*(?:[AaPp]\.\s*[Mm]\.?)?)\s*$")
        .expect("regex estacion")
});
static RE_CAJERO: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?im)^Cajero\s*:\s*(.+?)\s*$").expect("regex cajero"));

/// Clasifica un archivo suelto (los datasets mezclan tickets y cortes).
pub fn clasificar(texto: &str) -> ClaseArchivo {
    match RE_TITULO.captures(texto).and_then(|c| c.get(1)).map(|m| m.as_str().to_ascii_uppercase()) {
        Some(t) if t == "X" => ClaseArchivo::CorteX,
        Some(t) if t == "Z" => ClaseArchivo::CorteZ,
        _ => ClaseArchivo::NoEsCorte,
    }
}

/// Moneda del título (`MONEDA:MXN`), default MXN.
pub fn extraer_moneda(texto: &str) -> String {
    RE_MONEDA
        .captures(texto)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_ascii_uppercase())
        .unwrap_or_else(|| "MXN".to_string())
}

/// Folio del corte (`*** Corte Z 54`, `Corte X 1`).
pub fn extraer_folio(texto: &str) -> Option<String> {
    RE_FOLIO.captures(texto).and_then(|c| c.get(1)).map(|m| m.as_str().to_string())
}

/// Empresa: primera línea con contenido de la ZONA DE ENCABEZADO
/// (antes de que empiecen las secciones), que no sea folio, estación
/// ni cajero (p. ej. `EMPRESA, S.A. DE C.V.`). Si el corte arranca
/// directo en secciones, no hay empresa (None), no se inventa una
/// pescando renglones de Ingresos.
pub fn extraer_empresa(texto: &str) -> Option<String> {
    let lineas: Vec<&str> = texto.lines().collect();
    let titulo = lineas.iter().position(|l| RE_TITULO.is_match(l))?;
    for l in lineas.iter().skip(titulo + 1) {
        let t = l.trim();
        if t.is_empty() || RE_FOLIO.is_match(t) || RE_ESTACION.is_match(t) || RE_CAJERO.is_match(t) {
            continue;
        }
        // Ya empezaron las secciones: aquí no vive la empresa.
        if super::marcadores::marca_de(t).is_some() {
            break;
        }
        // Marcadores de página sueltos (`p0`): no son empresa.
        if t.len() <= 3 || t.starts_with('*') {
            continue;
        }
        return Some(t.to_string());
    }
    None
}

/// Estación + fecha ISO (`ESTACION01 01/01/2025 11:25:57 a. m.`).
pub fn extraer_estacion_fecha(texto: &str) -> (Option<String>, Option<String>) {
    match RE_ESTACION.captures(texto) {
        None => (None, None),
        Some(c) => {
            let estacion = c[1].to_string();
            let fecha = a_iso(&c[2], &c[3]);
            (Some(estacion), fecha)
        }
    }
}

/// Cajero del encabezado (`Cajero: GENERAL`); SISTEMA si no viene
/// (los Z del dataset no lo traen).
pub fn extraer_cajero(texto: &str) -> String {
    RE_CAJERO
        .captures(texto)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string())
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| "SISTEMA".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clasifica_x_z_y_descarta_tickets() {
        assert_eq!(clasificar("*** CORTE Z EN MONEDA:MXN***\n*** Corte Z 54"), ClaseArchivo::CorteZ);
        assert_eq!(clasificar("p0   *** CORTE X EN MONEDA:MXN***\nCorte X 1"), ClaseArchivo::CorteX);
        assert_eq!(clasificar("CORTE DE CAJA Z\nCorte de caja Z 12"), ClaseArchivo::CorteZ);
        assert_eq!(clasificar("2 Pan Bimbo 42.00 84.00\nTOTAL: $84.00"), ClaseArchivo::NoEsCorte);
    }

    #[test]
    fn encabezado_completo() {
        let t = "*** CORTE Z EN MONEDA:MXN***\nEMPRESA, S.A. DE C.V.\n\n*** Corte Z 54\nESTACION01 01/01/2025 11:25:57 a. m.\n**Ingresos**";
        assert_eq!(extraer_folio(t), Some("54".into()));
        assert_eq!(extraer_empresa(t), Some("EMPRESA, S.A. DE C.V.".into()));
        assert_eq!(extraer_estacion_fecha(t), (Some("ESTACION01".into()), Some("2025-01-01 11:25:57".into())));
        assert_eq!(extraer_cajero(t), "SISTEMA");
        assert_eq!(extraer_moneda(t), "MXN");
    }

    #[test]
    fn folio_sin_asteriscos_y_cajero_solo_en_x() {
        let t = "Corte X 1\nESTACION01 31/03/2026 09:35:08 p. m.\nCajero: GENERAL";
        assert_eq!(extraer_folio(t), Some("1".into()));
        assert_eq!(extraer_cajero(t), "GENERAL");
        assert_eq!(extraer_cajero("TICKET SIN ENCABEZADO"), "SISTEMA");
    }

    #[test]
    fn sin_empresa_no_se_inventa_con_renglones() {
        let t = "*** CORTE X EN MONEDA:MXN***\nCorte X 2\n**Ingresos**\nEFE Ventas $900.00";
        assert_eq!(extraer_empresa(t), None);
    }
}
