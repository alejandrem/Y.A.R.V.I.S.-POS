// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Mis cortes del empleado (empleatickets/mis_cortes).
// Operator-scoped por session.user_id: aislamiento entre empleados,
// ventana de días y dinero en centavos → pesos.
// ═══════════════════════════════════════════════════════════════════════════

#[path = "../common/mod.rs"]
mod common;

use common::db;
use yarvis_app_lib::backventanas::backempleado::empleatickets::mis_cortes::mis_cortes_impl;
use yarvis_app_lib::dinero::a_centavos;

async fn empleado(pool: &sqlx::SqlitePool, nombre: &str) -> i64 {
    let hash =
        yarvis_app_lib::backventanas::backadmin::adminconfig::auth::hash_password("x1234567");
    sqlx::query("INSERT INTO usuarios (nombre, password, rol) VALUES (?, ?, 'empleado')")
        .bind(nombre)
        .bind(hash)
        .execute(pool)
        .await
        .unwrap()
        .last_insert_rowid()
}

/// Corte directo con fecha de cierre relativa ("0" = hoy).
async fn corte(
    pool: &sqlx::SqlitePool,
    usuario_id: i64,
    total_pesos: f64,
    hace_dias: i64,
    tipo: &str,
    estado: &str,
) {
    let cierre = if hace_dias == 0 {
        "datetime('now','localtime')".to_string()
    } else {
        format!("datetime('now','localtime','-{hace_dias} days')")
    };
    let sql = format!(
        "INSERT INTO cortes_caja (fecha_apertura, fecha_cierre, monto_inicial, total_ventas, usuario_id, estado, tipo_corte)
         VALUES (datetime('now','localtime','-{hace_dias} days'), {cierre}, 0, ?, ?, ?, ?)"
    );
    sqlx::query(&sql)
        .bind(a_centavos(total_pesos))
        .bind(usuario_id)
        .bind(estado)
        .bind(tipo)
        .execute(pool)
        .await
        .unwrap();
}

#[tokio::test]
async fn mis_cortes_solo_los_mios_y_en_pesos() {
    let pool = db().await;
    let yo = empleado(&pool, "Yo").await;
    let otro = empleado(&pool, "Otro").await;
    corte(&pool, yo, 1250.50, 0, "Z", "cerrado").await;
    corte(&pool, otro, 9999.0, 0, "Z", "cerrado").await;

    let mios = mis_cortes_impl(&pool, yo, 30).await.unwrap();
    assert_eq!(mios.len(), 1);
    assert_eq!(mios[0].total_ventas, 1250.50);
    assert_eq!(mios[0].tipo_corte.as_deref(), Some("Z"));
    assert_eq!(mios[0].estado, "cerrado");
}

#[tokio::test]
async fn mis_cortes_respeta_ventana_de_dias() {
    let pool = db().await;
    let yo = empleado(&pool, "Yo").await;
    corte(&pool, yo, 100.0, 0, "Z", "cerrado").await;
    corte(&pool, yo, 200.0, 10, "X", "cerrado").await;

    let hoy = mis_cortes_impl(&pool, yo, 1).await.unwrap();
    assert_eq!(hoy.len(), 1);
    let mes = mis_cortes_impl(&pool, yo, 30).await.unwrap();
    assert_eq!(mes.len(), 2);
}
