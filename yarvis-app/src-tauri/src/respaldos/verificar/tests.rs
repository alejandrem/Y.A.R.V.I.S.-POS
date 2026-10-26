// Tests de la prueba real de respaldos: un archivo cifrado con basura
// dentro NO debe pasar; una DB sin tablas esenciales tampoco.

use super::*;
use rusqlite::Connection;
use std::time::{SystemTime, UNIX_EPOCH};

const PASS: &str = "clave-de-respaldo-ok";

fn dir_temp(nombre: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "yarvis-verificar-{nombre}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    fs::create_dir_all(&base).unwrap();
    base
}

fn crear_db(ruta: &Path) {
    let conn = Connection::open(ruta).unwrap();
    conn.execute_batch(
        "CREATE TABLE usuarios (id INTEGER PRIMARY KEY, nombre TEXT);
         CREATE TABLE ventas (id INTEGER PRIMARY KEY, total INTEGER);
         CREATE TABLE productos (id INTEGER PRIMARY KEY, nombre TEXT);
         CREATE TABLE detalle_ventas (id INTEGER PRIMARY KEY, venta_id INTEGER);
         INSERT INTO usuarios (nombre) VALUES ('Ana');
         INSERT INTO ventas (total) VALUES (900);
         INSERT INTO productos (nombre) VALUES ('Pan');",
    )
    .unwrap();
}

fn crear_respaldo(dir: &Path, nombre: &str, db: &Path) -> PathBuf {
    let destino = dir.join(nombre);
    let plano = fs::read(db).unwrap();
    fs::write(&destino, cifrado::cifrar(&plano, PASS).unwrap()).unwrap();
    destino
}

#[test]
fn prueba_un_respaldo_valido_y_cuenta_las_filas() {
    let dir = dir_temp("ok");
    let db = dir.join("origen.db");
    crear_db(&db);
    let respaldo = crear_respaldo(&dir, "bueno.yarvisbak", &db);

    let prueba = probar(&respaldo, PASS, &dir.join("tmp")).unwrap();

    assert!(prueba.ok);
    assert_eq!(prueba.usuarios, 1);
    assert_eq!(prueba.ventas, 1);
    assert_eq!(prueba.productos, 1);
    assert!(prueba.mensaje.contains("Respaldo válido"));

    // El temporal descifrado no debe sobrevivir.
    let temporales = fs::read_dir(dir.join("tmp"))
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().contains(".yarvis-restaurar-"))
        .count();
    assert_eq!(temporales, 0);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn una_db_corrupta_cifrada_no_pasa_la_prueba() {
    let dir = dir_temp("corrupta");
    let respaldo = dir.join("corrupta.yarvisbak");
    fs::write(
        &respaldo,
        cifrado::cifrar(b"esto no es sqlite", PASS).unwrap(),
    )
    .unwrap();

    let err = probar(&respaldo, PASS, &dir.join("tmp")).unwrap_err();
    assert!(
        err.contains("dañada") || err.contains("No se pudo abrir"),
        "{err}"
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn una_db_sin_tablas_esenciales_no_pasa() {
    let dir = dir_temp("incompleta");
    let db = dir.join("incompleta.db");
    let conn = Connection::open(&db).unwrap();
    conn.execute_batch("CREATE TABLE solo_esto (id INTEGER)")
        .unwrap();
    drop(conn);
    let respaldo = crear_respaldo(&dir, "incompleta.yarvisbak", &db);

    let err = probar(&respaldo, PASS, &dir.join("tmp")).unwrap_err();
    assert!(err.contains("falta la tabla"), "{err}");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn contrasena_equivocada_no_deja_temporal() {
    let dir = dir_temp("pass");
    let db = dir.join("origen.db");
    crear_db(&db);
    let respaldo = crear_respaldo(&dir, "bueno.yarvisbak", &db);
    let tmp = dir.join("tmp");

    assert!(probar(&respaldo, "otra-clave", &tmp).is_err());
    if tmp.exists() {
        let archivos = fs::read_dir(&tmp).unwrap().count();
        assert_eq!(archivos, 0, "quedó un temporal tras fallar el descifrado");
    }

    let _ = fs::remove_dir_all(&dir);
}
