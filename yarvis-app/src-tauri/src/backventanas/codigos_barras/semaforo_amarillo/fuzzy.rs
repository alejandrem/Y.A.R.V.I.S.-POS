// ============================================================
// fuzzy — Parecido de texto por trigramas (Jaccard).
//
// Tolera typos y pegados ("cocacola" vs "coca cola", "sabritas"
// vs "sabrita"): lo que el match exacto del verde no ve. Puro y
// determinista; el texto llega YA normalizado.
// ============================================================

use std::collections::HashSet;

/// Trigamas por palabra ("coca" -> coc, oca). Palabras cortas
/// entran enteras para no perderlas ("3l" suma igual).
pub fn trigramas(norm: &str) -> Vec<String> {
    let mut out = vec![];
    for w in norm.split_whitespace() {
        let chars: Vec<char> = w.chars().collect();
        if chars.len() < 3 {
            out.push(w.to_string());
            continue;
        }
        for i in 0..=chars.len() - 3 {
            out.push(chars[i..i + 3].iter().collect());
        }
    }
    out
}

/// Jaccard entre multisets: interseccion / union (0..1).
pub fn jaccard(a: &[String], b: &[String]) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let set_b: HashSet<&str> = b.iter().map(|s| s.as_str()).collect();
    let mut inter = 0usize;
    for t in a {
        if set_b.contains(t.as_str()) {
            inter += 1;
        }
    }
    let union = a.len() + b.len() - inter;
    inter as f64 / union as f64
}

/// 1.0 si iguales, 0.0 si vacio, Jaccard en otro caso.
pub fn similitud_fuzzy(a_norm: &str, b_norm: &str) -> f64 {
    if a_norm.is_empty() || b_norm.is_empty() {
        return 0.0;
    }
    if a_norm == b_norm {
        return 1.0;
    }
    jaccard(&trigramas(a_norm), &trigramas(b_norm))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iguales_y_vacios() {
        assert_eq!(similitud_fuzzy("coca", "coca"), 1.0);
        assert_eq!(similitud_fuzzy("", "coca"), 0.0);
    }

    #[test]
    fn tolera_pegados_y_typos() {
        let pegado = similitud_fuzzy("cocacola", "coca cola");
        assert!(pegado > 0.3, "pegado = {pegado}");
        let typo = similitud_fuzzy("sabritas", "sabrita");
        assert!(typo > 0.5, "typo = {typo}");
    }

    #[test]
    fn distintos_quedan_abajo() {
        let s = similitud_fuzzy("coca cola", "pepsi");
        assert!(s < 0.2, "distintos = {s}");
    }
}
