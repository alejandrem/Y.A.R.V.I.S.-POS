// ============================================================
// analizador_ticket — Análisis de un ticket con el Qwen 3 1.7B.
// Único modelo local del Y.A.R.V.I.S.: lo comparte el parseo y la
// conversación. 1 solo GGUF en RAM, sin escalada.
// Porción de analizador_llm.rs (espejo de `analizar_ticket`).
// ============================================================

#[cfg(feature = "llm-local")]
use super::analizador_inferencia::ejecutar_analisis;
#[cfg(feature = "llm-local")]
use super::analizador_json::con_status_ok;
#[cfg(feature = "llm-local")]
use super::analizador_modelos::cargar_modelo;

/// Analiza un ticket con el modelo 1.7B. Si no produce un `mapeo` válido,
/// se reporta error (sin reintento con otro modelo).
#[cfg(feature = "llm-local")]
pub fn analizar_ticket(texto_ticket: &str) -> serde_json::Value {
    if texto_ticket.trim().is_empty() {
        return serde_json::json!({ "status": "error", "error": "El texto del ticket está vacío" });
    }

    let modelo = match cargar_modelo("1.7B") {
        Ok(m) => m,
        Err(e) => {
            return serde_json::json!({ "status": "error", "error": format!("Error al analizar ticket: {e}") });
        }
    };

    match ejecutar_analisis(&modelo, texto_ticket) {
        Some(mut resultado) if resultado.get("mapeo").is_some() => {
            let confianza = resultado
                .get("confianza")
                .and_then(|c| c.as_f64())
                .unwrap_or(0.0);
            resultado["confianza"] = serde_json::json!(confianza);
            con_status_ok(resultado)
        }
        _ => {
            serde_json::json!({ "status": "error", "error": "El modelo 1.7B no devolvió un mapeo válido" })
        }
    }
}

#[cfg(not(feature = "llm-local"))]
pub fn analizar_ticket(_texto_ticket: &str) -> serde_json::Value {
    serde_json::json!({
        "status": "error",
        "error": "El feature 'llm-local' de src-ia no está habilitado (sin soporte llama.cpp)."
    })
}
