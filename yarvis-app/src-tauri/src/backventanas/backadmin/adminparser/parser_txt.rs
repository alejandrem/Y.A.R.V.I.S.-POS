// Parser TXT/Visual en Rust (envía al sidecar Python).
// Incluye parsing de tickets y catálogos visuales.
use std::fs;
use std::path;
use crate::models::TicketItem;
use crate::sidecar::AiSidecar;
use std::sync::Arc;
use super::utils::sanitize_path;
use tauri::Emitter;
use futures_util::StreamExt;

#[derive(serde::Serialize)]
pub struct ArchivoCarpeta {
    pub nombre: String,
    pub ruta: String,
    pub tamano: u64,
    pub preview: String,
}

#[tauri::command]
pub fn listar_archivos_carpeta(carpeta: String) -> Result<Vec<ArchivoCarpeta>, String> {
    let dir = path::Path::new(&carpeta);
    if !dir.is_dir() {
        return Err(format!("La ruta no es una carpeta: {}", carpeta));
    }

    let mut archivos = Vec::new();
    let entries = fs::read_dir(dir).map_err(|e| format!("Error leyendo carpeta: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Error leyendo entrada: {}", e))?;
        let file_path = entry.path();

        if !file_path.is_file() {
            continue;
        }

        let ext = file_path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if ext != "txt" {
            continue;
        }

        let nombre = file_path.file_name()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();

        let ruta = file_path.to_string_lossy().to_string();

        let tamano = fs::metadata(&file_path)
            .map(|m| m.len())
            .unwrap_or(0);

        // Leer primeras 5 lineas para preview
        let preview = fs::read_to_string(&file_path)
            .map(|content| {
                content.lines()
                    .take(5)
                    .collect::<Vec<&str>>()
                    .join("\n")
            })
            .unwrap_or_else(|_| "Error al leer archivo".to_string());

        archivos.push(ArchivoCarpeta {
            nombre,
            ruta,
            tamano,
            preview,
        });
    }

    archivos.sort_by(|a, b| a.nombre.cmp(&b.nombre));
    Ok(archivos)
}

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
    app_handle: tauri::AppHandle,
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

    if !resp.status().is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Error del motor de IA: {}", text));
    }

    // Leer la respuesta SSE como stream de bytes y emitir eventos en tiempo real
    let mut stream = resp.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Error leyendo stream: {}", e))?;
        let text = String::from_utf8_lossy(&chunk);
        buffer.push_str(&text);

        // Procesar líneas completas del buffer SSE
        while let Some(pos) = buffer.find("\n\n") {
            let line = buffer[..pos].to_string();
            buffer = buffer[pos + 2..].to_string();

            if let Some(data) = line.strip_prefix("data: ") {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                    let _ = app_handle.emit("batch-progress", json);
                }
            }
        }
    }

    // Procesar cualquier dato restante en el buffer
    if !buffer.is_empty() {
        for line in buffer.lines() {
            if let Some(data) = line.strip_prefix("data: ") {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                    let _ = app_handle.emit("batch-progress", json);
                }
            }
        }
    }

    Ok("ok".to_string())
}

