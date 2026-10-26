// ============================================================
// logs.rs — Log a archivo para diagnóstico en la tienda.
//
// Antes solo se escribía a stdout: en el .exe de Windows no hay consola,
// así que un fallo de arranque no dejaba rastro visible. Aquí se escribe
// en <app_data>/logs/yarvis.log y se conserva la ruta para el diálogo
// nativo de error.
//
// Rotación simple sin dependencias extra: si el log supera 10 MiB se
// renombra con sello de tiempo y se conservan los 5 más recientes.
// Nunca debe tumbar el arranque: si algo falla, se sigue sin archivo.
// ============================================================

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

use tauri::Manager;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

const ARCHIVO: &str = "yarvis.log";
const PREFIJO_ROTADO: &str = "yarvis-";
const MAX_BYTES: u64 = 10 * 1024 * 1024;
const MAX_ROTADOS: usize = 5;

static RUTA: OnceLock<PathBuf> = OnceLock::new();

/// Ruta del log activo, disponible aunque el logger no haya podido abrir
/// el archivo (el diálogo de error siempre debe poder mostrar una ruta).
pub fn ruta_actual() -> Option<PathBuf> {
    RUTA.get().cloned()
}

/// Prepara el log en archivo + consola. Devuelve la ruta del archivo.
/// Es idempotente: si el subscriber ya estaba inicializado, conserva la
/// ruta y no falla.
pub fn iniciar(app: &tauri::AppHandle) -> PathBuf {
    let ruta = ruta_log(app);
    if let Some(dir) = ruta.parent() {
        let _ = fs::create_dir_all(dir);
        let _ = rotar_si_grande(dir, MAX_BYTES);
    }

    if let Ok(archivo) = OpenOptions::new().create(true).append(true).open(&ruta) {
        let escritor = EscritorCompartido(Arc::new(Mutex::new(archivo)));
        // Si ya hay un subscriber global (tests, otra inicialización),
        // try_init devuelve Err y seguimos con la ruta calculada.
        let _ = tracing_subscriber::registry()
            .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
            .with(fmt::layer().with_ansi(false).with_writer(escritor))
            .with(fmt::layer().with_ansi(true).with_writer(io::stderr))
            .try_init();
    }

    let _ = RUTA.set(ruta.clone());
    ruta
}

fn ruta_log(app: &tauri::AppHandle) -> PathBuf {
    let base = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::temp_dir().join("yarvis-sin-appdata"));
    base.join("logs").join(ARCHIVO)
}

/// Renombra `yarvis.log` si supera `max_bytes` y limpia los rotados
/// sobrantes. Los errores se ignoran a propósito: el log nunca bloquea.
fn rotar_si_grande(dir: &Path, max_bytes: u64) -> Result<(), String> {
    let activo = dir.join(ARCHIVO);
    let grande = fs::metadata(&activo)
        .map(|m| m.len() > max_bytes)
        .unwrap_or(false);
    if grande {
        let sello = chrono::Local::now().format("%Y%m%d-%H%M%S");
        let mut destino = dir.join(format!("{PREFIJO_ROTADO}{sello}.log"));
        let mut n = 1;
        while destino.exists() {
            destino = dir.join(format!("{PREFIJO_ROTADO}{sello}-{n}.log"));
            n += 1;
        }
        fs::rename(&activo, &destino).map_err(|e| e.to_string())?;
    }
    limpiar_rotados(dir);
    Ok(())
}

fn limpiar_rotados(dir: &Path) {
    let mut rotados: Vec<PathBuf> = fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| {
                    p.extension().is_some_and(|ext| ext == "log")
                        && p.file_name()
                            .is_some_and(|n| n.to_string_lossy().starts_with(PREFIJO_ROTADO))
                })
                .collect()
        })
        .unwrap_or_default();
    rotados.sort();
    if rotados.len() > MAX_ROTADOS {
        for viejo in rotados.iter().take(rotados.len() - MAX_ROTADOS) {
            let _ = fs::remove_file(viejo);
        }
    }
}

/// Writer compartido entre capas de tracing. El Mutex evita entrelazar
/// líneas si dos hilos escriben a la vez; el volumen es bajo (info).
#[derive(Clone)]
struct EscritorCompartido(Arc<Mutex<File>>);

impl<'a> fmt::MakeWriter<'a> for EscritorCompartido {
    type Writer = Self;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

impl Write for EscritorCompartido {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut archivo = self
            .0
            .lock()
            .map_err(|_| io::Error::other("lock de log envenenado"))?;
        archivo.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut archivo = self
            .0
            .lock()
            .map_err(|_| io::Error::other("lock de log envenenado"))?;
        archivo.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir_temp(nombre: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!(
            "yarvis-logs-{nombre}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(&base).unwrap();
        base
    }

    #[test]
    fn rota_un_log_grande_y_conserva_solo_cinco() {
        let dir = dir_temp("rotar");
        fs::write(dir.join(ARCHIVO), vec![b'x'; 1024]).unwrap();
        for dia in 1..=7 {
            fs::write(
                dir.join(format!("{PREFIJO_ROTADO}2026010{dia}-120000.log")),
                b"viejo",
            )
            .unwrap();
        }

        rotar_si_grande(&dir, 512).unwrap();

        assert!(!dir.join(ARCHIVO).exists(), "el activo debe rotarse");
        let rotados: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with(PREFIJO_ROTADO))
            .collect();
        assert_eq!(rotados.len(), MAX_ROTADOS);
        // Los dos más viejos (20260101 y 20260102) deben haber salido.
        assert!(!dir.join("yarvis-20260101-120000.log").exists());
        assert!(!dir.join("yarvis-20260102-120000.log").exists());
        assert!(dir.join("yarvis-20260107-120000.log").exists());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn un_log_pequeno_no_se_rota() {
        let dir = dir_temp("pequeno");
        fs::write(dir.join(ARCHIVO), b"info").unwrap();

        rotar_si_grande(&dir, 512).unwrap();

        assert!(dir.join(ARCHIVO).exists());
        assert_eq!(fs::read_to_string(dir.join(ARCHIVO)).unwrap(), "info");
        let _ = fs::remove_dir_all(&dir);
    }
}
