// Tests del archivo de respaldo: que la foto cifrada contenga los datos,
// que la retención no borre archivos ajenos y que no queden .db en claro.

use super::*;
use sqlx::sqlite::SqliteConnectOptions;
use std::time::{SystemTime, UNIX_EPOCH};

const PASS: &str = "clave-de-respaldo-ok";
const SECRETO: &str = "PRODUCTO-SECRETO-2026";

fn dir_temp(nombre: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "yarvis-archivo-{nombre}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    fs::create_dir_all(&base).unwrap();
    base
}

async fn crear_pool(dir: &Path) -> SqlitePool {
    let opciones = SqliteConnectOptions::new()
        .filename(dir.join("yarvis.db"))
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(opciones).await.unwrap();
    sqlx::raw_sql(
        "CREATE TABLE usuarios (id INTEGER PRIMARY KEY, nombre TEXT);
         CREATE TABLE ventas (id INTEGER PRIMARY KEY, total INTEGER);
         CREATE TABLE productos (id INTEGER PRIMARY KEY, nombre TEXT);
         INSERT INTO usuarios (nombre) VALUES ('Ana');
         INSERT INTO ventas (total) VALUES (1500);",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO productos (nombre) VALUES (?1)")
        .bind(SECRETO)
        .execute(&pool)
        .await
        .unwrap();
    pool
}

#[tokio::test]
async fn crea_respaldo_cifrado_con_los_datos_reales() {
    let dir = dir_temp("crear");
    let destino = dir.join("usb");
    let tmp = dir.join("tmp");
    let pool = crear_pool(&dir).await;

    let info = crear_desde_pool(&pool, &destino, 30, PASS, &tmp)
        .await
        .unwrap();
    pool.close().await;

    assert!(Path::new(&info.archivo).exists());
    assert!(info.nombre.ends_with(".yarvisbak"));
    assert!(info.bytes > 0);
    assert_eq!(listar(&destino).len(), 1);

    let cifrado = fs::read(&info.archivo).unwrap();
    assert_eq!(&cifrado[..8], b"YRVBAK01");
    assert!(
        !cifrado.windows(SECRETO.len()).any(|v| v == SECRETO.as_bytes()),
        "el secreto apareció en el archivo cifrado"
    );

    let plano = cifrado::descifrar(&cifrado, PASS).unwrap();
    assert!(plano.starts_with(b"SQLite format 3\0"));
    assert!(
        plano.windows(SECRETO.len()).any(|v| v == SECRETO.as_bytes()),
        "el respaldo descifrado debe contener los datos"
    );

    // No debe quedar el .db en claro ni temporales.
    let en_tmp = fs::read_dir(&tmp).unwrap().count();
    assert_eq!(en_tmp, 0, "quedaron temporales en claro: {en_tmp}");
    let en_destino = fs::read_dir(&destino)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().contains(".tmp-"))
        .count();
    assert_eq!(en_destino, 0);

    let _ = fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn una_contrasena_equivocada_no_abre_el_respaldo() {
    let dir = dir_temp("pass");
    let pool = crear_pool(&dir).await;
    let info = crear_desde_pool(&pool, &dir.join("usb"), 30, PASS, &dir.join("tmp"))
        .await
        .unwrap();
    pool.close().await;

    let cifrado = fs::read(&info.archivo).unwrap();
    assert!(cifrado::descifrar(&cifrado, "clave-equivocada").is_err());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn rota_solo_los_respaldos_propios() {
    let dir = dir_temp("rotar");
    for i in 1..=6 {
        fs::write(dir.join(format!("yarvis-2026010{i}-120000.yarvisbak")), b"x").unwrap();
    }
    fs::write(dir.join("foto-familiar.yarvisbak"), b"ajeno").unwrap();
    fs::write(dir.join("notas.txt"), b"ajeno").unwrap();

    rotar(&dir, 2);

    assert_eq!(listar(&dir).len(), 2);
    assert!(!dir.join("yarvis-20260101-120000.yarvisbak").exists());
    assert!(dir.join("yarvis-20260106-120000.yarvisbak").exists());
    assert!(dir.join("foto-familiar.yarvisbak").exists());
    assert!(dir.join("notas.txt").exists());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn listar_ignora_temporales_y_extensiones_ajenas() {
    let dir = dir_temp("listar");
    fs::write(dir.join("yarvis-20260101-120000.yarvisbak"), b"ok").unwrap();
    fs::write(dir.join(".yarvis-20260101-120000.yarvisbak.tmp-1"), b"tmp").unwrap();
    fs::write(dir.join("yarvis-20260101-120000.db"), b"otro").unwrap();
    fs::write(dir.join("otro.yarvisbak"), b"otro").unwrap();

    let lista = listar(&dir);
    assert_eq!(lista.len(), 1);
    assert!(lista[0].nombre.ends_with(".yarvisbak"));

    let _ = fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn destino_imposible_devuelve_error_sin_panic() {
    let dir = dir_temp("destino-malo");
    let pool = crear_pool(&dir).await;
    let archivo = dir.join("soy-un-archivo");
    fs::write(&archivo, b"x").unwrap();

    let err = crear_desde_pool(&pool, &archivo, 30, PASS, &dir.join("tmp"))
        .await
        .unwrap_err();
    assert!(err.contains("carpeta de respaldo"), "{err}");
    pool.close().await;

    let _ = fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn limpia_temporales_de_un_cierre_abrupto() {
    let dir = dir_temp("limpiar");
    let tmp = dir.join("tmp");
    fs::create_dir_all(&tmp).unwrap();
    fs::write(tmp.join(".yarvis-viejo-123.db"), b"en claro").unwrap();
    let pool = crear_pool(&dir).await;

    crear_desde_pool(&pool, &dir.join("usb"), 30, PASS, &tmp)
        .await
        .unwrap();
    pool.close().await;

    assert!(!tmp.join(".yarvis-viejo-123.db").exists());
    let _ = fs::remove_dir_all(&dir);
}
