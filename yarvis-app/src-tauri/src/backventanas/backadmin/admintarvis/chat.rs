// ============================================================
// admintarvis/chat.rs — Comandos IPC para el chatbot de Y.A.R.V.I.S.
// Soporte streaming: cloud emitido por Rust; local = Qwen 1.7B nativo.
// ============================================================

use tauri::Emitter;

use crate::backventanas::auth::AuthState;
use futures_util::StreamExt;
use src_ia::motor_chat::cloud::apis_cloud::{
    generar_completo, generar_stream, nombre_proveedor, Evento,
};
use src_ia::motor_chat::cloud::prompts::{construir_mensajes_api_rol, Mensaje};
use src_ia::motor_chat::cloud::think::{SeparadorThink, TipoFragmento};
use src_ia::motor_chat::llm::{
    cargar_modelo_1_7, chat_1_7, descargar_modelo_1_7, modelo_1_7_cargado, nombre_modelo_local,
    ram_libre_gb, ram_total_gb, CONTEXTO_LOCAL, MODELO_CHAT, RAM_GB_MINIMA_1_7,
};

/// Máximo de palabras que se consideran razonamiento en el stream cloud
/// (espejo del `max_w` que usaba el motor original para `_separar_think`).
const CLOUD_MAX_W: usize = 1000;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ChatResponse {
    pub response: String,
    pub model_used: String,
}

/// Estado de los modelos locales y la RAM del sistema (nativo).
#[tauri::command]
pub async fn get_model_status(
    auth: tauri::State<'_, AuthState>,
) -> Result<serde_json::Value, String> {
    auth.require_operator()?;
    let ram_libre = ram_libre_gb().unwrap_or(0.0);
    let ram_total = ram_total_gb().unwrap_or(0.0);
    Ok(serde_json::json!({
        "status": "ok",
        // Único modelo local: el 1.7B (parseo de tickets y conversación).
        "models": { MODELO_CHAT: modelo_1_7_cargado() },
        "ram_gb": ram_total,
        "ram_libre_gb": ram_libre,
        "local_model_path": src_ia::motor_chat::llm::ruta_modelo_local().to_string_lossy(),
        "local_model_name": nombre_modelo_local(),
        "local_context_window": CONTEXTO_LOCAL,
    }))
}

/// Guarda la ruta GGUF elegida por el usuario y libera el modelo actual para
/// que la siguiente carga use el archivo nuevo.
#[tauri::command]
pub async fn set_local_model_path(
    auth: tauri::State<'_, AuthState>,
    path: String,
) -> Result<serde_json::Value, String> {
    auth.require_operator()?;
    let path = std::path::PathBuf::from(path.trim());
    if !path.is_file() {
        return Err(format!(
            "El archivo del modelo no existe: {}",
            path.display()
        ));
    }

    tokio::task::spawn_blocking(descargar_modelo_1_7)
        .await
        .map_err(|e| format!("El hilo de descarga falló: {e}"))?;
    src_ia::rutas::configurar_ruta_modelo(Some(path))?;

    Ok(serde_json::json!({
        "status": "ok",
        "path": src_ia::motor_chat::llm::ruta_modelo_local().to_string_lossy(),
        "name": nombre_modelo_local(),
        "context_window": CONTEXTO_LOCAL,
        "models": { MODELO_CHAT: false },
    }))
}

/// Carga el modelo local de conversación (1.7B) SOLO si hay ≥1GB de RAM libre.
#[tauri::command]
pub async fn load_chat_model(
    auth: tauri::State<'_, AuthState>,
    model: String,
) -> Result<serde_json::Value, String> {
    auth.require_operator()?;
    if model != MODELO_CHAT {
        return Err(format!(
            "El único modelo local es {MODELO_CHAT} (parseo de tickets y conversación)."
        ));
    }

    let libre = ram_libre_gb()?;
    if libre < RAM_GB_MINIMA_1_7 {
        return Err(format!(
            "RAM insuficiente para {MODELO_CHAT}: hay {libre:.2}GB libres, se necesitan ≥{RAM_GB_MINIMA_1_7}GB."
        ));
    }

    let modelo = tokio::task::spawn_blocking(cargar_modelo_1_7)
        .await
        .map_err(|e| format!("El hilo de carga falló: {e}"))??;

    Ok(serde_json::json!({
        "status": "ok",
        "model": modelo,
        "models": { MODELO_CHAT: true },
        "ram_gb": ram_total_gb().unwrap_or(0.0),
        "ram_libre_gb": ram_libre_gb().unwrap_or(0.0),
    }))
}

/// Lista los modelos disponibles de un proveedor de nube (dinámico).
///
/// Devuelve `{"models": [{"id": ..., "name": ...}]}` con el mismo JSON que
/// el endpoint `/cloud_models` al que reemplaza.
#[tauri::command]
pub async fn get_cloud_models(
    auth: tauri::State<'_, AuthState>,
    provider: String,
    api_key: Option<String>,
) -> Result<serde_json::Value, String> {
    auth.require_operator()?;
    let modelos = src_ia::motor_chat::cloud::apis_cloud::listar_modelos(
        &provider,
        api_key.as_deref().unwrap_or(""),
    )
    .await?;
    Ok(serde_json::json!({ "models": modelos }))
}

/// Detiene la generación en curso. La inferencia local del 1.7B es a bloqueo
/// (llama.cpp), así que no hay nada que interrumpir: se responde ok para no
/// romper el contrato con la UI (idéntico al endpoint `/stop` del motor original).
#[tauri::command]
pub async fn stop_chat_stream(auth: tauri::State<'_, AuthState>) -> Result<String, String> {
    auth.require_operator()?;
    Ok("ok".to_string())
}

/// Descarga el modelo local de conversación (1.7B) para liberar RAM.
#[tauri::command]
pub async fn unload_chat_model(
    auth: tauri::State<'_, AuthState>,
    model: String,
) -> Result<serde_json::Value, String> {
    auth.require_operator()?;
    let descargado = tokio::task::spawn_blocking(descargar_modelo_1_7)
        .await
        .map_err(|e| format!("El hilo de descarga falló: {e}"))?;
    println!("[YARVIS-CHAT] Descarga pedida para {model}: descargado = {descargado}");

    Ok(serde_json::json!({
        "status": "ok",
        "models": { MODELO_CHAT: false },
        "ram_gb": ram_total_gb().unwrap_or(0.0),
        "ram_libre_gb": ram_libre_gb().unwrap_or(0.0),
    }))
}

/// Chat sin streaming (respuesta completa).
#[tauri::command]
pub async fn send_chat_message(
    auth: tauri::State<'_, AuthState>,
    messages: Vec<serde_json::Value>,
    role: String,
    model: String,
    provider: Option<String>,
    api_key: Option<String>,
) -> Result<ChatResponse, String> {
    auth.require_operator()?;
    let provider = provider.unwrap_or_default();
    // Modo cloud: lo responde Rust directamente (port de generar_completo).
    // Si falla, cae al modelo local.
    if !provider.is_empty() {
        let api_key = api_key.unwrap_or_default();
        let es_empleado = auth.es_empleado();
        let chat = construir_mensajes_api_rol(&mensajes_serde_a_rust(&messages), es_empleado);
        match generar_completo(&provider, &api_key, &model, chat).await {
            Ok((respuesta, modelo_real)) => {
                let usado = if modelo_real.is_empty() {
                    nombre_proveedor(&provider)
                } else {
                    modelo_real
                };
                return Ok(ChatResponse {
                    response: respuesta,
                    model_used: usado,
                });
            }
            Err(e) => {
                println!("[YARVIS-CHAT] Error proveedor ({provider}): {e}");
                return _chat_local(messages).await;
            }
        }
    }

    // Modo local: Qwen 1.7B nativo de Rust.
    println!(
        "[YARVIS-CHAT] Modo local (model pedido: {model}, role: {role}) → usando {MODELO_CHAT}."
    );
    _chat_local(messages).await
}

/// Chat con streaming — modo cloud lo emite Rust (port de generar_stream),
/// el modo local trocea la respuesta del Qwen 1.7B nativo.
#[tauri::command]
pub async fn send_chat_stream(
    app: tauri::AppHandle,
    auth: tauri::State<'_, AuthState>,
    messages: Vec<serde_json::Value>,
    role: String,
    model: String,
    provider: Option<String>,
    api_key: Option<String>,
) -> Result<String, String> {
    auth.require_operator()?;
    let provider = provider.unwrap_or_default();

    // ---- Modo cloud: streaming en Rust. ----
    if !provider.is_empty() {
        let api_key = api_key.unwrap_or_default();
        let es_empleado = auth.es_empleado();
        let chat = construir_mensajes_api_rol(&mensajes_serde_a_rust(&messages), es_empleado);
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
                        let _ = app.emit(
                            "chat-token",
                            serde_json::json!({
                                "token": frag,
                                "model": modelo,
                            }),
                        );
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
            full_response.push_str(&frag);
            let _ = app.emit(
                "chat-token",
                serde_json::json!({
                    "token": frag,
                    "model": model_used,
                }),
            );
        }
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

/// Chat local (sin provider): responde el Qwen 1.7B nativo de Rust.
async fn _chat_local(messages: Vec<serde_json::Value>) -> Result<ChatResponse, String> {
    let msgs = mensajes_serde_a_rust(&messages);
    let response = tokio::task::spawn_blocking(move || chat_1_7(&msgs))
        .await
        .map_err(|e| format!("El hilo del modelo 1.7B falló: {e}"))??;
    Ok(ChatResponse {
        response,
        model_used: nombre_modelo_local(),
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

/// Convierte los mensajes `{role, content}` que manda el frontend a [`Mensaje`].
fn mensajes_serde_a_rust(messages: &[serde_json::Value]) -> Vec<Mensaje> {
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
