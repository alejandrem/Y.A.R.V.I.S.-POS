// Tests adversarios del arranque: el mensaje al dueño debe decir QUÉ
// pasó, DÓNDE está el log y DÓNDE buscar respaldos. Si falta cualquiera
// de los tres, en la tienda nadie puede recuperar la caja.

use super::*;

fn error_de_prueba() -> ErrorDb {
    ErrorDb::conexion("database disk image is malformed".to_string())
}

#[test]
fn el_mensaje_incluye_log_ruta_de_db_y_backups() {
    let mensaje = mensaje_error(
        &error_de_prueba(),
        "C:\\datos\\yarvis.db",
        "C:\\datos\\backups",
        "C:\\datos\\logs\\yarvis.log",
    );

    assert!(mensaje.contains("No se pudo abrir la base de datos"));
    assert!(mensaje.contains("malformed"));
    assert!(mensaje.contains("C:\\datos\\yarvis.db"));
    assert!(mensaje.contains("C:\\datos\\backups"));
    assert!(mensaje.contains("C:\\datos\\logs\\yarvis.log"));
    assert!(mensaje.contains("Qué hacer"));
    assert!(mensaje.contains("restaurar un respaldo cifrado"));
}

#[test]
fn el_mensaje_no_queda_vacio_ni_con_placeholders() {
    let mensaje = mensaje_error(
        &ErrorDb::migracion(
            sqlx::migrate::MigrateError::VersionMismatch(18),
            Path::new("C:\\datos\\yarvis.db"),
        ),
        "C:\\datos\\yarvis.db",
        "(no disponible)",
        "(log no disponible)",
    );

    assert!(!mensaje.trim().is_empty());
    assert!(mensaje.contains("otra versión del programa"));
    assert!(mensaje.contains("No edites migraciones ya aplicadas"));
    assert!(!mensaje.contains("{}"));
}

#[test]
fn validar_origen_rechaza_cifrados_y_extensiones_raras() {
    let db = Path::new("C:\\datos\\yarvis.db");
    assert!(validar_origen(Path::new("C:\\respaldo\\yarvis-20260101.db"), db).is_ok());
    assert!(validar_origen(Path::new("C:\\respaldo\\yarvis-20260101.DB"), db).is_ok());

    let err = validar_origen(Path::new("C:\\respaldo\\yarvis-20260101.yarvisbak"), db)
        .unwrap_err();
    assert!(err.contains(".yarvisbak"), "{err}");

    let err = validar_origen(Path::new("C:\\respaldo\\notas.txt"), db).unwrap_err();
    assert!(err.contains("respaldo local .db"), "{err}");
}

#[test]
fn validar_origen_rechaza_restaurar_el_mismo_archivo_actual() {
    let dir = std::env::temp_dir().join(format!(
        "yarvis-arranque-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let db = dir.join("yarvis.db");
    std::fs::write(&db, b"x").unwrap();

    let err = validar_origen(&db, &db).unwrap_err();
    assert!(err.contains("archivo que no pudo abrir"), "{err}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn db_lista_empieza_en_falso() {
    // No se puede probar el paso a true sin un AppHandle real, pero sí
    // que el gate NUNCA nace abierto (era el bug de la ventana vacía).
    assert!(!db_lista());
}
