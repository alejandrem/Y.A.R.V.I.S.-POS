// ============================================================
// umbral — LA constante del amarillo (issue #11).
//
// "El umbral vive en UNA constante para calibrarlo con datos
// reales." Es ESTA: `UMBRAL_AMARILLO`. >= sugiere, < cae a rojo.
// Nadie mas en el codigo hardcodea 0.80: todo pasa por `decide`.
// ============================================================

/// Score minimo para sugerir. Si los NO se acumulan arriba de este
/// valor (ver `stats_rechazos`), estaba bajo: subelo a mano aqui.
pub const UMBRAL_AMARILLO: f64 = 0.80;

/// ¿Se sugiere o cae a rojo?
pub fn decide(score: f64) -> bool {
    score >= UMBRAL_AMARILLO
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borde_exactamente_en_umbral() {
        assert!(decide(0.80));
        assert!(decide(0.99));
    }

    #[test]
    fn debajo_cae_a_rojo() {
        assert!(!decide(0.7999));
        assert!(!decide(0.0));
    }
}
