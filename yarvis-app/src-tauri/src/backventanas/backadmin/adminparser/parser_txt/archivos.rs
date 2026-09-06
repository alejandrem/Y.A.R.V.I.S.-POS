// ============================================================
// archivos — Exploración de carpetas de tickets (.txt).
// ============================================================

use super::super::utils::sanitize_path;
use crate::backventanas::auth::AuthState;
use std::fs;
use std::path;

#[derive(serde::Serialize)]
pub struct ArchivoCarpeta {
    pub nombre: String,
    pub ruta: String,
    pub tamano: u64,
    pub preview: String,
}

#[tauri::command]
pub fn listar_archivos_carpeta(
    auth: tauri::State<'_, AuthState>,
    carpeta: String,
) -> Result<Vec<ArchivoCarpeta>, String> {
    auth.require_admin()?;
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

        let ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if ext != "txt" {
            continue;
        }

        let nombre = file_path
            .file_name()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();

        let ruta = file_path.to_string_lossy().to_string();

        let tamano = fs::metadata(&file_path).map(|m| m.len()).unwrap_or(0);

        // Leer primeras 5 lineas para preview
        let preview = fs::read_to_string(&file_path)
            .map(|content| content.lines().take(5).collect::<Vec<&str>>().join("\n"))
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
pub fn leer_archivo_raw(auth: tauri::State<'_, AuthState>, path: String) -> Result<String, String> {
    auth.require_admin()?;
    let safe_path = sanitize_path(&path)?;
    fs::read_to_string(safe_path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn leer_archivo_bytes(
    auth: tauri::State<'_, AuthState>,
    path: String,
) -> Result<Vec<u8>, String> {
    auth.require_admin()?;
    let safe_path = sanitize_path(&path)?;
    fs::read(safe_path).map_err(|e| e.to_string())
}
