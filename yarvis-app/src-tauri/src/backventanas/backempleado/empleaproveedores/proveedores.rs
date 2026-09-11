// ============================================================
// proveedores — Alta y listado (el empleado puede agregar).
//
// Solo 3 campos: nombre forzoso, teléfono y correo opcionales.
// Sin duplicados silenciosos: UNIQUE NOCASE en DB + mensaje amigable.
// ============================================================

use crate::backventanas::auth::AuthState;
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Serialize, Debug, PartialEq)]
pub struct Proveedor {
    pub id: i64,
    pub nombre: String,
    pub telefono: Option<String>,
    pub correo: Option<String>,
    pub total_compras: i64,
    pub total_pagado: f64,
}

fn limpiar_opt(v: Option<String>) -> Option<String> {
    v.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

#[tauri::command]
pub async fn guardar_proveedor(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    nombre: String,
    telefono: Option<String>,
    correo: Option<String>,
) -> Result<i64, String> {
    auth.require_operator()?;
    guardar_proveedor_impl(&state, nombre, telefono, correo).await
}

/// Núcleo testeable sin runtime de Tauri.
pub async fn guardar_proveedor_impl(
    pool: &SqlitePool,
    nombre: String,
    telefono: Option<String>,
    correo: Option<String>,
) -> Result<i64, String> {
    let nombre = nombre.trim().to_string();
    if nombre.is_empty() {
        return Err("Escribe el nombre del proveedor.".into());
    }
    let r = sqlx::query("INSERT INTO proveedores (nombre, telefono, correo) VALUES (?, ?, ?)")
        .bind(&nombre)
        .bind(limpiar_opt(telefono))
        .bind(limpiar_opt(correo))
        .execute(pool)
        .await
        .map_err(|e| {
            let m = e.to_string();
            if m.contains("idx_proveedores_nombre") || m.contains("proveedores.nombre") {
                "Ese proveedor ya está registrado.".to_string()
            } else {
                m
            }
        })?;
    Ok(r.last_insert_rowid())
}

#[tauri::command]
pub async fn listar_proveedores(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
) -> Result<Vec<Proveedor>, String> {
    auth.require_operator()?;
    listar_proveedores_impl(&state).await
}

/// Núcleo testeable sin runtime de Tauri. Orden alfabético
/// insensible a mayúsculas + agregados por proveedor.
pub async fn listar_proveedores_impl(pool: &SqlitePool) -> Result<Vec<Proveedor>, String> {
    let rows = sqlx::query_as::<_, (i64, String, Option<String>, Option<String>, i64, i64)>(
        "SELECT p.id, p.nombre, p.telefono, p.correo,
                COUNT(c.id), COALESCE(SUM(c.monto_pagado), 0)
         FROM proveedores p LEFT JOIN compras c ON c.proveedor_id = p.id
         GROUP BY p.id ORDER BY p.nombre COLLATE NOCASE ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|r| Proveedor {
            id: r.0,
            nombre: r.1,
            telefono: r.2,
            correo: r.3,
            total_compras: r.4,
            total_pagado: crate::dinero::a_pesos(r.5),
        })
        .collect())
}

/// Alta exprés para pagos sin proveedor identificado: genera
/// MOSTRADOR 00001, 00002... (primer hueco libre, anti-duplicado
/// por el UNIQUE NOCASE aunque dos cajas lo pidan a la vez).
#[tauri::command]
pub async fn crear_proveedor_generico(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
) -> Result<Proveedor, String> {
    auth.require_operator()?;
    crear_proveedor_generico_impl(&state).await
}

/// Núcleo testeable sin runtime de Tauri.
pub async fn crear_proveedor_generico_impl(pool: &SqlitePool) -> Result<Proveedor, String> {
    for n in 1..=99999 {
        let nombre = format!("MOSTRADOR {n:05}");
        match guardar_proveedor_impl(pool, nombre, None, None).await {
            Ok(id) => {
                return Ok(Proveedor {
                    id,
                    nombre: format!("MOSTRADOR {n:05}"),
                    telefono: None,
                    correo: None,
                    total_compras: 0,
                    total_pagado: 0.0,
                })
            }
            Err(e) if e == "Ese proveedor ya está registrado." => continue,
            Err(e) => return Err(e),
        }
    }
    Err("No hay folios genéricos libres.".into())
}
