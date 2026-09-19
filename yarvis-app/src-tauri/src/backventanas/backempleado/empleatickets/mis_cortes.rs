// ============================================================
// mis_cortes — "Mis cortes" del operador actual.
//
// Operator-scoped como `mis_tickets.rs`: el cajero sale de la sesión
// (`session.user_id`), NUNCA de un parámetro del frontend. El Z escribe
// `cortes_caja.usuario_id` con ese mismo id (ver backcortes/cierre.rs),
// así que aquí no hay respaldo por nombre: el FK es canónico.
//
// El dinero vive en INTEGER centavos (migración 0005) y sale en pesos
// vía a_pesos, igual que `get_cortes_empleado`.
// ============================================================

use crate::backventanas::auth::AuthState;
use crate::dinero::a_pesos;
use serde::Serialize;
use sqlx::SqlitePool;

/// Límite por consulta (la lista del empleado es corta por diseño).
pub const MIS_CORTES_LIMITE_MAX: i64 = 60;

/// Corte propio resumido para la lista.
#[derive(Serialize, Debug, PartialEq)]
pub struct MiCorte {
    pub id: i64,
    pub fecha_apertura: Option<String>,
    pub fecha_cierre: Option<String>,
    pub tipo_corte: Option<String>,
    pub total_ventas: f64,
    pub estado: String,
}

#[tauri::command]
pub async fn get_mis_cortes(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    dias: Option<i64>,
) -> Result<Vec<MiCorte>, String> {
    let session = auth.require_operator()?;
    mis_cortes_impl(&state, session.user_id, dias.unwrap_or(30).clamp(1, 36500)).await
}

/// Núcleo testeable sin runtime de Tauri.
pub async fn mis_cortes_impl(
    pool: &SqlitePool,
    usuario_id: i64,
    dias: i64,
) -> Result<Vec<MiCorte>, String> {
    let dias = dias.clamp(1, 36500);
    // Ventana inclusiva sobre el cierre (apertura para cortes abiertos).
    let rows = sqlx::query_as::<_, (i64, Option<String>, Option<String>, Option<String>, i64, String)>(
        "SELECT id,
                strftime('%Y-%m-%d %H:%M:%S', fecha_apertura) as fecha_apertura,
                strftime('%Y-%m-%d %H:%M:%S', fecha_cierre) as fecha_cierre,
                tipo_corte, total_ventas, estado
         FROM cortes_caja
         WHERE usuario_id = ?1
           AND date(COALESCE(fecha_cierre, fecha_apertura)) >= date('now','localtime', printf('-%d days', ?2))
         ORDER BY COALESCE(fecha_cierre, fecha_apertura) DESC LIMIT 60",
    )
    .bind(usuario_id)
    .bind(dias - 1)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|r| MiCorte {
            id: r.0,
            fecha_apertura: r.1,
            fecha_cierre: r.2,
            tipo_corte: r.3,
            total_ventas: a_pesos(r.4),
            estado: r.5,
        })
        .collect())
}
