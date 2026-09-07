// ============================================================
// Encabezado del ticket: metadatos que no son líneas de producto
// ============================================================

use regex::Regex;
use std::sync::LazyLock;

/// Extrae el cajero/empleado que atendió (primeras 10 líneas del ticket).
pub fn extraer_cajero(texto: &str) -> String {
    for linea in texto.lines().take(10) {
        let lower = linea.to_lowercase();
        if lower.contains("cajero") || lower.contains("empleado") || lower.contains("vendedor") {
            if let Some(idx) = linea.find(':') {
                return limpiar_nombre_cajero(&linea[idx + 1..]);
            }
        }
    }
    "SISTEMA".to_string()
}

/// "MARIA G.     HORA: 09:14:22" → "MARIA G.": las impresoras ponen hora
/// y fecha en el MISMO renglón del cajero y antes se tragaba todo junto
/// (cientos de "empleados" distintos = nombre + hora).
fn limpiar_nombre_cajero(valor: &str) -> String {
    static RE_CORTE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)\b(HORA|FECHA)\b|\b\d{1,2}:\d{2}|\b\d{1,2}/\d{1,2}/\d{2,4}\b")
            .expect("regex corte cajero")
    });
    let hasta = RE_CORTE.find(valor).map(|m| m.start()).unwrap_or(valor.len());
    let nombre = valor[..hasta].trim().to_string();
    if nombre.is_empty() {
        "SISTEMA".to_string()
    } else {
        nombre
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cajero_se_corta_antes_de_la_hora() {
        assert_eq!(
            extraer_cajero("CAJERO: MARIA G.         HORA: 09:14:22"),
            "MARIA G."
        );
        assert_eq!(extraer_cajero("CAJERO: JUANA P."), "JUANA P.");
        assert_eq!(
            extraer_cajero("EMPLEADO: PEDRO A. FECHA: 03/03/2026"),
            "PEDRO A."
        );
        assert_eq!(extraer_cajero("VENDEDOR: LUIS"), "LUIS");
        assert_eq!(extraer_cajero("TICKET SIN ENCABEZADO"), "SISTEMA");
    }
}
