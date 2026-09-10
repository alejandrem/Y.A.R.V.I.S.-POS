// ============================================================
// importar — Importación masiva de catálogos con deduplicación.
// ============================================================
//
// Todo-o-nada por transacción: hash anti-duplicados (BUG-10), registro
// temprano del catálogo para vincular productos (BUG-02) y tope de 2
// productos con el mismo nombre. Con fallback para DBs viejas sin la
// columna catalogo_id (migración 0006).

use super::catalogo::{calcular_hash_catalogo, catalogo_ya_importado, contar_productos_por_nombre};
use crate::backventanas::auth::AuthState;
use crate::backventanas::codigos_barras::{
    mensaje_error_codigo, normalizar_codigo_barras, validar_codigo_barras,
};
use crate::models::InventoryItem;
use sqlx::SqlitePool;

#[tauri::command]
pub async fn importar_catalogo(
    state: tauri::State<'_, SqlitePool>,
    auth: tauri::State<'_, AuthState>,
    items: Vec<InventoryItem>,
    ruta_archivo: Option<String>,
    contenido_archivo: Option<String>,
) -> Result<String, String> {
    auth.require_admin()?;
    // 1. Verificar si el catálogo ya fue importado (por hash) — hash se calcula UNA vez (BUG-10)
    let hash_opt = contenido_archivo.as_ref().map(|c| calcular_hash_catalogo(c));
    if let Some(ref hash) = hash_opt {
        if catalogo_ya_importado(&*state, hash)
            .await
            .map_err(|e| e.to_string())?
        {
            return Err(
                "Este catálogo ya fue importado anteriormente. No se permiten duplicados."
                    .to_string(),
            );
        }
    }

    let mut tx = state.begin().await.map_err(|e| e.to_string())?;

    // 2. Registrar catálogo primero para obtener su id y poder vincular productos (BUG-02)
    let catalogo_id: Option<i64> = if let Some(hash) = hash_opt.as_deref() {
        let ruta = ruta_archivo.clone().unwrap_or_default();
        // Insert con total provisional 0, se actualizará al final si hace falta
        let res = sqlx::query(
            "INSERT INTO catalogos_importados (hash, ruta_archivo, total_productos) VALUES (?, ?, 0)",
        )
        .bind(hash)
        .bind(&ruta)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
        Some(res.last_insert_rowid())
    } else {
        None
    };

    // 3. Importar productos con deduplicación (máximo 2 con mismo nombre) y vinculados al catálogo
    let mut count = 0;
    let mut omitidos = 0;

    for item in items {
        let existentes = contar_productos_por_nombre(&mut *tx, &item.nombre)
            .await
            .map_err(|e| e.to_string())?;

        if existentes >= 2 {
            omitidos += 1;
            continue;
        }

        let codigo = normalizar_codigo_barras(item.codigo_barras.as_deref());
        if let Err(msg) = validar_codigo_barras(&codigo) {
            return Err(format!("Error insertando '{}': {}", item.nombre, msg));
        }

        // Intentar con catalogo_id (migración 0006), fallback sin él para DBs viejas sin migrar
        let res = sqlx::query("INSERT INTO productos (nombre, descripcion, precio_costo, precio_venta, stock, stock_minimo, codigo_barras, categoria, catalogo_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(&item.nombre)
            .bind(&item.descripcion)
            .bind(crate::dinero::a_centavos(item.precio_costo))
            .bind(crate::dinero::a_centavos(item.precio_venta))
            .bind(item.stock)
            .bind(item.stock_minimo)
            .bind(&codigo)
            .bind(&item.categoria)
            .bind(catalogo_id)
            .execute(&mut *tx)
            .await;

        let res = match res {
            Ok(r) => Ok(r),
            Err(e) if e.to_string().contains("no such column") || e.to_string().contains("has no column") => {
                sqlx::query("INSERT INTO productos (nombre, descripcion, precio_costo, precio_venta, stock, stock_minimo, codigo_barras, categoria) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
                    .bind(&item.nombre)
                    .bind(&item.descripcion)
                    .bind(crate::dinero::a_centavos(item.precio_costo))
                    .bind(crate::dinero::a_centavos(item.precio_venta))
                    .bind(item.stock)
                    .bind(item.stock_minimo)
                    .bind(&codigo)
                    .bind(&item.categoria)
                    .execute(&mut *tx)
                    .await
            }
            Err(e) => Err(e),
        };

        res.map_err(|e| {
            format!(
                "Error insertando '{}': {}",
                item.nombre,
                mensaje_error_codigo(e)
            )
        })?;

        count += 1;
    }

    // 4. Actualizar total_productos del catálogo ya insertado
    if let Some(cid) = catalogo_id {
        sqlx::query("UPDATE catalogos_importados SET total_productos = ? WHERE id = ?")
            .bind(count)
            .bind(cid)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;

    // 4. Retornar resultado con estadísticas
    let mensaje = if omitidos > 0 {
        format!(
            "Catálogo importado: {} productos insertados, {} omitidos por duplicados (máximo 2 con mismo nombre)",
            count, omitidos
        )
    } else {
        format!("Catálogo importado: {} productos", count)
    };

    Ok(mensaje)
}
