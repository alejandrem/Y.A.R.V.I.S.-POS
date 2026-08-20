// ============================================================
// totales.rs — Totales REALES del ticket (SUBTOTAL / IVA / TOTAL)
// ============================================================
//
// El procesador venía forzando `total = subtotal × 1.16`, ignorando los
// totales que imprime el propio ticket (descuentos, redondeos, promos).
// Aquí se extraen los valores de las líneas de cada segmento para que la
// venta guarde los números reales, con fallback al cálculo si no existen.

use regex::Regex;
use std::sync::LazyLock;

use super::es_linea_util;
use super::parser::limpiar_precio;

/// Totales reales declarados por el ticket (None si la línea no apareció).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TotalesTicket {
    pub subtotal: Option<f64>,
    pub iva: Option<f64>,
    pub total: Option<f64>,
}

static RE_SUBTOTAL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bsubtotal\b").expect("regex subtotal"));
static RE_IVA: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\biva\b").expect("regex iva"));
static RE_TOTAL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\btotal\b").expect("regex total"));
/// Cualquier monto: `$1,234.56`, `192.56`, `50`, `16`.
static RE_MONTO: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\$\s?\$?[\d.,]+|\$?\s?[\d][\d.,]*").expect("regex monto"));

/// Último monto numérico del texto, ignorando los porcentajes (p. ej. "16%").
fn ultimo_monto(tail: &str) -> Option<f64> {
    let mut mejor: Option<f64> = None;
    for m in RE_MONTO.find_iter(tail) {
        // Si el monto es un porcentaje ("16%") NO es el valor que buscamos.
        if tail[m.end()..].chars().next() == Some('%') {
            continue;
        }
        mejor = Some(limpiar_precio(m.as_str()));
    }
    mejor.filter(|v| *v > 0.0)
}

/// Extrae el monto de una línea tras su etiqueta. Rechaza líneas que no son
/// de totales limpios (ej. "TOTAL DE ARTICULOS 3" → contiene letras después
/// de la etiqueta) y los porcentajes de IVA.
fn monto_de_linea(linea: &str, etiqueta: &Regex) -> Option<f64> {
    let m = etiqueta.find(linea)?;
    let resto = &linea[m.end()..];
    // Líneas con letras tras la etiqueta no son un monto limpio.
    if resto.chars().any(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    ultimo_monto(resto)
}

/// Busca SUBTOTAL / IVA / TOTAL en un ticket y devuelve sus valores reales.
///
/// Solo mira líneas NO útiles (las de producto se ignoran). Para IVA, "16%" es
/// el porcentaje y no el monto: se descarta y se toma "26.56" de "IVA 16%: 26.56".
pub fn extraer_totales(texto: &str) -> TotalesTicket {
    let mut t = TotalesTicket::default();

    for linea in texto.lines() {
        if es_linea_util(linea) {
            continue;
        }
        if t.subtotal.is_none() && RE_SUBTOTAL.is_match(linea) {
            t.subtotal = monto_de_linea(linea, &RE_SUBTOTAL);
            continue;
        }
        if t.iva.is_none() && RE_IVA.is_match(linea) {
            t.iva = monto_de_linea(linea, &RE_IVA);
            continue;
        }
        if t.total.is_none() && RE_TOTAL.is_match(linea) {
            t.total = monto_de_linea(linea, &RE_TOTAL);
        }
    }

    t
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extrae_subtotal_iva_y_total_de_ticket_real() {
        let ticket = "Farmacia San Pablo\n\
                      Ticket: 004582\n\
                      Fecha: 15/03/2024\n\
                      2 Pan Bimbo Integral 42.00 84.00\n\
                      SUBTOTAL: 166.00\n\
                      IVA 16%: 26.56\n\
                      TOTAL: $192.56\n\
                      Gracias por su compra\n";

        let t = extraer_totales(ticket);
        assert_eq!(t.subtotal, Some(166.00));
        assert_eq!(t.iva, Some(26.56));
        assert_eq!(t.total, Some(192.56));
    }

    #[test]
    fn iva_porcentaje_no_se_confunde_con_monto() {
        assert_eq!(ultimo_monto("IVA 16%"), None);
        assert_eq!(ultimo_monto("16%: 26.56"), Some(26.56));
        assert_eq!(ultimo_monto(": $192.56"), Some(192.56));
    }

    #[test]
    fn sin_totales_devuelve_todo_none() {
        let ticket = "2 TAZAS $60.00 $120.00\n1 PLATO $80.00 $80.00\n";
        let t = extraer_totales(ticket);
        assert_eq!(t, TotalesTicket::default());
    }

    #[test]
    fn total_con_separador_y_sin_decimales() {
        assert_eq!(
            monto_de_linea("TOTAL ---- $1,234.56", &RE_TOTAL),
            Some(1234.56)
        );
        assert_eq!(monto_de_linea("TOTAL $50", &RE_TOTAL), Some(50.0));
    }

    #[test]
    fn linea_con_etiqueta_y_letras_se_descarta() {
        assert_eq!(monto_de_linea("TOTAL DE ARTICULOS 3", &RE_TOTAL), None);
    }
}
