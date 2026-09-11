// ============================================================
// compras — Recepción de mercancía + pago al proveedor (la "factura").
//
// Todo-o-nada por compra: cabecera + renglones + entrada a stock +
// egreso en caja, en una sola transacción. Si algo falla, no queda
// nada a medias (ni stock sumado sin factura, ni factura sin stock).
//
// Dinero en centavos INTEGER (regla de oro); cantidades en REAL.
//
// Egreso automático: si el cajero tiene corte ABIERTO y el monto > 0,
// se registra el retiro vinculado (movimiento_id exacto). Sin corte
// abierto (o monto 0), la compra queda guardada y el movimiento
// pendiente — nunca se inventa un corte.
// ============================================================

use crate::backventanas::auth::AuthState;
use crate::dinero::{a_centavos, a_pesos};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Deserialize, Clone)]
pub struct ItemCompraRequest {
    pub producto_id: Option<i64>,
    pub nombre: String,
    pub presentacion: String,
    pub cantidad: f64,
    /// Obligatorios si presentacion == "paquete": el total de unidades
    /// lo calcula el backend (piezas × paquetes), no el frontend.
    #[serde(default)]
    pub piezas_por_paquete: Option<f64>,
    #[serde(default)]
    pub paquetes: Option<f64>,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct CompraRegistrada {
    pub compra_id: i64,
    pub sugerido: f64,
    pub pagado: f64,
    pub movimiento_id: Option<i64>,
    /// true = se pagó pero no había corte abierto (egreso pendiente).
    pub movimiento_pendiente: bool,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct CompraRow {
    pub id: i64,
    pub proveedor: String,
    pub fecha: String,
    pub pagado: f64,
    pub sugerido: f64,
    pub metodo_pago: String,
    pub items: i64,
    pub movimiento_pendiente: bool,
    /// Id de la factura que esta rectifica (None = original).
    pub rectifica_a: Option<i64>,
    /// true = ya tiene una rectificativa encima (ver historial completo).
    pub rectificada: bool,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct ItemCompraRow {
    pub nombre: String,
    pub presentacion: String,
    pub cantidad: f64,
    pub precio_sugerido: f64,
    pub producto_id: Option<i64>,
    pub piezas_por_paquete: Option<f64>,
    pub paquetes: Option<f64>,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct CompraDetalle {
    pub id: i64,
    pub proveedor: String,
    pub fecha: String,
    pub pagado: f64,
    pub sugerido: f64,
    pub metodo_pago: String,
    pub comentario: Option<String>,
    pub movimiento_id: Option<i64>,
    pub rectifica_a: Option<i64>,
    /// Ids de rectificativas que apuntan a esta (vacío = nunca rectificada).
    pub rectificada_por: Vec<i64>,
    pub items: Vec<ItemCompraRow>,
}

/// Valida un renglón y devuelve (nombre, presentacion, unidades totales,
/// piezas_por_paquete, paquetes). En paquete, el total lo calcula el
/// backend (piezas × paquetes); en unidad van NULL.
fn validar_item(it: &ItemCompraRequest) -> Result<(String, String, f64, Option<f64>, Option<f64>), String> {
    let nombre = it.nombre.trim().to_string();
    if nombre.is_empty() {
        return Err("Hay un renglón sin producto.".into());
    }
    if it.presentacion != "unidad" && it.presentacion != "paquete" {
        return Err(format!("Presentación inválida en '{}': usa unidad o paquete.", nombre));
    }
    if it.presentacion == "paquete" {
        let (piezas, paqs) = match (it.piezas_por_paquete, it.paquetes) {
            (Some(pz), Some(pq)) => (pz, pq),
            _ => return Err(format!("En '{}' dime cuántas piezas trae el paquete y cuántos paquetes son.", nombre)),
        };
        if !piezas.is_finite() || piezas <= 0.0 || !paqs.is_finite() || paqs <= 0.0 {
            return Err(format!("Piezas y paquetes de '{}' deben ser mayores a 0.", nombre));
        }
        Ok((nombre, it.presentacion.clone(), piezas * paqs, Some(piezas), Some(paqs)))
    } else {
        if !it.cantidad.is_finite() || it.cantidad <= 0.0 {
            return Err(format!("Cantidad inválida en '{}'.", nombre));
        }
        Ok((nombre, it.presentacion.clone(), it.cantidad, None, None))
    }
}

#[tauri::command]
pub async fn registrar_compra(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    proveedor_id: i64,
    items: Vec<ItemCompraRequest>,
    monto_pagado: f64,
    metodo_pago: Option<String>,
    comentario: Option<String>,
) -> Result<CompraRegistrada, String> {
    let session = auth.require_operator()?;
    registrar_compra_impl(
        &state,
        session.user_id,
        proveedor_id,
        items,
        monto_pagado,
        metodo_pago.unwrap_or_else(|| "efectivo".into()),
        comentario,
    )
    .await
}

/// Renglón validado y costeado, listo para guardar.
struct RenglonValido {
    pid: Option<i64>,
    nombre: String,
    presentacion: String,
    cantidad: f64,
    piezas: Option<f64>,
    paqs: Option<f64>,
    costo: i64,
}

/// Valida renglones, verifica productos y calcula el sugerido.
/// Compartido por registrar y rectificar (misma vara para ambas).
async fn costear_renglones(
    pool: &SqlitePool,
    catalogo: &[(i64, String, i64)],
    items: &[ItemCompraRequest],
) -> Result<Vec<RenglonValido>, String> {
    let costo_de = |pid: Option<i64>, nombre: &str| -> i64 {
        if let Some(id) = pid {
            if let Some((_, _, c)) = catalogo.iter().find(|(i, _, _)| *i == id) {
                return *c;
            }
        }
        let norm = src_ia::embeddings::normalizar(nombre);
        let matches: Vec<i64> = catalogo
            .iter()
            .filter(|(_, n, _)| src_ia::embeddings::normalizar(n) == norm)
            .map(|(i, _, _)| *i)
            .collect();
        if matches.len() == 1 {
            catalogo.iter().find(|(i, _, _)| *i == matches[0]).map(|(_, _, c)| *c).unwrap_or(0)
        } else {
            0
        }
    };

    let mut out = Vec::new();
    for it in items {
        let (nombre, presentacion, cantidad, piezas, paqs) = validar_item(it)?;
        let costo = costo_de(it.producto_id, &nombre);
        // producto_id explícito debe existir (no fantasmas).
        if let Some(pid) = it.producto_id {
            let existe: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM productos WHERE id = ?")
                .bind(pid)
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())?;
            if existe == 0 {
                return Err(format!("El producto '{}' ya no existe en inventario.", nombre));
            }
        }
        out.push(RenglonValido { pid: it.producto_id, nombre, presentacion, cantidad, piezas, paqs, costo });
    }
    Ok(out)
}

/// Guarda renglones y suma su stock (entradas). Corre dentro de la tx.
async fn insertar_renglones(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    compra_id: i64,
    validados: &[RenglonValido],
) -> Result<(), String> {
    for v in validados {
        if let Some(id) = v.pid {
            sqlx::query("UPDATE productos SET stock = stock + ? WHERE id = ?")
                .bind(v.cantidad)
                .bind(id)
                .execute(&mut **tx)
                .await
                .map_err(|e| e.to_string())?;
        }
        sqlx::query(
            "INSERT INTO compras_items (compra_id, producto_id, nombre, presentacion, cantidad, precio_sugerido, piezas_por_paquete, paquetes)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(compra_id)
        .bind(v.pid)
        .bind(&v.nombre)
        .bind(&v.presentacion)
        .bind(v.cantidad)
        .bind(v.costo)
        .bind(v.piezas)
        .bind(v.paqs)
        .execute(&mut **tx)
        .await
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Egreso automático contra el corte ABIERTO del cajero + vínculo exacto.
/// Devuelve (movimiento_id, pendiente). Sin corte (o monto 0): pendiente.
async fn egreso_de_compra(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    cajero_id: i64,
    compra_id: i64,
    nombre_prov: &str,
    monto_pagado: f64,
    metodo_pago: &str,
) -> Result<(Option<i64>, bool), String> {
    if monto_pagado <= 0.0 {
        return Ok((None, false));
    }
    let corte: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM cortes_caja WHERE usuario_id = ? AND estado = 'abierto'
         ORDER BY fecha_apertura DESC LIMIT 1",
    )
    .bind(cajero_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;
    match corte {
        Some((corte_id,)) => {
            let mid: i64 = sqlx::query(
                "INSERT INTO movimientos_caja (corte_id, tipo, concepto, monto, metodo_pago, referencia_id)
                 VALUES (?, 'retiro', ?, ?, ?, ?)",
            )
            .bind(corte_id)
            .bind(format!("Compra #{compra_id} · {nombre_prov}"))
            .bind(a_centavos(monto_pagado))
            .bind(metodo_pago)
            .bind(compra_id)
            .execute(&mut **tx)
            .await
            .map_err(|e| e.to_string())?
            .last_insert_rowid();
            sqlx::query("UPDATE compras SET movimiento_id = ? WHERE id = ?")
                .bind(mid)
                .bind(compra_id)
                .execute(&mut **tx)
                .await
                .map_err(|e| e.to_string())?;
            Ok((Some(mid), false))
        }
        None => Ok((None, true)),
    }
}

/// Núcleo testeable sin runtime de Tauri.
#[allow(clippy::too_many_arguments)]
pub async fn registrar_compra_impl(
    pool: &SqlitePool,
    cajero_id: i64,
    proveedor_id: i64,
    items: Vec<ItemCompraRequest>,
    monto_pagado: f64,
    metodo_pago: String,
    comentario: Option<String>,
) -> Result<CompraRegistrada, String> {
    if items.is_empty() {
        return Err("La compra no trae productos.".into());
    }
    if !monto_pagado.is_finite() || monto_pagado < 0.0 {
        return Err("Monto pagado inválido.".into());
    }
    let proveedor: Option<(String,)> =
        sqlx::query_as("SELECT nombre FROM proveedores WHERE id = ?")
            .bind(proveedor_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;
    let Some((nombre_prov,)) = proveedor else {
        return Err("Registra primero al proveedor.".into());
    };

    // Costos para el sugerido (exacto no ambiguo; 0 si no hay).
    let catalogo: Vec<(i64, String, i64)> =
        sqlx::query_as("SELECT id, nombre, precio_costo FROM productos")
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
    let validados = costear_renglones(pool, &catalogo, &items).await?;
    let sugerido_cents: i64 = validados.iter().map(|v| (v.cantidad * v.costo as f64).round() as i64).sum();

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let compra_id: i64 = sqlx::query(
        "INSERT INTO compras (proveedor_id, monto_pagado, monto_sugerido, metodo_pago, comentario, cajero_id)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(proveedor_id)
    .bind(a_centavos(monto_pagado))
    .bind(sugerido_cents)
    .bind(&metodo_pago)
    .bind(comentario.map(|c| c.trim().to_string()).filter(|c| !c.is_empty()))
    .bind(cajero_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?
    .last_insert_rowid();

    insertar_renglones(&mut tx, compra_id, &validados).await?;
    let (movimiento_id, pendiente) =
        egreso_de_compra(&mut tx, cajero_id, compra_id, &nombre_prov, monto_pagado, &metodo_pago).await?;

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(CompraRegistrada {
        compra_id,
        sugerido: a_pesos(sugerido_cents),
        pagado: monto_pagado,
        movimiento_id,
        movimiento_pendiente: pendiente,
    })
}

#[tauri::command]
pub async fn rectificar_compra(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    compra_original_id: i64,
    items: Vec<ItemCompraRequest>,
    monto_pagado: f64,
    metodo_pago: Option<String>,
    comentario: Option<String>,
) -> Result<CompraRegistrada, String> {
    let session = auth.require_operator()?;
    rectificar_compra_impl(
        &state,
        session.user_id,
        compra_original_id,
        items,
        monto_pagado,
        metodo_pago.unwrap_or_else(|| "efectivo".into()),
        comentario,
    )
    .await
}

/// Rectificativa antifraude: la factura original JAMÁS se edita ni se
/// borra; se crea una compra NUEVA que apunta a ella (`rectifica_a`).
/// El stock de la original se REVERSA (resta lo que sumó) y se aplica
/// el nuevo; el egreso original queda intacto en caja (el dinero ya
/// salió: la diferencia la concilia el dueño viendo ambas facturas)
/// y se genera un egreso nuevo por el monto rectificado.
#[allow(clippy::too_many_arguments)]
pub async fn rectificar_compra_impl(
    pool: &SqlitePool,
    cajero_id: i64,
    compra_original_id: i64,
    items: Vec<ItemCompraRequest>,
    monto_pagado: f64,
    metodo_pago: String,
    comentario: Option<String>,
) -> Result<CompraRegistrada, String> {
    if items.is_empty() {
        return Err("La rectificativa no trae productos.".into());
    }
    if !monto_pagado.is_finite() || monto_pagado < 0.0 {
        return Err("Monto pagado inválido.".into());
    }
    let original: Option<(i64, String)> = sqlx::query_as(
        "SELECT c.proveedor_id, p.nombre FROM compras c
         JOIN proveedores p ON p.id = c.proveedor_id WHERE c.id = ?",
    )
    .bind(compra_original_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;
    let Some((proveedor_id, nombre_prov)) = original else {
        return Err("Compra no encontrada.".into());
    };

    let catalogo: Vec<(i64, String, i64)> =
        sqlx::query_as("SELECT id, nombre, precio_costo FROM productos")
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;
    let validados = costear_renglones(pool, &catalogo, &items).await?;
    let sugerido_cents: i64 = validados.iter().map(|v| (v.cantidad * v.costo as f64).round() as i64).sum();

    // Renglones originales para reversar su stock.
    let viejos: Vec<(Option<i64>, f64)> = sqlx::query_as(
        "SELECT producto_id, cantidad FROM compras_items WHERE compra_id = ?",
    )
    .bind(compra_original_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let compra_id: i64 = sqlx::query(
        "INSERT INTO compras (proveedor_id, monto_pagado, monto_sugerido, metodo_pago, comentario, cajero_id, rectifica_a)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(proveedor_id)
    .bind(a_centavos(monto_pagado))
    .bind(sugerido_cents)
    .bind(&metodo_pago)
    .bind(comentario.map(|c| c.trim().to_string()).filter(|c| !c.is_empty()))
    .bind(cajero_id)
    .bind(compra_original_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?
    .last_insert_rowid();

    // Reversa: resta del stock lo que la original había sumado.
    for (pid, cantidad) in &viejos {
        if let Some(id) = pid {
            sqlx::query("UPDATE productos SET stock = stock - ? WHERE id = ?")
                .bind(*cantidad)
                .bind(*id)
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;
        }
    }

    insertar_renglones(&mut tx, compra_id, &validados).await?;
    let (movimiento_id, pendiente) =
        egreso_de_compra(&mut tx, cajero_id, compra_id, &nombre_prov, monto_pagado, &metodo_pago).await?;

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(CompraRegistrada {
        compra_id,
        sugerido: a_pesos(sugerido_cents),
        pagado: monto_pagado,
        movimiento_id,
        movimiento_pendiente: pendiente,
    })
}

#[tauri::command]
pub async fn historial_compras(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    proveedor_id: Option<i64>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<CompraRow>, String> {
    auth.require_operator()?;
    historial_compras_impl(&state, proveedor_id, limit.unwrap_or(100), offset.unwrap_or(0)).await
}

/// Núcleo testeable sin runtime de Tauri.
pub async fn historial_compras_impl(
    pool: &SqlitePool,
    proveedor_id: Option<i64>,
    limit: i64,
    offset: i64,
) -> Result<Vec<CompraRow>, String> {
    let limit = limit.clamp(1, 500);
    let offset = offset.max(0);
    let base = "SELECT c.id, p.nombre, strftime('%Y-%m-%d %H:%M:%S', c.fecha) as fecha,
                c.monto_pagado, c.monto_sugerido, c.metodo_pago,
                COUNT(i.id), c.movimiento_id IS NULL AND c.monto_pagado > 0,
                c.rectifica_a, EXISTS(SELECT 1 FROM compras r WHERE r.rectifica_a = c.id)
         FROM compras c JOIN proveedores p ON p.id = c.proveedor_id
         LEFT JOIN compras_items i ON i.compra_id = c.id";
    let rows = match proveedor_id {
        Some(id) => {
            sqlx::query_as::<_, (i64, String, String, i64, i64, String, i64, bool, Option<i64>, bool)>(&format!(
                "{base} WHERE c.proveedor_id = ?3
                 GROUP BY c.id ORDER BY c.fecha DESC, c.id DESC LIMIT ?1 OFFSET ?2"
            ))
            .bind(limit)
            .bind(offset)
            .bind(id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?
        }
        None => {
            sqlx::query_as::<_, (i64, String, String, i64, i64, String, i64, bool, Option<i64>, bool)>(&format!(
                "{base} GROUP BY c.id ORDER BY c.fecha DESC, c.id DESC LIMIT ?1 OFFSET ?2"
            ))
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?
        }
    };

    Ok(rows
        .into_iter()
        .map(|r| CompraRow {
            id: r.0,
            proveedor: r.1,
            fecha: r.2,
            pagado: a_pesos(r.3),
            sugerido: a_pesos(r.4),
            metodo_pago: r.5,
            items: r.6,
            movimiento_pendiente: r.7,
            rectifica_a: r.8,
            rectificada: r.9,
        })
        .collect())
}

#[tauri::command]
pub async fn get_compra_detalle(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    compra_id: i64,
) -> Result<CompraDetalle, String> {
    auth.require_operator()?;
    get_compra_detalle_impl(&state, compra_id).await
}

/// Núcleo testeable sin runtime de Tauri.
pub async fn get_compra_detalle_impl(pool: &SqlitePool, compra_id: i64) -> Result<CompraDetalle, String> {
    let cab = sqlx::query_as::<_, (i64, String, String, i64, i64, String, Option<String>, Option<i64>, Option<i64>)>(
        "SELECT c.id, p.nombre, strftime('%Y-%m-%d %H:%M:%S', c.fecha),
                c.monto_pagado, c.monto_sugerido, c.metodo_pago, c.comentario, c.movimiento_id, c.rectifica_a
         FROM compras c JOIN proveedores p ON p.id = c.proveedor_id WHERE c.id = ?",
    )
    .bind(compra_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "Compra no encontrada.".to_string())?;

    let filas = sqlx::query_as::<_, (String, String, f64, i64, Option<i64>, Option<f64>, Option<f64>)>(
        "SELECT nombre, presentacion, cantidad, precio_sugerido, producto_id,
                piezas_por_paquete, paquetes
         FROM compras_items WHERE compra_id = ? ORDER BY id ASC",
    )
    .bind(compra_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let hijas: Vec<i64> = sqlx::query_scalar("SELECT id FROM compras WHERE rectifica_a = ? ORDER BY id ASC")
        .bind(compra_id)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(CompraDetalle {
        id: cab.0,
        proveedor: cab.1,
        fecha: cab.2,
        pagado: a_pesos(cab.3),
        sugerido: a_pesos(cab.4),
        metodo_pago: cab.5,
        comentario: cab.6,
        movimiento_id: cab.7,
        rectifica_a: cab.8,
        rectificada_por: hijas,
        items: filas
            .into_iter()
            .map(|f| ItemCompraRow {
                nombre: f.0,
                presentacion: f.1,
                cantidad: f.2,
                precio_sugerido: a_pesos(f.3),
                producto_id: f.4,
                piezas_por_paquete: f.5,
                paquetes: f.6,
            })
            .collect(),
    })
}
