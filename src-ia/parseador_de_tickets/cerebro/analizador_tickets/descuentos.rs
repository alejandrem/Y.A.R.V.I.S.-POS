use regex::Regex;
use std::sync::LazyLock;

static RE_DESCUENTO_SEPARADO: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^\s*descuento\b.*?(-?\$?\s*[\d,]+(?:\.\d{1,2})?)\s*$")
        .expect("regex descuento separado")
});

pub fn descuento_separado(linea: &str) -> Option<f64> {
    let monto = RE_DESCUENTO_SEPARADO.captures(linea)?.get(1)?.as_str();
    let limpio: String = monto
        .chars()
        .filter(|c| !matches!(c, '$' | ',' | ' '))
        .collect();
    limpio.parse::<f64>().ok().map(f64::abs)
}

#[cfg(test)]
mod tests;
