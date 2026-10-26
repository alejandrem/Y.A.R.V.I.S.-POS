// ============================================================
// respaldos::archivo — Crear, listar y rotar respaldos cifrados.
//
// Un respaldo es una foto consistente de SQLite (VACUUM INTO) cifrada
// con la contraseña del dueño. El .db temporal en claro se crea en la
// carpeta temporal de la app y se borra en el mismo proceso; el archivo
// que llega al USB es siempre .yarvisbak.
// ============================================================

use std::fs;
use std::path::{Path, PathBuf};

use chrono::Local;
use serde::Serialize;
use sqlx::SqlitePool;

use crate::respaldos::cifrado;

pub const EXTENSION: &str = "yarvisbak";
const PREFIJO: &str = "yarvis-";
const PREFIJO_TEMP: &str = ".yarvis-";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct InfoRespaldo {
    pub archivo: String,
    pub nombre: String,
    pub bytes: u64,
    pub fecha: String,
}

/// Crea un respaldo cifrado del pool abierto y aplica retención.
/// `tmp_dir` debe ser local (no el USB): ahí vive el .db en claro unos
/// milisegundos antes de cifrarse.
pub async fn crear_desde_pool(
    pool: &SqlitePool,
    destino: &Path,
    retener: usize,
    passphrase: &str,
    tmp_dir: &Path,
) -> Result<InfoRespaldo, String> {
    fs::create_dir_all(destino)
        .map_err(|e| format!("No se pudo crear la carpeta de respaldo {}: {e}", destino.display()))?;
    fs::create_dir_all(tmp_dir)
        .map_err(|e| format!("No se pudo crear la carpeta temporal {}: {e}", tmp_dir.display()))?;
    limpiar_temporales(tmp_dir);

    let sello = Local::now().format("%Y%m%d%H%M%S%f").to_string();
    let temp_db = tmp_dir.join(format!("{PREFIJO_TEMP}{}-{sello}.db", std::process::id()));
    let _limpiar = LimpiarArchivo(temp_db.clone());

    let dest_sql = temp_db.to_string_lossy().replace('\'', "''");
    sqlx::query(&format!("VACUUM INTO '{dest_sql}'"))
        .execute(pool)
        .await
        .map_err(|e| format!("No se pudo generar la foto de la base: {e}"))?;
    if !temp_db.exists() {
        return Err("VACUUM INTO no generó el archivo temporal".to_string());
    }

    let plano = fs::read(&temp_db)
        .map_err(|e| format!("No se pudo leer la foto temporal: {e}"))?;
    let cifrado = cifrado::cifrar(&plano, passphrase)?;

    let final_ = ruta_unica(destino);
    guardar_atomico(&final_, &cifrado)?;
    rotar(destino, retener);

    info_de(&final_)
}

/// Lista los respaldos YARVIS de una carpeta. Si no existe, devuelve
/// lista vacía (el USB puede estar desconectado). Nunca inventa.
pub fn listar(destino: &Path) -> Vec<InfoRespaldo> {
    let mut out: Vec<InfoRespaldo> = fs::read_dir(destino)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| es_respaldo(p))
                .filter_map(|p| info_de(&p).ok())
                .collect()
        })
        .unwrap_or_default();
    out.sort_by(|a, b| b.nombre.cmp(&a.nombre));
    out
}

/// Conserva los `retener` más recientes. Solo borra archivos con el
/// nombre/extension exactos de YARVIS; nunca toca archivos ajenos.
pub fn rotar(destino: &Path, retener: usize) {
    let mut propios: Vec<PathBuf> = fs::read_dir(destino)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| es_respaldo(p))
                .collect()
        })
        .unwrap_or_default();
    propios.sort();
    let retener = retener.max(1);
    if propios.len() > retener {
        for viejo in propios.iter().take(propios.len() - retener) {
            let _ = fs::remove_file(viejo);
        }
    }
}

fn es_respaldo(ruta: &Path) -> bool {
    let Some(nombre) = ruta.file_name().map(|n| n.to_string_lossy().to_string()) else {
        return false;
    };
    ruta.extension().is_some_and(|ext| ext == EXTENSION)
        && nombre.starts_with(PREFIJO)
        && !nombre.starts_with(PREFIJO_TEMP)
}

fn ruta_unica(destino: &Path) -> PathBuf {
    let sello = Local::now().format("%Y%m%d-%H%M%S").to_string();
    let mut ruta = destino.join(format!("{PREFIJO}{sello}.{EXTENSION}"));
    let mut n = 1;
    while ruta.exists() {
        ruta = destino.join(format!("{PREFIJO}{sello}-{n}.{EXTENSION}"));
        n += 1;
    }
    ruta
}

fn info_de(ruta: &Path) -> Result<InfoRespaldo, String> {
    let meta = fs::metadata(ruta).map_err(|e| e.to_string())?;
    let fecha = meta
        .modified()
        .ok()
        .map(|t| chrono::DateTime::<Local>::from(t).format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_default();
    Ok(InfoRespaldo {
        archivo: ruta.to_string_lossy().to_string(),
        nombre: ruta
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default(),
        bytes: meta.len(),
        fecha,
    })
}

fn guardar_atomico(destino: &Path, bytes: &[u8]) -> Result<(), String> {
    let temporal = destino.with_file_name(format!(
        ".{}.tmp-{}",
        destino
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "respaldo".to_string()),
        std::process::id()
    ));
    let resultado = (|| -> Result<(), String> {
        fs::write(&temporal, bytes)
            .map_err(|e| format!("No se pudo escribir {}: {e}", temporal.display()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&temporal, fs::Permissions::from_mode(0o600))
                .map_err(|e| format!("No se pudieron aplicar permisos 0600: {e}"))?;
        }
        fs::rename(&temporal, destino)
            .map_err(|e| format!("No se pudo instalar {}: {e}", destino.display()))
    })();
    if resultado.is_err() {
        let _ = fs::remove_file(&temporal);
    }
    resultado
}

/// Borra .db temporales que hayan quedado de un cierre abrupto.
pub fn limpiar_temporales(tmp_dir: &Path) -> usize {
    let mut borrados = 0;
    if let Ok(rd) = fs::read_dir(tmp_dir) {
        for entrada in rd.filter_map(|e| e.ok()) {
            let p = entrada.path();
            let nombre = p.file_name().map(|n| n.to_string_lossy().to_string());
            if let Some(nombre) = nombre {
                if nombre.starts_with(PREFIJO_TEMP) && p.extension().is_some_and(|e| e == "db") {
                    if fs::remove_file(&p).is_ok() {
                        borrados += 1;
                    }
                }
            }
        }
    }
    borrados
}

struct LimpiarArchivo(PathBuf);

impl Drop for LimpiarArchivo {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

#[cfg(test)]
mod tests;
