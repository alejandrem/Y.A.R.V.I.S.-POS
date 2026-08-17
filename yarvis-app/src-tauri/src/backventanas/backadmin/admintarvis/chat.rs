// ============================================================
// admintarvis/chat.rs — Comandos IPC para el chatbot de Y.A.R.V.I.S.
// Soporte streaming: lee SSE de Python y envía tokens vía Tauri events.
// ============================================================

use std::sync::Arc;
use tauri::Emitter;
use crate::sidecar::AiSidecar;

use src_ia::motor_chat::cloud::apis_cloud::{generar_completo, generar_stream, nombre_proveedor, Evento};
use src_ia::motor_chat::cloud::prompts::{construir_mensajes_api, Mensaje};
use src_ia::motor_chat::cloud::think::{SeparadorThink, TipoFragmento};
use futures_util::StreamExt;
use src_ia::motor_chat::llm::{MODELO_CHAT, chat_1_7};

/// Máximo de palabras que se consideran razonamiento en el stream cloud
/// (espejo del `max_w` que Python usa para `_separar_think`).
const CLOUD_MAX_W: usize = 1000;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ChatResponse {
    pub response: String,
    pub model_used: String,
}

/// Estado de los modelos y RAM del sistema.
#[tauri::command]
pub async fn get_model_status(
    sidecar: tauri::State<'_, Arc<AiSidecar>>,
) -> Result<serde_json::Value, String> {
    let base_url = sidecar.base_url()
        .ok_or("El motor de IA no está disponible")?;
    let resp = sidecar.http_client
        .get(format!("{}/model_status", base_url))
        .send().await
        .map_err(|e| e.to_string())?;
    resp.json().await.map_err(|e| e.to_string())
}

/// Carga un modelo bajo demanda (0.5B, 0.8B, 1.7B).
#[tauri::command]
pub async fn load_chat_model(
    sidecar: tauri::State<'_, Arc<AiSidecar>>,
    model: String,
) -> Result<serde_json::Value, String> {
    let base_url = sidecar.base_url()
        .ok_or("El motor de IA no está disponible")?;
    let resp = sidecar.http_client
        .post(format!("{}/load_model", base_url))
        .json(&serde_json::json!({"model": model}))
        .send().await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(text);
    }
    resp.json().await.map_err(|e| e.to_string())
}

/// Lista los modelos disponibles de un proveedor de nube (dinámico).
///
/// Devuelve `{"models": [{"id": ..., "name": ...}]}` igual que el endpoint
/// Python `/cloud_models` que reemplaza (mismos modelos, mismo JSON).
#[tauri::command]
pub async fn get_cloud_models(
    provider: String,
    api_key: Option<String>,
) -> Result<serde_json::Value, String> {
    let modelos = src_ia::motor_chat::cloud::apis_cloud::listar_modelos(
        &provider,
        api_key.as_deref().unwrap_or(""),
    )
    .await?;
    Ok(serde_json::json!({ "models": modelos }))
}

/// Detiene la generación en curso (local o nube) en el motor de IA.
#[tauri::command]
pub async fn stop_chat_stream(
    sidecar: tauri::State<'_, Arc<AiSidecar>>,
) -> Result<String, String> {
    let base_url = sidecar.base_url()
        .ok_or("El motor de IA no está disponible")?;
    let resp = sidecar.http_client
        .post(format!("{}/stop", base_url))
        .send().await
        .map_err(|e| format!("Error al detener: {}", e))?;
    resp.text().await.map_err(|e| e.to_string())
}

/// Descarga un modelo para liberar RAM.
#[tauri::command]
pub async fn unload_chat_model(
    sidecar: tauri::State<'_, Arc<AiSidecar>>,
    model: String,
) -> Result<serde_json::Value, String> {
    let base_url = sidecar.base_url()
        .ok_or("El motor de IA no está disponible")?;
    let resp = sidecar.http_client
        .post(format!("{}/unload_model", base_url))
        .json(&serde_json::json!({"model": model}))
        .send().await
        .map_err(|e| e.to_string())?;
    resp.json().await.map_err(|e| e.to_string())
}

/// Chat sin streaming (respuesta completa).
#[tauri::command]
pub async fn send_chat_message(
    messages: Vec<serde_json::Value>,
    role: String,
    model: String,
    provider: Option<String>,
    api_key: Option<String>,
) -> Result<ChatResponse, String> {
    let provider = provider.unwrap_or_default();
    // Modo cloud: lo responde Rust directamente (port de generar_completo).
    // Si falla, cae al modelo local (igual que hacía Python).
    if !provider.is_empty() {
        let api_key = api_key.unwrap_or_default();
        let chat = construir_mensajes_api(&mensajes_serde_a_rust(&messages));
        match generar_completo(&provider, &api_key, &model, chat).await {
            Ok(respuesta) => {
                return Ok(ChatResponse {
                    response: respuesta,
                    model_used: nombre_proveedor(&provider),
                });
            }
            Err(e) => {
                println!("[YARVIS-CHAT] Error proveedor ({provider}): {e}");
                return _chat_local(messages).await;
            }
        }
    }

    // Modo local: Qwen 1.7B nativo de Rust (sin sidecar Python).
    println!(
        "[YARVIS-CHAT] Modo local (model pedido: {model}, role: {role}) → usando {MODELO_CHAT}."
    );
    _chat_local(messages).await
}

/// Chat con streaming — modo cloud lo emite Rust (port de generar_stream),
/// el modo local trocea la respuesta del Qwen 1.7B nativo (sin sidecar Python).
#[tauri::command]
pub async fn send_chat_stream(
    app: tauri::AppHandle,
    messages: Vec<serde_json::Value>,
    role: String,
    model: String,
    provider: Option<String>,
    api_key: Option<String>,
) -> Result<String, String> {
    let provider = provider.unwrap_or_default();

    // ---- Modo cloud: streaming en Rust (sin dependencia del sidecar). ----
    if !provider.is_empty() {
        let api_key = api_key.unwrap_or_default();
        let chat = construir_mensajes_api(&mensajes_serde_a_rust(&messages));
        match _stream_cloud(&app, &provider, &api_key, &model, chat).await {
            Ok(respuesta) => return Ok(respuesta),
            Err(e) => {
                println!("[YARVIS-CHAT] Error proveedor ({provider}), fallback local: {e}");
                return _stream_local(&app, messages).await;
            }
        }
    }

    // ---- Modo local: Qwen 1.7B nativo de Rust. ----
    println!(
        "[YARVIS-CHAT] Modo local (model pedido: {model}, role: {role}) → usando {MODELO_CHAT}."
    );
    _stream_local(&app, messages).await
}

/// Emite un stream cloud completo con Rust y devuelve la respuesta acumulada.
async fn _stream_cloud(
    app: &tauri::AppHandle,
    provider: &str,
    api_key: &str,
    model: &str,
    chat: Vec<Mensaje>,
) -> Result<String, String> {
    let stream = generar_stream(provider, api_key, model, chat);
    futures_util::pin_mut!(stream);

    let mut sep = SeparadorThink::new(CLOUD_MAX_W);
    let mut full_response = String::new();
    let mut model_used = String::from("unknown");

    while let Some(item) = stream.next().await {
        match item {
            Ok(Evento::Texto { texto, modelo }) => {
                if model_used == "unknown" {
                    model_used = modelo.clone();
                }
                for (tipo, frag) in sep.procesar(&texto) {
                    if tipo == TipoFragmento::Token {
                        full_response.push_str(&frag);
                        let _ = app.emit("chat-token", serde_json::json!({
                            "token": frag,
                            "model": modelo,
                        }));
                    } else {
                        let _ = app.emit("chat-think", serde_json::json!({
                            "token": frag,
                            "model": modelo,
                        }));
                    }
                }
            }
            Ok(Evento::Uso { usage, modelo }) => {
                if model_used == "unknown" {
                    model_used = modelo;
                }
                let _ = app.emit("chat-usage", serde_json::json!({
                    "usage": serde_json::json!({
                        "prompt_tokens": usage.prompt_tokens,
                        "completion_tokens": usage.completion_tokens,
                        "total_tokens": usage.total_tokens,
                    }),
                }));
            }
            Err(e) => return Err(e),
        }
    }
    for (tipo, frag) in sep.finalizar() {
        if tipo == TipoFragmento::Token {
            full_response.push_str(&frag);
            let _ = app.emit("chat-token", serde_json::json!({
                "token": frag,
                "model": model_used,
            }));
        }
    }

    let _ = app.emit("chat-done", serde_json::json!({ "model": model_used }));
    let _ = app.emit("chat-complete", serde_json::json!({
        "response": full_response,
        "model": model_used,
    }));
    Ok(full_response)
}

/// Chat local (sin provider): responde el Qwen 1.7B nativo de Rust.
async fn _chat_local(messages: Vec<serde_json::Value>) -> Result<ChatResponse, String> {
    let msgs = mensajes_serde_a_rust(&messages);
    let response = tokio::task::spawn_blocking(move || chat_1_7(&msgs))
        .await
        .map_err(|e| format!("El hilo del modelo 1.7B falló: {e}"))??;
    Ok(ChatResponse {
        response,
        model_used: MODELO_CHAT.to_string(),
    })
}

/// Chat local con streaming: la inferencia del 1.7B es a bloqueo (llama.cpp),
/// así que se genera la respuesta completa (ya limpia de bloques  thinking ,
/// con el limpiador local robusto) y se emite en trozos para que la UI la
/// muestre de forma progresiva (eventos `chat-token`).
async fn _stream_local(
    app: &tauri::AppHandle,
    messages: Vec<serde_json::Value>,
) -> Result<String, String> {
    let msgs = mensajes_serde_a_rust(&messages);

    let cleaned = tokio::task::spawn_blocking(move || chat_1_7(&msgs))
        .await
        .map_err(|e| format!("El hilo del modelo 1.7B falló: {e}"))??;

    // Emite la respuesta en trozos (~40 chars) preservando saltos de línea.
    let mut seg = String::new();
    let mut n = 0;
    for c in cleaned.chars() {
        seg.push(c);
        n += 1;
        if n >= 40 {
            let _ = app.emit("chat-token", serde_json::json!({
                "token": seg,
                "model": MODELO_CHAT,
            }));
            seg.clear();
            n = 0;
        }
    }
    if !seg.is_empty() {
        let _ = app.emit("chat-token", serde_json::json!({
            "token": seg,
            "model": MODELO_CHAT,
        }));
    }

    let _ = app.emit("chat-done", serde_json::json!({ "model": MODELO_CHAT }));
    let _ = app.emit("chat-complete", serde_json::json!({
        "response": cleaned,
        "model": MODELO_CHAT,
    }));
    Ok(cleaned)
}

/// Convierte los mensajes `{role, content}` que manda el frontend a [`Mensaje`].
fn mensajes_serde_a_rust(messages: &[serde_json::Value]) -> Vec<Mensaje> {
    messages
        .iter()
        .map(|m| Mensaje {
            role: m.get("role").and_then(|r| r.as_str()).unwrap_or("user").to_string(),
            content: m.get("content").and_then(|c| c.as_str()).unwrap_or("").to_string(),
        })
        .collect()
}
