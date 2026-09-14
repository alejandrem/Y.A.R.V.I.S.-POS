// ============================================================
// rechazos — NO ES y NINGUNO (issue #11).
//
// NO ES: el rechazo se guarda en `rechazos_amarillos`; esos NO
// alimentan el umbral futuro via `stats` (el dueno recalibra
// UMBRAL_AMARILLO a mano, nunca auto).
// NINGUNO: cae a rojo (captura manual, issue #12).
// ============================================================

use super::tipos::StatsRechazos;
use sqlx::SqlitePool;

/// Guarda UN no (pendiente + producto + score con que se sugirio).
pub async fn rechazar_impl(pool: &SqlitePool, pid: i64, prod: i64, score: f64) -> Result<(), String> {
    sqlx::query("INSERT INTO rechazos_amarillos (pendiente_id, producto_id, score) VALUES (?, ?, ?)")
        .bind(pid)
        .bind(prod)
        .bind(score)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// NINGUNO de los 5: a rojo, sin candidato.
pub async fn ninguno_impl(pool: &SqlitePool, pid: i64) -> Result<(), String> {
    sqlx::query("UPDATE pendientes_codigos SET estado = 'rojo', mejor_candidato_id = NULL, mejor_score = NULL WHERE id = ?")
        .bind(pid)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// ¿El umbral estaba bajo? Si el promedio/maximo de NOs supera al
/// umbral, hay que subirlo a mano en `umbral.rs`.
pub async fn stats_rechazos_impl(pool: &SqlitePool) -> Result<StatsRechazos, String> {
    let row: (i64, Option<f64>, Option<f64>) =
        sqlx::query_as("SELECT COUNT(*), AVG(score), MAX(score) FROM rechazos_amarillos")
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
    Ok(StatsRechazos { total: row.0, score_promedio: row.1.unwrap_or(0.0), score_maximo: row.2.unwrap_or(0.0) })
}
