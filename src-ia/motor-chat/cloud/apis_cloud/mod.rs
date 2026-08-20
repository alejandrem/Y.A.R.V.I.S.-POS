//! apis_cloud — Respuestas por API de proveedores de IA (Gemini y OpenCode Zen).
//!
//! Port de `yarvis-IA/chatbot/motor_chat/modelos_API/apis_cloud.py`.
//!
//! Dividido por TAREA (archivos planos, espejo de `parseador_de_tickets/cerebro`):
//!   errores.rs    → clasificación de errores (HTTP vs red) + esperas 429
//!   tipos.rs      → eventos del stream, uso de tokens y modelos disponibles
//!   helpers.rs    → nombre amigable, modelos free, cola de relevo 429, normalización
//!   sse.rs        → lectura del cuerpo SSE (líneas `data: ...`)
//!   proveedores.rs→ streams específicos por proveedor (OpenAI-compatible + Google)
//!   generacion.rs → API pública: `generar_stream` / `generar_completo` (con relevo)
//!   catalogo.rs   → listado de modelos con caché TTL
//!
//! No toca hardware ni base de datos: recibe los mensajes ya construidos.
//! (A diferencia de Python, aquí se omiten las tools/function calling: el modelo
//! cloud ya no llama search_inventory.)

mod catalogo;
mod errores;
mod generacion;
mod helpers;
mod proveedores;
mod sse;
mod tipos;

pub use catalogo::listar_modelos;
pub use generacion::{generar_completo, generar_stream};
pub use helpers::nombre_proveedor;
pub use tipos::{Evento, ModeloDisponible, Usage};
