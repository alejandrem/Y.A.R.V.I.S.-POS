// ============================================================
// comandos — Comandos Tauri del semaforo amarillo (issue #11).
//
// Roles: sugerir / rechazar / ninguno / stats = operario (caja);
// confirmar = solo admin (escribe inventario + vinculo).
// ============================================================

use super::confirmar::confirmar_impl;
use super::rechazos::{ninguno_impl, rechazar_impl, stats_rechazos_impl};
use super::sugerencias::sugerir_impl;
use super::tipos::{StatsRechazos, SugerenciaTop};
use crate::backventanas::auth::AuthState;
use sqlx::SqlitePool;

#[tauri::command]
pub async fn amarillo_sugerir(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    nombre_crudo: String,
    marca: Option<String>,
    ean: Option<String>,
) -> Result<SugerenciaTop, String> {
    auth.require_operator()?;
    sugerir_impl(&*state, &nombre_crudo, marca.as_deref(), ean.as_deref()).await
}

#[tauri::command]
pub async fn amarillo_confirmar(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    pendiente_id: i64,
    producto_id: i64,
    ean_override: Option<String>,
) -> Result<(), String> {
    let ses = auth.require_admin()?;
    confirmar_impl(&*state, pendiente_id, producto_id, ean_override.as_deref(), Some(ses.user_id)).await
}

#[tauri::command]
pub async fn amarillo_rechazar(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    pendiente_id: i64,
    producto_id: i64,
    score: f64,
) -> Result<(), String> {
    auth.require_operator()?;
    rechazar_impl(&*state, pendiente_id, producto_id, score).await
}

#[tauri::command]
pub async fn amarillo_ninguno(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    pendiente_id: i64,
) -> Result<(), String> {
    auth.require_operator()?;
    ninguno_impl(&*state, pendiente_id).await
}

#[tauri::command]
pub async fn amarillo_stats_rechazos(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
) -> Result<StatsRechazos, String> {
    auth.require_operator()?;
    stats_rechazos_impl(&*state).await
}
