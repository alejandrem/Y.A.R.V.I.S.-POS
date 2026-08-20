// ============================================================
// src-ia — Núcleo de IA en Rust
// Migración progresiva desde yarvis-IA (Python → Rust).
// Estrategia Strangler Fig: cada módulo se porta con tests
// de equivalencia contra el comportamiento de Python.
// ============================================================

pub mod cerebro;
pub mod formatos;
pub mod rutas;

// Motor de chat (vive en `src-ia/motor-chat/`, hermano de `parseador_de_tickets/`).
#[path = "../motor-chat/mod.rs"]
pub mod motor_chat;

// Matemáticas pesadas de predicción (viven en `src-ia/predicciones/`).
#[path = "../predicciones/mod.rs"]
pub mod predicciones;
