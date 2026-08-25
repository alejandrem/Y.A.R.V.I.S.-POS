// Diagnóstico: correr el MIGRATOR real contra una copia de la DB del usuario,
// con el mismo flujo en dos fases que db.rs (migrar sin FK, reabrir con FK).
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use std::path::Path;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[tokio::main]
async fn main() {
    let ruta = std::env::args().nth(1).expect("pasa la ruta de la DB");
    assert!(Path::new(&ruta).exists());

    let base = SqliteConnectOptions::new()
        .filename(&ruta)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(std::time::Duration::from_secs(5));

    // FASE 1 — migrar sin foreign_keys
    let pool = sqlx::SqlitePool::connect_with(base.clone().foreign_keys(false))
        .await
        .unwrap();

    println!("¿FK activas en fase 1?: {:?}",
        sqlx::query_scalar::<_, i64>("PRAGMA foreign_keys").fetch_one(&pool).await.unwrap());

    match MIGRATOR.run(&pool).await {
        Ok(()) => println!("MIGRACIÓN OK"),
        Err(e) => eprintln!("FALLO: {e}"),
    }
    pool.close().await;

    // FASE 2 — reabrir con FK y validar datos convertidos
    let pool = sqlx::SqlitePool::connect_with(base.foreign_keys(true))
        .await
        .unwrap();
    let aplicadas: Vec<i64> =
        sqlx::query_scalar("SELECT version FROM _sqlx_migrations ORDER BY version")
            .fetch_all(&pool)
            .await
            .unwrap();
    println!("aplicadas: {aplicadas:?}");

    let fila = sqlx::query_as::<_, (i64, i64)>(
        "SELECT COUNT(*), CAST(SUM(total) AS INTEGER) FROM ventas",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    println!("ventas: {} | SUM(total) en centavos: {}", fila.0, fila.1);
}
