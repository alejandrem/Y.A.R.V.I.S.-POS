// ============================================================
// embeddings — Módulo canónico de embeddings de Y.A.R.V.I.S.
//
// VIVE TODO AQUÍ. No busques en vinculador_inventario/similitud.rs:
// ese archivo es solo un shim que re-exporta desde aquí.
//
// Estado actual: implementación propia sin modelo ML externo.
// Usa hashing trick (384 dims) + normalización + n-gramas para
// dar búsqueda semántica real sin descargar ningún GGUF/ONNX.
// Cuando exista un modelo propio (Fase 4), se inyectará via
// trait Embedder y reemplazará a HashEmbedder sin tocar el
// resto del código.
//
// Flujo:
//   texto --normalizar--> tokens --hash--> vec[384] --normalize--> embedding
//   embedding --embedding_a_blob--> BLOB LE --blob_a_embedding--> vec
//   vec --cosine_similarity--> score
// ============================================================

use regex::Regex;
use std::sync::LazyLock;

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/// Dimensión del embedding. 384 = estándar MiniLM size, compatible
/// con knowledge_base.embedding (384 * 4 bytes = 1536 bytes blob).
pub const DIM: usize = 384;

static RE_NO_WORD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[^\w\s]").expect("regex no-word"));
static RE_ESPACIOS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s+").expect("regex espacios"));

// ---------------------------------------------------------------------------
// Normalización
// ---------------------------------------------------------------------------

/// Normaliza un nombre para comparación: minúsculas, sin especiales,
/// sin espacios extra. Usado tanto para match exacto como para
/// generación de embeddings.
pub fn normalizar(nombre: &str) -> String {
    let limpio = nombre.trim().to_lowercase();
    let limpio = RE_NO_WORD.replace_all(&limpio, "").into_owned();
    RE_ESPACIOS.replace_all(&limpio, " ").into_owned()
}

// ---------------------------------------------------------------------------
// Blob helpers
// ---------------------------------------------------------------------------

/// Deserializa un BLOB de SQLite a vector f32 (little-endian).
/// Cada fila de `knowledge_base.embedding` son 384 floats.
pub fn blob_a_embedding(blob: &[u8]) -> Vec<f32> {
    blob.chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect()
}

/// Serializa un embedding a BLOB LE para SQLite.
pub fn embedding_a_blob(emb: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(emb.len() * 4);
    for v in emb {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

// ---------------------------------------------------------------------------
// Coseno
// ---------------------------------------------------------------------------

/// Similitud coseno entre dos vectores.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| (*x as f64) * (*y as f64)).sum();
    let norm_a: f64 = a.iter().map(|x| (*x as f64) * (*x as f64)).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| (*x as f64) * (*x as f64)).sum::<f64>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// Generador de embeddings de texto.
pub trait Embedder: Send + Sync {
    fn texto_a_embedding(&self, texto: &str) -> Option<Vec<f32>>;
}

// ---------------------------------------------------------------------------
// Hash determinístico (FNV-1a 64) — estable entre runs, no como DefaultHasher
// ---------------------------------------------------------------------------

fn fnv1a64(s: &str) -> u64 {
    const FNV_OFFSET: u64 = 14695981039346656037;
    const FNV_PRIME: u64 = 1099511628211;
    let mut hash = FNV_OFFSET;
    for b in s.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

// ---------------------------------------------------------------------------
// Implementación propia: HashEmbedder
// ---------------------------------------------------------------------------

/// Embedder propio sin ML: hashing trick + n-gramas.
///
/// - Tokeniza por palabras del texto normalizado
/// - Cada palabra -> hash % DIM += 1.0
/// - Cada trigram de cada palabra -> hash % DIM += 0.5 (tolera typos)
/// - L2-normaliza al final (coseno = dot directo)
///
/// No necesita modelo descargado, es instantáneo y determinístico.
/// Supera a `LIKE '%q%'` en casos como "coca cl light" -> "Coca-Cola Light 600ml".
#[derive(Default, Clone, Copy)]
pub struct HashEmbedder;

impl Embedder for HashEmbedder {
    fn texto_a_embedding(&self, texto: &str) -> Option<Vec<f32>> {
        let norm = normalizar(texto);
        if norm.is_empty() {
            return None;
        }
        let mut vec = vec![0.0f32; DIM];
        for word in norm.split_whitespace() {
            // palabra completa
            let h = fnv1a64(word);
            let idx = (h % DIM as u64) as usize;
            vec[idx] += 1.0;

            // trigramas para fuzzy (coca -> coc, oca, etc. tolera "coca" vs "cocacola")
            if word.len() >= 3 {
                let chars: Vec<char> = word.chars().collect();
                for i in 0..=chars.len() - 3 {
                    let tri: String = chars[i..i + 3].iter().collect();
                    let th = fnv1a64(&tri);
                    let tidx = (th % DIM as u64) as usize;
                    vec[tidx] += 0.5;
                }
            }
        }
        // L2 normalize
        let norm_val: f64 = vec.iter().map(|x| (*x as f64) * (*x as f64)).sum::<f64>().sqrt();
        if norm_val > 0.0 {
            for v in &mut vec {
                *v = (*v as f64 / norm_val) as f32;
            }
        }
        Some(vec)
    }
}

/// Atajo global: genera embedding con el Embedder propio sin instanciar.
pub fn embed_text(texto: &str) -> Option<Vec<f32>> {
    HashEmbedder.texto_a_embedding(texto)
}
