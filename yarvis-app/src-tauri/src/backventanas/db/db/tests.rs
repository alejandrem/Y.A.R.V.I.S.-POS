// Tests de arranque de la DB: el objetivo es que una base corrupta
// DEVUELVA error (nunca panic) y que la app pueda decidir mostrarlo.

use super::*;
use std::time::{SystemTime, UNIX_EPOCH};

fn dir_temp(nombre: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "yarvis-db-{nombre}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    fs::create_dir_all(&base).unwrap();
    base
}

#[tokio::test]
async fn base_corrupta_devuelve_error_y_no_hace_panic() {
    let dir = dir_temp("corrupta");
    fs::write(dir.join("yarvis.db"), b"esto no es sqlite").unwrap();

    let resultado = inicializar_en(&dir).await;
    let err = resultado.err().expect("una DB corrupta debe devolver Err");

    assert!(!err.resumen.is_empty());
    assert!(!err.detalle.is_empty());
    assert!(!err.version_mismatch);

    let _ = fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn inicializa_y_aplica_las_migraciones_embebidas() {
    let dir = dir_temp("ok");

    let (pool, ruta) = inicializar_en(&dir).await.unwrap();
    let migraciones: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(&pool)
        .await
        .unwrap();
    let usuarios: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM usuarios")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert!(migraciones > 0, "deben aplicarse migraciones");
    assert_eq!(usuarios, 0, "una base nueva no tiene usuarios");
    assert!(ruta.ends_with("yarvis.db"));

    pool.close().await;
    let _ = fs::remove_dir_all(&dir);
}
