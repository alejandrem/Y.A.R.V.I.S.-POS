// ============================================================
// respaldos::config — Configuración del respaldo programado.
//
// Se guarda en <app_data>/respaldo_config.json. La contraseña del
// respaldo NUNCA se serializa en claro: se protege con DPAPI en Windows
// (atada a la cuenta) o se deja 0600 en Unix. Aun así, para restaurar en
// otra máquina el dueño debe recordar su contraseña: DPAPI no viaja.
// ============================================================

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use tauri::Manager;

use crate::api_config::almacen;

const ARCHIVO: &str = "respaldo_config.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigRespaldo {
    #[serde(default)]
    pub activo: bool,
    #[serde(default)]
    pub destino: Option<String>,
    #[serde(default = "cada_horas_default")]
    pub cada_horas: u32,
    #[serde(default = "retener_default")]
    pub retener: usize,
    #[serde(default)]
    pub ultimo_exito: Option<String>,
    /// Base64 de los bytes protegidos (DPAPI/0600). No es la contraseña.
    #[serde(default)]
    pub passphrase_protegida: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ConfigPublica {
    pub activo: bool,
    pub destino: Option<String>,
    pub cada_horas: u32,
    pub retener: usize,
    pub ultimo_exito: Option<String>,
    pub tiene_passphrase: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigEntrada {
    pub activo: bool,
    pub destino: Option<String>,
    pub cada_horas: u32,
    pub retener: usize,
    /// Contraseña nueva en claro; si viene vacía/None se conserva la actual.
    pub passphrase: Option<String>,
}

impl Default for ConfigRespaldo {
    fn default() -> Self {
        Self {
            activo: false,
            destino: None,
            cada_horas: cada_horas_default(),
            retener: retener_default(),
            ultimo_exito: None,
            passphrase_protegida: None,
        }
    }
}

impl ConfigRespaldo {
    pub fn publica(&self) -> ConfigPublica {
        ConfigPublica {
            activo: self.activo,
            destino: self.destino.clone(),
            cada_horas: self.cada_horas,
            retener: self.retener,
            ultimo_exito: self.ultimo_exito.clone(),
            tiene_passphrase: self.passphrase_protegida.is_some(),
        }
    }

    /// Recupera la contraseña en claro solo en memoria, para el job o una
    /// restauración. Puede fallar si el archivo se copió a otra cuenta.
    pub fn passphrase(&self) -> Result<Option<String>, String> {
        let Some(b64) = &self.passphrase_protegida else {
            return Ok(None);
        };
        let bytes = STANDARD
            .decode(b64)
            .map_err(|_| "La contraseña guardada del respaldo está corrupta".to_string())?;
        let claro = almacen::desproteger_secreto(&bytes)?;
        String::from_utf8(claro)
            .map(Some)
            .map_err(|_| "La contraseña guardada del respaldo no es texto válido".to_string())
    }

    pub fn validar(&self) -> Result<(), String> {
        if !(1..=168).contains(&self.cada_horas) {
            return Err("El intervalo debe estar entre 1 y 168 horas".to_string());
        }
        if !(1..=100).contains(&self.retener) {
            return Err("La retención debe estar entre 1 y 100 respaldos".to_string());
        }
        if self.activo {
            let Some(destino) = self.destino.as_deref() else {
                return Err("Elige una carpeta o USB para el respaldo automático".to_string());
            };
            if destino.trim().is_empty() {
                return Err("La carpeta de respaldo no puede estar vacía".to_string());
            }
            if !Path::new(destino).is_absolute() {
                return Err("La carpeta de respaldo debe ser una ruta absoluta".to_string());
            }
            if self.passphrase_protegida.is_none() {
                return Err(
                    "Configura una contraseña de respaldo antes de activar los respaldos automáticos"
                        .to_string(),
                );
            }
        }
        Ok(())
    }
}

/// Aplica los cambios de la UI sobre la configuración existente.
pub fn aplicar(
    actual: &ConfigRespaldo,
    entrada: ConfigEntrada,
) -> Result<ConfigRespaldo, String> {
    let destino = entrada
        .destino
        .map(|d| d.trim().to_string())
        .filter(|d| !d.is_empty());
    let mut nueva = ConfigRespaldo {
        activo: entrada.activo,
        destino,
        cada_horas: entrada.cada_horas,
        retener: entrada.retener,
        ultimo_exito: actual.ultimo_exito.clone(),
        passphrase_protegida: actual.passphrase_protegida.clone(),
    };

    if let Some(pass) = entrada.passphrase {
        if !pass.trim().is_empty() {
            if pass.trim().chars().count() < crate::respaldos::cifrado::MIN_PASSPHRASE {
                return Err(format!(
                    "La contraseña del respaldo debe tener al menos {} caracteres",
                    crate::respaldos::cifrado::MIN_PASSPHRASE
                ));
            }
            let protegida = almacen::proteger_secreto(pass.as_bytes())?;
            nueva.passphrase_protegida = Some(STANDARD.encode(protegida));
        }
    }

    nueva.validar()?;
    Ok(nueva)
}

pub fn ruta(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("No se pudo resolver el directorio de datos: {e}"))?;
    Ok(dir.join(ARCHIVO))
}

pub fn leer(app: &tauri::AppHandle) -> Result<ConfigRespaldo, String> {
    leer_en(&ruta(app)?)
}

pub fn leer_en(ruta: &Path) -> Result<ConfigRespaldo, String> {
    if !ruta.exists() {
        return Ok(ConfigRespaldo::default());
    }
    let contenido = fs::read_to_string(ruta)
        .map_err(|e| format!("No se pudo leer {}: {e}", ruta.display()))?;
    serde_json::from_str::<ConfigRespaldo>(&contenido)
        .map_err(|e| format!("La configuración de respaldos está corrupta: {e}"))
}

pub fn guardar(app: &tauri::AppHandle, cfg: &ConfigRespaldo) -> Result<(), String> {
    guardar_en(&ruta(app)?, cfg)
}

pub fn guardar_en(ruta: &Path, cfg: &ConfigRespaldo) -> Result<(), String> {
    cfg.validar()?;
    let dir = ruta
        .parent()
        .ok_or_else(|| "Ruta de configuración sin carpeta padre".to_string())?;
    fs::create_dir_all(dir).map_err(|e| format!("No se pudo crear {}: {e}", dir.display()))?;
    let json = serde_json::to_vec_pretty(cfg).map_err(|e| e.to_string())?;
    escribir_atomico(ruta, &json)
}

fn escribir_atomico(ruta: &Path, bytes: &[u8]) -> Result<(), String> {
    let dir = ruta.parent().ok_or("Ruta sin padre")?;
    let temporal = dir.join(format!(
        ".{ARCHIVO}.tmp-{}-{}",
        std::process::id(),
        chrono::Local::now().format("%Y%m%d%H%M%S%f")
    ));

    let resultado = (|| -> Result<(), String> {
        let mut archivo = fs::File::create(&temporal)
            .map_err(|e| format!("No se pudo crear {}: {e}", temporal.display()))?;
        archivo.write_all(bytes).map_err(|e| e.to_string())?;
        archivo.sync_all().map_err(|e| e.to_string())?;
        drop(archivo);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&temporal, fs::Permissions::from_mode(0o600))
                .map_err(|e| format!("No se pudieron aplicar permisos 0600: {e}"))?;
        }

        fs::rename(&temporal, ruta).map_err(|e| e.to_string())
    })();

    if resultado.is_err() {
        let _ = fs::remove_file(&temporal);
    }
    resultado
}

fn cada_horas_default() -> u32 {
    24
}

fn retener_default() -> usize {
    30
}

#[cfg(test)]
mod tests;
