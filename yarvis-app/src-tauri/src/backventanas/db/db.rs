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
use std::path::{Path, PathBuf};

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

/// Cuántos backups se conservan (rotación simple, los más nuevos).
pub const MAX_BACKUPS: usize = 5;

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

    // Backup automático ANTES de migrar: si la migración falla a la mitad,
    // el dueño puede restaurar el .db de `backups/`. Nunca debe tumbar el
    // arranque: un backup fallido solo se registra con warn.
    if db_path.exists() {
        match backup_db_antes_de_migrar(&app_dir, &db_path) {
            Some(ruta) => tracing::info!("[DB] backup pre-migración: {}", ruta.display()),
            None => tracing::warn!("[DB] no se pudo crear backup pre-migración, se continúa sin él"),
        }
    }

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
        let base_options = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(std::time::Duration::from_secs(5));

        // FASE 1 — migraciones SIN foreign_keys.
        // Por qué: sqlx-sqlite envuelve SIEMPRE cada migración en una
        // transacción (ignora el marcador `-- no-transaction`) y SQLite
        // ignora `PRAGMA foreign_keys` DENTRO de una transacción. La
        // reconstrucción de tablas de 0005 necesita FKs apagadas, así que
        // el flag debe venir apagado desde la propia conexión.
        let pool_migraciones = SqlitePool::connect_with(base_options.clone().foreign_keys(false))
            .await
            .expect("Fallo al conectar a SQLite");

        // Aplica el esquema versionado (0001_inicial + futuras).
        // `sqlx::migrate!` embebe los .sql al compilar, así que el esquema
        // viaja DENTRO del binario (sigue siendo portable al 100%).
        //
        // BLINDAJE VersionMismatch (fail-closed, issue #1): sqlx valida el
        // checksum de cada migración ya aplicada. Si el .sql cambió de hash
        // sin cambiar de versión (típico: checkout Windows convirtió
        // LF->CRLF, o se editó una migración vieja), la app NO arranca y
        // NO toca la DB: el backup pre-migración ya preserva los datos.
        // Abrir una tienda vacía ("primer inicio") sobre un mismatch sería
        // bifurcar la historia y hacer creer al dueño que perdió todo.
        // Prevención real: `.gitattributes` fuerza `*.sql eol=lf` y las
        // migraciones viejas nunca se editan (solo se añade 0011_...).
        if let Err(e) = MIGRATOR.run(&pool_migraciones).await {
            match e {
                sqlx::migrate::MigrateError::VersionMismatch(version) => {
                    panic!(
                        "[DB] VersionMismatch en migración v{version}: el .sql embebido \
                         no coincide con el aplicado en {}. La base de datos NO se modificó \
                         y hay un backup previo en backups/. Causas típicas: se editó una \
                         migración ya aplicada o un checkout cambió LF->CRLF (revisa \
                         .gitattributes eol=lf). Restaura el backup o corrige las \
                         migraciones y vuelve a arrancar. Ver issue #1.",
                        db_path.display()
                    );
                }
                otro => panic!(
                    "Fallo al aplicar migraciones de la DB en {}: {otro} \
                     (backup previo en backups/; si es VersionMismatch, revisa \
                     .gitattributes eol=lf y no edites migraciones aplicadas)",
                    db_path.display()
                ),
            }
        }

        pool_migraciones.close().await;

        // FASE 2 — operación normal CON foreign_keys activas: integridad
        // referencial real para ventas/detalles/inventario.
        let pool = SqlitePool::connect_with(base_options.foreign_keys(true))
            .await
            .expect("Fallo al reconectar a SQLite");

        // Mantenimiento WAL (issue #1, best-effort: nunca debe tumbar el
        // arranque). Acota el -wal a 32 MiB y lo compacta al arrancar para
        // que no crezca sin fin en la PC de la tienda.
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

        (pool, db_path_str)
    })
}

/// Crea una copia consistente de `yarvis.db` en `<app_dir>/backups/` con
/// nombre `yarvis-YYYYMMDD-HHMMSS.db` y rota para conservar solo los
/// últimos [`MAX_BACKUPS`].
///
/// Usa `VACUUM INTO` (snapshot consistente aunque haya WAL pendiente) con
/// fallback a `fs::copy` si VACUUM falla. Devuelve la ruta del backup o
/// `None` si no se pudo crear (el arranque debe continuar igual).
pub fn backup_db_antes_de_migrar(app_dir: &Path, db_path: &Path) -> Option<PathBuf> {
    let backups_dir = app_dir.join("backups");
    if fs::create_dir_all(&backups_dir).is_err() {
        return None;
    }
    let sello = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
    let destino = backups_dir.join(format!("yarvis-{sello}.db"));

    // Ruta 1 (preferida): VACUUM INTO produce un .db compacto y consistente
    // sin copiar los -wal/-shm a mano. El open sale del helper compartido
    // (busy_timeout + WAL, issue #2).
    let vacuum_ok = src_ia::sqlite::abrir_db(db_path)
        .ok()
        .and_then(|conn| {
            // El path va como literal SQL: se escapan comillas simples.
            let dest_sql = destino.to_string_lossy().replace('\'', "''");
            conn.execute_batch(&format!("VACUUM INTO '{dest_sql}';"))
                .ok()
        })
        .is_some()
        && destino.exists();

    if !vacuum_ok {
        // Ruta 2 (fallback): copia cruda del archivo principal.
        if fs::copy(db_path, &destino).is_err() {
            return None;
        }
    }

    rotar_backups(&backups_dir);
    Some(destino)
}

/// Borra los backups más viejos dejando solo los últimos [`MAX_BACKUPS`].
/// Los errores de limpieza se ignoran a propósito (no deben romper nada).
fn rotar_backups(backups_dir: &Path) {
    let mut archivos: Vec<PathBuf> = fs::read_dir(backups_dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| {
                    p.extension().is_some_and(|ext| ext == "db")
                        && p.file_name()
                            .is_some_and(|n| n.to_string_lossy().starts_with("yarvis-"))
                })
                .collect()
        })
        .unwrap_or_default();
    archivos.sort();
    if archivos.len() > MAX_BACKUPS {
        for viejo in archivos.iter().take(archivos.len() - MAX_BACKUPS) {
            let _ = fs::remove_file(viejo);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotar_backups_conserva_solo_los_ultimos_5() {
        let base = std::env::temp_dir().join(format!(
            "yarvis_backup_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&base).unwrap();
        for i in 1..=7 {
            std::fs::write(base.join(format!("yarvis-2026010{i}-120000.db")), b"x").unwrap();
        }
        rotar_backups(&base);
        let restantes = std::fs::read_dir(&base).unwrap().count();
        assert_eq!(restantes, MAX_BACKUPS);
        // Sobreviven los más nuevos (orden lexicográfico = cronológico).
        assert!(!base.join("yarvis-20260101-120000.db").exists());
        assert!(!base.join("yarvis-20260102-120000.db").exists());
        assert!(base.join("yarvis-20260107-120000.db").exists());
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn backup_ignora_nombres_que_no_son_yarvis() {
        let base = std::env::temp_dir().join(format!(
            "yarvis_backup_test2_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&base).unwrap();
        std::fs::write(base.join("otro.db"), b"x").unwrap();
        std::fs::write(base.join("yarvis-20260101-120000.db"), b"x").unwrap();
        rotar_backups(&base);
        assert!(base.join("otro.db").exists(), "no debe borrar archivos ajenos");
        let _ = std::fs::remove_dir_all(&base);
    }
}
