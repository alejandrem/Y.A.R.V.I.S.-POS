// ============================================================
// respaldos::comandos — API Tauri del wizard de respaldos.
//
// Todos los comandos exigen sesión de ADMIN. La restauración crea una
// copia de seguridad de la DB actual ANTES de cerrar el pool; si algo
// falla, reinicia la app para que vuelva a abrir la base original.
// ============================================================

use std::path::{Path, PathBuf};

use chrono::Local;
use sqlx::SqlitePool;
use tauri::{Manager, State};

use crate::backventanas::auth::AuthState;
use crate::backventanas::db::{respaldo, DbPath};
use crate::respaldos::archivo::{self, InfoRespaldo};
use crate::respaldos::config::{self, ConfigEntrada, ConfigPublica, ConfigRespaldo};
use crate::respaldos::verificar::{self, PruebaRespaldo};
use crate::respaldos::CANDADO;

#[tauri::command]
pub fn respaldo_config_leer(
    app: tauri::AppHandle,
    auth: State<'_, AuthState>,
) -> Result<ConfigPublica, String> {
    auth.require_admin()?;
    Ok(config::leer(&app)?.publica())
}

#[tauri::command]
pub fn respaldo_config_guardar(
    app: tauri::AppHandle,
    auth: State<'_, AuthState>,
    entrada: ConfigEntrada,
) -> Result<ConfigPublica, String> {
    auth.require_admin()?;
    let actual = config::leer(&app)?;
    let nueva = config::aplicar(&actual, entrada)?;
    if nueva.activo {
        if let Some(destino) = nueva.destino.as_deref() {
            validar_destino(&app, destino)?;
        }
    }
    config::guardar(&app, &nueva)?;
    Ok(nueva.publica())
}

#[tauri::command]
pub async fn respaldo_crear_ahora(
    app: tauri::AppHandle,
    auth: State<'_, AuthState>,
    pool: State<'_, SqlitePool>,
    passphrase: Option<String>,
) -> Result<InfoRespaldo, String> {
    auth.require_admin()?;
    let mut cfg = config::leer(&app)?;
    let destino = cfg
        .destino
        .clone()
        .ok_or_else(|| "Configura primero una carpeta de respaldo".to_string())?;
    validar_destino(&app, &destino)?;
    let pass = resolver_pass(&cfg, passphrase)?;
    let tmp = tmp_dir(&app)?;
    let pool = pool.inner().clone();

    let _guard = CANDADO.lock().await;
    let info = archivo::crear_desde_pool(
        &pool,
        Path::new(&destino),
        cfg.retener,
        &pass,
        &tmp,
    )
    .await?;

    cfg.ultimo_exito = Some(Local::now().to_rfc3339());
    if let Err(e) = config::guardar(&app, &cfg) {
        tracing::warn!("[RESPALDO] creado {} pero no se guardó el estado: {e}", info.archivo);
    }
    Ok(info)
}

#[tauri::command]
pub fn respaldo_listar(
    app: tauri::AppHandle,
    auth: State<'_, AuthState>,
) -> Result<Vec<InfoRespaldo>, String> {
    auth.require_admin()?;
    let cfg = config::leer(&app)?;
    Ok(cfg
        .destino
        .as_deref()
        .map(|d| archivo::listar(Path::new(d)))
        .unwrap_or_default())
}

#[tauri::command]
pub fn respaldo_probar(
    app: tauri::AppHandle,
    auth: State<'_, AuthState>,
    archivo: String,
    passphrase: Option<String>,
) -> Result<PruebaRespaldo, String> {
    auth.require_admin()?;
    let cfg = config::leer(&app)?;
    let pass = resolver_pass(&cfg, passphrase)?;
    let ruta = validar_archivo(&archivo)?;
    verificar::probar(&ruta, &pass, &tmp_dir(&app)?)
}

#[tauri::command]
pub async fn respaldo_restaurar(
    app: tauri::AppHandle,
    auth: State<'_, AuthState>,
    pool: State<'_, SqlitePool>,
    archivo: String,
    passphrase: Option<String>,
) -> Result<(), String> {
    auth.require_admin()?;
    let cfg = config::leer(&app)?;
    let pass = resolver_pass(&cfg, passphrase)?;
    let ruta = validar_archivo(&archivo)?;
    let tmp = tmp_dir(&app)?;
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("No se pudo resolver app_data_dir: {e}"))?;
    let db_path = PathBuf::from(&app.state::<DbPath>().inner().0);

    // 1. Descifrar y validar ANTES de cerrar el pool.
    let temp = verificar::descifrar_a_temp(&ruta, &pass, &tmp)?;
    let prueba = verificar::probar_plana(temp.ruta())?;
    if !prueba.ok {
        return Err("El respaldo no pasó la prueba; no se restauró nada".to_string());
    }

    let pool = pool.inner().clone();
    let _guard = CANDADO.lock().await;

    // 2. Copia de seguridad de la DB ACTUAL con VACUUM (consistente con WAL).
    let seguridad = respaldo::backup_local_desde_pool(&pool, &app_dir)
        .await
        .map_err(|e| {
            format!("Restauración cancelada: no se pudo respaldar la base actual antes de tocarla. {e}")
        })?;
    tracing::info!("[RESPALDO] copia previa a restaurar: {}", seguridad.display());

    // 3. Cerrar el pool para que Windows permita reemplazar el archivo.
    pool.close().await;

    // 4. Instalar. Si falla, se reinicia con la base original (rollback).
    if let Err(e) = respaldo::instalar_db(temp.ruta(), &db_path) {
        tracing::error!("[RESPALDO] falló la restauración: {e}. Se reinicia con la base actual.");
        app.restart();
    }

    tracing::info!("[RESPALDO] restauración completada; reiniciando la app");
    app.restart()
}

fn resolver_pass(cfg: &ConfigRespaldo, passphrase: Option<String>) -> Result<String, String> {
    if let Some(pass) = passphrase.filter(|p| !p.trim().is_empty()) {
        return Ok(pass);
    }
    cfg.passphrase()?.ok_or_else(|| {
        "Escribe la contraseña del respaldo (o guárdala en la configuración)".to_string()
    })
}

fn validar_archivo(archivo: &str) -> Result<PathBuf, String> {
    let ruta = PathBuf::from(archivo);
    if !ruta.is_file() {
        return Err(format!("No existe el respaldo {}", ruta.display()));
    }
    if ruta.extension().is_none_or(|ext| ext != archivo::EXTENSION) {
        return Err("Selecciona un archivo de respaldo .yarvisbak".to_string());
    }
    Ok(ruta)
}

/// Un respaldo dentro de app_data es la misma carpeta oculta (y casi
/// siempre el mismo disco) que la base: si el disco muere, mueren los
/// dos. Se rechaza para no vender una falsa red de seguridad.
fn validar_destino(app: &tauri::AppHandle, destino: &str) -> Result<(), String> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("No se pudo resolver el directorio de datos: {e}"))?;
    if es_dentro_de(&base, Path::new(destino)) {
        return Err(
            "Elige una carpeta o USB FUERA de la carpeta de datos de YARVIS. \
             Un respaldo dentro de AppData muere con el mismo disco que la base."
                .to_string(),
        );
    }
    Ok(())
}

fn es_dentro_de(base: &Path, destino: &Path) -> bool {
    let base = base.canonicalize().unwrap_or_else(|_| base.to_path_buf());
    let destino = destino
        .canonicalize()
        .unwrap_or_else(|_| destino.to_path_buf());

    #[cfg(windows)]
    {
        let base = base.to_string_lossy().to_lowercase();
        let destino = destino.to_string_lossy().to_lowercase();
        let con_barra = format!("{base}\\");
        let con_slash = format!("{base}/");
        destino == base || destino.starts_with(&con_barra) || destino.starts_with(&con_slash)
    }
    #[cfg(not(windows))]
    {
        destino == base || destino.starts_with(&base)
    }
}

fn tmp_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("No se pudo resolver app_data_dir: {e}"))?
        .join("tmp");
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("No se pudo crear la carpeta temporal: {e}"))?;
    Ok(dir)
}

#[cfg(test)]
mod tests;
