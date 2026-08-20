// ============================================================
// analizador_inferencia — Generación llama.cpp bajo lock global
// (feature `llm-local`). Porción de analizador_llm.rs.
// ============================================================

#[cfg(feature = "llm-local")]
use std::num::NonZeroU32;
#[cfg(feature = "llm-local")]
use std::sync::Arc;

#[cfg(feature = "llm-local")]
use llama_cpp_4::prelude::*;

#[cfg(feature = "llm-local")]
use super::analizador_json::extraer_json;
#[cfg(feature = "llm-local")]
use super::analizador_modelos::{
    backend_global, n_threads_llm, ModeloChat, Resultado, INFERENCIA_LOCK, MAX_TOKENS,
    MAX_TOKENS_PARSEO, N_BATCH, N_CTX, TEMPERATURA, TOP_P,
};
#[cfg(feature = "llm-local")]
use super::analizador_prompt::SISTEMA_PROMPT;

/// Genera texto completo dado el prompt ya formateado con el chat template.
/// `max_tokens` es un TECHO de generación por línea de uso: el chat usa 2048 y
/// el parseo 1024 (su JSON cabe en <500 tokens), así un "divague" del modelo
/// no se paga dos veces.
#[cfg(feature = "llm-local")]
fn generar(modelo: &ModeloChat, prompt: &str, max_tokens: i32) -> Resultado<String> {
    let model = &modelo.model;
    let backend = backend_global()?;

    let hilos = n_threads_llm();
    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(NonZeroU32::new(N_CTX))
        .with_n_batch(N_BATCH as u32)
        .with_n_threads(hilos)
        .with_n_threads_batch(hilos);
    let mut ctx = model
        .new_context(backend, ctx_params)
        .map_err(|e| format!("No se pudo crear el contexto: {e}"))?;

    let tokens = model
        .str_to_token(prompt, AddBos::Always)
        .map_err(|e| format!("No se pudo tokenizar el prompt: {e}"))?;
    let n_prompt = tokens.len() as i32;
    if n_prompt >= N_CTX as i32 {
        return Err("El prompt excede n_ctx".to_string());
    }

    // Prefill por CHUNKS de N_BATCH: el prompt (SISTEMA_PROMPT + ticket)
    // suele superar las 512 posiciones y `add` falla con "Insufficient Space"
    // si no cabe en un solo batch (llama.cpp decodifica en lotes del tamaño
    // de `n_batch`).
    let mut batch = LlamaBatch::new(N_BATCH, 1);
    let ultimo_token = tokens.len() - 1;
    for (start, seg) in tokens.chunks(N_BATCH).enumerate() {
        batch.clear();
        for (idx, token) in seg.iter().enumerate() {
            let pos = start * N_BATCH + idx;
            let es_ultimo = pos == ultimo_token;
            batch
                .add(*token, pos as i32, &[0], es_ultimo)
                .map_err(|e| format!("Error llenando batch del prompt: {e}"))?;
        }
        ctx.decode(&mut batch)
            .map_err(|e| format!("decode(prompt) falló: {e}"))?;
    }

    let sampler = LlamaSampler::chain_simple([
        // Mismo conjunto de samplers por defecto que llama-cpp-python (el
        // resto de create_chat_completion quedó en defaults): top_k=40,
        // repeat_penalty=1.1, min_p=0.05 (además de temp/top_p del .py).
        // Orden = default de llama.cpp: penalties → top_k → top_p → min_p → temp.
        LlamaSampler::penalties_simple(64, 1.1),
        LlamaSampler::top_k(40),
        LlamaSampler::top_p(TOP_P, 1),
        LlamaSampler::min_p(0.05, 1),
        LlamaSampler::temp(TEMPERATURA),
        LlamaSampler::dist(0),
    ]);

    // Se acumulan los bytes de todos los tokens y se decodifica UTF-8 una sola
    // vez al final: llama.cpp puede partir un carácter multi-byte entre dos
    // tokens, y `String::from_utf8_lossy` aplicado al buffer completo lo
    // reconstruye igual que el `decode('utf-8')` de Python (sin dep externa).
    let mut salida = Vec::with_capacity(max_tokens as usize * 4);
    let llena = n_prompt + max_tokens;
    // Posición del siguiente token = longitud TOTAL del prompt (no el tamaño
    // del último chunk del prefill; si no, el token se solapa con el KV cache
    // y llama_decode devuelve -1).
    let mut n_cur = n_prompt;

    while n_cur < llena {
        let token = sampler.sample(&ctx, batch.n_tokens() - 1);
        if model.is_eog_token(token) {
            break;
        }

        let bytes = model
            .token_to_bytes(token, Special::Tokenize)
            .map_err(|e| format!("Error decodificando token: {e}"))?;
        salida.extend_from_slice(&bytes);

        batch.clear();
        batch
            .add(token, n_cur, &[0], true)
            .map_err(|e| format!("Error llenando batch de generación: {e}"))?;
        n_cur += 1;
        ctx.decode(&mut batch)
            .map_err(|e| format!("decode(generación) falló: {e}"))?;
    }

    Ok(String::from_utf8_lossy(&salida).into_owned())
}

/// Aplica el chat template a los mensajes y genera la respuesta bajo el lock
/// global de inferencia (compartido con el parseo de tickets: llama.cpp no
/// tolera dos generaciones a la vez sobre el mismo backend).
///
/// Lo consume el chat local (`motor-chat/llm`) para el modelo 1.7B de
/// conversación, reutilizando el MISMO caché de modelos del parseo.
#[cfg(feature = "llm-local")]
pub fn generar_bajo_lock(
    modelo: &Arc<ModeloChat>,
    messages: &[LlamaChatMessage],
) -> Resultado<String> {
    let prompt = modelo
        .model
        .apply_chat_template(None, messages, true)
        .map_err(|e| format!("No se pudo aplicar el chat template: {e}"))?;
    let _lock = INFERENCIA_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    generar(modelo, &prompt, MAX_TOKENS)
}

/// Ejecuta un análisis sobre el modelo indicado (espejo de `_ejecutar_analisis`).
#[cfg(feature = "llm-local")]
pub(crate) fn ejecutar_analisis(
    modelo: &Arc<ModeloChat>,
    texto: &str,
) -> Option<serde_json::Value> {
    let lineas: Vec<&str> = texto
        .trim()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .collect();
    let texto_analizar = lineas[..lineas.len().min(20)].join("\n");

    let user_prompt = format!(
        "TICKET A ANALIZAR:\n---\n{texto_analizar}\n---\n\nAnaliza este ticket y responde SOLAMENTE con el JSON válido."
    );

    let messages = vec![
        LlamaChatMessage::new("system".to_string(), SISTEMA_PROMPT.to_string()).ok()?,
        LlamaChatMessage::new("user".to_string(), user_prompt).ok()?,
    ];

    let prompt = modelo
        .model
        .apply_chat_template(None, &messages, true)
        .ok()?;

    // La inferencia está serializada por el lock global (llama-cpp-python igual).
    let _lock = INFERENCIA_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let contenido = generar(modelo, &prompt, MAX_TOKENS_PARSEO).ok()?;
    extraer_json(&contenido)
}
