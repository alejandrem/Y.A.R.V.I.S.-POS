// ============================================================
// embeddings — Modulo de embeddings interino de Y.A.R.V.I.S.
//
// Antes vivia en `parseador_de_tickets/cerebro/vinculador_inventario/similitud.rs`.
// Se separo a `src-ia/embeddings/` para aislar la logica de normalizacion,
// coseno y trait Embedder del vinculador y dejar via libre al futuro
// modelo de embeddings propio.
//
// Estado actual (interino): TF-IDF + fuzzy no vectorial se resuelve aca
// via `normalizar` + `cosine_similarity`; el Embedder real (modelo propio)
// se inyectara cuando exista (Fase 4). Sin embedder solo funciona el
// match exacto y `por_embedding` queda en 0.
// ============================================================

use regex::Regex;
use std::sync::LazyLock;

static RE_NO_WORD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[^\w\s]").expect("regex no-word"));

static RE_ESPACIOS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").expect("regex espacios"));

/// Normaliza un nombre para comparacion: minusculas, sin especiales, sin
/// espacios extra. Espejo de `_normalizar`.
pub fn normalizar(nombre: &str) -> String {
    let limpio = nombre.trim().to_lowercase();
    let limpio = RE_NO_WORD.replace_all(&limpio, "").into_owned();
    RE_ESPACIOS.replace_all(&limpio, " ").into_owned()
}

/// Deserializa un BLOB de SQLite a vector f32 (little-endian). Espejo de
/// `blob_a_embedding` (384 floats filas de `knowledge_base.embedding`).
pub fn blob_a_embedding(blob: &[u8]) -> Vec<f32> {
    blob.chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

/// Similitud coseno entre dos vectores. Espejo de `cosine_similarity`.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    let dot: f64 = a
        .iter()
        .zip(b.iter())
        .map(|(x, y)| (*x as f64) * (*y as f64))
        .sum();
    let norm_a: f64 = a
        .iter()
        .map(|x| (*x as f64) * (*x as f64))
        .sum::<f64>()
        .sqrt();
    let norm_b: f64 = b
        .iter()
        .map(|x| (*x as f64) * (*x as f64))
        .sum::<f64>()
        .sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

// ---------------------------------------------------------------------------
// Tipos
// ---------------------------------------------------------------------------

/// Generador de embeddings de texto (Fase 4: modelo propio).
pub trait Embedder {
    fn texto_a_embedding(&self, texto: &str) -> Option<Vec<f32>>;
}
