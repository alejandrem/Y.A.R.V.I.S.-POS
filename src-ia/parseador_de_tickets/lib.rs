// ============================================================
// src-ia — Núcleo de IA en Rust
// Migración progresiva desde yarvis-IA (Python → Rust).
// Estrategia Strangler Fig: cada módulo se porta con tests
// de equivalencia contra el comportamiento de Python.
// ============================================================

pub mod cerebro;
pub mod formatos;
pub mod rutas;

// Embeddings interino (TF-IDF + fuzzy / trait Embedder) — vive en `src-ia/embeddings/`.
// Se expone desde la raiz para que `src_ia::embeddings::*` sea la ruta canonica;
// el modulo viejo en `cerebro/vinculador_inventario/similitud.rs` re-exporta desde aqui.
#[path = "../embeddings/mod.rs"]
pub mod embeddings;

// Motor de chat (vive en `src-ia/motor-chat/`, hermano de `parseador_de_tickets/`).
#[path = "../motor-chat/mod.rs"]
pub mod motor_chat;

// Matemáticas pesadas de predicción (viven en `src-ia/predicciones/`).
#[path = "../predicciones/mod.rs"]
pub mod predicciones;
