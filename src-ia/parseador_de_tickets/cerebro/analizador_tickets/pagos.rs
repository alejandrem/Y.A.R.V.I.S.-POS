use regex::Regex;
use std::sync::LazyLock;

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
    // "TARJETA" suelto (sin DÉBITO/CRÉDITO) también es pago con tarjeta.
    (
        r"(?:FORMA\s+DE\s+PAGO|METODO\s+DE\s+PAGO|PAGO\s+CON|PAGO:?)\s*:?\s*(\d+\s*[-:]?\s*)?(TARJETA)",
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
    (r"^TARJETA[\s\.]+(?:\$|[\d])", "tarjeta"),
    (r"^DEBITO[\s\.]+(?:\$|[\d])", "tarjeta"),
    (r"^CREDITO[\s\.]+(?:\$|[\d])", "tarjeta"),
    (r"^TRANSFERENCIA[\s\.]+(?:\$|[\d])", "transferencia"),
    (r"^CHEQUE[\s\.]+(?:\$|[\d])", "cheque"),
    (r"^EFECTIVO\s*:", "efectivo"),
    (r"^TARJETA\s+DEBITO\s*:", "tarjeta"),
    (r"^TARJETA\s+CREDITO\s*:", "tarjeta"),
    (r"^TARJETA\s*:", "tarjeta"),
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

