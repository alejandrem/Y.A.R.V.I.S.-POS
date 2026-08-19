use std::path::Path;

/// Lista de archivos .txt en la carpeta (ordenados por nombre).
pub fn obtener_archivos_txt(carpeta: &str) -> Vec<String> {
    let mut archivos: Vec<String> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(carpeta) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let es_txt = path
                .extension()
                .map(|e| e.to_string_lossy().to_ascii_lowercase() == "txt")
                .unwrap_or(false);
            if es_txt {
                archivos.push(path.to_string_lossy().to_string());
            }
        }
    }
    archivos.sort();
    archivos
}

pub(super) fn nombre_de_archivo(ruta: &str) -> String {
    Path::new(ruta)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| ruta.to_string())
}

/// Igual que Python `open(..., errors="ignore")`: bytes inválidos se descartan.
pub(super) fn leer_archivo_tolerante(ruta: &str) -> std::io::Result<String> {
    let bytes = std::fs::read(ruta)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}
