// ============================================================
// comandos — Comandos Tauri del semaforo rojo (issue #12).
//
// Separado de `rojo.rs` por la regla de ~400 lineas por archivo.
// Roles: registrar/listar/contar = operario (caja: admin o empleado);
// resolver = solo admin (escribe inventario).
// ============================================================

use super::rojo::{
    contar_pendientes_impl, listar_pendientes_impl, registrar_pendiente_impl,
    resolver_asignando_impl, resolver_con_alta_impl, ConteosPendientes, Pendiente,
};
use crate::backventanas::auth::AuthState;
use sqlx::SqlitePool;

#[tauri::command]
pub async fn rojo_registrar_pendiente(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    ean: Option<String>,
    nombre_crudo: String,
) -> Result<i64, String> {
    auth.require_operator()?;
    registrar_pendiente_impl(&*state, ean.as_deref(), &nombre_crudo).await
}

#[tauri::command]
pub async fn rojo_listar_pendientes(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    estado: Option<String>,
    limite: Option<i64>,
) -> Result<Vec<Pendiente>, String> {
    auth.require_operator()?;
    listar_pendientes_impl(&*state, estado.as_deref(), limite.unwrap_or(100)).await
}

#[tauri::command]
pub async fn rojo_contar_pendientes(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
) -> Result<ConteosPendientes, String> {
    auth.require_operator()?;
    contar_pendientes_impl(&*state).await
}

#[tauri::command]
pub async fn rojo_resolver_con_alta(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    pendiente_id: i64,
    nombre: String,
    codigo_barras: Option<String>,
    categoria: Option<String>,
    marca: Option<String>,
    cantidad: Option<f64>,
    unidad: Option<String>,
) -> Result<i64, String> {
    let ses = auth.require_admin()?;
    resolver_con_alta_impl(
        &*state,
        pendiente_id,
        &nombre,
        codigo_barras.as_deref(),
        categoria.as_deref(),
        marca.as_deref(),
        cantidad,
        unidad.as_deref(),
        Some(ses.user_id),
    )
    .await
}

#[tauri::command]
pub async fn rojo_resolver_asignando(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    pendiente_id: i64,
    producto_id: i64,
    ean_override: Option<String>,
) -> Result<(), String> {
    let ses = auth.require_admin()?;
    resolver_asignando_impl(
        &*state,
        pendiente_id,
        producto_id,
        ean_override.as_deref(),
        Some(ses.user_id),
    )
    .await
}
