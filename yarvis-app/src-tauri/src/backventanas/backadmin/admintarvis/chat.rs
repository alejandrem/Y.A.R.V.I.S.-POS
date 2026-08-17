// ============================================================
// admintarvis/chat.rs — Comandos IPC para el chatbot de Y.A.R.V.I.S.
// Soporte streaming: lee SSE de Python y envía tokens vía Tauri events.
// ============================================================

use std::sync::Arc;
use tauri::Emitter;
use crate::sidecar::AiSidecar;
use sqlx::SqlitePool;

use src_ia::motor_chat::cloud::apis_cloud::{generar_completo, generar_stream, nombre_proveedor, Evento};
use src_ia::motor_chat::cloud::prompts::{construir_mensajes_api, Mensaje};
use src_ia::motor_chat::cloud::think::{SeparadorThink, TipoFragmento};
use futures_util::StreamExt;

/// Máximo de palabras que se consideran razonamiento en el stream cloud
/// (espejo del `max_w` que Python usa para `_separar_think`).
const CLOUD_MAX_W: usize = 1000;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ChatResponse {
    pub response: String,
    pub model_used: String,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct TiendaInfo {
    pub nombre: Option<String>,
    pub ubicacion: Option<String>,
    pub cp: Option<String>,
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
    sidecar: tauri::State<'_, Arc<AiSidecar>>,
    pool: tauri::State<'_, SqlitePool>,
    messages: Vec<serde_json::Value>,
    role: String,
    model: String,
    provider: Option<String>,
    api_key: Option<String>,
) -> Result<ChatResponse, String> {
    let provider = provider.unwrap_or_default();
    // Modo cloud: lo responde Rust directamente (port de generar_completo).
    // Si falla, cae al modelo local vía sidecar (igual que hacía Python).
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
                return _chat_local(sidecar, pool, messages, role, model).await;
            }
        }
    }

    _chat_local(sidecar, pool, messages, role, model).await
}

/// Chat con streaming — modo cloud lo emite Rust (port de generar_stream),
/// el modo local sigue leyendo SSE del sidecar Python. Si la nube falla,
/// cae al local (igual que el fallback que Python hacía con _fallback_local).
#[tauri::command]
pub async fn send_chat_stream(
    sidecar: tauri::State<'_, Arc<AiSidecar>>,
    pool: tauri::State<'_, SqlitePool>,
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
                return _stream_local(sidecar, pool, &app, messages, role, model).await;
            }
        }
    }

    _stream_local(sidecar, pool, &app, messages, role, model).await
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

/// Chat local (sin provider) contra el sidecar Python, sin streaming.
async fn _chat_local(
    sidecar: tauri::State<'_, Arc<AiSidecar>>,
    pool: tauri::State<'_, SqlitePool>,
    messages: Vec<serde_json::Value>,
    role: String,
    model: String,
) -> Result<ChatResponse, String> {
    let base_url = sidecar.base_url()
        .ok_or("El motor de IA no está disponible (sidecar no iniciado)")?;

    let tienda_info = _obtener_tienda_info(&pool).await.unwrap_or_default();
    let client = &sidecar.http_client;
    let url = format!("{}/chat", base_url);

    let body = serde_json::json!({
        "messages": messages,
        "role": role,
        "model": model,
        "provider": "",
        "api_key": "",
        "tienda_info": {
            "nombre": tienda_info.nombre,
            "ubicacion": tienda_info.ubicacion,
            "cp": tienda_info.cp,
        }
    });

    let resp = client.post(&url).json(&body).send().await
        .map_err(|e| format!("Error al conectar con el motor de IA: {}", e))?;

    if !resp.status().is_success() {
        let err_msg = format!("Error {}", resp.status());
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("{}: {}", err_msg, text));
    }

    resp.json().await.map_err(|e| format!("Error al leer respuesta: {}", e))
}

/// Chat local con streaming: lee SSE del sidecar Python y emite eventos.
async fn _stream_local(
    sidecar: tauri::State<'_, Arc<AiSidecar>>,
    pool: tauri::State<'_, SqlitePool>,
    app: &tauri::AppHandle,
    messages: Vec<serde_json::Value>,
    role: String,
    model: String,
) -> Result<String, String> {
    let base_url = sidecar.base_url()
        .ok_or("El motor de IA no está disponible (sidecar no iniciado)")?;

    let tienda_info = _obtener_tienda_info(&pool).await.unwrap_or_default();
    let client = &sidecar.http_client;
    let url = format!("{}/chat_stream", base_url);

    let body = serde_json::json!({
        "messages": messages,
        "role": role,
        "model": model,
        "provider": "",
        "api_key": "",
        "tienda_info": {
            "nombre": tienda_info.nombre,
            "ubicacion": tienda_info.ubicacion,
            "cp": tienda_info.cp,
        }
    });

    let resp = client.post(&url).json(&body).send().await
        .map_err(|e| format!("Error al conectar: {}", e))?;

    if !resp.status().is_success() {
        let err_msg = format!("Error {}", resp.status());
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("{}: {}", err_msg, text));
    }

    let mut full_response = String::new();
    let mut model_used = String::from("unknown");
    let mut buffer = String::new();

    let mut stream = resp.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let bytes = chunk.map_err(|e| format!("Error leyendo stream: {}", e))?;
        buffer.push_str(&String::from_utf8_lossy(&bytes));

        while let Some(newline_pos) = buffer.find('\n') {
            let line = buffer[..newline_pos].trim().to_string();
            buffer = buffer[newline_pos + 1..].to_string();

            if line.starts_with("data: ") {
                let data_str = &line[6..];
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(data_str) {
                    if let Some(token) = data.get("token").and_then(|t| t.as_str()) {
                        full_response.push_str(token);
                        model_used = data.get("model")
                            .and_then(|m| m.as_str())
                            .unwrap_or("unknown")
                            .to_string();
                        let _ = app.emit("chat-token", serde_json::json!({
                            "token": token,
                            "model": model_used,
                        }));
                    }
                    if let Some(think) = data.get("think").and_then(|t| t.as_str()) {
                        if model_used == "unknown" {
                            model_used = data.get("model")
                                .and_then(|m| m.as_str())
                                .unwrap_or("unknown")
                                .to_string();
                        }
                        let _ = app.emit("chat-think", serde_json::json!({
                            "token": think,
                            "model": model_used,
                        }));
                    }
                    if let Some(usage) = data.get("usage").and_then(|u| u.as_object()) {
                        if !usage.is_empty() {
                            model_used = data.get("model")
                                .and_then(|m| m.as_str())
                                .unwrap_or(&model_used)
                                .to_string();
                            let _ = app.emit("chat-usage", serde_json::json!({
                                "usage": usage,
                            }));
                        }
                    }
                    if data.get("done").and_then(|d| d.as_bool()).unwrap_or(false) {
                        model_used = data.get("model")
                            .and_then(|m| m.as_str())
                            .unwrap_or(&model_used)
                            .to_string();
                        let _ = app.emit("chat-done", serde_json::json!({
                            "model": model_used,
                        }));
                    }
                    if let Some(err) = data.get("error").and_then(|e| e.as_str()) {
                        let _ = app.emit("chat-error", serde_json::json!({
                            "error": err,
                        }));
                        return Err(err.to_string());
                    }
                }
            }
        }
    }

    let _ = app.emit("chat-complete", serde_json::json!({
        "response": full_response,
        "model": model_used,
    }));

    Ok(full_response)
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

async fn _obtener_tienda_info(pool: &SqlitePool) -> Result<TiendaInfo, sqlx::Error> {
    let row: (Option<String>, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT tienda, ubicacion, cp FROM usuarios WHERE rol = 'admin' LIMIT 1"
    )
    .fetch_one(pool)
    .await?;

    Ok(TiendaInfo {
        nombre: row.0,
        ubicacion: row.1,
        cp: row.2,
    })
}
