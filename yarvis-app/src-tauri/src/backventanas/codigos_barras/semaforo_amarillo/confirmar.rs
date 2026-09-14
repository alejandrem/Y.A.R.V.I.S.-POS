// ============================================================
// confirmar — SI ES ESTE (issue #11).
//
// Asigna + audita origen 'confirmado-amarillo' + quien confirmo +
// resuelve el pendiente, todo en una transaccion. Si el ean ya esta
// en otro producto, el pendiente va a conflicto (igual que el rojo).
// ============================================================

use crate::backventanas::codigos_barras::semaforo_verde::validar_ean;
use crate::backventanas::codigos_barras::{mensaje_error_codigo, normalizar_codigo_barras, validar_codigo_barras};
use sqlx::{Sqlite, SqlitePool, Transaction};

type Tx<'a> = Transaction<'a, Sqlite>;

async fn marcar_conflicto(pool: &SqlitePool, pid: i64) -> Result<(), String> {
    sqlx::query("UPDATE pendientes_codigos SET estado = 'conflicto' WHERE id = ?")
        .bind(pid)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

async fn checar_ean(pool: &SqlitePool, pid: i64, ean: &str, prod: i64) -> Result<(), String> {
    validar_codigo_barras(&Some(ean.into()))?;
    if !validar_ean(ean) { return Err(format!("El ean {ean} tiene dígito inválido.")); }
    let otro: Option<i64> = sqlx::query_scalar("SELECT id FROM productos WHERE codigo_barras = ?")
        .bind(ean)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?
        .flatten();
    if otro.is_some_and(|id| id != prod) {
        marcar_conflicto(pool, pid).await?;
        return Err("El código ya está en otro producto: a conflicto.".into());
    }
    Ok(())
}

async fn leer_pendiente(pool: &SqlitePool, pid: i64) -> Result<(String, Option<String>, Option<f64>), String> {
    sqlx::query_as::<_, (String, Option<String>, Option<f64>)>(
        "SELECT estado, ean, mejor_score FROM pendientes_codigos WHERE id = ?",
    )
    .bind(pid)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "Pendiente no existe.".to_string())
}

fn ean_final(ean_override: Option<&str>, ean_pend: Option<String>) -> Result<Option<String>, String> {
    match ean_override.map(|s| s.to_string()).or(ean_pend) {
        Some(raw) => Ok(normalizar_codigo_barras(Some(&raw))),
        None => Ok(None),
    }
}

async fn existe_producto(pool: &SqlitePool, prod: i64) -> Result<(), String> {
    let id: Option<i64> = sqlx::query_scalar("SELECT id FROM productos WHERE id = ?")
        .bind(prod)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?
        .flatten();
    id.map(|_| ()).ok_or_else(|| "El producto destino no existe.".into())
}

async fn pegar_codigo(tx: &mut Tx<'_>, prod: i64, ean: &Option<String>) -> Result<(), String> {
    if let Some(ref e) = ean {
        sqlx::query("UPDATE productos SET codigo_barras = ? WHERE id = ?")
            .bind(e)
            .bind(prod)
            .execute(&mut **tx)
            .await
            .map_err(mensaje_error_codigo)?;
    }
    Ok(())
}

async fn leer_nombres(tx: &mut Tx<'_>, pid: i64) -> Result<(String, String), String> {
    sqlx::query_as::<_, (String, String)>(
        "SELECT nombre_crudo, nombre_norm FROM pendientes_codigos WHERE id = ?",
    )
    .bind(pid)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| e.to_string())
    .map(|par| par.unwrap_or_default())
}

async fn insertar_vinculo(tx: &mut Tx<'_>, pid: i64, prod: i64, ean: &Option<String>, score: Option<f64>, por: Option<i64>) -> Result<(), String> {
    let (crudo, norm) = leer_nombres(tx, pid).await?;
    sqlx::query(
        "INSERT OR IGNORE INTO vinculos_codigos (ean, producto_id, nombre_ticket_crudo, nombre_ticket_norm, origen, score, confirmado_por) VALUES (?, ?, ?, ?, 'confirmado-amarillo', ?, ?)",
    )
    .bind(ean)
    .bind(prod)
    .bind(crudo)
    .bind(norm)
    .bind(score)
    .bind(por)
    .execute(&mut **tx)
    .await
    .map(|_| ())
    .map_err(|e| e.to_string())
}

async fn auditar(tx: &mut Tx<'_>, pid: i64, prod: i64, ean: &Option<String>, score: Option<f64>, por: Option<i64>) -> Result<(), String> {
    insertar_vinculo(tx, pid, prod, ean, score, por).await?;
    sqlx::query("UPDATE pendientes_codigos SET estado = 'resuelto' WHERE id = ?")
        .bind(pid)
        .execute(&mut **tx)
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// SI ES ESTE: asigna + audita + resuelve.
pub async fn confirmar_impl(pool: &SqlitePool, pid: i64, prod: i64, ean_override: Option<&str>, por: Option<i64>) -> Result<(), String> {
    let (estado, ean_pend, score) = leer_pendiente(pool, pid).await?;
    if estado == "resuelto" { return Err("Ese pendiente ya está resuelto.".into()); }
    let ean = ean_final(ean_override, ean_pend)?;
    if let Some(ref e) = ean { checar_ean(pool, pid, e, prod).await?; }
    existe_producto(pool, prod).await?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    pegar_codigo(&mut tx, prod, &ean).await?;
    auditar(&mut tx, pid, prod, &ean, score, por).await?;
    tx.commit().await.map_err(|e| e.to_string())
}
