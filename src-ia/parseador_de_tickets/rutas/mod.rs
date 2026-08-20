// ============================================================
// rutas — Resolución de rutas de modelos y análisis de tickets
// con LLM local. Dividido por TAREA (archivos planos):
//   analizador_*    → port de analizador_llm.py
//   rutas_modelos_* → port de rutas_modelos.py
// Espejo de yarvis-IA/parseador_de_tickets/llm/
// ============================================================

mod analizador_inferencia;
mod analizador_json;
mod analizador_modelos;
mod analizador_prompt;
mod analizador_ticket;
mod rutas_modelos_api;
mod rutas_modelos_config;
mod rutas_modelos_detect;

#[cfg(feature = "llm-local")]
pub use analizador_inferencia::generar_bajo_lock;
pub use analizador_json::extraer_json;
pub use analizador_modelos::descargar_modelos;
#[cfg(feature = "llm-local")]
pub use analizador_modelos::{cargar_modelo, descargar_modelo, modelo_cargado};
pub use analizador_prompt::SISTEMA_PROMPT;
pub use analizador_ticket::analizar_ticket;
pub use rutas_modelos_api::{
    configurar_ruta_modelo, qwen1_7, ruta_modelo, ruta_modelo_personalizada, verificar_modelos,
};
pub use rutas_modelos_config::InfoModelo;
