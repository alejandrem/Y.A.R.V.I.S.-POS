//! variables.rs — Constantes y parámetros del motor de chat EN LA NUBE (APIs).
//!
//! Solo datos, SIN imports del proyecto (evita imports circulares).
//! Única fuente de verdad para proveedores/timeouts; apis_cloud.rs importa desde aquí.
//! Espejo de `yarvis-IA/chatbot/motor_chat/modelos_API/variables.py`.

/// Proveedor de nube soportado: URL base + modelo por defecto.
#[derive(Debug, Clone, Copy)]
pub struct Provider {
    pub key: &'static str,
    pub name: &'static str,
    pub base_url: &'static str,
    pub default_model: &'static str,
}

/// Proveedores de nube soportados (espejo de PROVIDERS de Python).
/// Gemini + NVIDIA (OpenAI-compatible, sin candado de cliente: la key
/// `nvapi-` jala donde sea). OpenCode Zen se retiró 2026-09-19 (su free
/// tier bloquea a terceros y no hay facturación; ver bitácora).
pub const PROVIDERS: &[Provider] = &[
    Provider {
        key: "google",
        name: "Gemini",
        base_url: "https://generativelanguage.googleapis.com/v1beta",
        // gemini-2.x fue RETIRADO para keys nuevas ("no longer available to new
        // users", error 404). Verificado en vivo 2026-09-19 con key AIza real:
        // gemini-3.5-flash-lite responde 200 + stream SSE correcto; es además
        // el más ligero/barato (ideal para caja). 3.6-flash también responde.
        default_model: "gemini-3.5-flash-lite",
    },
    Provider {
        key: "nvidia",
        name: "NVIDIA",
        base_url: "https://integrate.api.nvidia.com/v1",
        // NIM OpenAI-compatible (Bearer nvapi-…): nano 8b, ligero y rápido
        // para caja. La lista completa sale de GET /models en la app.
        default_model: "nvidia/llama-3.1-nemotron-nano-8b-v1",
    },
];

/// Timeout de CONEXIÓN hacia los proveedores (segundos).
pub const TIMEOUT_CONNECT_SECS: u64 = 30;

/// Timeout de INACTIVIDAD (segundos): máximo silencio entre chunks del
/// stream SSE. NO es un timeout total — una generación larga pero viva
/// (llegan chunks) nunca se corta; un servidor colgado sí se detecta.
/// Antes había un timeout global de 120 s que mataba streams largos a
/// mitad de respuesta.
pub const TIMEOUT_IDLE_SECS: u64 = 90;

/// Límite de tokens de SALIDA para el transporte OpenAI-compatible
/// (`/chat/completions` usa `max_tokens`). Techo seguro 4096 para el día
/// que entre otro proveedor por esta vía (p. ej. NVIDIA).
pub const MAX_TOKENS: u32 = 4096;

/// Límite de tokens de SALIDA para Gemini (`generationConfig.maxOutputTokens`).
/// Los modelos flash tienen techos de salida bajos (8192 en gemini-2.0-flash):
/// enviarles un valor mayor responde 400 INVALID_ARGUMENT en CADA llamada.
/// Antes se enviaba MAX_TOKENS tal cual y Gemini estaba roto por esto.
pub const MAX_TOKENS_GOOGLE: u32 = 8192;

/// Segundos a esperar ante un 429 entre reintentos (rango corto para no
/// frenar el chat: si está saturado, mejor reintentar rápido o caer al local).
pub const ESPERA_429_MIN_SECS: u64 = 2;
pub const ESPERA_429_MAX_SECS: u64 = 4;

/// TTL (segundos) de la caché del listado de modelos de /cloud_models.
/// Evita golpear los endpoints /models de los proveedores en cada apertura.
pub const MODELOS_CACHE_TTL_SECS: f64 = 60.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proveedores_tienen_claves_unicas() {
        let mut claves: Vec<&str> = PROVIDERS.iter().map(|p| p.key).collect();
        claves.sort();
        claves.dedup();
        assert_eq!(claves.len(), PROVIDERS.len());
    }

    #[test]
    fn google_arranca_en_flash_lite_verificado() {
        let g = PROVIDERS.iter().find(|p| p.key == "google").unwrap();
        assert_eq!(g.default_model, "gemini-3.5-flash-lite");
    }

    #[test]
    fn nvidia_usa_transporte_openai_compatible() {
        let n = PROVIDERS.iter().find(|p| p.key == "nvidia").unwrap();
        assert_eq!(n.base_url, "https://integrate.api.nvidia.com/v1");
        assert!(n.default_model.starts_with("nvidia/"));
    }
}
