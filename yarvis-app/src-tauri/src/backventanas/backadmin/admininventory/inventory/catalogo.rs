// ============================================================
// catalogo — Huella e historial de catálogos importados.
// ============================================================
//
// Cada catálogo se identifica por SHA256 de su contenido: re-importar el
// mismo archivo se rechaza sin tocar la DB. Los helpers aceptan pool o
// transacción para participar de escrituras atómicas.

/// Calcula SHA256 del contenido del catálogo
pub(super) fn calcular_hash_catalogo(contenido: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(contenido.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Verifica si un catálogo ya fue importado (por hash).
/// Acepta pool o transacción para poder participar de escrituras atómicas.
pub(super) async fn catalogo_ya_importado<'a, E>(
    executor: E,
    hash: &str,
) -> Result<bool, sqlx::Error>
where
    E: sqlx::Executor<'a, Database = sqlx::Sqlite>,
{
    let result =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM catalogos_importados WHERE hash = ?")
            .bind(hash)
            .fetch_one(executor)
            .await?;
    Ok(result > 0)
}

#[allow(dead_code)] // Se usa vía INSERT directo en importar_catalogo para obtener el id
/// Registra un catálogo como importado (dentro de la transacción de importación).
pub(super) async fn registrar_catalogo_importado<'a, E>(
    executor: E,
    hash: &str,
    ruta: &str,
    total_productos: i32,
) -> Result<(), sqlx::Error>
where
    E: sqlx::Executor<'a, Database = sqlx::Sqlite>,
{
    sqlx::query(
        "INSERT INTO catalogos_importados (hash, ruta_archivo, total_productos) VALUES (?, ?, ?)",
    )
    .bind(hash)
    .bind(ruta)
    .bind(total_productos)
    .execute(executor)
    .await?;
    Ok(())
}

/// Cuenta cuántos productos con el mismo nombre ya existen en la DB
pub(super) async fn contar_productos_por_nombre<'a, E>(
    executor: E,
    nombre: &str,
) -> Result<i64, sqlx::Error>
where
    E: sqlx::Executor<'a, Database = sqlx::Sqlite>,
{
    let result = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM productos WHERE nombre = ?")
        .bind(nombre)
        .fetch_one(executor)
        .await?;
    Ok(result)
}

/// Struct para catálogos importados
#[derive(serde::Serialize)]
pub struct CatalogoImportado {
    pub id: i64,
    pub hash: String,
    pub ruta_archivo: String,
    pub fecha_importacion: String,
    pub total_productos: i64,
}
