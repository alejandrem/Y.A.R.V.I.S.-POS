// ============================================================
// historial — Lectura de cortes importados y su detalle completo.
// Los montos salen en pesos para el IPC (viven en centavos).
// La verificación se recalcula al leer: caja == ingresos − egresos
// y ventas == Σ renglones vendibles (ARTICULO/TICKET).
// ============================================================

use crate::backventanas::auth::AuthState;
use crate::dinero::a_pesos;
use sqlx::SqlitePool;

/// Renglón del historial.
#[derive(serde::Serialize, Debug)]
pub struct CorteImportadoRow {
    pub id: i64,
    pub tipo: String,
    pub folio: Option<String>,
    pub estacion: Option<String>,
    pub cajero: String,
    pub fecha: Option<String>,
    pub total_caja: f64,
    pub total_ventas: f64,
    pub clientes_atendidos: i64,
    /// Chequeo rápido sin re-parsear (el detalle trae el completo).
    pub verificado: bool,
}

#[tauri::command]
pub async fn get_cortes_importados(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<CorteImportadoRow>, String> {
    auth.require_admin()?;
    let limit = limit.unwrap_or(100).clamp(1, 500);
    let offset = offset.unwrap_or(0).max(0);
    let rows = sqlx::query(
        "SELECT id, tipo, folio, estacion, cajero, fecha, total_ingresos,
                total_egresos, total_caja, total_ventas, clientes_atendidos
         FROM cortes_importados ORDER BY fecha DESC, id DESC LIMIT ? OFFSET ?",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    use sqlx::Row;
    Ok(rows
        .into_iter()
        .map(|r| {
            let ing: i64 = r.try_get("total_ingresos").unwrap_or(0);
            let egr: i64 = r.try_get("total_egresos").unwrap_or(0);
            let caja: i64 = r.try_get("total_caja").unwrap_or(0);
            let ventas: i64 = r.try_get("total_ventas").unwrap_or(0);
            CorteImportadoRow {
                id: r.try_get("id").unwrap_or(0),
                tipo: r.try_get("tipo").unwrap_or_default(),
                folio: r.try_get("folio").ok().flatten(),
                estacion: r.try_get("estacion").ok().flatten(),
                cajero: r.try_get("cajero").unwrap_or_default(),
                fecha: r.try_get("fecha").ok().flatten(),
                total_caja: a_pesos(caja),
                total_ventas: a_pesos(ventas),
                clientes_atendidos: r.try_get("clientes_atendidos").unwrap_or(0),
                verificado: caja == ing - egr,
            }
        })
        .collect())
}

/// Renglón del detalle.
#[derive(serde::Serialize, Debug)]
pub struct ItemImportado {
    pub kind: String,
    pub nombre: String,
    pub cantidad: Option<f64>,
    pub precio_unitario: f64,
    pub subtotal: f64,
}

/// Detalle completo de un corte importado.
#[derive(serde::Serialize, Debug)]
pub struct CorteImportadoDetalle {
    pub id: i64,
    pub tipo: String,
    pub folio: Option<String>,
    pub estacion: Option<String>,
    pub cajero: String,
    pub empresa: Option<String>,
    pub moneda: String,
    pub fecha: Option<String>,
    pub total_ingresos: f64,
    pub total_egresos: f64,
    pub total_caja: f64,
    pub total_ventas: f64,
    pub ventas_gravadas: f64,
    pub impuesto: f64,
    pub ventas_no_gravadas: f64,
    pub redondeos: f64,
    pub ventas_credito: f64,
    pub total_unidades: f64,
    pub clientes_atendidos: i64,
    pub caja_ok: bool,
    pub ventas_ok: bool,
    pub items: Vec<ItemImportado>,
}

#[tauri::command]
pub async fn get_corte_importado_detalle(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    corte_id: i64,
) -> Result<CorteImportadoDetalle, String> {
    auth.require_admin()?;
    use sqlx::Row;
    let r = sqlx::query("SELECT * FROM cortes_importados WHERE id = ?")
        .bind(corte_id)
        .fetch_optional(&*state)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Corte no encontrado".to_string())?;

    let get = |col: &str| -> i64 { r.try_get(col).unwrap_or(0) };
    let ing = get("total_ingresos");
    let egr = get("total_egresos");
    let caja = get("total_caja");
    let ventas = get("total_ventas");

    let filas = sqlx::query(
        "SELECT kind, nombre, cantidad, precio_unitario, subtotal
         FROM cortes_importados_items WHERE corte_id = ? ORDER BY id ASC",
    )
    .bind(corte_id)
    .fetch_all(&*state)
    .await
    .map_err(|e| e.to_string())?;

    let mut suma_vendible: i64 = 0;
    let items: Vec<ItemImportado> = filas
        .into_iter()
        .map(|f| {
            let kind: String = f.try_get("kind").unwrap_or_default();
            let sub: i64 = f.try_get("subtotal").unwrap_or(0);
            if kind == "ARTICULO" || kind == "TICKET" {
                suma_vendible += sub;
            }
            ItemImportado {
                kind,
                nombre: f.try_get("nombre").unwrap_or_default(),
                cantidad: f.try_get("cantidad").ok().flatten(),
                precio_unitario: a_pesos(f.try_get("precio_unitario").unwrap_or(0)),
                subtotal: a_pesos(sub),
            }
        })
        .collect();

    Ok(CorteImportadoDetalle {
        id: r.try_get("id").unwrap_or(0),
        tipo: r.try_get("tipo").unwrap_or_default(),
        folio: r.try_get("folio").ok().flatten(),
        estacion: r.try_get("estacion").ok().flatten(),
        cajero: r.try_get("cajero").unwrap_or_default(),
        empresa: r.try_get("empresa").ok().flatten(),
        moneda: r.try_get("moneda").unwrap_or_default(),
        fecha: r.try_get("fecha").ok().flatten(),
        total_ingresos: a_pesos(ing),
        total_egresos: a_pesos(egr),
        total_caja: a_pesos(caja),
        total_ventas: a_pesos(ventas),
        ventas_gravadas: a_pesos(get("ventas_gravadas")),
        impuesto: a_pesos(get("impuesto")),
        ventas_no_gravadas: a_pesos(get("ventas_no_gravadas")),
        redondeos: a_pesos(get("redondeos")),
        ventas_credito: a_pesos(get("ventas_credito")),
        total_unidades: r.try_get("total_unidades").unwrap_or(0.0),
        clientes_atendidos: r.try_get("clientes_atendidos").unwrap_or(0),
        caja_ok: caja == ing - egr,
        ventas_ok: ventas == suma_vendible,
        items,
    })
}
