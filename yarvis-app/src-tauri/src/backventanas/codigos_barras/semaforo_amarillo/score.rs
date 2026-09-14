// ============================================================
// score — `0.6*coseno + 0.4*fuzzy ± marca` (issue #11).
//
// Embeddings con el `HashEmbedder` existente (coseno, determinista,
// sin modelo que descargar) + fuzzy de texto sobre el nombre.
// Marca: la misma suma poco, la DISTINTA resta fuerte (Coca vs
// Pepsi con igual medida jamas deben sugerirse).
// ============================================================

use super::fuzzy::similitud_fuzzy;
use src_ia::embeddings::{cosine_similarity, Embedder, HashEmbedder};

/// 60% semantica, 40% texto: el embedding manda, el fuzzy desempata.
pub const PESO_EMBEDDING: f64 = 0.6;
/// Misma marca suma; distinta resta FUERTE.
pub const DELTA_MARCA_MISMA: f64 = 0.05;
pub const DELTA_MARCA_DISTINTA: f64 = -0.30;

fn marca_norm(m: Option<&str>) -> Option<String> {
    let t = m?.trim().to_lowercase();
    if t.is_empty() {
        None
    } else {
        Some(t)
    }
}

/// +0.05 misma, -0.30 distinta, 0.0 si falta dato (ausente != distinta).
pub fn delta_marca(marca_ticket: Option<&str>, marca_prod: Option<&str>) -> f64 {
    match (marca_norm(marca_ticket), marca_norm(marca_prod)) {
        (Some(a), Some(b)) if a == b => DELTA_MARCA_MISMA,
        (Some(_), Some(_)) => DELTA_MARCA_DISTINTA,
        _ => 0.0,
    }
}

/// Coseno entre embeddings del HashEmbedder (0.0 si no hay texto).
pub fn coseno(ticket_norm: &str, prod_norm: &str) -> f64 {
    let emb = HashEmbedder;
    let pareja = emb.texto_a_embedding(ticket_norm);
    let otra = emb.texto_a_embedding(prod_norm);
    match (pareja, otra) {
        (Some(a), Some(b)) => cosine_similarity(&a, &b),
        _ => 0.0,
    }
}

/// Mezcla + clamp a [0,1] + redondeo a 4 decimales.
pub fn combinar(cos: f64, fuzzy: f64, delta: f64) -> f64 {
    let bruto = PESO_EMBEDDING * cos + (1.0 - PESO_EMBEDDING) * fuzzy + delta;
    (bruto.clamp(0.0, 1.0) * 10000.0).round() / 10000.0
}

/// Score final ticket vs producto (nombres YA normalizados).
pub fn puntuar(
    ticket_norm: &str,
    prod_norm: &str,
    marca_ticket: Option<&str>,
    marca_prod: Option<&str>,
) -> f64 {
    let c = coseno(ticket_norm, prod_norm);
    let f = similitud_fuzzy(ticket_norm, prod_norm);
    combinar(c, f, delta_marca(marca_ticket, marca_prod))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marca_distinta_hunde() {
        let misma = puntuar("coca 600 ml", "coca 600 ml", Some("Coca"), Some("coca"));
        let distinta = puntuar("coca 600 ml", "coca 600 ml", Some("Pepsi"), Some("coca"));
        assert!(distinta + 0.2 < misma, "{distinta} vs {misma}");
    }

    #[test]
    fn sin_marca_no_castiga() {
        let s = puntuar("sabritas 42 g", "sabritas 42 g", None, Some("Sabritas"));
        assert!(s > 0.8, "score = {s}");
    }
}
