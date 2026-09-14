// ============================================================
// rojo — Sin match: captura manual que alimenta el aprendizaje.
//
// Regla (issue #12): aqui cae lo que el motor no reconoce para nada.
// Mensaje honesto: no lo conozco, registralo tu. Cada rojo resuelto
// guarda el par (nombre_norm -> producto_id + ean) como vinculo
// 'manual-rojo': la proxima vez el MISMO nombre cae minimo en
// amarillo (el futuro scoring consulta `vinculos_codigos` primero)
// y si ademas cuadra presentacion, en verde.
//
// Idempotencia: mismo (nombre_norm, ean) = mismo pendiente
// (veces_visto++). Re-importar no duplica lo ya resuelto: lo
// resuelto queda en 'resuelto' y no vuelve a 'rojo'.
// ============================================================

use super::super::semaforo_verde::validar_ean;
use crate::backventanas::codigos_barras::{mensaje_error_codigo, normalizar_codigo_barras};
use serde::Serialize;
use sqlx::SqlitePool;

// ---------- Tipos ----------

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Pendiente {
    pub id: i64,
    pub ean: Option<String>,
    pub nombre_crudo: String,
    pub nombre_norm: String,
    pub estado: String,
    pub mejor_candidato_id: Option<i64>,
    pub mejor_score: Option<f64>,
    pub veces_visto: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConteosPendientes {
    pub rojo: i64,
    pub amarillo: i64,
    pub conflicto: i64,
    pub resuelto: i64,
}

// ---------- Nucleo: registrar (idempotente) ----------

/// Registra un pendiente rojo. `ean` puede ser None (ticket sin codigo,
/// solo nombre crudo). Devuelve el id (nuevo o existente bumped).
pub async fn registrar_pendiente_impl(
    pool: &SqlitePool,
    ean_crudo: Option<&str>,
    nombre_crudo: &str,
) -> Result<i64, String> {
    let ean = match ean_crudo {
        Some(raw) => normalizar_codigo_barras(Some(raw)),
        None => None,
    };
    // Solo charset/longitud aqui: el rojo acepta lo pitado tal cual;
    // el checksum se exige al RESOLVER (asignar un ean malo seria verde).
    if let Some(ref e) = ean {
        crate::backventanas::codigos_barras::validar_codigo_barras(&Some(e.clone()))?;
    }
    let nombre_norm = src_ia::embeddings::normalizar(nombre_crudo);
    if nombre_norm.is_empty() {
        return Err("El nombre del ticket es obligatorio.".into());
    }
    // ¿Ya existe? (NULL-safe: `ean IS NULL` no iguala con `=`).
    let existente: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM pendientes_codigos WHERE nombre_norm = ? AND ((ean IS NULL AND ? IS NULL) OR ean = ?) LIMIT 1",
    )
    .bind(&nombre_norm)
    .bind(&ean)
    .bind(&ean)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;
    if let Some((id,)) = existente {
        sqlx::query(
            "UPDATE pendientes_codigos SET veces_visto = veces_visto + 1, actualizado_en = CURRENT_TIMESTAMP WHERE id = ?",
        )
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
        return Ok(id);
    }
    let r = sqlx::query(
        "INSERT INTO pendientes_codigos (ean, nombre_crudo, nombre_norm, estado) VALUES (?, ?, ?, 'rojo')",
    )
    .bind(&ean)
    .bind(nombre_crudo)
    .bind(&nombre_norm)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(r.last_insert_rowid())
}

// ---------- Lecturas para la cola (front #13 las consume) ----------

pub async fn listar_pendientes_impl(
    pool: &SqlitePool,
    estado: Option<&str>,
    limite: i64,
) -> Result<Vec<Pendiente>, String> {
    let lim = limite.clamp(1, 500);
    let rows = match estado {
        Some(e) => {
            sqlx::query_as::<_, Pendiente>(
                "SELECT id, ean, nombre_crudo, nombre_norm, estado, mejor_candidato_id, mejor_score, veces_visto FROM pendientes_codigos WHERE estado = ? ORDER BY veces_visto DESC, actualizado_en DESC LIMIT ?",
            )
            .bind(e)
            .bind(lim)
            .fetch_all(pool)
            .await
        }
        None => {
            sqlx::query_as::<_, Pendiente>(
                "SELECT id, ean, nombre_crudo, nombre_norm, estado, mejor_candidato_id, mejor_score, veces_visto FROM pendientes_codigos WHERE estado != 'resuelto' ORDER BY veces_visto DESC, actualizado_en DESC LIMIT ?",
            )
            .bind(lim)
            .fetch_all(pool)
            .await
        }
    }
    .map_err(|e| e.to_string())?;
    Ok(rows)
}

pub async fn contar_pendientes_impl(pool: &SqlitePool) -> Result<ConteosPendientes, String> {
    let rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT estado, COUNT(*) FROM pendientes_codigos GROUP BY estado",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;
    let mut c = ConteosPendientes { rojo: 0, amarillo: 0, conflicto: 0, resuelto: 0 };
    for (estado, n) in rows {
        match estado.as_str() {
            "rojo" => c.rojo = n,
            "amarillo" => c.amarillo = n,
            "conflicto" => c.conflicto = n,
            "resuelto" => c.resuelto = n,
            _ => {}
        }
    }
    Ok(c)
}

/// Aprendizaje: ¿este nombre ya se resolvio antes? El amarillo futuro
/// lo consulta primero para no sugerir desde cero.
pub async fn buscar_aprendizaje_impl(
    pool: &SqlitePool,
    nombre_norm: &str,
) -> Result<Option<i64>, String> {
    let id: Option<i64> = sqlx::query_scalar(
        "SELECT producto_id FROM vinculos_codigos WHERE nombre_ticket_norm = ? ORDER BY id DESC LIMIT 1",
    )
    .bind(nombre_norm)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .flatten();
    Ok(id)
}

// ---------- Resolucion: alta nueva ----------

#[allow(clippy::too_many_arguments)]
pub async fn resolver_con_alta_impl(
    pool: &SqlitePool,
    pendiente_id: i64,
    nombre: &str,
    codigo_barras: Option<&str>,
    categoria: Option<&str>,
    marca: Option<&str>,
    cantidad: Option<f64>,
    unidad: Option<&str>,
    confirmado_por: Option<i64>,
) -> Result<i64, String> {
    if nombre.trim().is_empty() {
        return Err("El nombre del producto es obligatorio.".into());
    }
    let pend: Option<Pendiente> = sqlx::query_as::<_, Pendiente>(
        "SELECT id, ean, nombre_crudo, nombre_norm, estado, mejor_candidato_id, mejor_score, veces_visto FROM pendientes_codigos WHERE id = ?",
    )
    .bind(pendiente_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;
    let pend = pend.ok_or_else(|| "Pendiente no existe.".to_string())?;
    if pend.estado == "resuelto" {
        return Err("Ese pendiente ya está resuelto.".into());
    }
    // ean: lo que mande el form, si no lo del pendiente.
    let ean_raw = codigo_barras
        .map(|s| s.to_string())
        .or(pend.ean.clone());
    let ean = match ean_raw.as_deref() {
        Some(raw) => normalizar_codigo_barras(Some(raw)),
        None => None,
    };
    if let Some(ref e) = ean {
        crate::backventanas::codigos_barras::validar_codigo_barras(&Some(e.clone()))?;
        if !validar_ean(e) {
            return Err(format!("El ean {e} tiene dígito verificador inválido."));
        }
        let ocupado: Option<i64> =
            sqlx::query_scalar("SELECT id FROM productos WHERE codigo_barras = ?")
                .bind(e)
                .fetch_optional(pool)
                .await
                .map_err(|e| e.to_string())?
                .flatten();
        if ocupado.is_some() {
            return Err("El código de barras ya está registrado en otro producto.".into());
        }
    }
    let unidad_canon = match unidad {
        Some(u) => {
            let c = u.trim().to_ascii_lowercase();
            if !["ml", "l", "g", "kg", "pzs"].contains(&c.as_str()) {
                return Err(format!("Unidad '{u}' fuera de catálogo (ml|l|g|kg|pzs)."));
            }
            Some(c)
        }
        None => None,
    };
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let r = sqlx::query(
        "INSERT INTO productos (nombre, descripcion, precio_costo, precio_venta, stock, stock_minimo, vendido, codigo_barras, categoria, marca, cantidad_presentacion, unidad_presentacion) VALUES (?, NULL, 0, 0, 0, 0, 0, ?, ?, ?, ?, ?)",
    )
    .bind(nombre.trim())
    .bind(&ean)
    .bind(categoria)
    .bind(marca)
    .bind(cantidad)
    .bind(&unidad_canon)
    .execute(&mut *tx)
    .await
    .map_err(mensaje_error_codigo)?;
    let producto_id = r.last_insert_rowid();
    sqlx::query(
        "INSERT OR IGNORE INTO vinculos_codigos (ean, producto_id, nombre_ticket_crudo, nombre_ticket_norm, origen, score, confirmado_por) VALUES (?, ?, ?, ?, 'manual-rojo', 1.0, ?)",
    )
    .bind(&ean)
    .bind(producto_id)
    .bind(&pend.nombre_crudo)
    .bind(&pend.nombre_norm)
    .bind(confirmado_por)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;
    sqlx::query(
        "UPDATE pendientes_codigos SET estado = 'resuelto', actualizado_en = CURRENT_TIMESTAMP WHERE id = ?",
    )
    .bind(pendiente_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(producto_id)
}

// ---------- Resolucion: asignar a existente ----------

/// Pega el ean del pendiente (o el override) a un producto existente
/// y cierra el pendiente como resuelto + vinculo manual-rojo.
/// Loop de oro en caja: pitaste desconocido -> eliges producto -> aprende.
pub async fn resolver_asignando_impl(
    pool: &SqlitePool,
    pendiente_id: i64,
    producto_id: i64,
    ean_override: Option<&str>,
    confirmado_por: Option<i64>,
) -> Result<(), String> {
    let pend: Option<Pendiente> = sqlx::query_as::<_, Pendiente>(
        "SELECT id, ean, nombre_crudo, nombre_norm, estado, mejor_candidato_id, mejor_score, veces_visto FROM pendientes_codigos WHERE id = ?",
    )
    .bind(pendiente_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;
    let pend = pend.ok_or_else(|| "Pendiente no existe.".to_string())?;
    if pend.estado == "resuelto" {
        return Err("Ese pendiente ya está resuelto.".into());
    }
    let ean_raw = ean_override.map(|s| s.to_string()).or(pend.ean.clone());
    let ean = normalizar_codigo_barras(ean_raw.as_deref())
        .ok_or_else(|| "Sin ean que asignar (el pendiente no trae y no mandaste override).".to_string())?;
    crate::backventanas::codigos_barras::validar_codigo_barras(&Some(ean.clone()))?;
    if !validar_ean(&ean) {
        return Err(format!("El ean {ean} tiene dígito verificador inválido."));
    }
    let existe: Option<i64> = sqlx::query_scalar("SELECT id FROM productos WHERE id = ?")
        .bind(producto_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?
        .flatten();
    if existe.is_none() {
        return Err("El producto destino no existe.".into());
    }
    let ocupado_por_otro: Option<i64> =
        sqlx::query_scalar("SELECT id FROM productos WHERE codigo_barras = ?")
            .bind(&ean)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?
            .flatten();
    if ocupado_por_otro.is_some_and(|id| id != producto_id) {
        // A cola de conflicto: 2 productos reclaman el mismo ean.
        sqlx::query(
            "UPDATE pendientes_codigos SET estado = 'conflicto', actualizado_en = CURRENT_TIMESTAMP WHERE id = ?",
        )
        .bind(pendiente_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
        return Err("El código ya está en otro producto: pendiente a conflicto.".into());
    }
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    sqlx::query("UPDATE productos SET codigo_barras = ? WHERE id = ?")
        .bind(&ean)
        .bind(producto_id)
        .execute(&mut *tx)
        .await
        .map_err(mensaje_error_codigo)?;
    sqlx::query(
        "INSERT OR IGNORE INTO vinculos_codigos (ean, producto_id, nombre_ticket_crudo, nombre_ticket_norm, origen, score, confirmado_por) VALUES (?, ?, ?, ?, 'manual-rojo', 1.0, ?)",
    )
    .bind(&ean)
    .bind(producto_id)
    .bind(&pend.nombre_crudo)
    .bind(&pend.nombre_norm)
    .bind(confirmado_por)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;
    sqlx::query(
        "UPDATE pendientes_codigos SET estado = 'resuelto', actualizado_en = CURRENT_TIMESTAMP WHERE id = ?",
    )
    .bind(pendiente_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}
