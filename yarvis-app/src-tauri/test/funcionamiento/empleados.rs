// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Módulo EMPLEADOS (adminempleados/modalempleado).
// Prueba editar_empleado_impl (validación de bloques, recálculo de salario
// diario/semanal, cambio de contraseña) y set_estado_empleado_impl.
// ═══════════════════════════════════════════════════════════════════════════

#[path = "../common/mod.rs"]
mod common;

use common::{db, seed_empleado};
use sqlx::Row;
use yarvis_app_lib::backventanas::backadmin::adminconfig::auth::{verify_password, BloqueHorario};
use yarvis_app_lib::backventanas::backadmin::adminempleados::modalempleado::{
    editar_empleado_impl, set_estado_empleado_impl,
};

fn bloque(dias: Vec<i32>, inicio: &str, fin: &str) -> BloqueHorario {
    BloqueHorario {
        dias,
        hora_inicio: inicio.into(),
        hora_fin: fin.into(),
    }
}

#[tokio::test]
async fn edicion_recalcula_salarios_y_espeja_primer_bloque() {
    let pool = db().await;
    let id = seed_empleado(&pool, "Peter", "clave123").await;

    editar_empleado_impl(
        &pool,
        id as i32,
        "Peter Parker".into(),
        Some(1500.0),
        Some(vec![
            bloque(vec![0, 2, 3], "08:00", "17:00"),
            bloque(vec![5, 6], "08:00", "12:00"),
        ]),
        None,
    )
    .await
    .unwrap();

    let fila = sqlx::query("SELECT nombre, salario_semanal, salario_diario, dias_semana, horario_inicio FROM usuarios WHERE id = ?")
        .bind(id).fetch_one(&pool).await.unwrap();
    assert_eq!(fila.get::<String,_>("nombre"), "Peter Parker");
    assert_eq!(fila.get::<f64,_>("salario_semanal"), 1500.0);
    // 5 días laborables → 1500/5 = 300
    assert_eq!(fila.get::<f64,_>("salario_diario"), 300.0);
    assert_eq!(fila.get::<i32,_>("dias_semana"), 5);
    // Columnas legacy espejan el primer bloque
    assert_eq!(fila.get::<String,_>("horario_inicio"), "08:00");

    // Los bloques quedaron persistidos completos
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM empleado_horarios WHERE empleado_id = ?")
        .bind(id).fetch_one(&pool).await.unwrap();
    assert_eq!(n, 2);
}

#[tokio::test]
async fn dia_repetido_entre_bloques_rechazado() {
    let pool = db().await;
    let id = seed_empleado(&pool, "Gwen", "clave456").await;
    let r = editar_empleado_impl(
        &pool,
        id as i32,
        "Gwen".into(),
        Some(100.0),
        Some(vec![
            bloque(vec![0], "08:00", "12:00"),
            bloque(vec![0, 1], "14:00", "18:00"), // lunes repetido
        ]),
        None,
    )
    .await;
    assert!(r.is_err());
}

#[tokio::test]
async fn bloque_sin_dias_rechazado() {
    let pool = db().await;
    let id = seed_empleado(&pool, "Miles", "clave789").await;
    let r = editar_empleado_impl(
        &pool,
        id as i32,
        "Miles".into(),
        None,
        Some(vec![bloque(vec![], "08:00", "16:00")]),
        None,
    )
    .await;
    assert!(r.is_err());
}

#[tokio::test]
async fn cambio_de_contraseña_se_hashea_y_verifica() {
    let pool = db().await;
    let id = seed_empleado(&pool, "Hulk", "viejita1").await;

    editar_empleado_impl(
        &pool,
        id as i32,
        "Hulk".into(),
        None,
        None,
        Some("nuevaclave1".into()),
    )
    .await
    .unwrap();

    let hash: String = sqlx::query_scalar("SELECT password FROM usuarios WHERE id = ?")
        .bind(id).fetch_one(&pool).await.unwrap();
    assert!(!verify_password("viejita1", &hash));
    assert!(verify_password("nuevaclave1", &hash));
}

#[tokio::test]
async fn desactivar_y_reactivar_cambian_el_estado() {
    let pool = db().await;
    let id = seed_empleado(&pool, "Strange", "magia123").await;

    set_estado_empleado_impl(&pool, id as i32, "inactivo".into()).await.unwrap();
    let estado: String = sqlx::query_scalar("SELECT estado FROM usuarios WHERE id = ?")
        .bind(id).fetch_one(&pool).await.unwrap();
    assert_eq!(estado, "inactivo");

    set_estado_empleado_impl(&pool, id as i32, "activo".into()).await.unwrap();
    let estado: String = sqlx::query_scalar("SELECT estado FROM usuarios WHERE id = ?")
        .bind(id).fetch_one(&pool).await.unwrap();
    assert_eq!(estado, "activo");
}

#[tokio::test]
async fn estado_invalido_e_inexistente_rechazados() {
    let pool = db().await;
    assert!(set_estado_empleado_impl(&pool, 1, "volador".into()).await.is_err());
    assert!(set_estado_empleado_impl(&pool, 424242, "activo".into()).await.is_err());
}
