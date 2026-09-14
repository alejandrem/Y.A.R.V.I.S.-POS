// ============================================================
// sugerencias — Derivacion del amarillo (issue #11).
//
// `sugerir` decide UN camino: conflicto | aprendizaje | scoring |
// rojo. Solo orquesta (cada rama vive en su helper). Lo resuelto
// jamas se reabre; re-sugerir actualiza el pendiente.
// ============================================================

use super::candidatos::{aprendizaje, hay_conflicto_ean, top5};
use super::tipos::{Candidato, SugerenciaTop};
use crate::backventanas::codigos_barras::semaforo_rojo::registrar_pendiente_impl;
use crate::backventanas::codigos_barras::normalizar_codigo_barras;
use sqlx::SqlitePool;

fn vacia(pendiente_id: i64, estado: &str) -> SugerenciaTop {
    SugerenciaTop { pendiente_id, estado: estado.into(), candidatos: vec![] }
}

async fn guardar_amarillo(pool: &SqlitePool, pid: i64, mejor: &Candidato) -> Result<(), String> {
    sqlx::query(
        "UPDATE pendientes_codigos SET estado = 'amarillo', mejor_candidato_id = ?, mejor_score = ?, actualizado_en = CURRENT_TIMESTAMP WHERE id = ?",
    )
    .bind(mejor.producto_id)
    .bind(mejor.score)
    .bind(pid)
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(|e| e.to_string())
}

async fn a_conflicto(pool: &SqlitePool, ean: &str, nombre: &str) -> Result<SugerenciaTop, String> {
    let pid = registrar_pendiente_impl(pool, Some(ean), nombre).await?;
    sqlx::query("UPDATE pendientes_codigos SET estado = 'conflicto' WHERE id = ?")
        .bind(pid)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(vacia(pid, "conflicto"))
}

async fn a_aprendizaje(pool: &SqlitePool, c: Candidato, nombre: &str, ean: Option<&str>) -> Result<SugerenciaTop, String> {
    let pid = registrar_pendiente_impl(pool, ean, nombre).await?;
    guardar_amarillo(pool, pid, &c).await?;
    Ok(SugerenciaTop { pendiente_id: pid, estado: "aprendizaje".into(), candidatos: vec![c] })
}

async fn a_rojo(pool: &SqlitePool, nombre: &str, ean: Option<&str>) -> Result<SugerenciaTop, String> {
    let pid = registrar_pendiente_impl(pool, ean, nombre).await?;
    Ok(vacia(pid, "rojo"))
}

async fn a_scoring(pool: &SqlitePool, nombre: &str, marca: Option<&str>, ean: Option<&str>) -> Result<SugerenciaTop, String> {
    let lista = top5(pool, nombre, marca).await?;
    if lista.is_empty() {
        return a_rojo(pool, nombre, ean).await;
    }
    let pid = registrar_pendiente_impl(pool, ean, nombre).await?;
    guardar_amarillo(pool, pid, &lista[0]).await?;
    Ok(SugerenciaTop { pendiente_id: pid, estado: "amarillo".into(), candidatos: lista })
}

/// Sugiere top-5 o deriva (conflicto / aprendizaje / rojo).
pub async fn sugerir_impl(pool: &SqlitePool, nombre_crudo: &str, marca: Option<&str>, ean_crudo: Option<&str>) -> Result<SugerenciaTop, String> {
    let norm = src_ia::embeddings::normalizar(nombre_crudo);
    if norm.is_empty() { return Err("El nombre del ticket es obligatorio.".into()); }
    let ean = ean_crudo.and_then(|e| normalizar_codigo_barras(Some(e)));
    if let Some(ref e) = ean {
        if hay_conflicto_ean(pool, e).await? { return a_conflicto(pool, e, nombre_crudo).await; }
    }
    if let Some(c) = aprendizaje(pool, &norm).await? {
        return a_aprendizaje(pool, c, nombre_crudo, ean.as_deref()).await;
    }
    a_scoring(pool, nombre_crudo, marca, ean.as_deref()).await
}
