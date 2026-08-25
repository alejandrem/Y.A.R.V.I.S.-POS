// ============================================================
// admintarvis/chat.rs — Comandos IPC para el chatbot de Y.A.R.V.I.S.
// Soporte streaming: cloud emitido por Rust; local = Qwen 1.7B nativo.
// Las piezas internas viven en cancelacion, herramientas_rol,
// ciclo_tools y rutas (hermanos de este módulo).
// ============================================================

use crate::backventanas::auth::AuthState;
use src_ia::motor_chat::cloud::apis_cloud::{generar_completo, nombre_proveedor};
use src_ia::motor_chat::llm::{
    cargar_modelo_1_7, descargar_modelo_1_7, modelo_1_7_cargado, nombre_modelo_local,
    ram_libre_gb, ram_total_gb, CONTEXTO_LOCAL, MODELO_CHAT, RAM_GB_MINIMA_1_7,
};

use super::cancelacion::{reset_stream_cancelado, STREAM_CANCELADO};
use super::ciclo_tools::{resolver_ciclo_tools, Generador};
use super::rutas::{
    construir_historial, db_path_de, _chat_local, _stream_cloud, _stream_local, ChatResponse,
};

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
/// Detiene la emisión del stream en curso: levanta la bandera que los bucles
/// de generación (cloud y local) consultan entre tokens y rondas de tools.
/// La inferencia local a bloqueo puede tardar en llegar a un punto de chequeo,
/// pero ningún token posterior se emite a la UI tras esta llamada.
#[tauri::command]
pub async fn stop_chat_stream(auth: tauri::State<'_, AuthState>) -> Result<String, String> {
    auth.require_operator()?;
    STREAM_CANCELADO.store(true, std::sync::atomic::Ordering::Relaxed);
    tracing::info!("[YARVIS-CHAT] stop solicitado por el usuario");
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
    tracing::info!("[YARVIS-CHAT] Descarga pedida para {model}: descargado = {descargado}");

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
    state: tauri::State<'_, sqlx::SqlitePool>,
    messages: Vec<serde_json::Value>,
    role: String,
    model: String,
    provider: Option<String>,
    api_key: Option<String>,
) -> Result<ChatResponse, String> {
    auth.require_operator()?;
    let provider = provider.unwrap_or_default();
    let db_path = db_path_de(&state);
    // Modo cloud: lo responde Rust directamente (port de generar_completo).
    // Si falla, cae al modelo local.
    if !provider.is_empty() {
        let api_key = api_key.unwrap_or_default();
        let es_empleado = auth.es_empleado();
        let chat = construir_historial(&messages, es_empleado);
        match generar_completo(&provider, &api_key, &model, chat.clone()).await {
            Ok((respuesta, modelo_real)) => {
                let usado = if modelo_real.is_empty() {
                    nombre_proveedor(&provider)
                } else {
                    modelo_real
                };
                let mut generador: Generador = Box::new(move |hist| {
                    let p = provider.clone();
                    let k = api_key.clone();
                    let m = model.clone();
                    Box::pin(async move { generar_completo(&p, &k, &m, hist).await.map(|(r, _)| r) })
                });
                let final_resp =
                    resolver_ciclo_tools(respuesta, chat, db_path, es_empleado, &mut generador).await?;
                return Ok(ChatResponse {
                    response: final_resp,
                    model_used: usado,
                });
            }
            Err(e) => {
                tracing::warn!("[YARVIS-CHAT] Error proveedor ({provider}): {e}");
                return _chat_local(messages, db_path, auth.es_empleado()).await;
            }
        }
    }

    // Modo local: Qwen 1.7B nativo de Rust.
    tracing::info!(
        "[YARVIS-CHAT] Modo local (model pedido: {model}, role: {role}) → usando {MODELO_CHAT}."
    );
    _chat_local(messages, db_path, auth.es_empleado()).await
}

/// Chat con streaming — modo cloud lo emite Rust (port de generar_stream),
/// el modo local trocea la respuesta del Qwen 1.7B nativo.
#[tauri::command]
pub async fn send_chat_stream(
    app: tauri::AppHandle,
    auth: tauri::State<'_, AuthState>,
    state: tauri::State<'_, sqlx::SqlitePool>,
    messages: Vec<serde_json::Value>,
    role: String,
    model: String,
    provider: Option<String>,
    api_key: Option<String>,
) -> Result<String, String> {
    auth.require_operator()?;
    reset_stream_cancelado(); // nueva generación: cancelación limpia
    let provider = provider.unwrap_or_default();
    let db_path = db_path_de(&state);

    // ---- Modo cloud: streaming en Rust. ----
    if !provider.is_empty() {
        let api_key = api_key.unwrap_or_default();
        let es_empleado = auth.es_empleado();
        let chat = construir_historial(&messages, es_empleado);
        match _stream_cloud(&app, &provider, &api_key, &model, chat, &db_path, es_empleado, 0).await {
            Ok(respuesta) => return Ok(respuesta),
            Err(e) => {
                tracing::warn!("[YARVIS-CHAT] Error proveedor ({provider}), fallback local: {e}");
                return _stream_local(&app, messages, db_path, es_empleado).await;
            }
        }
    }

    // ---- Modo local: Qwen 1.7B nativo de Rust. ----
    tracing::info!(
        "[YARVIS-CHAT] Modo local (model pedido: {model}, role: {role}) → usando {MODELO_CHAT}."
    );
    _stream_local(&app, messages, db_path, auth.es_empleado()).await
}
