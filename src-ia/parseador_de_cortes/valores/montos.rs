// ============================================================
// montos — Limpieza de montos impresos a centavos INTEGER.
//
// Acepta `$2,462.80`, `2,462.80`, `$702.00` y el tramposo `$.00`
// (signo sin entero = cero, NO error). Devuelve None si no hay
// número que rescatar.
// ============================================================

/// Convierte un monto impreso a centavos. None si es ilegible.
pub fn limpiar_monto(raw: &str) -> Option<i64> {
    let mut s = raw.trim();
    // Signos y espacios al inicio.
    s = s.trim_start_matches(['$', ' ']);
    if s.is_empty() {
        return None;
    }
    // Separador de miles.
    let sin_miles: String = s.chars().filter(|c| *c != ',').collect();
    // Debe quedar algo numérico (dígitos y a lo más un punto).
    if !sin_miles.chars().any(|c| c.is_ascii_digit()) {
        return None;
    }
    if sin_miles.chars().filter(|c| *c == '.').count() > 1 {
        return None;
    }
    let pesos: f64 = sin_miles.parse().ok()?;
    if !pesos.is_finite() {
        return None;
    }
    Some((pesos * 100.0).round() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn montos_reales_del_dataset() {
        assert_eq!(limpiar_monto("$2,462.80"), Some(246280));
        assert_eq!(limpiar_monto("  $2,023.80"), Some(202380));
        assert_eq!(limpiar_monto("$439.00"), Some(43900));
        assert_eq!(limpiar_monto("$702.00"), Some(70200));
        assert_eq!(limpiar_monto("94.00"), Some(9400));
        assert_eq!(limpiar_monto("$136.80"), Some(13680));
    }

    #[test]
    fn signo_sin_entero_es_cero_no_error() {
        assert_eq!(limpiar_monto("$.00"), Some(0));
        assert_eq!(limpiar_monto("$ .00"), Some(0));
    }

    #[test]
    fn basura_devuelve_none() {
        assert_eq!(limpiar_monto(""), None);
        assert_eq!(limpiar_monto("$"), None);
        assert_eq!(limpiar_monto("---"), None);
        assert_eq!(limpiar_monto("1.2.3"), None);
    }
}
