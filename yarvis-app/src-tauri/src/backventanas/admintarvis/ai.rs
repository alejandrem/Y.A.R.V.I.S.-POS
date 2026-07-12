// ============================================================
// commands/ai.rs — Comandos IPC relacionados con el motor de IA
// ============================================================

use std::sync::Arc;
use crate::sidecar::{AiSidecar, AiStatus};

/// Respuesta serializable que el frontend puede leer.
#[derive(serde::Serialize)]
pub struct AiStatusResponse {
    pub status: String,        // "not_running" | "starting" | "ready" | "error"
    pub port: Option<u16>,     // Puerto donde corre Python (útil para debugging)
    pub message: Option<String>, // Mensaje de error si aplica
}

impl From<(AiStatus, Option<u16>)> for AiStatusResponse {
    fn from((status, port): (AiStatus, Option<u16>)) -> Self {
        match status {
            AiStatus::NotRunning => AiStatusResponse {
                status: "not_running".to_string(),
                port,
                message: None,
            },
            AiStatus::Starting => AiStatusResponse {
                status: "starting".to_string(),
                port,
                message: Some("El motor de IA está arrancando...".to_string()),
            },
            AiStatus::Ready => AiStatusResponse {
                status: "ready".to_string(),
                port,
                message: Some(format!("Motor listo en puerto {}", port.unwrap_or(0))),
            },
            AiStatus::Error(msg) => AiStatusResponse {
                status: "error".to_string(),
                port,
                message: Some(msg),
            },
        }
    }
}

/// Consulta el estado del motor de IA desde el frontend.
/// El React puede llamar a invoke("get_ai_status") para saber
/// si puede mostrar el chatbot o no.
#[tauri::command]
pub async fn get_ai_status(
    sidecar: tauri::State<'_, Arc<AiSidecar>>,
) -> Result<AiStatusResponse, String> {
    let status = sidecar.get_status();
    let port = sidecar.get_port();
    Ok(AiStatusResponse::from((status, port)))
}
