// Shim de compatibilidad: el modulo canónico vive en `src-ia/embeddings/mod.rs`.
// NO agregues lógica aquí. Todo lo de embeddings va en `src-ia/embeddings/`.
pub use crate::embeddings::{
    blob_a_embedding, cosine_similarity, embedding_a_blob, normalizar, Embedder, HashEmbedder, DIM,
};
