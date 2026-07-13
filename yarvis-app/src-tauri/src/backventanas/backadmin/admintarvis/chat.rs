// ============================================================
// admintarvis/chat.rs — Comandos IPC para el chatbot de Y.A.R.V.I.S.
// Soporte streaming: lee SSE de Python y envía tokens vía Tauri events.
// ============================================================

use std::sync::Arc;
use tauri::Emitter;
use crate::sidecar::AiSidecar;
use crate::backventanas::db::db::DbPath;
use sqlx::SqlitePool;

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
    _pool: tauri::State<'_, SqlitePool>,
    _db_path: tauri::State<'_, DbPath>,
    messages: Vec<serde_json::Value>,
    role: String,
    model: String,
) -> Result<ChatResponse, String> {
    let base_url = sidecar.base_url()
        .ok_or("El motor de IA no está disponible (sidecar no iniciado)")?;

    let tienda_info = _obtener_tienda_info(&_pool).await.unwrap_or_default();
    let client = &sidecar.http_client;
    let url = format!("{}/chat", base_url);

    let body = serde_json::json!({
        "messages": messages,
        "role": role,
        "model": model,
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

/// Chat con streaming — lee SSE de Python y emite eventos por token.
#[tauri::command]
pub async fn send_chat_stream(
    sidecar: tauri::State<'_, Arc<AiSidecar>>,
    _pool: tauri::State<'_, SqlitePool>,
    _db_path: tauri::State<'_, DbPath>,
    app: tauri::AppHandle,
    messages: Vec<serde_json::Value>,
    role: String,
    model: String,
) -> Result<String, String> {
    let base_url = sidecar.base_url()
        .ok_or("El motor de IA no está disponible (sidecar no iniciado)")?;

    let tienda_info = _obtener_tienda_info(&_pool).await.unwrap_or_default();
    let client = &sidecar.http_client;
    let url = format!("{}/chat_stream", base_url);

    let body = serde_json::json!({
        "messages": messages,
        "role": role,
        "model": model,
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
    use futures_util::StreamExt;

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
