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

/// Estimación conservadora: 1 token ≈ 4 caracteres en español.
const CHARS_POR_TOKEN: usize = 4;
/// Espacio reservado a la SALIDA del modelo dentro del contexto.
const RESERVA_SALIDA_TOKENS: usize = 1024;
/// Margen para el overhead del chat template (roles, marcadores).
const MARGEN_TEMPLATE_TOKENS: usize = 128;

/// Recorta el historial del chat local para que quepa en `CONTEXTO_LOCAL`.
///
/// Antes un chat largo moría con "El prompt excede n_ctx" sin recuperación.
/// Estrategia: los mensajes SYSTEM jamás se recortan (identidad + tools);
/// del resto se conservan los MÁS RECIENTES que quepan en el presupuesto.
/// Si ni siquiera el último mensaje del usuario cabe, se conserva su cola
/// (la parte más cercana a la pregunta actual). Es una estimación por
/// caracteres, no tokenización exacta: deliberadamente conservadora.
pub fn recortar_historial(messages: &[Mensaje]) -> Vec<Mensaje> {
    let presupuesto_chars = ((CONTEXTO_LOCAL as usize)
        .saturating_sub(RESERVA_SALIDA_TOKENS)
        .saturating_sub(MARGEN_TEMPLATE_TOKENS))
        .saturating_mul(CHARS_POR_TOKEN);

    let sistema: Vec<Mensaje> =
        messages.iter().filter(|m| m.role == "system").cloned().collect();
    let conversacion: Vec<Mensaje> =
        messages.iter().filter(|m| m.role != "system").cloned().collect();

    let chars_sistema: usize = sistema.iter().map(|m| m.content.chars().count()).sum();
    let mut disponibles = presupuesto_chars.saturating_sub(chars_sistema);

    let mut elegidos: Vec<Mensaje> = Vec::new();
    for m in conversacion.iter().rev() {
        let costo = m.content.chars().count() + 8; // + overhead de rol/template
        if costo > disponibles {
            if elegidos.is_empty() && m.role == "user" {
                // El último mensaje del usuario es innegociable: se conserva
                // su cola completa en caracteres (sin partir multibyte).
                let tomar = disponibles.saturating_sub(16);
                let cola: String = m
                    .content
                    .chars()
                    .rev()
                    .take(tomar)
                    .collect::<Vec<char>>()
                    .into_iter()
                    .rev()
                    .collect();
                elegidos.push(Mensaje::new("user", cola));
            }
            break; // todo lo más viejo ya no cabe
        }
        disponibles -= costo;
        elegidos.push(m.clone());
    }
    elegidos.reverse();

    if sistema.len() + elegidos.len() == messages.len() {
        return messages.to_vec(); // cupo completo: sin cambios
    }

    let mut out = sistema;
    out.extend(elegidos);
    out
}

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
    // El historial se ajusta al contexto ANTES de tokenizar: chats largos
    // se truncán por el final (se pierde lo viejo, nunca la pregunta actual)
    // en lugar de fallar con "El prompt excede n_ctx".
    let chat = recortar_historial(&chat);
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

    #[test]
    fn historial_corto_no_se_toca() {
        let msgs = vec![
            Mensaje::new("system", "eres yarvis"),
            Mensaje::new("user", "hola"),
            Mensaje::new("assistant", "qué onda"),
            Mensaje::new("user", "cuánto vendí hoy"),
        ];
        assert_eq!(recortar_historial(&msgs), msgs);
    }

    #[test]
    fn historial_largo_conserva_recientes_y_sistema() {
        let mut msgs = vec![Mensaje::new("system", "S")]; // system corto
        // 40 mensajes viejos de ~1000 caracteres cada uno ≈ 40k chars >> 11.7k
        for i in 0..40 {
            msgs.push(Mensaje::new("user", format!("viejo {i}: {}", "x".repeat(1000))));
            msgs.push(Mensaje::new("assistant", "ok"));
        }
        let ultima = "¿cuánto vendí HOY?";
        msgs.push(Mensaje::new("user", ultima));

        let recortado = recortar_historial(&msgs);
        assert!(recortado.len() < msgs.len(), "debió recortar");
        assert_eq!(recortado[0].role, "system", "el system jamás se recorta");
        let ultima_msg = recortado.last().unwrap();
        assert_eq!(ultima_msg.role, "user");
        assert!(ultima_msg.content.ends_with(ultima), "la pregunta actual va completa");
    }

    #[test]
    fn usuario_gigante_se_trunca_por_el_final_sin_partir_multibyte() {
        let msgs = vec![
            Mensaje::new("system", "s"),
            Mensaje::new("user", "ñ".repeat(50_000) + "PREGUNTA FINAL ñ"),
        ];
        let r = recortar_historial(&msgs);
        let ultima = r.last().unwrap();
        // 'ñ' es multibyte: si el corte partiera un carácter, ends_with fallaría.
        assert!(ultima.content.ends_with("PREGUNTA FINAL ñ"));
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
