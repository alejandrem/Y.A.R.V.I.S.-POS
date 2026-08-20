// ============================================================
// db.rs — Conexión a SQLite + migraciones versionadas (sqlx).
//
// El esquema NO vive en código: vive en `migrations/0001_inicial.sql`
// (y siguientes). sqlx valida el hash de cada migración aplicada, así
// que un cambio ad-hoc a una migración vieja rompe en el arranque en
// vez de corromper la DB silenciosamente. Los cambios de esquema a
// futuro = archivo nuevo en `migrations/`, nunca editar uno aplicado.
// ============================================================
use std::fs;

use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use sqlx::SqlitePool;
use tauri::Manager;

/// Estado simple para exponer la ruta de la DB al frontend.
pub struct DbPath(pub String);

/// Migraciones embebidas en el binario (compiladas desde `migrations/`).
/// `sqlx::migrate!` calcula el hash de cada archivo: modificar una migración
/// ya aplicada rompe el arranque a propósito (nunca edites una aplicada,
/// añade una nueva con numeración creciente).
static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub fn initialize_db(app: &tauri::AppHandle) -> (SqlitePool, String) {
    let app_dir = app
        .path()
        .app_data_dir()
        .expect("No se pudo obtener el directorio de datos");
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir).expect("No se pudo crear el directorio de datos");
    }

    let db_path = app_dir.join("yarvis.db");
    let db_path_str = db_path.to_string_lossy().to_string();

    tauri::async_runtime::block_on(async move {
        // Journal=WAL, FK y busy_timeout van en las OPTIONS de conexión y NO
        // como PRAGMA suelto al crear el pool: así aplican a CUALQUIER
        // conexión que sqlx abra, no solo a la primera.
        //
        // FIX (auditoría): sin foreign_keys, TODAS las FOREIGN KEY ...
        // ON DELETE CASCADE son decorativas (SQLite las trae apagadas por
        // default y borrar una venta dejaría huérfanos en detalle_ventas).
        // FIX (auditoría): sin busy_timeout, escrituras concurrentes desde
        // distintos comandos Tauri devuelven SQLITE_BUSY (reintento 5s).
        let options = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .foreign_keys(true)
            .busy_timeout(std::time::Duration::from_secs(5));

        let pool = SqlitePool::connect_with(options)
            .await
            .expect("Fallo al conectar a SQLite");

        // Aplica el esquema versionado (0001_inicial + futuras).
        // `sqlx::migrate!` embebe los .sql al compilar, así que el esquema
        // viaja DENTRO del binario (sigue siendo portable al 100%).
        MIGRATOR
            .run(&pool)
            .await
            .expect("Fallo al aplicar migraciones de la DB");

        (pool, db_path_str)
    })
}
