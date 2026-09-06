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

/// Ordena archivos por la fecha mínima de sus tickets (pre-pass barato:
/// segmentar es solo regex en memoria, no toca la DB). Los archivos sin
/// fecha detectable van al final; en empates manda el nombre.
///
/// Así los IDs de venta crecen en el orden en que se generaron los
/// tickets aunque los archivos vengan desordenados (exportaciones,
/// copias, descargas). El costo es una lectura extra por archivo
/// (los .txt son de KB y el SO los cachea).
pub fn ordenar_archivos_cronologicamente(archivos: Vec<String>) -> Vec<String> {
    let mut con_fecha: Vec<(Option<String>, String)> = archivos
        .into_iter()
        .map(|a| {
            let min = std::fs::read(&a).ok().and_then(|bytes| {
                let texto = String::from_utf8_lossy(&bytes);
                crate::cerebro::analizador_tickets::segmentar(&texto)
                    .into_iter()
                    .filter_map(|s| s.fecha_hora)
                    .min()
            });
            (min, a)
        })
        .collect();
    con_fecha.sort_by(|(fa, na), (fb, nb)| match (fa, fb) {
        (Some(x), Some(y)) => x.cmp(y).then(na.cmp(nb)),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => na.cmp(nb),
    });
    con_fecha.into_iter().map(|(_, a)| a).collect()
}
