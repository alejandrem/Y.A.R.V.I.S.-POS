//! llm — Chat LOCAL en Rust con el modelo Qwen 1.7B (fine-tune en curso).
//!
//! Reemplaza a `yarvis-IA/chatbot/motor_chat/modelos_local/` en su función de
//! conversación: se conecta directo al 1.7B con llama.cpp. SIN RAG, SIN
//! consultas a la BD y SIN gestión de hardware: ahora solo se cargan
//! `0.5B` (parseo de tickets) y `1.7B` (conversación/consultas).
//!
//! El system prompt marca que Y.A.R.V.I.S. está en fase de TESTING: es el
//! prompt que verá el 1.7B durante su fine-tuning.
//!
//! Espejo ligero de `modelos_local/prompts.py` (construir_mensajes) y de
//! `gestion_hardware.py` (ejecutar_chat), SIN toda la capa de RAG/caché.
//!
//! La inferencia vive detrás del feature `llm-local` (igual que el parseo de
//! tickets); el backend Tauri la activa con `features = ["llm-local"]`.

use super::cloud::prompts::Mensaje;

/// Clave del modelo de conversación (único modelo local para el chat).
pub const MODELO_CHAT: &str = "1.7B";

/// System prompt del 1.7B local: marca que está siendo TESTEADO (fase previa
/// al fine-tuning). Sin contexto de BD: solo identidad + reglas de prueba.
pub const SYSTEM_PROMPT_TEST: &str = r#"Eres Y.A.R.V.I.S., el asistente inteligente de negocios.

ACTUALMENTE ESTÁS EN FASE DE TESTING: estás siendo probado y evaluado antes de pasar a producción (estás preparado para fine-tuning sobre el modelo Qwen 1.7B).

REGLAS:
1. Responde en el idioma que te hablen (español por defecto).
2. Sé claro, directo y conciso. Usa markdown ligero (listas, negritas) cuando ayude.
3. Si no sabes algo o no tienes la información, dilo con honestidad en lugar de inventar.
4. Estás siendo probado: si algo falla o no puedes hacerlo, explícalo. Es parte del test.
5. No inventes datos de la tienda (precios, stocks, ventas) si no están en la conversación."#;

/// Arma los mensajes [system (test) + historial] para el modelo local 1.7B.
///
/// Espejo de `construir_mensajes` de `prompts.py` pero SIN contexto de RAG/BD.
pub fn construir_mensajes_locales(messages: &[Mensaje]) -> Vec<Mensaje> {
    let mut chat = vec![Mensaje::new("system", SYSTEM_PROMPT_TEST)];
    chat.extend_from_slice(messages);
    chat
}

// ---------------------------------------------------------------------------
// Inferencia llama.cpp (feature `llm-local`)
// ---------------------------------------------------------------------------

#[cfg(feature = "llm-local")]
use crate::rutas::analizador_llm::{cargar_modelo, generar_bajo_lock};
#[cfg(feature = "llm-local")]
use regex::Regex;
#[cfg(feature = "llm-local")]
use std::sync::OnceLock;

/// Limpiador de los bloques de razonamiento del Qwen3 local.
///
/// El GGUF de Qwen3 1.7B emite sus marcadores en VARIAS variantes según el
/// template/tokenizador (se han observado las tres):
///   - `<think>... </think>`        (etiquetas HTML con cierre /think)
///   - `<think>... <response>`      (etiquetas HTML con cierre /response)
///   - `" think" ... " response"`    (con espacios y razonamiento a inicio de línea)
/// Las palabras inglesas tipo "the response should..." NO deben cerrar el
/// bloque, o la respuesta final se mutila. Por eso:
///   - las variantes HTML exigen el `>` del cierre (la prosa jamás lo tiene);
///   - la variante con espacio exige que ` response` esté a inicio de línea.
#[cfg(feature = "llm-local")]
fn limpiar_think_local(texto: &str) -> String {
    static RE_HTML: OnceLock<Regex> = OnceLock::new();
    static RE_HTML_RESP: OnceLock<Regex> = OnceLock::new();
    static RE_SPACE: OnceLock<Regex> = OnceLock::new();

    let re_html = RE_HTML.get_or_init(|| {
        Regex::new(r"(?s)<think(?:ing)?>.*?</think(?:ing)?>").expect("regex de think HTML válida")
    });
    let re_html_resp = RE_HTML_RESP.get_or_init(|| {
        Regex::new(r"(?s)<think(?:ing)?>.*?<(?:/\s*)?response>").expect("regex de think HTML válida")
    });
    let re_space = RE_SPACE.get_or_init(|| {
        Regex::new(r#"(?s)(?:^|\n)\s*think(?:ing)?\b.*?(?:\n\s*)response(?:\n|$)"#)
            .expect("regex de think con espacio válida")
    });

    let s = re_html.replace_all(texto, "");
    let s = re_html_resp.replace_all(&s, "");
    let s = re_space.replace_all(&s, "");
    s.trim().to_string()
}

/// Genera la respuesta CRUDA (sin limpiar) del 1.7B local.
#[cfg(feature = "llm-local")]
fn generar_1_7(messages: &[Mensaje]) -> Result<String, String> {
    use llama_cpp_4::prelude::LlamaChatMessage;

    let chat = construir_mensajes_locales(messages);
    let modelo = cargar_modelo(MODELO_CHAT)?;

    let mut llm_messages = Vec::with_capacity(chat.len());
    for m in &chat {
        let msg = LlamaChatMessage::new(m.role.clone(), m.content.clone())
            .map_err(|e| format!("Error armando mensaje para el modelo: {e}"))?;
        llm_messages.push(msg);
    }

    generar_bajo_lock(&modelo, &llm_messages)
}

/// Respuesta completa del 1.7B local para el chat (sin bloques  thinking ).
#[cfg(feature = "llm-local")]
pub fn chat_1_7(messages: &[Mensaje]) -> Result<String, String> {
    generar_1_7(messages).map(|raw| limpiar_think_local(&raw))
}

/// Respuesta CRUDA del 1.7B (conservando bloques  think... response ) para
/// que el frontend pueda mostrar el razonamiento por separado (streaming).
#[cfg(feature = "llm-local")]
pub fn chat_1_7_raw(messages: &[Mensaje]) -> Result<String, String> {
    generar_1_7(messages)
}

// ---------------------------------------------------------------------------
// Sin feature `llm-local`: API presente pero reporta que no hay backend.
// ---------------------------------------------------------------------------

#[cfg(not(feature = "llm-local"))]
pub fn chat_1_7(_messages: &[Mensaje]) -> Result<String, String> {
    Err("El feature 'llm-local' de src-ia no está habilitado (sin soporte llama.cpp).".to_string())
}

#[cfg(not(feature = "llm-local"))]
pub fn chat_1_7_raw(_messages: &[Mensaje]) -> Result<String, String> {
    Err("El feature 'llm-local' de src-ia no está habilitado (sin soporte llama.cpp).".to_string())
}

// ---------------------------------------------------------------------------
// Tests (la lógica pura no depende de llama.cpp)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_prompt_marca_que_esta_siendo_testeado() {
        assert!(SYSTEM_PROMPT_TEST.contains("TESTING"));
        assert!(SYSTEM_PROMPT_TEST.contains("fine-tuning"));
        assert!(SYSTEM_PROMPT_TEST.contains("1.7B"));
    }

    #[test]
    fn construir_mensajes_locales_prepende_system_test() {
        let historial = vec![
            Mensaje::new("user", "hola"),
            Mensaje::new("assistant", "hola!"),
        ];
        let chat = construir_mensajes_locales(&historial);
        assert_eq!(chat.len(), 3);
        assert_eq!(chat[0].role, "system");
        assert!(chat[0].content.contains("TESTING"));
        assert_eq!(chat[1].content, "hola");
        assert_eq!(chat[2].content, "hola!");
    }

    #[test]
    fn sin_feature_devuelve_error_claro() {
        #[cfg(not(feature = "llm-local"))]
        {
            assert!(chat_1_7(&[]).is_err());
            assert!(chat_1_7_raw(&[]).is_err());
        }
    }

    #[cfg(feature = "llm-local")]
    #[test]
    fn temp_real_raw_se_limpia() {
        let raw = std::fs::read_to_string("/tmp/chat_raw3.txt").unwrap();
        let limpio = limpiar_think_local(&raw);
        println!("LIMPIO=[{limpio}]");
        assert!(!limpio.contains("thinking"), "sigue el marcador");
    }

    #[cfg(feature = "llm-local")]
    #[test]
    fn limpiar_think_local_no_mutila_la_respuesta() {
        // Variante con espacio: el cierre " response" va SOLO a inicio de línea
        // y la palabra inglesa "response" en medio del texto NO debe cerrar.
        let crudo = " think\nOkay, la respuesta should be natural.\n response\n\nEstoy en fase de testing y puedo ayudarte.";
        assert_eq!(limpiar_think_local(crudo), "Estoy en fase de testing y puedo ayudarte.");

        // Variante HTML (etiquetas con <>), como emitió el GGUF real.
        let crudo_html = " thinking\nOkay, la respuesta should be natural.\n response\n\nEstoy en fase de testing y puedo ayudarte.";
        assert_eq!(limpiar_think_local(crudo_html), "Estoy en fase de testing y puedo ayudarte.");

        // Respuesta sin razonamiento no se toca.
        assert_eq!(limpiar_think_local("Hola, ¿en qué te ayudo?"), "Hola, ¿en qué te ayudo?");
    }
}