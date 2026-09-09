// ============================================================
// empleatickets — "Mis tickets" del operador actual.
//
// Todo aquí es operator-scoped: el cajero sale de la sesión
// (`session.user_id` + `session.name`), NUNCA de un parámetro del
// frontend (un nombre por parámetro se puede suplantar; el de la
// sesión no). Mismo patrón que `get_employee_profile` cuando
// `role == Employee`.
//
// Vinculación en dos niveles: primero la canónica (`cajero_id`);
// como respaldo, ventas históricas nunca vinculadas (`cajero_id
// IS NULL`) cuya etiqueta `cajero` es exactamente mi nombre — pasa
// cuando el usuario se creó manual DESPUÉS de importar tickets o la
// migración 0003 no pudo vincularlas. Las de IMPORTADOR/SISTEMA
// nunca caen aquí (su etiqueta no es mi nombre).
//
// El dinero vive en INTEGER centavos y sale en pesos vía a_pesos.
// ============================================================

use crate::backventanas::auth::AuthState;
use crate::dinero::a_pesos;
use serde::Serialize;
use sqlx::SqlitePool;

/// Límite máximo por página (igual que el historial admin: 12k+
/// tickets de golpe congelan el webview).
pub const MIS_TICKETS_LIMITE_MAX: i64 = 500;

/// Ticket propio resumido para la lista.
#[derive(Serialize, Debug, PartialEq)]
pub struct MiTicket {
    pub id: i64,
    pub folio_ticket: Option<String>,
    pub fecha: String,
    pub total: f64,
    pub metodo_pago: String,
}

/// Agregado del periodo seleccionado (el rango lo elige el KPI).
#[derive(Serialize, Debug, PartialEq)]
pub struct MisKpis {
    pub total: f64,
    pub tickets: i64,
    pub ticket_promedio: f64,
}

/// Venta agrupada por día para la gráfica vs sueldo.
#[derive(Serialize, Debug, PartialEq, Clone)]
pub struct VentaDia {
    /// Día local "YYYY-MM-DD".
    pub fecha: String,
    pub total: f64,
}

/// Renglón del detalle de un ticket propio.
#[derive(Serialize, Debug, PartialEq)]
pub struct MiTicketItem {
    pub producto_nombre: String,
    pub cantidad: f64,
    pub precio_unitario: f64,
    pub subtotal: f64,
}

/// Detalle completo de un ticket propio (modal).
#[derive(Serialize, Debug, PartialEq)]
pub struct MiTicketDetalle {
    pub id: i64,
    pub folio_ticket: Option<String>,
    pub fecha: String,
    pub total: f64,
    pub subtotal: f64,
    pub descuento: f64,
    pub metodo_pago: String,
    pub items: Vec<MiTicketItem>,
}

/// Ventana máxima de días: el rango "Todos" del frontend manda esto
/// (~100 años, cubre todo el historial real sin quitar el filtro de
/// fecha de los queries).
pub const DIAS_TODOS: i64 = 36500;

/// Días válidos para los filtros: [1, DIAS_TODOS].
fn normalizar_dias(dias: Option<i64>) -> i64 {
    dias.unwrap_or(1).clamp(1, DIAS_TODOS)
}

/// Recorta días al rango válido en los núcleos testeables.
fn acotar_dias(dias: i64) -> i64 {
    dias.clamp(1, DIAS_TODOS)
}

// ── Mis tickets (paginado, solo míos) ────────────────────────

#[tauri::command]
pub async fn get_mis_tickets(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    limit: Option<i64>,
    offset: Option<i64>,
    dias: Option<i64>,
) -> Result<Vec<MiTicket>, String> {
    let session = auth.require_operator()?;
    mis_tickets_impl(
        &state,
        session.user_id,
        &session.name,
        limit.unwrap_or(100),
        offset.unwrap_or(0),
        normalizar_dias(dias),
    )
    .await
}

/// Núcleo testeable sin runtime de Tauri.
pub async fn mis_tickets_impl(
    pool: &SqlitePool,
    cajero_id: i64,
    cajero_nombre: &str,
    limit: i64,
    offset: i64,
    dias: i64,
) -> Result<Vec<MiTicket>, String> {
    let limit = limit.clamp(1, MIS_TICKETS_LIMITE_MAX);
    let offset = offset.max(0);
    let dias = acotar_dias(dias);
    // Ventana inclusiva: dias=1 → solo hoy; dias=7 → hoy + 6 atrás.
    let rows = sqlx::query_as::<_, (i64, Option<String>, String, i64, String)>(
        "SELECT id, folio_ticket, strftime('%Y-%m-%d %H:%M:%S', fecha) as fecha, total, metodo_pago
         FROM ventas
         WHERE estado = 'completada'
           AND (cajero_id = ?1 OR (cajero_id IS NULL AND cajero = ?2))
           AND date(fecha) >= date('now','localtime', printf('-%d days', ?3))
         ORDER BY fecha DESC LIMIT ?4 OFFSET ?5",
    )
    .bind(cajero_id)
    .bind(cajero_nombre)
    .bind(dias - 1)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|r| MiTicket {
            id: r.0,
            folio_ticket: r.1,
            fecha: r.2,
            total: a_pesos(r.3),
            metodo_pago: r.4,
        })
        .collect())
}

// ── Mis KPIs del periodo ─────────────────────────────────────

#[tauri::command]
pub async fn get_mis_kpis(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    dias: Option<i64>,
) -> Result<MisKpis, String> {
    let session = auth.require_operator()?;
    mis_kpis_impl(&state, session.user_id, &session.name, normalizar_dias(dias)).await
}

/// Núcleo testeable sin runtime de Tauri.
pub async fn mis_kpis_impl(
    pool: &SqlitePool,
    cajero_id: i64,
    cajero_nombre: &str,
    dias: i64,
) -> Result<MisKpis, String> {
    let dias = acotar_dias(dias);
    let row = sqlx::query_as::<_, (i64, i64)>(
        "SELECT COALESCE(SUM(total), 0), COUNT(*)
         FROM ventas
         WHERE estado = 'completada'
           AND (cajero_id = ?1 OR (cajero_id IS NULL AND cajero = ?2))
           AND date(fecha) >= date('now','localtime', printf('-%d days', ?3))",
    )
    .bind(cajero_id)
    .bind(cajero_nombre)
    .bind(dias - 1)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())?;

    let total = a_pesos(row.0);
    let tickets = row.1;
    Ok(MisKpis {
        total,
        tickets,
        ticket_promedio: if tickets > 0 {
            total / tickets as f64
        } else {
            0.0
        },
    })
}

// ── Mis ventas por día (gráfica vs sueldo) ───────────────────

#[tauri::command]
pub async fn get_mis_ventas_por_dia(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    dias: Option<i64>,
) -> Result<Vec<VentaDia>, String> {
    let session = auth.require_operator()?;
    mis_ventas_por_dia_impl(&state, session.user_id, &session.name, normalizar_dias(dias)).await
}

/// Núcleo testeable sin runtime de Tauri. Solo días con venta;
/// el frontend rellena los huecos con ceros (igual que admin).
pub async fn mis_ventas_por_dia_impl(
    pool: &SqlitePool,
    cajero_id: i64,
    cajero_nombre: &str,
    dias: i64,
) -> Result<Vec<VentaDia>, String> {
    let dias = acotar_dias(dias);
    let rows = sqlx::query_as::<_, (String, i64)>(
        "SELECT date(fecha) as dia, SUM(total)
         FROM ventas
         WHERE estado = 'completada'
           AND (cajero_id = ?1 OR (cajero_id IS NULL AND cajero = ?2))
           AND date(fecha) >= date('now','localtime', printf('-%d days', ?3))
         GROUP BY dia ORDER BY dia ASC",
    )
    .bind(cajero_id)
    .bind(cajero_nombre)
    .bind(dias - 1)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|r| VentaDia {
            fecha: r.0,
            total: a_pesos(r.1),
        })
        .collect())
}

// ── Detalle de un ticket propio (modal) ──────────────────────

#[tauri::command]
pub async fn get_mi_ticket_detalle(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    venta_id: i64,
) -> Result<MiTicketDetalle, String> {
    let session = auth.require_operator()?;
    mi_ticket_detalle_impl(&state, session.user_id, &session.name, venta_id).await
}

/// Núcleo testeable sin runtime de Tauri. Si el ticket no es del
/// operador (o no existe) devuelve el mismo error genérico para
/// no revelar tickets ajenos.
pub async fn mi_ticket_detalle_impl(
    pool: &SqlitePool,
    cajero_id: i64,
    cajero_nombre: &str,
    venta_id: i64,
) -> Result<MiTicketDetalle, String> {
    let cab = sqlx::query_as::<_, (i64, Option<String>, String, i64, i64, i64, String)>(
        "SELECT id, folio_ticket, strftime('%Y-%m-%d %H:%M:%S', fecha) as fecha,
                total, COALESCE(subtotal, total), COALESCE(descuento, 0), metodo_pago
         FROM ventas
         WHERE id = ?1 AND estado = 'completada'
           AND (cajero_id = ?2 OR (cajero_id IS NULL AND cajero = ?3))",
    )
    .bind(venta_id)
    .bind(cajero_id)
    .bind(cajero_nombre)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    let c = match cab {
        Some(c) => c,
        None => return Err("Ticket no encontrado".into()),
    };

    let filas = sqlx::query_as::<_, (String, f64, i64, i64)>(
        "SELECT producto_nombre, cantidad, precio_unitario, subtotal
         FROM detalle_ventas WHERE venta_id = ? ORDER BY id ASC",
    )
    .bind(venta_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(MiTicketDetalle {
        id: c.0,
        folio_ticket: c.1,
        fecha: c.2,
        total: a_pesos(c.3),
        subtotal: a_pesos(c.4),
        descuento: a_pesos(c.5),
        metodo_pago: c.6,
        items: filas
            .into_iter()
            .map(|f| MiTicketItem {
                producto_nombre: f.0,
                cantidad: f.1,
                precio_unitario: a_pesos(f.2),
                subtotal: a_pesos(f.3),
            })
            .collect(),
    })
}
