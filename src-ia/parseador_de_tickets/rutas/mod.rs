// ============================================================
// rutas — Resolución de rutas de modelos y generación local para
// el CHAT (motor-chat). El parseo de tickets ya NO usa LLM: el
// detector estadístico (`analizador_tickets::detector`) lo sustituyó.
//
//   analizador_inferencia → generación llama.cpp bajo lock global
//   analizador_modelos    → carga/caché/descarga del GGUF
//   rutas_modelos_*       → rutas y detección de archivos de modelo
// ============================================================

mod analizador_inferencia;
mod analizador_json;
mod analizador_modelos;
mod rutas_modelos_api;
mod rutas_modelos_config;
mod rutas_modelos_detect;

#[cfg(feature = "llm-local")]
pub use analizador_inferencia::generar_bajo_lock;
pub use analizador_json::extraer_json;
pub use analizador_modelos::descargar_modelos;
#[cfg(feature = "llm-local")]
pub use analizador_modelos::{cargar_modelo, descargar_modelo, modelo_cargado};
pub use rutas_modelos_api::{
    configurar_ruta_modelo, qwen1_7, ruta_modelo, ruta_modelo_personalizada, verificar_modelos,
};
pub use rutas_modelos_config::InfoModelo;
