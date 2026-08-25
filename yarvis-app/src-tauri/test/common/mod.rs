// ═══════════════════════════════════════════════════════════════════════════
// COMMON — Fixture compartido de los tests de backend.
// Tarea única: crear una base de datos SQLite REAL (archivo temporal, WAL,
// foreign_keys y busy_timeout idénticos a producción), aplicar las migraciones
// embebidas y ofrecer helpers de seed. Cada test obtiene su propia DB
// aislada; nada toca la yarvis.db del usuario.
// ═══════════════════════════════════════════════════════════════════════════

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use sqlx::SqlitePool;
use std::sync::atomic::{AtomicU64, Ordering};

static SEQ: AtomicU64 = AtomicU64::new(0);

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

/// DB temporal única por test, con las mismas opciones de conexión que producción.
/// Dos fases como en db.rs: migrar SIN foreign_keys (sqlx envuelve cada
/// migración en una transacción donde PRAGMA foreign_keys es un no-op y la
/// reconstrucción de 0005 lo requiere) y reabrir CON foreign_keys.
pub async fn db() -> SqlitePool {
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "yarvis_test_{}_{}_{}.db",
        std::process::id(),
        nanos,
        n
    ));

    let base = SqliteConnectOptions::new()
        .filename(&path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(std::time::Duration::from_secs(5));

    let pool_migraciones = SqlitePool::connect_with(base.clone().foreign_keys(false))
        .await
        .expect("conectar sqlite de prueba");
    MIGRATOR.run(&pool_migraciones).await.expect("aplicar migraciones");
    pool_migraciones.close().await;

    let pool = SqlitePool::connect_with(base.foreign_keys(true))
        .await
        .expect("reconectar sqlite de prueba");
    pool
}

/// Inserta un producto y devuelve su id.
pub async fn seed_producto(pool: &SqlitePool, nombre: &str, stock: f64, precio_venta: f64) -> i64 {
    // Los precios van en CENTAVOS (la DB es INTEGER); el stock sigue en f64.
    let r = sqlx::query(
        "INSERT INTO productos (nombre, precio_costo, precio_venta, stock, stock_minimo, vendido) VALUES (?, ?, ?, ?, ?, 0)",
    )
    .bind(nombre)
    .bind(yarvis_app_lib::dinero::a_centavos(precio_venta * 0.5))
    .bind(yarvis_app_lib::dinero::a_centavos(precio_venta))
    .bind(stock)
    .bind(1.0)
    .execute(pool)
    .await
    .unwrap();
    r.last_insert_rowid()
}

/// Inserta un empleado con contraseña ya hasheada y devuelve su id.
pub async fn seed_empleado(pool: &SqlitePool, nombre: &str, pass: &str) -> i64 {
    let hash = yarvis_app_lib::backventanas::backadmin::adminconfig::auth::hash_password(pass);
    let r = sqlx::query("INSERT INTO usuarios (nombre, password, rol) VALUES (?, ?, 'empleado')")
        .bind(nombre)
        .bind(hash)
        .execute(pool)
        .await
        .unwrap();
    r.last_insert_rowid()
}

/// Consulta rápida de una celda escalar.
pub async fn escalar_i64(pool: &SqlitePool, sql: &str) -> i64 {
    sqlx::query_scalar(sql).fetch_one(pool).await.unwrap()
}
