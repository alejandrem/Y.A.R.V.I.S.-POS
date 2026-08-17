// ============================================================
// src-ia — Núcleo de IA en Rust
// Migración progresiva desde yarvis-IA (Python → Rust).
// Estrategia Strangler Fig: cada módulo se porta con tests
// de equivalencia contra el comportamiento de Python.
// ============================================================

pub mod cerebro;
pub mod formatos;
pub mod rutas;