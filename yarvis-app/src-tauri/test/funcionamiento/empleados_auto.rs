// TEST FUNCIONAL — Alta automática de empleados desde tickets.
// Cubre: creación con pass nombre+123 hasheada, reutilización por nombre
// normalizado (sin duplicados en re-importes), sufijo 1234/12345 ante
// colisión de contraseña (login solo-password), omisiones (SISTEMA),
// vinculación de cajero_id y flag password_defecto.
use rusqlite::Connection;
use yarvis_app_lib::backventanas::backadmin::adminconfig::auth::{hash_password, verify_password};
use yarvis_app_lib::backventanas::backadmin::adminparser::empleados_auto::{
    normalizar_empleado, resolver_empleados_desde_conn,
};

fn db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "CREATE TABLE usuarios (id INTEGER PRIMARY KEY AUTOINCREMENT, nombre TEXT NOT NULL,
          password TEXT NOT NULL, rol TEXT NOT NULL, estado TEXT DEFAULT 'activo',
          password_defecto INTEGER DEFAULT 0);
         CREATE TABLE ventas (id INTEGER PRIMARY KEY AUTOINCREMENT, cajero TEXT NOT NULL,
          cajero_id INTEGER);",
    )
    .unwrap();
    conn
}

fn venta(conn: &Connection, cajero: &str) {
    conn.execute(
        "INSERT INTO ventas (cajero) VALUES (?1)",
        rusqlite::params![cajero],
    )
    .unwrap();
}

fn usuario(conn: &Connection, nombre: &str, pass_plana: &str, defecto: i64) -> i64 {
    conn.execute(
        "INSERT INTO usuarios (nombre, password, rol, estado, password_defecto) VALUES (?1, ?2, 'empleado', 'activo', ?3)",
        rusqlite::params![nombre, hash_password(pass_plana), defecto],
    )
    .unwrap();
    conn.last_insert_rowid()
}

fn cajero_id_de(conn: &Connection, venta_id: i64) -> Option<i64> {
    conn.query_row(
        "SELECT cajero_id FROM ventas WHERE id = ?1",
        rusqlite::params![venta_id],
        |r| r.get(0),
    )
    .unwrap()
}

#[test]
fn normaliza_para_comparar_personas() {
    assert_eq!(normalizar_empleado("MARIA G."), "MARIA G");
    assert_eq!(normalizar_empleado("  maria   g. "), "MARIA G");
    assert_eq!(normalizar_empleado("SISTEMA"), "SISTEMA");
}

#[test]
fn crea_reutiliza_vincula_y_es_idempotente() {
    let conn = db();
    let ana_id = usuario(&conn, "ana", "ana123", 0);
    venta(&conn, "MARIA G.");
    venta(&conn, "  maria g. ");
    venta(&conn, "ANA");
    venta(&conn, "SISTEMA");

    let r = resolver_empleados_desde_conn(&conn).unwrap();
    // Solo MARIA es nueva ("  maria g. " es la misma persona; ANA ya existe).
    assert_eq!(r.creados.len(), 1);
    assert_eq!(r.creados[0].nombre, "MARIA G.");
    assert_eq!(r.creados[0].password_plana, "MARIA G.123");
    // La contraseña guardada es hash verificable, no texto plano.
    let hash: String = conn
        .query_row(
            "SELECT password FROM usuarios WHERE nombre = 'MARIA G.'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(verify_password("MARIA G.123", &hash));
    // Flag de contraseña débil prendido; el de ANA intacto.
    let flag: i64 = conn
        .query_row(
            "SELECT password_defecto FROM usuarios WHERE nombre = 'MARIA G.'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(flag, 1);
    // Vinculación: las 2 de MARIA al nuevo id, la de ANA a ana_id, SISTEMA libre.
    assert_eq!(cajero_id_de(&conn, 1), cajero_id_de(&conn, 2));
    assert!(cajero_id_de(&conn, 1).unwrap() != ana_id);
    assert_eq!(cajero_id_de(&conn, 3), Some(ana_id));
    assert_eq!(cajero_id_de(&conn, 4), None);
    assert_eq!(r.vinculados, 3);

    // Segunda corrida: nada nuevo, nada re-vinculado.
    let r2 = resolver_empleados_desde_conn(&conn).unwrap();
    assert!(r2.creados.is_empty());
    assert_eq!(r2.vinculados, 0);
}

#[test]
fn colision_de_password_agrega_entero_al_final() {
    let conn = db();
    // Alguien YA tiene la contraseña que le tocaría a "Y".
    usuario(&conn, "otro", "Y123", 0);
    venta(&conn, "Y");

    let r = resolver_empleados_desde_conn(&conn).unwrap();
    assert_eq!(r.creados.len(), 1);
    assert_eq!(r.creados[0].password_plana, "Y1234");
    // Y si también existe esa, sigue con 12345.
    venta(&conn, "Z");
    usuario(&conn, "alguien", "Z123", 0);
    usuario(&conn, "alguien2", "Z1234", 0);
    let r2 = resolver_empleados_desde_conn(&conn).unwrap();
    assert_eq!(r2.creados[0].password_plana, "Z12345");
}

#[test]
fn mismo_nombre_no_duplica_entre_corridas() {
    let conn = db();
    venta(&conn, "JUAN M");
    let r1 = resolver_empleados_desde_conn(&conn).unwrap();
    assert_eq!(r1.creados.len(), 1);
    // Llega otro ticket del mismo JUAN (re-importe o turno siguiente).
    venta(&conn, "JUAN M");
    let r2 = resolver_empleados_desde_conn(&conn).unwrap();
    assert!(r2.creados.is_empty(), "no debe duplicar al mismo nombre");
    assert_eq!(r2.vinculados, 1);
    let total: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM usuarios WHERE rol = 'empleado'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(total, 1);
}
