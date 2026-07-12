// Parser Excel en Rust (envía al sidecar Python).
use crate::sidecar::AiSidecar;
use std::sync::Arc;

/// Parsea catálogo Excel (.xlsx) - recibe bytes del archivo
#[tauri::command]
pub async fn parsear_excel(
    archivo: Vec<u8>,
    sidecar: tauri::State<'_, Arc<AiSidecar>>,
) -> Result<serde_json::Value, String> {
    let base_url = sidecar.base_url()
        .ok_or("El motor de IA no está disponible (sidecar no iniciado)")?;

    let resp = sidecar.http_client
        .post(format!("{}/parsear_excel", base_url))
        .body(archivo)
        .header("Content-Type", "application/octet-stream")
        .send()
        .await
        .map_err(|e| format!("Error conectando al motor de IA: {}", e))?;

    let status = resp.status();
    let resultado: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Error leyendo respuesta del motor de IA: {}", e))?;

    if status.is_success() {
        Ok(resultado)
    } else {
        let msg = resultado.get("detail")
            .and_then(|d| d.as_str())
            .unwrap_or("Error desconocido del motor de IA");
        Err(msg.to_string())
    }
}
