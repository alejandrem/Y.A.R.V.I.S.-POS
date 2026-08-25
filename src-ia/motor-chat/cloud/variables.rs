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
pub const PROVIDERS: &[Provider] = &[
    Provider {
        key: "google",
        name: "Gemini",
        base_url: "https://generativelanguage.googleapis.com/v1beta",
        default_model: "gemini-2.0-flash",
    },
    Provider {
        key: "opencode",
        name: "OpenCode",
        base_url: "https://opencode.ai/zen/v1",
        default_model: "mimo-v2.5-free",
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

/// Límite de tokens de SALIDA para OpenCode Zen (OpenAI-compatible usa
/// `max_tokens`). Los modelos free toleran valores altos.
pub const MAX_TOKENS: u32 = 39800;

/// Límite de tokens de SALIDA para Gemini (`generationConfig.maxOutputTokens`).
/// Los modelos flash tienen techos de salida bajos (8192 en gemini-2.0-flash):
/// enviarles un valor mayor responde 400 INVALID_ARGUMENT en CADA llamada.
/// Antes se enviaba MAX_TOKENS tal cual y Gemini estaba roto por esto.
pub const MAX_TOKENS_GOOGLE: u32 = 8192;

/// Modelos gratuitos de OpenCode que NO terminan en "-free" pero sí lo son.
pub const MODELOS_FREE_EXTRA: &[&str] = &["big-pickle"];

/// Orden de fallback cuando un modelo free de OpenCode satura (429): se cambia
/// automáticamente al siguiente de la lista hasta agotarlos.
pub const ORDEN_FALLBACK_FREE: &[&str] = &[
    "mimo-v2.5-free",
    "nemotron-3-ultra-free",
    "nemotron-3.5-lightning-free",
    "hy3-free",
    "laguna-s-2.1-free",
    "deepseek-v4-flash-free",
    "big-pickle",
];

/// Máximo de modelos a probar en un solo mensaje (incluye el pedido). Si todos
/// fallan, se cae al modelo local. Evita que el relevo tarde una eternidad.
pub const MAX_MODELOS_A_PROBAR: usize = 3;

/// Segundos a esperar ante un 429 entre modelo y modelo (rango corto para no
/// frenar el chat: si está saturado, mejor pasar al siguiente rápido o caer al local).
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
    fn opencode_arranca_en_mimo_v2_5_free() {
        let oc = PROVIDERS.iter().find(|p| p.key == "opencode").unwrap();
        assert_eq!(oc.default_model, "mimo-v2.5-free");
    }

    #[test]
    fn modelos_free_extra_no_estan_en_orden_fallback() {
        for extra in MODELOS_FREE_EXTRA {
            assert!(
                ORDEN_FALLBACK_FREE.contains(extra),
                "{extra} debe estar en el relevo"
            );
        }
    }
}
