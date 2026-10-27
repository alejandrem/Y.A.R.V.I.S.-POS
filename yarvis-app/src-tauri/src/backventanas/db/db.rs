// ============================================================
// db.rs — Conexión a SQLite + migraciones versionadas (sqlx).
//
// El esquema NO vive en código: vive en `migrations/0001_inicial.sql`
// (y siguientes). sqlx valida el hash de cada migración aplicada, así
// que un cambio ad-hoc a una migración vieja rompe en el arranque en
// vez de corromper la DB silenciosamente. Los cambios de esquema a
// futuro = archivo nuevo en `migrations/`, nunca editar uno aplicado.
//
// IMPORTANTE: esta función ya NO hace panic!. Devuelve `ErrorDb` para
// que `arranque` muestre un diálogo nativo con log + restaurar respaldo.
// Un panic dentro del hilo de arranque dejaba el proceso vivo, la
// ventana principal vacía y ningún comando funcionando.
// ============================================================

use std::fs;
use std::path::{Path, PathBuf};

use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use sqlx::{Row, SqlitePool};
use tauri::Manager;

use crate::backventanas::db::{error::ErrorDb, respaldo};

/// Estado simple para exponer la ruta de la DB al frontend.
pub struct DbPath(pub String);

/// Migraciones embebidas en el binario (compiladas desde `migrations/`).
/// `sqlx::migrate!` calcula el hash de cada archivo: modificar una migración
/// ya aplicada rompe el arranque a propósito (nunca edites una aplicada,
/// añade una nueva con numeración creciente).
static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// Checksum de la migración 23 que quedó aplicada antes de que se añadiera
/// `sesion_inicio`. Solo se acepta junto con la validación del esquema abajo.
const CHECKSUM_V23_ANTIGUO: &str =
    "1C2EDABBD3ABC42EFBA67ED92BC8AFCEF4652430B250E4ECD395F4412539EE8C7043F820BFF88B24A4CB34341E1E7F7A";

/// Carpeta de datos de la app (donde viven yarvis.db, backups/ y logs/).
pub fn ruta_db(app: &tauri::AppHandle) -> Result<PathBuf, ErrorDb> {
    let app_dir = app.path().app_data_dir().map_err(|e| {
        ErrorDb::directorio(format!(
            "No se pudo resolver el directorio de datos de la aplicación: {e}"
        ))
    })?;
    Ok(app_dir.join("yarvis.db"))
}

/// Inicializa SQLite y aplica migraciones. Nunca entra en panic:
/// cualquier fallo se devuelve para que el arranque lo atienda.
pub fn initialize_db(app: &tauri::AppHandle) -> Result<(SqlitePool, String), ErrorDb> {
    let app_dir = app.path().app_data_dir().map_err(|e| {
        ErrorDb::directorio(format!(
            "No se pudo resolver el directorio de datos de la aplicación: {e}"
        ))
    })?;
    tauri::async_runtime::block_on(inicializar_en(&app_dir))
}

/// Núcleo testeable: recibe la carpeta de datos, no un AppHandle.
pub(crate) async fn inicializar_en(app_dir: &Path) -> Result<(SqlitePool, String), ErrorDb> {
    if !app_dir.exists() {
        fs::create_dir_all(app_dir).map_err(|e| {
            ErrorDb::directorio(format!(
                "No se pudo crear el directorio de datos {}: {e}",
                app_dir.display()
            ))
        })?;
    }

    let db_path = app_dir.join("yarvis.db");
    let db_path_str = db_path.to_string_lossy().to_string();

    // Backup automático ANTES de migrar. Nunca tumba el arranque: si
    // falla, solo se registra el warning y se intenta migrar igual.
    if db_path.exists() {
        match respaldo::backup_db_antes_de_migrar(app_dir, &db_path) {
            Some(ruta) => tracing::info!("[DB] backup pre-migración: {}", ruta.display()),
            None => tracing::warn!("[DB] no se pudo crear backup pre-migración, se continúa sin él"),
        }
    }

    // Journal=WAL, FK y busy_timeout van en las OPTIONS de conexión y NO
    // como PRAGMA suelto al crear el pool: así aplican a CUALQUIER
    // conexión que sqlx abra, no solo a la primera.
    let base_options = SqliteConnectOptions::new()
        .filename(&db_path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(std::time::Duration::from_secs(5));

    // FASE 1 — migraciones SIN foreign_keys. sqlx-sqlite envuelve cada
    // migración en una transacción y SQLite ignora PRAGMA foreign_keys
    // dentro de una transacción; la reconstrucción de 0005 necesita FKs
    // apagadas desde la conexión.
    let pool_migraciones = SqlitePool::connect_with(base_options.clone().foreign_keys(false))
        .await
        .map_err(|e| {
            ErrorDb::conexion(format!(
                "No se pudo conectar a {}: {e}",
                db_path.display()
            ))
        })?;

    if let Err(e) = reparar_v23_conocida(&pool_migraciones).await {
        pool_migraciones.close().await;
        return Err(ErrorDb::operacion(format!(
            "No se pudo completar la compatibilidad de la migración 23: {e}"
        )));
    }

    if let Err(e) = MIGRATOR.run(&pool_migraciones).await {
        pool_migraciones.close().await;
        return Err(ErrorDb::migracion(e, &db_path));
    }

    pool_migraciones.close().await;

    // FASE 2 — operación normal CON foreign_keys activas: integridad
    // referencial real para ventas/detalles/inventario.
    let pool = SqlitePool::connect_with(base_options.foreign_keys(true))
        .await
        .map_err(|e| {
            ErrorDb::conexion(format!(
                "No se pudo reconectar a {}: {e}",
                db_path.display()
            ))
        })?;

    // Mantenimiento WAL (best-effort: nunca debe tumbar el arranque).
    if let Err(e) = sqlx::query("PRAGMA journal_size_limit = 33554432")
        .execute(&pool)
        .await
    {
        tracing::warn!("[DB] no se pudo fijar journal_size_limit: {e}");
    }
    if let Err(e) = sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
        .execute(&pool)
        .await
    {
        tracing::warn!("[DB] checkpoint inicial omitido: {e}");
    }

    Ok((pool, db_path_str))
}

/// Repara únicamente la transición conocida de v23: algunas instalaciones
/// ya tienen `turno_activo`, pero su checksum es el anterior y falta
/// `sesion_inicio`. No se usa para ignorar otros VersionMismatch.
async fn reparar_v23_conocida(pool: &SqlitePool) -> Result<(), String> {
    let existe_migraciones: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = '_sqlx_migrations'",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;
    if existe_migraciones.is_none() {
        return Ok(());
    }

    let aplicada = sqlx::query(
        "SELECT UPPER(HEX(checksum)) AS checksum, success
         FROM _sqlx_migrations WHERE version = 23",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;
    let Some(aplicada) = aplicada else {
        return Ok(());
    };

    let checksum: String = aplicada.try_get("checksum").map_err(|e| e.to_string())?;
    let success: bool = aplicada.try_get("success").map_err(|e| e.to_string())?;
    if !success || checksum != CHECKSUM_V23_ANTIGUO {
        return Ok(());
    }

    let columnas = sqlx::query("PRAGMA table_info(asistencias)")
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|row| row.try_get::<String, _>("name").map_err(|e| e.to_string()))
        .collect::<Result<std::collections::HashSet<_>, _>>()?;
    for requerida in ["id", "empleado_id", "fecha", "ultimo_login", "turno_activo"] {
        if !columnas.contains(requerida) {
            return Err(format!(
                "la base tiene el checksum antiguo de v23, pero falta la columna esperada `{requerida}` en asistencias; no se modificó"
            ));
        }
    }

    let migracion_actual = MIGRATOR
        .iter()
        .find(|m| m.version == 23)
        .ok_or_else(|| "no se encontró la migración 23 en el ejecutable".to_string())?;
    let checksum_actual = migracion_actual.checksum.as_ref();
    let checksum_anterior = (0..CHECKSUM_V23_ANTIGUO.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&CHECKSUM_V23_ANTIGUO[i..i + 2], 16))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("checksum conocido inválido: {e}"))?;

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    if !columnas.contains("sesion_inicio") {
        sqlx::query("ALTER TABLE asistencias ADD COLUMN sesion_inicio TEXT")
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }
    let actualizadas = sqlx::query(
        "UPDATE _sqlx_migrations SET checksum = ?
         WHERE version = 23 AND success = 1 AND checksum = ?",
    )
    .bind(checksum_actual)
    .bind(checksum_anterior)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?
    .rows_affected();
    if actualizadas != 1 {
        return Err("el registro de v23 cambió durante la recuperación; se revirtió la transacción".to_string());
    }
    tx.commit().await.map_err(|e| e.to_string())?;
    tracing::warn!("[DB] compatibilidad aplicada para el checksum antiguo conocido de v23; se continuará con la migración 24");
    Ok(())
}

#[cfg(test)]
mod tests;
