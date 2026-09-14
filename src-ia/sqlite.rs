// ============================================================
// sqlite — Apertura uniforme de conexiones rusqlite productivas.
//
// TODO open productivo de la app pasa por [`abrir_db`]: busy_timeout
// de 5 s + journal_mode WAL, igual que el pool sqlx del backend
// (`yarvis-app/.../db/db.rs`). Sin esto, una venta por sqlx en el
// milisegundo exacto en que corre un parseo masivo o un backfill
// truena al instante con `database is locked` (el timeout default
// de SQLite es 0 ms). Ver issue #2.
//
// Lo que NO pasa por aquí (a propósito):
// - tools del chat: abren READ_ONLY (en WAL los lectores no bloquean).
// - tests/examples: DBs en memoria o desechables.
// - foreign_keys: se deja APAGADO como antes (cambio mínimo; los
//   escritores rusqlite siempre corrieron así, encenderlo ahora
//   podría rechazar escrituras históricas que hoy pasan).
// ============================================================

use std::path::Path;
use std::time::Duration;

/// Espera ante locks, igual que el pool sqlx de producción.
pub const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

/// Abre `yarvis.db` con los pragmas de producción.
/// El error se devuelve tal cual (cada llamador ya lo mapea a String).
pub fn abrir_db<P: AsRef<Path>>(ruta: P) -> Result<rusqlite::Connection, rusqlite::Error> {
    let conn = rusqlite::Connection::open(ruta)?;
    conn.busy_timeout(BUSY_TIMEOUT)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir_temp(prefijo: &str) -> std::path::PathBuf {
        let base = std::env::temp_dir().join(format!(
            "yarvis_{prefijo}_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&base).unwrap();
        base
    }

    #[test]
    fn helper_deja_wal_activo() {
        let base = dir_temp("wal");
        let conn = abrir_db(base.join("t.db")).unwrap();
        let modo: String = conn
            .query_row("PRAGMA journal_mode", [], |r| r.get(0))
            .unwrap();
        assert_eq!(modo.to_uppercase(), "WAL");
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn helper_espera_lock_mientras_open_pelon_truena() {
        let base = dir_temp("lock");
        let db = base.join("t.db");
        abrir_db(&db)
            .unwrap()
            .execute_batch("CREATE TABLE t(x INTEGER)")
            .unwrap();

        // Escritor que retiene el lock de escritura.
        let escritor = abrir_db(&db).unwrap();
        escritor
            .execute_batch("BEGIN IMMEDIATE; INSERT INTO t VALUES (1)")
            .unwrap();

        // Open pelón (timeout 0): truena al instante, el bug del issue #2.
        let pelona = rusqlite::Connection::open(&db).unwrap();
        let err = pelona.execute("INSERT INTO t VALUES (2)", []).unwrap_err();
        assert!(
            err.to_string().contains("database is locked"),
            "se esperaba DatabaseBusy, fue: {err}"
        );

        // Helper: espera con busy_timeout y lo logra al liberarse el lock.
        let db_hilo = db.clone();
        let lector = std::thread::spawn(move || {
            abrir_db(&db_hilo)
                .unwrap()
                .execute("INSERT INTO t VALUES (3)", [])
                .unwrap();
        });
        std::thread::sleep(Duration::from_millis(300));
        escritor.execute_batch("COMMIT").unwrap();
        lector.join().unwrap();

        let n: i64 = abrir_db(&db)
            .unwrap()
            .query_row("SELECT COUNT(*) FROM t", [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 2);
        let _ = std::fs::remove_dir_all(&base);
    }
}
