// ============================================================
// creacion — Apertura de cortes X y Z.
// ============================================================
//
// El tipo de corte lo define el payload (`tipo_corte`); los comandos
// `crear_corte_x`/`crear_corte_z` lo fijan para que cada botón de la UI
// cree siempre su tipo, pero la lógica real vive en un solo sitio.

use crate::backventanas::auth::AuthState;
use crate::backventanas::backadmin::adminfinanzas::models::*;
use crate::dinero::a_centavos;
use sqlx::SqlitePool;

// Crear un corte de caja. El tipo de corte lo define el payload (`tipo_corte`);
// los comandos `crear_corte_x`/`crear_corte_z` lo fijan para que cada botón
// de la UI cree siempre su tipo, pero la lógica real vive en un solo sitio.
async fn crear_corte_impl(
    state: &SqlitePool,
    auth: &tauri::State<'_, AuthState>,
    datos: CrearCorteRequest,
) -> Result<i64, String> {
    let tipo = match datos.tipo_corte.as_str() {
        "X" => "X",
        _ => "Z",
    };
    let usuario_id = auth.require_admin()?.user_id;

    let result = sqlx::query(
        r#"INSERT INTO cortes_caja (monto_inicial, tipo_corte, turno, observaciones, usuario_id, estado)
           VALUES (?, ?, ?, ?, ?, 'abierto')"#
    )
    .bind(a_centavos(datos.monto_inicial))
    .bind(tipo)
    .bind(&datos.turno)
    .bind(&datos.observaciones)
    .bind(usuario_id)
    .execute(state)
    .await
    .map_err(|e| e.to_string())?;

    Ok(result.last_insert_rowid())
}

#[tauri::command]
pub async fn crear_corte_x(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    mut datos: CrearCorteRequest,
) -> Result<i64, String> {
    datos.tipo_corte = "X".to_string();
    crear_corte_impl(&*state, &auth, datos).await
}

#[tauri::command]
pub async fn crear_corte_z(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    mut datos: CrearCorteRequest,
) -> Result<i64, String> {
    datos.tipo_corte = "Z".to_string();
    crear_corte_impl(&*state, &auth, datos).await
}
