// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Módulo AUTH (adminconfig/auth).
// Prueba hash/verify de contraseñas (Argon2id), guardar_empleado_impl:
// cálculo de salario semanal, persistencia de bloques y el bloqueo de
// contraseñas duplicadas entre empleados (login solo por clave).
// ═══════════════════════════════════════════════════════════════════════════

#[path = "../common/mod.rs"]
mod common;

use common::{db, seed_empleado};
use sqlx::Row;
use yarvis_app_lib::backventanas::backadmin::adminconfig::auth::{
    guardar_empleado_impl, hash_password, verify_password, BloqueHorario,
};

#[test]
fn hash_y_verify_roundtrip() {
    let hash = hash_password("clave123");
    assert!(verify_password("clave123", &hash));
    assert!(!verify_password("clave124", &hash));
    // Dos hashes de la misma clave difieren (salt único)
    let otro = hash_password("clave123");
    assert_ne!(hash, otro);
}

#[tokio::test]
async fn alta_calcula_salario_semanal_y_guarda_bloques() {
    let pool = db().await;
    let r = guardar_empleado_impl(
        &pool,
        "Peter Parker".into(),
        "webshot1".into(),
        Some(2000.0),
        Some(vec![BloqueHorario { dias: vec![0, 1], hora_inicio: "08:00".into(), hora_fin: "16:00".into() }]),
    )
    .await
    .unwrap();

    let fila = sqlx::query(
        "SELECT rol, salario_diario, dias_semana, salario_semanal FROM usuarios WHERE nombre = 'Peter Parker'",
    )
    .fetch_one(&pool).await.unwrap();
    assert_eq!(fila.get::<String,_>("rol"), "empleado");
    assert_eq!(fila.get::<i64,_>("salario_semanal"), 200_000);
    assert_eq!(fila.get::<i32,_>("dias_semana"), 2);
    assert_eq!(fila.get::<i64,_>("salario_diario"), 100_000);

    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM empleado_horarios").fetch_one(&pool).await.unwrap();
    assert_eq!(n, 1);
    assert!(r.contains("guardado"));
}

#[tokio::test]
async fn alta_sin_campos_opcionales_funciona_primer_inicio() {
    let pool = db().await;
    let r = guardar_empleado_impl(&pool, "PrimerEmpleado".into(), "abc123".into(), None, None).await;
    assert!(r.is_ok());
}

#[tokio::test]
async fn contraseña_duplicada_entre_empleados_rechazada() {
    let pool = db().await;
    seed_empleado(&pool, "Empleado Uno", "secreta99").await;

    let r = guardar_empleado_impl(&pool, "Empleado Dos".into(), "secreta99".into(), None, None).await;
    assert!(r.is_err());
    // Y no se insertó nada
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM usuarios WHERE nombre = 'Empleado Dos'")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(n, 0);
}

#[tokio::test]
async fn contraseñas_distintas_aceptadas() {
    let pool = db().await;
    seed_empleado(&pool, "Uno", "claveaaa").await;
    let r = guardar_empleado_impl(&pool, "Dos".into(), "clavebbb".into(), None, None).await;
    assert!(r.is_ok());
}


#[test]
fn password_sin_hash_es_rechazada_regresion() {
    // REGRESIÓN: verify_password tuvo un fallback que comparaba en claro
    // cuando el hash no parseaba ("perpetúa credenciales sin hash para
    // siempre"). Fue eliminado: un stored sin formato Argon2 debe negar
    // el acceso SIEMPRE, aunque la contraseña coincida literalmente.
    assert!(verify_password("secreto", "secreto") == false);
    assert!(verify_password("secreto", "no-es-mi-password") == false);
    // Y el flujo normal sigue funcionando:
    let hash = hash_password("clave-segura");
    assert!(verify_password("clave-segura", &hash));
}

/// TOCTOU de guardar_admin: dos configuraciones iniciales simultáneas no
/// deben crear DOS admins.
///
/// NOTA: `guardar_admin` recibe `tauri::State<'_, SqlitePool>` y
/// `tauri::State<'_, AuthState>`, que no pueden construirse fuera del
/// runtime de Tauri, así que NO es invocable directamente desde tests.
/// La atomicidad se verifica al nivel SQL replicando EXACTAMENTE la única
/// sentencia que usa la implementación actual (INSERT..SELECT..WHERE COUNT=0,
/// ver adminconfig/auth.rs líneas 64-67) ejecutándola DOS veces en paralelo
/// sobre una DB limpia. Si alguien reintroduce el patrón COUNT-then-INSERT
/// de dos sentencias, este test deja de representar el código real.
#[tokio::test]
async fn toctou_dos_guardar_admin_concurrentes_crean_un_solo_admin() {
    let pool = db().await;

    let sentencia = |nombre: &'static str| {
        sqlx::query(
            "INSERT INTO usuarios (nombre, tienda, password, rol)
             SELECT ?, ?, ?, 'admin'
             WHERE (SELECT COUNT(*) FROM usuarios WHERE rol = 'admin') = 0",
        )
        .bind(nombre)
        .bind("Tienda")
        .bind("hash-falso")
    };

    // Ambas llamadas comparten el pool (como los dos commands concurrentes).
    let (r1, r2) = tokio::join!(
        sentencia("Admin A").execute(&pool),
        sentencia("Admin B").execute(&pool),
    );

    let afectadas_1 = r1.unwrap().rows_affected();
    let afectadas_2 = r2.unwrap().rows_affected();

    // Exactamente UNO gana; el otro obtiene rows_affected == 0, que en
    // guardar_admin se traduce en Err("La configuración inicial ya fue completada").
    assert_eq!(
        (afectadas_1 + afectadas_2) as usize, 1,
        "la sentencia atómica debe admitir UN solo admin concurrente"
    );
    assert!(afectadas_1 == 1 || afectadas_2 == 1);

    let admins: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM usuarios WHERE rol = 'admin'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(admins, 1, "TOCTOU roto: hay más de un admin");
}
