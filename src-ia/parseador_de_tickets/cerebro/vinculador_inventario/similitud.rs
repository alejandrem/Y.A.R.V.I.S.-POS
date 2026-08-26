// Shim de compatibilidad: el modulo canónico ahora vive en `src-ia/embeddings/mod.rs`.
// Se mantiene este archivo para no romper imports viejos ni cachés de binarios que
// aún resuelvan `crate::cerebro::vinculador_inventario::similitud`.
// Todos los símbolos se re-exportan desde `crate::embeddings`.

pub use crate::embeddings::{blob_a_embedding, cosine_similarity, normalizar, Embedder};
