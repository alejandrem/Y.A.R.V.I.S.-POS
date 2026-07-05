// Parser TXT/Visual en Rust (envía al sidecar Python).
// Incluye parsing de tickets y catálogos visuales.
use std::fs;
use crate::models::TicketItem;
use crate::sidecar::AiSidecar;
use std::sync::Arc;
use super::utils::sanitize_path;

#[tauri::command]
pub fn leer_archivo_raw(path: String) -> Result<String, String> {
    let safe_path = sanitize_path(&path)?;
    fs::read_to_string(safe_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn leer_archivo_bytes(path: String) -> Result<Vec<u8>, String> {
    let safe_path = sanitize_path(&path)?;
    fs::read(safe_path).map_err(|e| e.to_string())
}

// ============================================================
// Parser de catálogo visual (envía al sidecar Python)
// ============================================================

#[tauri::command]
pub async fn parsear_catalogo_visual(
    path: String,
    sidecar: tauri::State<'_, Arc<AiSidecar>>,
) -> Result<serde_json::Value, String> {
    let safe_path = sanitize_path(&path)?;
    let content = fs::read_to_string(safe_path).map_err(|e| e.to_string())?;

    let base_url = sidecar.base_url()
        .ok_or("El motor de IA no está disponible (sidecar no iniciado)")?;

    let body = serde_json::json!({ "texto": content });

    let resp = sidecar.http_client
        .post(format!("{}/parsear_catalogo_visual", base_url))
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

// ============================================================
// Parser de tickets
// ============================================================

#[tauri::command]
pub fn parsear_ticket(path: String) -> Result<Vec<TicketItem>, String> {
    let safe_path = sanitize_path(&path)?;
    let content = fs::read_to_string(safe_path).map_err(|e| e.to_string())?;
    let mut items = Vec::new();

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            if let Ok(cantidad) = parts[0].parse::<f64>() {
                if cantidad > 0.0 {
                    let total_str = parts.last().unwrap().replace('$', "").replace(',', "");
                    let precio_str = parts[parts.len()-2].replace('$', "").replace(',', "");

                    let total = total_str.parse::<f64>().unwrap_or(0.0);
                    let precio = precio_str.parse::<f64>().unwrap_or(0.0);

                    let producto = parts[1..parts.len()-2].join(" ");

                    if !producto.is_empty() && total > 0.0 {
                        items.push(TicketItem {
                            producto: producto.to_uppercase(),
                            cantidad,
                            precio,
                            total
                        });
                    }
                }
            }
        }
    }
    Ok(items)
}

// ============================================================
// Análisis de tickets con LLM
// ============================================================

#[tauri::command]
pub async fn analizar_ticket_llm(
    path: String,
    sidecar: tauri::State<'_, Arc<AiSidecar>>,
) -> Result<serde_json::Value, String> {
    let safe_path = sanitize_path(&path)?;
    let contenido = fs::read_to_string(&safe_path)
        .map_err(|e| format!("No se pudo leer el archivo: {}", e))?;

    let base_url = sidecar.base_url()
        .ok_or("El motor de IA no está disponible (sidecar no iniciado)")?;

    let body = serde_json::json!({ "texto": contenido });

    let resp = sidecar.http_client
        .post(format!("{}/analizar_ticket", base_url))
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
pub async fn analizar_ticket_con_ia(
    sidecar: tauri::State<'_, Arc<AiSidecar>>,
    texto: String,
) -> Result<serde_json::Value, String> {
    let base_url = sidecar.base_url()
        .ok_or("El motor de IA no está disponible (sidecar no iniciado)")?;

    let body = serde_json::json!({ "texto": texto });

    let resp = sidecar.http_client
        .post(format!("{}/analizar_ticket", base_url))
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

// ============================================================
// Parseo con mapeo de columnas
// ============================================================

#[tauri::command]
pub async fn parsear_con_mapeo(
    sidecar: tauri::State<'_, Arc<AiSidecar>>,
    texto: String,
    mapeo: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let base_url = sidecar.base_url()
        .ok_or("El motor de IA no está disponible (sidecar no iniciado)")?;

    let body = serde_json::json!({ "texto": texto, "mapeo": mapeo });

    let resp = sidecar.http_client
        .post(format!("{}/parsear_con_mapeo", base_url))
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

// ============================================================
// Parseo de carpetas
// ============================================================

#[tauri::command]
pub async fn parsear_carpeta(
    sidecar: tauri::State<'_, Arc<AiSidecar>>,
    carpeta: String,
    mapeo: serde_json::Value,
    db_path: String,
) -> Result<serde_json::Value, String> {
    let base_url = sidecar.base_url()
        .ok_or("El motor de IA no está disponible (sidecar no iniciado)")?;

    let body = serde_json::json!({
        "carpeta": carpeta,
        "mapeo": mapeo,
        "db_path": db_path
    });

    let resp = sidecar.http_client
        .post(format!("{}/parsear_carpeta", base_url))
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
pub async fn parsear_carpeta_stream(
    sidecar: tauri::State<'_, Arc<AiSidecar>>,
    carpeta: String,
    mapeo: serde_json::Value,
    db_path: String,
) -> Result<String, String> {
    let base_url = sidecar.base_url()
        .ok_or("El motor de IA no está disponible (sidecar no iniciado)")?;

    let body = serde_json::json!({
        "carpeta": carpeta,
        "mapeo": mapeo,
        "db_path": db_path
    });

    let resp = sidecar.http_client
        .post(format!("{}/parsear_carpeta_stream", base_url))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Error conectando al motor de IA: {}", e))?;

    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| format!("Error leyendo respuesta del motor de IA: {}", e))?;

    if status.is_success() {
        Ok(text)
    } else {
        Err(format!("Error del motor de IA: {}", text))
    }
}
