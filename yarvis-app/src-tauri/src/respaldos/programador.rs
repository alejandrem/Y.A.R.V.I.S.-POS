// ============================================================
// respaldos::programador — Respaldo automático cada N horas.
//
// Revisa cada 30 minutos si toca respaldar según la configuración.
// Si el USB no está conectado, el error se registra y la caja sigue
// cobrando: un respaldo jamás debe bloquear una venta.
// ============================================================

use std::path::Path;
use std::time::Duration;

use chrono::{DateTime, Local};
use sqlx::SqlitePool;
use tauri::Manager;

use crate::respaldos::archivo::{self, InfoRespaldo};
use crate::respaldos::config::{self, ConfigRespaldo};
use crate::respaldos::CANDADO;

/// Arranca el job de respaldo en segundo plano. No bloquea el setup.
pub fn iniciar(app: tauri::AppHandle, pool: SqlitePool) {
    tauri::async_runtime::spawn(async move {
        // El primer tick es inmediato: si nunca se ha respaldado y el
        // dueño ya configuró todo, se crea la primera copia al arrancar.
        let mut tick = tokio::time::interval(Duration::from_secs(30 * 60));
        loop {
            tick.tick().await;
            match ejecutar_si_toca(&app, &pool).await {
                Ok(Some(info)) => {
                    tracing::info!("[RESPALDO] creado automáticamente: {}", info.archivo)
                }
                Ok(None) => {}
                Err(e) => tracing::warn!("[RESPALDO] automático omitido: {e}"),
            }
        }
    });
}

/// Un solo ciclo del job, expuesto para que los tests no esperen 30 min.
pub async fn ejecutar_si_toca(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
) -> Result<Option<InfoRespaldo>, String> {
    let cfg = config::leer(app)?;
    if !toca(&cfg, Local::now()) {
        return Ok(None);
    }

    let destino = cfg
        .destino
        .clone()
        .ok_or_else(|| "Respaldo automático sin carpeta configurada".to_string())?;
    let passphrase = cfg
        .passphrase()?
        .ok_or_else(|| "Respaldo automático sin contraseña configurada".to_string())?;
    let tmp_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("No se pudo resolver app_data_dir: {e}"))?
        .join("tmp");

    // Evita que el job y el botón "Crear ahora" escriban a la vez.
    let _guard = CANDADO.lock().await;
    let info = archivo::crear_desde_pool(
        pool,
        Path::new(&destino),
        cfg.retener,
        &passphrase,
        &tmp_dir,
    )
    .await?;

    let mut actualizada = cfg.clone();
    actualizada.ultimo_exito = Some(Local::now().to_rfc3339());
    if let Err(e) = config::guardar(app, &actualizada) {
        tracing::warn!("[RESPALDO] se creó {} pero no se pudo guardar el estado: {e}", info.archivo);
    }
    Ok(Some(info))
}

/// ¿Toca respaldar ahora? Es una función pura para poder romperla en tests.
pub fn toca(cfg: &ConfigRespaldo, ahora: DateTime<Local>) -> bool {
    if !cfg.activo || cfg.destino.is_none() {
        return false;
    }
    let Some(ultimo) = cfg.ultimo_exito.as_deref() else {
        return true;
    };
    match DateTime::parse_from_rfc3339(ultimo) {
        Ok(fecha) => {
            let horas = ahora
                .signed_duration_since(fecha.with_timezone(&Local))
                .num_hours();
            // Reloj atrasado (horas negativas): no entrar en bucle de
            // respaldos. Se espera a que el reloj vuelva a avanzar.
            horas >= i64::from(cfg.cada_horas)
        }
        Err(_) => true,
    }
}

#[cfg(test)]
mod tests;
