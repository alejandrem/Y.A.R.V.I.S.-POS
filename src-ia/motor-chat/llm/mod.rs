//! llm — Chat LOCAL en Rust con el modelo Qwen 1.7B (fine-tune en curso).
//!
//! Reemplaza a `yarvis-IA/chatbot/motor_chat/modelos_local/` en su función de
//! conversación: se conecta directo al 1.7B con llama.cpp. SIN RAG, SIN
//! consultas a la BD y SIN gestión de hardware: hay UN SOLO modelo local, el
//! `1.7B`, compartido entre el parseo de tickets y la conversación.
//!
//! El system prompt marca que Y.A.R.V.I.S. está en fase de TESTING: es el
//! prompt que verá el 1.7B durante su fine-tuning.
//!
//! Espejo ligero de `modelos_local/prompts.py` (construir_mensajes) y de
//! `gestion_hardware.py` (ejecutar_chat), SIN toda la capa de RAG/caché.
//!
//! La inferencia vive detrás del feature `llm-local` (igual que el parseo de
//! tickets); el backend Tauri la activa con `features = ["llm-local"]`.

pub mod tools;

use super::cloud::prompts::Mensaje;
use std::path::PathBuf;

/// Clave del modelo de conversación (único modelo local para el chat).
pub const MODELO_CHAT: &str = "1.7B";
/// Contexto seguro por defecto para modelos locales cargados por llama.cpp.
pub const CONTEXTO_LOCAL: u64 = 4096;

/// Ruta efectiva del GGUF usado por el parser y el chat local.
pub fn ruta_modelo_local() -> PathBuf {
    crate::rutas::ruta_modelo(MODELO_CHAT)
}

/// Nombre legible del modelo local configurado por el usuario.
pub fn nombre_modelo_local() -> String {
    ruta_modelo_local()
        .file_name()
        .and_then(|nombre| nombre.to_str())
        .unwrap_or(MODELO_CHAT)
        .to_string()
}

/// System prompt del 1.7B local: marca que está en fase de TESTING (previa al
/// fine-tuning). Sin contexto de BD: solo identidad + reglas de prueba.
pub const SYSTEM_PROMPT_TEST: &str = r#"Eres un asistente de tienda con acceso a herramientas: [query_sales, compare_periods, get_top_products, query_inventory, forecast_sales, get_product_info, get_restock_analysis]
Eres Y.A.R.V.I.S., el asistente inteligente de negocios. Respuesta con sinceridad: si no tienes informacion o no sabes hacer algo, dilo claro y explica por que.
Cuando la pregunta pueda responderse con una herramienta (ventas, inventario, productos, pronosticos o resurtido), responde UNICAMENTE con:
<tool_call>
{"name": "nombre_de_tool", "arguments": { ... }}
</tool_call>
Si ninguna herramienta aplica, responde directo sin tool_call. No inventes datos.
Se directo y util."#;

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
use crate::rutas::{cargar_modelo, descargar_modelo, generar_bajo_lock, modelo_cargado};
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
        Regex::new(r"(?s)<think(?:ing)?>.*?<(?:/\s*)?response>")
            .expect("regex de think HTML válida")
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
// Gestión del modelo de conversación + RAM disponible (para `get_model_status`
// y `load_chat_model` del backend Tauri: carga local SIN sidecar Python).
// ---------------------------------------------------------------------------

/// RAM mínima (GB *disponibles*) para poder cargar el 1.7B de conversación.
pub const RAM_GB_MINIMA_1_7: f64 = 1.0;

/// RAM disponible del sistema en GB (`MemAvailable` de `/proc/meminfo`).
pub fn ram_libre_gb() -> Result<f64, String> {
    let meminfo = std::fs::read_to_string("/proc/meminfo")
        .map_err(|e| format!("No se pudo leer /proc/meminfo: {e}"))?;
    for linea in meminfo.lines() {
        if let Some(resto) = linea.strip_prefix("MemAvailable:") {
            let kb: f64 = resto
                .split_whitespace()
                .next()
                .and_then(|v| v.parse().ok())
                .ok_or_else(|| "MemAvailable sin valor en /proc/meminfo".to_string())?;
            return Ok(kb / (1024.0 * 1024.0));
        }
    }
    Err("No se encontró MemAvailable en /proc/meminfo".to_string())
}

/// RAM total del sistema en GB (`MemTotal` de `/proc/meminfo`).
pub fn ram_total_gb() -> Result<f64, String> {
    let meminfo = std::fs::read_to_string("/proc/meminfo")
        .map_err(|e| format!("No se pudo leer /proc/meminfo: {e}"))?;
    for linea in meminfo.lines() {
        if let Some(resto) = linea.strip_prefix("MemTotal:") {
            let kb: f64 = resto
                .split_whitespace()
                .next()
                .and_then(|v| v.parse().ok())
                .ok_or_else(|| "MemTotal sin valor en /proc/meminfo".to_string())?;
            return Ok(kb / (1024.0 * 1024.0));
        }
    }
    Err("No se encontró MemTotal en /proc/meminfo".to_string())
}

/// Carga el modelo de conversación 1.7B (mismo caché que el parseo de
/// tickets) SOLO si hay RAM disponible suficiente. Real-RAM de "fin de plazo"
/// usado por `load_chat_model` del Tauri.
#[cfg(feature = "llm-local")]
pub fn cargar_modelo_1_7() -> Result<String, String> {
    let libre = ram_libre_gb().unwrap_or(0.0);
    if libre < RAM_GB_MINIMA_1_7 {
        return Err(format!(
            "RAM insuficiente para {MODELO_CHAT}: hay {libre:.2}GB libres, se necesitan ≥{RAM_GB_MINIMA_1_7}GB."
        ));
    }
    cargar_modelo(MODELO_CHAT)?;
    Ok(MODELO_CHAT.to_string())
}

/// ¿Está cargado el modelo de conversación 1.7B?
#[cfg(feature = "llm-local")]
pub fn modelo_1_7_cargado() -> bool {
    modelo_cargado(MODELO_CHAT)
}

/// Descarga el 1.7B de conversación para liberar RAM (devuelve si estaba cargado).
#[cfg(feature = "llm-local")]
pub fn descargar_modelo_1_7() -> bool {
    descargar_modelo(MODELO_CHAT)
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

#[cfg(not(feature = "llm-local"))]
pub fn cargar_modelo_1_7() -> Result<String, String> {
    Err("El feature 'llm-local' de src-ia no está habilitado (sin soporte llama.cpp).".to_string())
}

#[cfg(not(feature = "llm-local"))]
pub fn modelo_1_7_cargado() -> bool {
    false
}

#[cfg(not(feature = "llm-local"))]
pub fn descargar_modelo_1_7() -> bool {
    false
}

// ---------------------------------------------------------------------------
// Tests (la lógica pura no depende de llama.cpp)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

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
    fn limpiar_think_local_no_mutila_la_respuesta() {
        // Variante con espacio: el cierre " response" va SOLO a inicio de línea
        // y la palabra inglesa "response" en medio del texto NO debe cerrar.
        let crudo = " think\nOkay, la respuesta should be natural.\n response\n\nEstoy en fase de testing y puedo ayudarte.";
        assert_eq!(
            limpiar_think_local(crudo),
            "Estoy en fase de testing y puedo ayudarte."
        );

        // Variante HTML (etiquetas con <>), como emitió el GGUF real.
        let crudo_html = " thinking\nOkay, la respuesta should be natural.\n response\n\nEstoy en fase de testing y puedo ayudarte.";
        assert_eq!(
            limpiar_think_local(crudo_html),
            "Estoy en fase de testing y puedo ayudarte."
        );

        // Respuesta sin razonamiento no se toca.
        assert_eq!(
            limpiar_think_local("Hola, ¿en qué te ayudo?"),
            "Hola, ¿en qué te ayudo?"
        );
    }
}
