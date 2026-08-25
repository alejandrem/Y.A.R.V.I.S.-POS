// ============================================================
// admintarvis/rutas.rs — Rutas internas del chat: stream cloud,
// stream local (Qwen 1.7B), chat a bloqueo y emisión troceada.
// ============================================================

use tauri::Emitter;

use futures_util::StreamExt;
use src_ia::motor_chat::cloud::apis_cloud::{
    generar_completo, generar_stream, Evento,
};
use src_ia::motor_chat::cloud::prompts::{construir_mensajes_api_rol, Mensaje};
use src_ia::motor_chat::cloud::think::{SeparadorThink, TipoFragmento};
use src_ia::motor_chat::llm::{chat_1_7, nombre_modelo_local, tools};

use super::cancelacion::stream_cancelado;
use super::ciclo_tools::{resolver_ciclo_tools, Generador, SupresorToolCall};
use super::herramientas_rol::ejecutar_tool_con_rol;

/// Máximo de palabras que se consideran razonamiento en el stream cloud
/// (espejo del `max_w` que usaba el motor original para `_separar_think`).
const CLOUD_MAX_W: usize = 1000;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ChatResponse {
    pub response: String,
    pub model_used: String,
}

/// Ruta del archivo SQLite desde el pool (para el ejecutor de tools).
pub(super) fn db_path_de(pool: &sqlx::SqlitePool) -> String {
    pool.connect_options().get_filename().to_string_lossy().to_string()
}

/// Convierte los mensajes `{role, content}` que manda el frontend a [`Mensaje`].
pub(super) fn mensajes_serde_a_rust(messages: &[serde_json::Value]) -> Vec<Mensaje> {
    messages
        .iter()
        .map(|m| Mensaje {
            role: m
                .get("role")
                .and_then(|r| r.as_str())
                .unwrap_or("user")
                .to_string(),
            content: m
                .get("content")
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .to_string(),
        })
        .collect()
}

/// Emite un stream cloud completo con Rust y devuelve la respuesta acumulada.
pub(super) async fn _stream_cloud(
    app: &tauri::AppHandle,
    provider: &str,
    api_key: &str,
    model: &str,
    chat: Vec<Mensaje>,
    db_path: &str,
    es_empleado: bool,
    ronda: usize,
) -> Result<String, String> {
    let stream = generar_stream(provider, api_key, model, chat.clone());
    futures_util::pin_mut!(stream);

    let mut sep = SeparadorThink::new(CLOUD_MAX_W);
    // Supresor: los bloques <tool_call> NUNCA llegan a la UI.
    let mut supresor = SupresorToolCall::new();
    let mut full_response = String::new();
    let mut model_used = String::from("unknown");

    while let Some(item) = stream.next().await {
        if stream_cancelado() {
            tracing::info!("[YARVIS-CHAT] stream cancelado por el usuario");
            return Ok(full_response); // nada más se emite tras el stop
        }
        match item {
            Ok(Evento::Texto { texto, modelo }) => {
                if model_used == "unknown" {
                    model_used = modelo.clone();
                }
                for (tipo, frag) in sep.procesar(&texto) {
                    if tipo == TipoFragmento::Token {
                        let visible = supresor.procesar(&frag);
                        full_response.push_str(&frag);
                        if !visible.is_empty() {
                            let _ = app.emit(
                                "chat-token",
                                serde_json::json!({
                                    "token": visible,
                                    "model": modelo,
                                }),
                            );
                        }
                    } else {
                        let _ = app.emit(
                            "chat-think",
                            serde_json::json!({
                                "token": frag,
                                "model": modelo,
                            }),
                        );
                    }
                }
            }
            Ok(Evento::Uso { usage, modelo }) => {
                if model_used == "unknown" {
                    model_used = modelo;
                }
                let _ = app.emit(
                    "chat-usage",
                    serde_json::json!({
                        "usage": serde_json::json!({
                            "prompt_tokens": usage.prompt_tokens,
                            "completion_tokens": usage.completion_tokens,
                            "total_tokens": usage.total_tokens,
                        }),
                    }),
                );
            }
            Err(e) => return Err(e),
        }
    }
    for (tipo, frag) in sep.finalizar() {
        if tipo == TipoFragmento::Token {
            let visible = supresor.procesar(&frag);
            full_response.push_str(&frag);
            if !visible.is_empty() {
                let _ = app.emit(
                    "chat-token",
                    serde_json::json!({
                        "token": visible,
                        "model": model_used,
                    }),
                );
            }
        }
    }
    let _ = supresor.finalizar();

    // ¿El stream pidió herramientas? Ejecutarlas y re-generar en silencio;
    // la respuesta final (sin tool_calls) se emite troceada al usuario.
    if !stream_cancelado()
        && tools::detectar_tool_call(&full_response).is_some()
        && ronda < tools::MAX_RONDAS_TOOLS
    {
        tracing::info!("[YARVIS-TOOLS] cloud pidió tool en ronda {ronda}");
        let mut generador: Generador = Box::new({
            let provider = provider.to_string();
            let api_key = api_key.to_string();
            let model = model.to_string();
            move |hist| {
                let p = provider.clone();
                let k = api_key.clone();
                let m = model.clone();
                Box::pin(async move { generar_completo(&p, &k, &m, hist).await.map(|(r, _)| r) })
            }
        });
        let final_resp =
            resolver_ciclo_tools(full_response, chat, db_path.to_string(), es_empleado, &mut generador).await?;
        return _emitir_como_stream(app, &final_resp, &model_used);
    }

    let _ = app.emit("chat-done", serde_json::json!({ "model": model_used }));
    let _ = app.emit(
        "chat-complete",
        serde_json::json!({
            "response": full_response,
            "model": model_used,
        }),
    );
    Ok(full_response)
}

/// Emite un texto final troceado (~40 chars) como si fuera streaming local.
fn _emitir_como_stream(
    app: &tauri::AppHandle,
    texto: &str,
    modelo: &str,
) -> Result<String, String> {
    let mut seg = String::new();
    for c in texto.chars() {
        if stream_cancelado() {
            tracing::info!("[YARVIS-CHAT] emisión cancelada por el usuario");
            return Ok(String::new());
        }
        seg.push(c);
        if seg.chars().count() >= 40 {
            let _ = app.emit("chat-token", serde_json::json!({ "token": seg, "model": modelo }));
            seg = String::new();
        }
    }
    if !seg.is_empty() {
        let _ = app.emit("chat-token", serde_json::json!({ "token": seg, "model": modelo }));
    }
    let _ = app.emit("chat-done", serde_json::json!({ "model": modelo }));
    let _ = app.emit(
        "chat-complete",
        serde_json::json!({ "response": texto, "model": modelo }),
    );
    Ok(texto.to_string())
}

/// Chat local (sin provider): responde el Qwen 1.7B nativo de Rust.
pub(super) async fn _chat_local(
    messages: Vec<serde_json::Value>,
    db_path: String,
    es_empleado: bool,
) -> Result<ChatResponse, String> {
    let mut historial = mensajes_serde_a_rust(&messages);
    let mut respuesta = tokio::task::spawn_blocking({
        let h = historial.clone();
        move || chat_1_7(&h)
    })
    .await
    .map_err(|e| format!("El hilo del modelo 1.7B falló: {e}"))??;

    for _ in 0..tools::MAX_RONDAS_TOOLS {
        let Some((nombre, args)) = tools::detectar_tool_call(&respuesta) else { break };
        tracing::info!("[YARVIS-TOOLS] ejecutando {nombre}({args})");
        let json_res = ejecutar_tool_con_rol(&nombre, &args, &db_path, es_empleado).await;
        historial.push(Mensaje::new("assistant", respuesta));
        historial.push(Mensaje::new("tool", json_res));
        respuesta = tokio::task::spawn_blocking({
            let h = historial.clone();
            move || chat_1_7(&h)
        })
        .await
        .map_err(|e| format!("El hilo del modelo 1.7B falló: {e}"))??;
    }
    let response = tools::respuesta_final_segura(&respuesta);

    Ok(ChatResponse {
        response,
        model_used: nombre_modelo_local(),
    })
}

/// Chat local con streaming: la inferencia del 1.7B es a bloqueo (llama.cpp),
/// así que se genera la respuesta completa (ya limpia de bloques  thinking ,
/// con el limpiador local robusto) y se emite en trozos para que la UI la
/// muestre de forma progresiva (eventos `chat-token`).
pub(super) async fn _stream_local(
    app: &tauri::AppHandle,
    messages: Vec<serde_json::Value>,
    db_path: String,
    es_empleado: bool,
) -> Result<String, String> {
    let mut historial = mensajes_serde_a_rust(&messages);

    let mut cleaned = tokio::task::spawn_blocking({
        let h = historial.clone();
        move || chat_1_7(&h)
    })
    .await
    .map_err(|e| format!("El hilo del modelo 1.7B falló: {e}"))??;

    // Ciclo de herramientas ANTES de emitir nada a la UI.
    for _ in 0..tools::MAX_RONDAS_TOOLS {
        let Some((nombre, args)) = tools::detectar_tool_call(&cleaned) else { break };
        tracing::info!("[YARVIS-TOOLS] ejecutando {nombre}({args})");
        let json_res = ejecutar_tool_con_rol(&nombre, &args, &db_path, es_empleado).await;
        historial.push(Mensaje::new("assistant", cleaned));
        historial.push(Mensaje::new("tool", json_res));
        cleaned = tokio::task::spawn_blocking({
            let h = historial.clone();
            move || chat_1_7(&h)
        })
        .await
        .map_err(|e| format!("El hilo del modelo 1.7B falló: {e}"))??;
    }
    let cleaned = tools::respuesta_final_segura(&cleaned);

    // Emite la respuesta en trozos (~40 chars) preservando saltos de línea.
    let mut seg = String::new();
    let mut n = 0;
    for c in cleaned.chars() {
        if stream_cancelado() {
            tracing::info!("[YARVIS-CHAT] emisión local cancelada por el usuario");
            return Ok(String::new());
        }
        seg.push(c);
        n += 1;
        if n >= 40 {
            let _ = app.emit(
                "chat-token",
                serde_json::json!({
                    "token": seg,
                    "model": nombre_modelo_local(),
                }),
            );
            seg.clear();
            n = 0;
        }
    }
    if !seg.is_empty() {
        let _ = app.emit(
            "chat-token",
            serde_json::json!({
                "token": seg,
                "model": nombre_modelo_local(),
            }),
        );
    }

    let _ = app.emit(
        "chat-done",
        serde_json::json!({ "model": nombre_modelo_local() }),
    );
    let _ = app.emit(
        "chat-complete",
        serde_json::json!({
            "response": cleaned,
            "model": nombre_modelo_local(),
        }),
    );
    Ok(cleaned)
}

/// Prepara los mensajes API según rol (helper compartido por los comandos).
pub(super) fn construir_historial(
    messages: &[serde_json::Value],
    es_empleado: bool,
) -> Vec<Mensaje> {
    construir_mensajes_api_rol(&mensajes_serde_a_rust(messages), es_empleado)
}
