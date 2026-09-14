// ============================================================
// sugerencia — "Se debería pagar tanto" desde el costo registrado.
//
// Busca el producto por nombre normalizado exacto y no ambiguo;
// multiplica su precio_costo por la cantidad. Sin costo registrado
// devuelve None y la UI muestra "sin recomendación" (nunca inventa).
// Es solo sugerencia: el empleado escribe el monto final.
// ============================================================

use crate::backventanas::auth::AuthState;
use serde::Serialize;
use sqlx::SqlitePool;
use std::collections::HashMap;

#[derive(Serialize, Debug, PartialEq)]
pub struct SugerenciaPago {
    /// Total sugerido en pesos. None = sin recomendación.
    pub sugerido: Option<f64>,
    /// Costo unitario encontrado (0 si no hay).
    pub precio_costo: f64,
}

#[tauri::command]
pub async fn sugerir_pago(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    nombre_producto: String,
    cantidad: f64,
) -> Result<SugerenciaPago, String> {
    auth.require_operator()?;
    sugerir_pago_impl(&state, &nombre_producto, cantidad).await
}

/// Núcleo testeable sin runtime de Tauri.
pub async fn sugerir_pago_impl(
    pool: &SqlitePool,
    nombre_producto: &str,
    cantidad: f64,
) -> Result<SugerenciaPago, String> {
    if !cantidad.is_finite() || cantidad <= 0.0 {
        return Err("Cantidad inválida.".into());
    }
    let filas: Vec<(i64, String, i64)> =
        sqlx::query_as("SELECT id, nombre, precio_costo FROM productos")
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;

    let mut mapa: HashMap<String, Vec<(i64, i64)>> = HashMap::new();
    for (id, nombre, costo) in filas {
        mapa.entry(src_ia::embeddings::normalizar(&nombre)).or_default().push((id, costo));
    }
    let ids = mapa.get(&src_ia::embeddings::normalizar(nombre_producto));
    let costo_cents = match ids {
        Some(v) if v.len() == 1 => v[0].1,
        _ => 0,
    };
    let costo = crate::dinero::a_pesos(costo_cents);
    Ok(SugerenciaPago {
        sugerido: if costo > 0.0 { Some(costo * cantidad) } else { None },
        precio_costo: costo,
    })
}
