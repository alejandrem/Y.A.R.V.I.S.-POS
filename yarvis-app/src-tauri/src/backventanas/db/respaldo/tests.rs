// Tests adversarios del respaldo local: intentan restaurar basura,
// borrar archivos ajenos y dejar la caja sin base. Si algo de esto pasa,
// el POS queda inservible en una tienda real.

use super::*;
use std::time::{SystemTime, UNIX_EPOCH};

fn dir_temp(nombre: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "yarvis-respaldo-{nombre}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    fs::create_dir_all(&base).unwrap();
    base
}

fn crear_db(ruta: &Path, marca: &str) {
    let conn = Connection::open(ruta).unwrap();
    conn.execute_batch(
        "CREATE TABLE usuarios (id INTEGER PRIMARY KEY, nombre TEXT);
         CREATE TABLE ventas (id INTEGER PRIMARY KEY, total INTEGER);
         CREATE TABLE productos (id INTEGER PRIMARY KEY, nombre TEXT);
         CREATE TABLE detalle_ventas (id INTEGER PRIMARY KEY, venta_id INTEGER);
         CREATE TABLE _sqlx_migrations (version INTEGER PRIMARY KEY);",
    )
    .unwrap();
    conn.execute("INSERT INTO usuarios (nombre) VALUES (?1)", [marca])
        .unwrap();
    conn.execute("INSERT INTO ventas (total) VALUES (1250)", [])
        .unwrap();
    conn.execute("INSERT INTO productos (nombre) VALUES ('Coca')", [])
        .unwrap();
    conn.execute("INSERT INTO detalle_ventas (venta_id) VALUES (1)", [])
        .unwrap();
    conn.execute("INSERT INTO _sqlx_migrations (version) VALUES (18)", [])
        .unwrap();
}

fn marca_usuarios(ruta: &Path) -> String {
    let conn = Connection::open(ruta).unwrap();
    conn.query_row("SELECT nombre FROM usuarios LIMIT 1", [], |r| r.get(0))
        .unwrap()
}

#[test]
fn rota_solo_los_backups_propios_y_protege_los_de_seguridad() {
    let dir = dir_temp("rotar");
    for i in 1..=7 {
        fs::write(dir.join(format!("yarvis-2026010{i}-120000.db")), b"x").unwrap();
    }
    let seguridad = dir.join("yarvis-antes-restaurar-20260101-120000.db");
    fs::write(&seguridad, b"no borrar").unwrap();
    fs::write(dir.join("otro.db"), b"ajeno").unwrap();

    rotar_backups(&dir, 3);

    let propios = fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| es_backup_rotativo(&e.path()))
        .count();
    assert_eq!(propios, 3);
    assert!(!dir.join("yarvis-20260101-120000.db").exists());
    assert!(dir.join("yarvis-20260107-120000.db").exists());
    assert!(seguridad.exists(), "los respaldos de seguridad no se rotan");
    assert!(dir.join("otro.db").exists(), "no se toca lo ajeno");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn probar_db_acepta_una_base_valida() {
    let dir = dir_temp("probar-ok");
    let db = dir.join("yarvis.db");
    crear_db(&db, "Ana");

    let resumen = probar_db(&db).unwrap();
    assert_eq!(resumen.usuarios, 1);
    assert_eq!(resumen.ventas, 1);
    assert_eq!(resumen.productos, 1);
    assert_eq!(resumen.detalle_ventas, 1);
    assert_eq!(resumen.version_esquema, 18);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn probar_db_rechaza_archivo_que_no_es_sqlite() {
    let dir = dir_temp("probar-basura");
    let db = dir.join("basura.db");
    fs::write(&db, b"esto no es una base de datos").unwrap();

    let err = probar_db(&db).unwrap_err();
    assert!(err.contains("No se pudo abrir") || err.contains("dañada"), "{err}");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn probar_db_rechaza_db_sin_tablas_esenciales() {
    let dir = dir_temp("probar-incompleta");
    let db = dir.join("incompleta.db");
    let conn = Connection::open(&db).unwrap();
    conn.execute_batch("CREATE TABLE otra (id INTEGER)")
        .unwrap();
    drop(conn);

    let err = probar_db(&db).unwrap_err();
    assert!(err.contains("falta la tabla"), "{err}");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn restaurar_rechaza_corrupto_y_no_toca_la_base_actual() {
    let dir = dir_temp("restaurar-corrupto");
    let actual = dir.join("yarvis.db");
    crear_db(&actual, "ACTUAL");
    let corrupto = dir.join("corrupto.db");
    fs::write(&corrupto, b"basura").unwrap();

    let err = restaurar_db_plana(&corrupto, &actual, &dir).unwrap_err();
    assert!(!err.is_empty());
    assert_eq!(marca_usuarios(&actual), "ACTUAL", "la base actual debe seguir intacta");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn restaurar_reemplaza_y_conserva_copia_de_la_anterior() {
    let dir = dir_temp("restaurar-ok");
    let actual = dir.join("yarvis.db");
    crear_db(&actual, "ANTES");
    let nueva = dir.join("nueva.db");
    crear_db(&nueva, "DESPUES");

    let seguridad = restaurar_db_plana(&nueva, &actual, &dir).unwrap().unwrap();

    assert_eq!(marca_usuarios(&actual), "DESPUES");
    assert_eq!(marca_usuarios(&seguridad), "ANTES");
    assert!(seguridad
        .file_name()
        .unwrap()
        .to_string_lossy()
        .starts_with("yarvis-antes-restaurar-"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn backup_antes_de_migrar_copia_datos_validos() {
    let dir = dir_temp("pre-migracion");
    let db = dir.join("yarvis.db");
    crear_db(&db, "VIEJA");

    let copia = backup_db_antes_de_migrar(&dir, &db).unwrap();

    assert!(copia.exists());
    assert_eq!(marca_usuarios(&copia), "VIEJA");
    assert_eq!(probar_db(&copia).unwrap().ventas, 1);

    let _ = fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn backup_local_desde_pool_copia_una_db_abierta() {
    let dir = dir_temp("pool");
    let db = dir.join("yarvis.db");
    let pool = SqlitePool::connect_with(
        sqlx::sqlite::SqliteConnectOptions::new()
            .filename(&db)
            .create_if_missing(true),
    )
    .await
    .unwrap();
    sqlx::raw_sql(
        "CREATE TABLE usuarios (id INTEGER PRIMARY KEY, nombre TEXT);
         CREATE TABLE ventas (id INTEGER PRIMARY KEY, total INTEGER);
         CREATE TABLE productos (id INTEGER PRIMARY KEY, nombre TEXT);
         INSERT INTO usuarios (nombre) VALUES ('POOL');
         INSERT INTO ventas (total) VALUES (99);
         INSERT INTO productos (nombre) VALUES ('Pan');",
    )
    .execute(&pool)
    .await
    .unwrap();

    let copia = backup_local_desde_pool(&pool, &dir).await.unwrap();
    pool.close().await;

    assert!(copia.exists());
    assert_eq!(marca_usuarios(&copia), "POOL");
    assert_eq!(probar_db(&copia).unwrap().ventas, 1);

    let _ = fs::remove_dir_all(&dir);
}
