// Utilidades compartidas para parsers de archivos.
use std::path;

/// Sanitiza una ruta de archivo para prevenir path traversal.
/// Canonicaliza, bloquea rutas del sistema, y valida la extensión.
pub fn sanitize_path(path: &str) -> Result<String, String> {
    let canonical = path::Path::new(path)
        .canonicalize()
        .map_err(|e| format!("Ruta inválida: {}", e))?;

    let path_str = canonical.to_string_lossy().to_string();

    let blocked = [
        "/etc", "/proc", "/sys", "/dev", "/root",
        "C:\\Windows", "C:\\Program Files", "C:\\ProgramData",
    ];

    let path_lower = path_str.to_lowercase();
    for b in &blocked {
        if path_lower.starts_with(&b.to_lowercase()) {
            return Err(format!("Acceso denegado a ruta del sistema: {}", b));
        }
    }

    let allowed_ext = ["txt", "csv", "tsv", "xlsx", "xls"];
    if let Some(ext) = canonical.extension().and_then(|e| e.to_str()) {
        if !allowed_ext.contains(&ext.to_lowercase().as_str()) {
            return Err(format!("Extensión no permitida: .{}", ext));
        }
    } else {
        return Err("El archivo no tiene extensión".to_string());
    }

    Ok(path_str)
}
