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
}

#[derive(Serialize, Debug, PartialEq)]
pub struct ItemCompraRow {
    pub nombre: String,
    pub presentacion: String,
    pub cantidad: f64,
    pub precio_sugerido: f64,
    pub producto_id: Option<i64>,
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
    pub items: Vec<ItemCompraRow>,
}

fn validar_item(it: &ItemCompraRequest) -> Result<(String, f64), String> {
    let nombre = it.nombre.trim().to_string();
    if nombre.is_empty() {
        return Err("Hay un renglón sin producto.".into());
    }
    if it.presentacion != "unidad" && it.presentacion != "paquete" {
        return Err(format!("Presentación inválida en '{}': usa unidad o paquete.", nombre));
    }
    if !it.cantidad.is_finite() || it.cantidad <= 0.0 {
        return Err(format!("Cantidad inválida en '{}'.", nombre));
    }
    Ok((nombre, it.cantidad))
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

    let mut validados: Vec<(Option<i64>, String, String, f64, i64)> = Vec::new();
    let mut sugerido_cents: i64 = 0;
    for it in &items {
        let (nombre, cantidad) = validar_item(it)?;
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
        sugerido_cents += (cantidad * costo as f64).round() as i64;
        validados.push((it.producto_id, nombre, it.presentacion.clone(), cantidad, costo));
    }

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

    for (pid, nombre, presentacion, cantidad, costo) in &validados {
        // Entrada a stock (v1: unidad y paquete suman directo; el factor
        // piezas-por-paquete llega en fase 2).
        if let Some(id) = pid {
            sqlx::query("UPDATE productos SET stock = stock + ? WHERE id = ?")
                .bind(*cantidad)
                .bind(*id)
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;
        }
        sqlx::query(
            "INSERT INTO compras_items (compra_id, producto_id, nombre, presentacion, cantidad, precio_sugerido)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(compra_id)
        .bind(*pid)
        .bind(nombre)
        .bind(presentacion)
        .bind(*cantidad)
        .bind(*costo)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    // Egreso automático contra el corte ABIERTO del cajero.
    let mut movimiento_id: Option<i64> = None;
    let mut pendiente = false;
    if monto_pagado > 0.0 {
        let corte: Option<(i64,)> = sqlx::query_as(
            "SELECT id FROM cortes_caja WHERE usuario_id = ? AND estado = 'abierto'
             ORDER BY fecha_apertura DESC LIMIT 1",
        )
        .bind(cajero_id)
        .fetch_optional(&mut *tx)
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
                .bind(&metodo_pago)
                .bind(compra_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?
                .last_insert_rowid();
                sqlx::query("UPDATE compras SET movimiento_id = ? WHERE id = ?")
                    .bind(mid)
                    .bind(compra_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| e.to_string())?;
                movimiento_id = Some(mid);
            }
            None => pendiente = true,
        }
    }

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
                COUNT(i.id), c.movimiento_id IS NULL AND c.monto_pagado > 0
         FROM compras c JOIN proveedores p ON p.id = c.proveedor_id
         LEFT JOIN compras_items i ON i.compra_id = c.id";
    let rows = match proveedor_id {
        Some(id) => {
            sqlx::query_as::<_, (i64, String, String, i64, i64, String, i64, bool)>(&format!(
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
            sqlx::query_as::<_, (i64, String, String, i64, i64, String, i64, bool)>(&format!(
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
    let cab = sqlx::query_as::<_, (i64, String, String, i64, i64, String, Option<String>, Option<i64>)>(
        "SELECT c.id, p.nombre, strftime('%Y-%m-%d %H:%M:%S', c.fecha),
                c.monto_pagado, c.monto_sugerido, c.metodo_pago, c.comentario, c.movimiento_id
         FROM compras c JOIN proveedores p ON p.id = c.proveedor_id WHERE c.id = ?",
    )
    .bind(compra_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "Compra no encontrada.".to_string())?;

    let filas = sqlx::query_as::<_, (String, String, f64, i64, Option<i64>)>(
        "SELECT nombre, presentacion, cantidad, precio_sugerido, producto_id
         FROM compras_items WHERE compra_id = ? ORDER BY id ASC",
    )
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
        items: filas
            .into_iter()
            .map(|f| ItemCompraRow {
                nombre: f.0,
                presentacion: f.1,
                cantidad: f.2,
                precio_sugerido: a_pesos(f.3),
                producto_id: f.4,
            })
            .collect(),
    })
}
