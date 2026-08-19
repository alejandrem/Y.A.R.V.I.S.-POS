// ============================================================
// Encabezado del ticket: metadatos que no son líneas de producto
// ============================================================

/// Extrae el cajero/empleado que atendió (primeras 10 líneas del ticket).
pub fn extraer_cajero(texto: &str) -> String {
    for linea in texto.lines().take(10) {
        let lower = linea.to_lowercase();
        if lower.contains("cajero") || lower.contains("empleado") || lower.contains("vendedor") {
            if let Some(idx) = linea.find(':') {
                return linea[idx + 1..].trim().to_string();
            }
        }
    }
    "SISTEMA".to_string()
}
