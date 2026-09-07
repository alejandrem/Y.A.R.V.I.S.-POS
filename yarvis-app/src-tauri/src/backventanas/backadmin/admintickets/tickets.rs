use crate::backventanas::auth::AuthState;
use crate::backventanas::db::db::DbPath;
use crate::backventanas::backadmin::adminfinanzas::metricas::calcular_utilidad_neta_periodo;
use crate::dinero::a_centavos;
use crate::models::{CorteDb, TicketDb, TicketItem};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::path::PathBuf;
use src_ia::embeddings::{cosine_similarity, normalizar, HashEmbedder, Embedder};

#[tauri::command]
pub async fn get_tickets(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
) -> Result<Vec<TicketDb>, String> {
    auth.require_admin()?;
    let rows = sqlx::query_as::<_, (i32, Option<String>, String, i64, String)>(
        "SELECT id, folio_ticket, strftime('%Y-%m-%d %H:%M:%S', fecha) as fecha, total, metodo_pago FROM ventas ORDER BY fecha DESC LIMIT 500"
    )
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    let tickets = rows
        .into_iter()
        .map(|row| TicketDb {
            id: row.0,
            folio_ticket: row.1,
            fecha: row.2,
            total: crate::dinero::a_pesos(row.3),
            metodo_pago: row.4,
        })
        .collect();

    Ok(tickets)
}

/// Total de ventas en la DB (el historial pagina: `get_tickets` trae los
/// 500 más recientes; esto dice cuántos hay en realidad).
#[tauri::command]
pub async fn get_tickets_total(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
) -> Result<i64, String> {
    auth.require_admin()?;
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM ventas")
        .fetch_one(&*state)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_cortes(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
) -> Result<Vec<CorteDb>, String> {
    auth.require_admin()?;
    let rows = sqlx::query_as::<_, (i32, String, i64, i64)>(
        "SELECT id, strftime('%Y-%m-%d %H:%M:%S', fecha_cierre) as fecha, total_ventas, total_efectivo FROM cortes_caja ORDER BY fecha_cierre DESC LIMIT 500"
    )
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    let cortes = rows
        .into_iter()
        .map(|row| CorteDb {
            id: row.0,
            fecha: row.1,
            total_ventas: crate::dinero::a_pesos(row.2),
            total_efectivo: crate::dinero::a_pesos(row.3),
        })
        .collect();

    Ok(cortes)
}

// FIX Bug 2: producto_id se guarda como NULL (None) en lugar de 0
// Asi no se viola la foreign key ni causa colision con productos reales
#[tauri::command]
pub async fn guardar_ticket_parseado(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    items: Vec<TicketItem>,
    total: f64,
    fecha: Option<String>,
    hora: Option<String>,
    metodo_pago: Option<String>,
) -> Result<String, String> {
    auth.require_admin()?;
    guardar_ticket_parseado_impl(&*state, items, total, fecha, hora, metodo_pago).await
}

/// Núcleo de importación de ticket, testeable sin runtime de Tauri.
pub async fn guardar_ticket_parseado_impl(
    pool: &SqlitePool,
    items: Vec<TicketItem>,
    total: f64,
    fecha: Option<String>,
    hora: Option<String>,
    metodo_pago: Option<String>,
) -> Result<String, String> {
    let metodo_pago = metodo_pago.unwrap_or_else(|| "efectivo".into());
    let fecha_iso = match (fecha, hora) {
        (Some(f), Some(h)) => {
            if !f.is_empty() && !h.is_empty() {
                Some(format!("{} {}:00", f, h))
            } else if !f.is_empty() {
                Some(format!("{} 00:00:00", f))
            } else {
                None
            }
        }
        (Some(f), None) => {
            if !f.is_empty() {
                Some(format!("{} 00:00:00", f))
            } else {
                None
            }
        }
        _ => None,
    };

    // TRANSACCIÓN todo-o-nada: la venta importada, sus detalles y los
    // ajustes de inventario se escriben como una sola unidad. Si cualquier
    // paso falla, SQLite revierte TODO (antes quedaba venta sin items o
    // stock descuadrado a medias).
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let result = if let Some(ref f_iso) = fecha_iso {
        sqlx::query("INSERT INTO ventas (total, subtotal, cajero, metodo_pago, fecha) VALUES (?, ?, ?, ?, ?)")
            .bind(a_centavos(total))
            .bind(a_centavos(total))
            .bind("IMPORTADOR")
            .bind(&metodo_pago)
            .bind(f_iso)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?
    } else {
        sqlx::query("INSERT INTO ventas (total, subtotal, cajero, metodo_pago) VALUES (?, ?, ?, ?)")
            .bind(a_centavos(total))
            .bind(a_centavos(total))
            .bind("IMPORTADOR")
            .bind(&metodo_pago)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?
    };

    let venta_id = result.last_insert_rowid();
    let total_items = items.len();

    // Precargar inventario para matching normalizado + fuzzy (evita LOWER frágil)
    // LOWER("ACEITE") nunca matchea "ACEITE 123 1L" y causaba fantasmas en el pasado.
    let rows_prod: Vec<(i64, String)> = sqlx::query_as::<_, (i64, String)>(
        "SELECT id, nombre FROM productos",
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    let mut mapa_norm: HashMap<String, Vec<i64>> = HashMap::new();
    for (id, nombre) in &rows_prod {
        mapa_norm
            .entry(normalizar(nombre))
            .or_default()
            .push(*id);
    }
    // Mapa exacto no ambiguo
    let mut productos_por_nombre: HashMap<String, Option<i64>> = HashMap::new();
    for (k, ids) in mapa_norm {
        productos_por_nombre.insert(k, if ids.len() == 1 { Some(ids[0]) } else { None });
    }
    // Cache de productos para fuzzy
    let inventario: Vec<(i64, String)> = rows_prod.clone();
    let embedder = HashEmbedder;

    let mut sin_vincular: usize = 0;

    for item in items {
        // Resolver producto_id: exacto -> fuzzy embedding -> None
        let norm = normalizar(&item.producto);
        let mut producto_id: Option<i64> = productos_por_nombre.get(&norm).copied().flatten();

        if producto_id.is_none() && !inventario.is_empty() {
            // Solo fuzzy si no hay exacto y no es ambiguo
            if !productos_por_nombre.contains_key(&norm) {
                if let Some(q_emb) = embedder.texto_a_embedding(&item.producto) {
                    let mut mejor: Option<(i64, f64)> = None;
                    let mut segundo = 0.0;
                    for (pid, nombre) in &inventario {
                        if let Some(p_emb) = embedder.texto_a_embedding(nombre) {
                            let score = cosine_similarity(&q_emb, &p_emb);
                            if let Some((_, best)) = mejor {
                                if score > best {
                                    segundo = best;
                                    mejor = Some((*pid, score));
                                } else if score > segundo {
                                    segundo = score;
                                }
                            } else {
                                mejor = Some((*pid, score));
                            }
                        }
                    }
                    if let Some((pid, best)) = mejor {
                        if best >= 0.55 && segundo < 0.52 {
                            producto_id = Some(pid);
                        }
                    }
                }
            }
        }

        // Insertar detalle con producto_id resuelto (puede ser None = sin vincular, sin fantasma)
        sqlx::query("INSERT INTO detalle_ventas (venta_id, producto_id, producto_nombre, cantidad, precio_unitario, subtotal) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(venta_id)
            .bind(producto_id)
            .bind(&item.producto)
            .bind(item.cantidad)
            .bind(a_centavos(item.precio))
            .bind(a_centavos(item.total))
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

        // Solo tocar stock si hay producto_id fiable. Si es None (truncado "ACEITE" ambiguo
        // o producto nuevo real), NO hacer UPDATE por LOWER — eso creaba stock -158 sin freno.
        if let Some(pid) = producto_id {
            sqlx::query("UPDATE productos SET stock = stock - ?, vendido = vendido + ? WHERE id = ?")
                .bind(item.cantidad)
                .bind(item.cantidad)
                .bind(pid)
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;
        } else {
            sin_vincular += 1;
        }
    }

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(if sin_vincular > 0 {
        format!(
            "Ticket importado ({}/{} items vinculados al inventario; {} sin coincidencia por nombre)",
            total_items - sin_vincular,
            total_items,
            sin_vincular
        )
    } else {
        "Ticket importado correctamente".into()
    })
}

#[tauri::command]
pub async fn get_predictions(
    days: i32,
    db_path: tauri::State<'_, DbPath>,
    auth: tauri::State<'_, AuthState>,
) -> Result<serde_json::Value, String> {
    auth.require_admin()?;

    let horizonte = validar_horizonte_prediccion(days)?;
    let ruta_db = PathBuf::from(db_path.0.clone());
    let data = tokio::task::spawn_blocking(move || {
        src_ia::predicciones::predecir_ventas(&ruta_db, horizonte)
    })
    .await
    .map_err(|e| format!("Falló el cálculo de predicciones: {e}"))??;

    // El frontend existente consume un envoltorio `{ data: [...] }`.
    Ok(serde_json::json!({ "data": data }))
}

fn validar_horizonte_prediccion(days: i32) -> Result<usize, String> {
    if !(1..=365).contains(&days) {
        return Err("El horizonte de predicción debe estar entre 1 y 365 días".to_string());
    }
    Ok(days as usize)
}

// ============================================================
// Vista Ventas: historial + pronóstico, KPIs y desglose por empleado.
// ============================================================

/// Histórico denso + pronóstico en un solo viaje: la gráfica dibuja la
/// curva continua sin pegar dos fuentes con fechas desfasadas.
#[tauri::command]
pub async fn get_ventas_con_pronostico(
    days: i32,
    db_path: tauri::State<'_, DbPath>,
    auth: tauri::State<'_, AuthState>,
) -> Result<serde_json::Value, String> {
    auth.require_admin()?;

    let horizonte = validar_horizonte_prediccion(days)?;
    let ruta_db = PathBuf::from(db_path.0.clone());
    let (historial, pronostico) =
        get_ventas_con_pronostico_impl(ruta_db, horizonte).await?;

    Ok(serde_json::json!({ "historial": historial, "pronostico": pronostico }))
}

/// Núcleo testeable: historia densa + pronóstico desde un archivo SQLite.
pub async fn get_ventas_con_pronostico_impl(
    ruta_db: PathBuf,
    horizonte: usize,
) -> Result<
    (
        Vec<src_ia::predicciones::PuntoHistorial>,
        Vec<src_ia::predicciones::PuntoConFecha>,
    ),
    String,
> {
    tokio::task::spawn_blocking(move || {
        src_ia::predicciones::historia_y_pronostico(&ruta_db, horizonte)
    })
    .await
    .map_err(|e| format!("Falló el cálculo de predicciones: {e}"))?
}

#[derive(serde::Serialize)]
struct KpiDia {
    total: f64,
    tickets: i64,
    ticket_promedio: f64,
    utilidad_neta: f64,
    margen_pct: f64,
}

/// KPIs de un bloque de días vs el bloque anterior, en una sola llamada
/// (totales, conteo, ticket promedio y utilidad neta con margen).
/// `dias` = tamaño del bloque, `desfase` = días hacia atrás (0 = hoy).
/// Hoy→(1,0), ayer→(1,1), últimos 7→(7,0). Robusto con alto volumen:
/// agrega en SQL en vez de paginar `get_tickets`.
#[tauri::command]
pub async fn get_kpis_ventas(
    dias: i32,
    desfase: Option<i32>,
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
) -> Result<serde_json::Value, String> {
    auth.require_admin()?;
    if !(1..=365).contains(&dias) {
        return Err("El rango debe estar entre 1 y 365 días".to_string());
    }
    let desfase = desfase.unwrap_or(0);
    if !(0..=365).contains(&desfase) {
        return Err("El desfase debe estar entre 0 y 365 días".to_string());
    }

    get_kpis_ventas_impl(&state, dias, desfase).await
}

/// Núcleo testeable: KPIs de un bloque de días vs el bloque anterior.
pub async fn get_kpis_ventas_impl(
    pool: &SqlitePool,
    dias: i32,
    desfase: i32,
) -> Result<serde_json::Value, String> {
    // Bloque actual: [hoy-desfase-dias+1, hoy-desfase]; anterior: los `dias` previos.
    let fin: String = sqlx::query_scalar("SELECT date('now','localtime', ?)")
        .bind(format!("-{} days", desfase))
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
    let inicio_actual: String =
        sqlx::query_scalar("SELECT date(?, ?)")
            .bind(&fin)
            .bind(format!("-{} days", dias - 1))
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;
    let fin_anterior: String = sqlx::query_scalar("SELECT date(?, '-1 day')")
        .bind(&inicio_actual)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
    let inicio_anterior: String = sqlx::query_scalar("SELECT date(?, ?)")
        .bind(&fin_anterior)
        .bind(format!("-{} days", dias - 1))
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

    async fn kpi(pool: &SqlitePool, inicio: &str, fin: &str) -> Result<KpiDia, String> {
        let (total_c, tickets): (i64, i64) = sqlx::query_as(
            "SELECT COALESCE(SUM(total), 0), COUNT(*) FROM ventas WHERE date(fecha) BETWEEN ? AND ? AND estado = 'completada'",
        )
        .bind(inicio)
        .bind(fin)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;
        let total = crate::dinero::a_pesos(total_c);
        let utilidad_neta =
            calcular_utilidad_neta_periodo(pool, inicio, fin).await?;
        let margen_pct = if total > 0.0 {
            utilidad_neta / total * 100.0
        } else {
            0.0
        };
        Ok(KpiDia {
            total,
            tickets,
            ticket_promedio: if tickets > 0 {
                total / tickets as f64
            } else {
                0.0
            },
            utilidad_neta,
            margen_pct,
        })
    }

    Ok(serde_json::json!({
        "actual": kpi(pool, &inicio_actual, &fin).await?,
        "anterior": kpi(pool, &inicio_anterior, &fin_anterior).await?,
        "inicio": inicio_actual,
        "fin": fin,
    }))
}

#[derive(serde::Serialize)]
pub struct VentaEmpleadoDia {
    pub fecha: String,
    pub cajero: String,
    pub total: f64,
}
/// Venta por cajero por día (últimos N días): alimenta las barras apiladas
/// de ganancia/venta por empleado. Sin cajero asignado → "SIN ASIGNAR".
#[tauri::command]
pub async fn get_ventas_por_empleado_dia(
    days: i32,
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
) -> Result<Vec<VentaEmpleadoDia>, String> {
    auth.require_admin()?;
    if !(1..=365).contains(&days) {
        return Err("El rango debe estar entre 1 y 365 días".to_string());
    }

    get_ventas_por_empleado_dia_impl(&state, days).await
}

/// Núcleo testeable: venta por cajero por día.
pub async fn get_ventas_por_empleado_dia_impl(
    pool: &SqlitePool,
    days: i32,
) -> Result<Vec<VentaEmpleadoDia>, String> {
    let rows = sqlx::query_as::<_, (String, Option<String>, i64)>(
        r#"SELECT date(fecha) as dia, cajero, COALESCE(SUM(total), 0)
           FROM ventas
           WHERE estado = 'completada' AND date(fecha) >= date('now','localtime', ?)
           GROUP BY dia, cajero
           ORDER BY dia ASC"#,
    )
    .bind(format!("-{} days", days - 1))
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|(fecha, cajero, total_c)| VentaEmpleadoDia {
            fecha,
            cajero: cajero
                .map(|c| {
                    let c = c.trim().to_string();
                    if c.is_empty() { "SIN ASIGNAR".to_string() } else { c }
                })
                .unwrap_or_else(|| "SIN ASIGNAR".to_string()),
            total: crate::dinero::a_pesos(total_c),
        })
        .collect())
}

#[derive(serde::Serialize)]
pub struct TopProducto {
    pub nombre: String,
    pub total: f64,
    pub cantidad: f64,
}

/// Top 5 productos por ingreso en los últimos N días (detalle real, no
/// paginado: robusto con alto volumen).
#[tauri::command]
pub async fn get_top_productos(
    days: i32,
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
) -> Result<Vec<TopProducto>, String> {
    auth.require_admin()?;
    if !(1..=365).contains(&days) {
        return Err("El rango debe estar entre 1 y 365 días".to_string());
    }

    get_top_productos_impl(&state, days).await
}

/// Núcleo testeable: top 5 productos por ingreso.
pub async fn get_top_productos_impl(
    pool: &SqlitePool,
    days: i32,
) -> Result<Vec<TopProducto>, String> {
    let rows = sqlx::query_as::<_, (String, i64, f64)>(
        r#"SELECT dv.producto_nombre, COALESCE(SUM(dv.subtotal), 0), COALESCE(SUM(dv.cantidad), 0)
           FROM detalle_ventas dv
           JOIN ventas v ON dv.venta_id = v.id
           WHERE v.estado = 'completada' AND date(v.fecha) >= date('now','localtime', ?)
           GROUP BY dv.producto_nombre
           ORDER BY SUM(dv.subtotal) DESC
           LIMIT 5"#,
    )
    .bind(format!("-{} days", days - 1))
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|(nombre, subtotal_c, cantidad)| TopProducto {
            nombre,
            total: crate::dinero::a_pesos(subtotal_c),
            cantidad,
        })
        .collect())
}
