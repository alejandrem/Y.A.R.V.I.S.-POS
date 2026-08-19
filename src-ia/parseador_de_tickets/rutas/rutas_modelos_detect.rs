// ============================================================
// rutas_modelos_detect — Exploración del filesystem de LM
// Studio: home, búsqueda de .gguf y resolución con fallback.
// Porción de rutas_modelos.rs.
// ============================================================

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use super::rutas_modelos_config::{InfoModelo, MODELOS_CONFIG, PREFERENCIA_QUANT};

/// Detecta el home del usuario (Linux/macOS: HOME; Windows: USERPROFILE).
pub(crate) fn obtener_dir_lmstudio() -> PathBuf {
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());

    Path::new(&home).join(".lmstudio").join("models")
}

/// Devuelve la primera ruta `.gguf` (no `mmproj`) del directorio del modelo.
pub(crate) fn buscar_gguf(base: &Path, rel_path: &str) -> Option<PathBuf> {
    let folder = base.join(rel_path);

    if !folder.is_dir() {
        return None;
    }

    let mut archivos_gguf: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = fs::read_dir(&folder) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let es_gguf = path
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase() == "gguf")
                .unwrap_or(false);
            let es_mmproj = path
                .file_name()
                .map(|f| f.to_string_lossy().to_lowercase().contains("mmproj"))
                .unwrap_or(false);
            if es_gguf && !es_mmproj {
                archivos_gguf.push(path);
            }
        }
    }

    if archivos_gguf.is_empty() {
        return None;
    }

    for preferida in PREFERENCIA_QUANT {
        for archivo in &archivos_gguf {
            let nombre = archivo.file_name().map(|f| f.to_string_lossy().to_lowercase());
            if let Some(nombre) = nombre {
                if nombre.contains(preferida) {
                    return Some(archivo.clone());
                }
            }
        }
    }

    archivos_gguf.into_iter().next()
}

/// Busca en los directorios candidatos; fallback a la ruta predecible
/// `modelo_no_encontrado.gguf` (mismo comportamiento que Python).
pub(crate) fn resolver(candidatos: &[&str], base: &Path) -> PathBuf {
    for rel in candidatos {
        if let Some(ruta) = buscar_gguf(base, rel) {
            return ruta;
        }
    }

    base.join(candidatos[0]).join("modelo_no_encontrado.gguf")
}

pub(crate) fn verificar_en(base: &Path) -> Vec<(&'static str, InfoModelo)> {
    let mut resultado = Vec::new();

    for &(key, candidatos) in MODELOS_CONFIG {
        let ruta = resolver(candidatos, base);
        let existe = ruta.exists();
        let mut tamano_mb = 0.0;

        if existe {
            if let Ok(metadata) = fs::metadata(&ruta) {
                tamano_mb = (metadata.len() as f64 / (1024.0 * 1024.0) * 10.0).round() / 10.0;
            }
        }

        resultado.push((key, InfoModelo { ruta, existe, tamano_mb }));
    }

    resultado
}
