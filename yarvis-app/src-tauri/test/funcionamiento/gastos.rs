// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Módulo GASTOS (adminfinanzas/gastos).
// Prueba crear_gasto_impl y registrar_pago_gasto_impl: persistencia,
// acumulación de monto_real y transición de estado pendiente→pagado.
// ═══════════════════════════════════════════════════════════════════════════

#[path = "../common/mod.rs"]
mod common;

use common::{db, escalar_i64};
use sqlx::Row;
use yarvis_app_lib::backventanas::backadmin::adminfinanzas::gastos::{
    crear_gasto_impl, registrar_pago_gasto_impl,
};
use yarvis_app_lib::backventanas::backadmin::adminfinanzas::models::{CrearGastoRequest, RegistrarPagoRequest};

fn gasto(monto: f64) -> CrearGastoRequest {
    CrearGastoRequest {
        nombre: "Renta".into(),
        tipo: "fijo".into(),
        categoria: "operativo".into(),
        monto_proyectado: monto,
        frecuencia: "mensual".into(),
        dia_pago: Some(1),
        intervalo_dias: None,
        fecha_inicio: "2026-01-01".into(),
        fecha_fin: None,
        folio_comprobante: None,
        notas: None,
    }
}

fn pago(gasto_id: i64, monto: f64) -> RegistrarPagoRequest {
    RegistrarPagoRequest {
        gasto_id,
        fecha_pago: "2026-08-21 12:00:00".into(),
        monto_pagado: monto,
        metodo_pago: Some("efectivo".into()),
        folio_comprobante: None,
        notas: None,
    }
}

#[tokio::test]
async fn crear_gasto_persiste_los_campos() {
    let pool = db().await;
    let id = crear_gasto_impl(&pool, &gasto(2500.0)).await.unwrap();
    let (nombre, monto): (String, f64) = sqlx::query_as(
        "SELECT nombre, monto_proyectado FROM gastos_recurrentes WHERE id = ?",
    ).bind(id).fetch_one(&pool).await.unwrap();
    assert_eq!(nombre, "Renta");
    assert_eq!(monto, 2500.0);
}

#[tokio::test]
async fn pago_parcial_deja_estado_pendiente_y_acumula() {
    let pool = db().await;
    let id = crear_gasto_impl(&pool, &gasto(1000.0)).await.unwrap();

    registrar_pago_gasto_impl(&pool, &pago(id, 400.0)).await.unwrap();

    let fila = sqlx::query("SELECT monto_real, estado_pago FROM gastos_recurrentes WHERE id = ?")
        .bind(id).fetch_one(&pool).await.unwrap();
    let real: f64 = Row::get(&fila, "monto_real");
    let estado: String = Row::get(&fila, "estado_pago");
    assert_eq!(real, 400.0);
    assert_eq!(estado, "pendiente");
    assert_eq!(escalar_i64(&pool, "SELECT COUNT(*) FROM pagos_gastos").await, 1);
}

#[tokio::test]
async fn pagos_que_cubren_el_proyectado_marcan_pagado() {
    let pool = db().await;
    let id = crear_gasto_impl(&pool, &gasto(1000.0)).await.unwrap();

    registrar_pago_gasto_impl(&pool, &pago(id, 600.0)).await.unwrap();
    registrar_pago_gasto_impl(&pool, &pago(id, 500.0)).await.unwrap(); // excede 100 → pagado

    let (real, estado): (f64, String) = sqlx::query_as(
        "SELECT monto_real, estado_pago FROM gastos_recurrentes WHERE id = ?",
    ).bind(id).fetch_one(&pool).await.unwrap();
    assert_eq!(real, 1100.0);
    assert_eq!(estado, "pagado");
}

#[tokio::test]
async fn pago_a_gasto_inexistente_rechazado_por_fk() {
    let pool = db().await;
    // La foreign key de pagos_gastos → gastos_recurrentes rechaza pagos
    // huérfanos: integridad referencial funcionando como debe.
    let r = registrar_pago_gasto_impl(&pool, &pago(9999, 100.0)).await;
    assert!(r.is_err());
    assert_eq!(escalar_i64(&pool, "SELECT COUNT(*) FROM pagos_gastos").await, 0);
}
