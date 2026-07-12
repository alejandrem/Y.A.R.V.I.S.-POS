use crate::sidecar::AiSidecar;
use crate::backventanas::db::db::DbPath;
use std::sync::Arc;

/// Retorna la ruta absoluta de la base de datos SQLite.
#[tauri::command]
pub fn get_db_path(db_path: tauri::State<'_, DbPath>) -> Result<String, String> {
    Ok(db_path.0.clone())
}

// ============================================================
// Comandos que siguen en commands/parser (no movidos a parser_rs)
// ============================================================

#[tauri::command]
pub async fn vincular_inventario(
    sidecar: tauri::State<'_, Arc<AiSidecar>>,
    productos: serde_json::Value,
    db_path: String,
    umbral: f64,
) -> Result<serde_json::Value, String> {
    let base_url = sidecar.base_url()
        .ok_or("El motor de IA no está disponible (sidecar no iniciado)")?;

    let body = serde_json::json!({
        "productos": productos,
        "db_path": db_path,
        "umbral": umbral,
    });

    let resp = sidecar.http_client
        .post(format!("{}/vincular_inventario", base_url))
        .json(&body)
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

#[tauri::command]
pub async fn guardar_vinculacion(
    sidecar: tauri::State<'_, Arc<AiSidecar>>,
    vinculaciones: serde_json::Value,
    db_path: String,
) -> Result<serde_json::Value, String> {
    let base_url = sidecar.base_url()
        .ok_or("El motor de IA no está disponible (sidecar no iniciado)")?;

    let body = serde_json::json!({
        "vinculaciones": vinculaciones,
        "db_path": db_path,
    });

    let resp = sidecar.http_client
        .post(format!("{}/guardar_vinculacion", base_url))
        .json(&body)
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

#[tauri::command]
pub async fn descargar_modelos(
    sidecar: tauri::State<'_, Arc<AiSidecar>>,
) -> Result<serde_json::Value, String> {
    let base_url = sidecar.base_url()
        .ok_or("El motor de IA no está disponible (sidecar no iniciado)")?;

    let resp = sidecar.http_client
        .post(format!("{}/unload_llm", base_url))
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
