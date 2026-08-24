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
    assert_eq!(fila.get::<i64,_>("salario_semanal"), 150_000);
    // 5 días laborables → 1500/5 = 300
    assert_eq!(fila.get::<i64,_>("salario_diario"), 30_000);
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

// ── ASISTENCIA ────────────────────────────────────────────────────────────

use yarvis_app_lib::backventanas::backempleado::empleaperfil::asistencia::registrar_asistencia;

#[tokio::test]
async fn primer_login_del_dia_se_conserva_ante_relogins() {
    let pool = db().await;
    let id = seed_empleado(&pool, "Pepito", "clave123").await;

    // Primer login del día: crea el registro de entrada real.
    registrar_asistencia(&pool, id).await.unwrap();
    let fila = sqlx::query("SELECT primer_login, ultimo_login FROM asistencias WHERE empleado_id = ?")
        .bind(id).fetch_one(&pool).await.unwrap();
    let primer_login: String = Row::get(&fila, "primer_login");

    // Segundo login el MISMO día: NO cambia la entrada, solo refresca último.
    std::thread::sleep(std::time::Duration::from_millis(1100)); // asegurar timestamp distinto
    registrar_asistencia(&pool, id).await.unwrap();

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM asistencias").fetch_one(&pool).await.unwrap();
    assert_eq!(total, 1, "no debe duplicar renglones por día");

    let fila = sqlx::query("SELECT primer_login, ultimo_login FROM asistencias WHERE empleado_id = ?")
        .bind(id).fetch_one(&pool).await.unwrap();
    assert_eq!(Row::get::<String, _>(&fila, "primer_login"), primer_login,
        "el primer login del día es inmutable");
}

#[tokio::test]
async fn asistencias_de_dias_distintos_se_separan() {
    let pool = db().await;
    let id = seed_empleado(&pool, "Lupita", "clave456").await;

    // Simular registro de ayer insertando directamente con fecha distinta.
    sqlx::query("INSERT INTO asistencias (empleado_id, fecha, primer_login) VALUES (?, date('now','localtime','-1 day'), '2026-01-01 07:00:00')")
        .bind(id).execute(&pool).await.unwrap();

    // Login de hoy crea renglón aparte sin tocar el de ayer.
    registrar_asistencia(&pool, id).await.unwrap();

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM asistencias WHERE empleado_id = ?")
        .bind(id).fetch_one(&pool).await.unwrap();
    assert_eq!(total, 2);
}

// ── HISTORIAL DE HORAS EXTRA ──────────────────────────────────────────────

use yarvis_app_lib::backventanas::backempleado::empleaperfil::asistencia::historial_horas_extra_impl;

#[tokio::test]
async fn historial_calcula_extras_pre_y_post_y_descarta_sin_extra() {
    let pool = db().await;
    let id = seed_empleado(&pool, "Pepito Extra", "clave123").await;

    // Bloque de los LUNES (día chip 0): 08:00 — 16:00
    sqlx::query("INSERT INTO empleado_horarios (empleado_id, dias, hora_inicio, hora_fin) VALUES (?, '0', '08:00', '16:00')")
        .bind(id).execute(&pool).await.unwrap();

    // 2026-01-05 es LUNES: llegó 90 min antes y se quedó 90 después.
    sqlx::query(
        "INSERT INTO asistencias (empleado_id, fecha, primer_login, ultimo_login)
         VALUES (?, '2026-01-05', '2026-01-05 06:30', '2026-01-05 17:30')",
    ).bind(id).execute(&pool).await.unwrap();

    // Martes 2026-01-06 sin turno asignado → nunca cuenta.
    sqlx::query(
        "INSERT INTO asistencias (empleado_id, fecha, primer_login, ultimo_login)
         VALUES (?, '2026-01-06', '2026-01-06 07:50', '2026-01-06 18:00')",
    ).bind(id).execute(&pool).await.unwrap();

    // Lunes 2026-01-12: llegó solo 10 min antes (< umbral 15) y salió puntual → SIN extras.
    sqlx::query(
        "INSERT INTO asistencias (empleado_id, fecha, primer_login, ultimo_login)
         VALUES (?, '2026-01-12', '2026-01-12 07:50', '2026-01-12 16:00')",
    ).bind(id).execute(&pool).await.unwrap();

    let hist = historial_horas_extra_impl(&pool, id).await.unwrap();

    assert_eq!(hist.len(), 1, "solo el lunes con extras reales aparece");
    let d = &hist[0];
    assert_eq!(d.fecha, "2026-01-05");
    assert_eq!(d.dia_label, "Lunes");
    assert_eq!(d.extra_pre_min, 90);   // 06:30 → 08:00
    assert_eq!(d.extra_post_min, 90);  // 16:00 → 17:30
    assert_eq!(d.trabajo_min, 660);    // 06:30 → 17:30
    assert_eq!(d.entrada_oficial, "08:00");
    assert_eq!(d.salida_oficial, "16:00");
}
