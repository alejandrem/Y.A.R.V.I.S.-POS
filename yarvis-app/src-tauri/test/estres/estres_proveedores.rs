// ═══════════════════════════════════════════════════════════════════════════
// TEST DE ESTRÉS — Proveedores del empleado.
// Presión: 300 recepciones secuenciales con integridad exacta
// (compras == renglones, stock exacto) y 30 concurrentes sobre
// productos independientes. Mide tiempo total.
// ═══════════════════════════════════════════════════════════════════════════

#[path = "../common/mod.rs"]
mod common;

use common::{db, escalar_i64, seed_producto};
use sqlx::Row;
use yarvis_app_lib::backventanas::backempleado::empleaproveedores::compras::{
    registrar_compra_impl, ItemCompraRequest,
};
use yarvis_app_lib::backventanas::backempleado::empleaproveedores::proveedores::guardar_proveedor_impl;

async fn prov(pool: &sqlx::SqlitePool) -> i64 {
    guardar_proveedor_impl(pool, "Estrés SA".into(), None, None).await.unwrap()
}

fn recepcion(pid: i64, nombre: &str) -> (Vec<ItemCompraRequest>, f64) {
    (
        vec![ItemCompraRequest {
            producto_id: Some(pid),
            nombre: nombre.into(),
            presentacion: "unidad".into(),
            cantidad: 2.0,
            piezas_por_paquete: None,
            paquetes: None,
        }],
        10.0,
    )
}

#[tokio::test]
async fn trescientas_recepciones_secuenciales_integridad_exacta() {
    let pool = db().await;
    let p = prov(&pool).await;
    let pid = seed_producto(&pool, "Masivo", 0.0, 13.5).await;
    let t0 = std::time::Instant::now();

    for i in 0..300 {
        let (items, monto) = recepcion(pid, "Masivo");
        let r = registrar_compra_impl(&pool, 1, p, items, monto, "efectivo".into(), None).await;
        assert!(r.is_ok(), "compra #{} falló: {:?}", i, r.err());
    }
    let ms = t0.elapsed().as_millis();
    println!("[estres] 300 compras secuenciales en {} ms ({} ms/compra)", ms, ms / 300);

    // Integridad: cada compra con su renglón, nada huérfano ni parcial.
    assert_eq!(escalar_i64(&pool, "SELECT COUNT(*) FROM compras").await, 300);
    assert_eq!(escalar_i64(&pool, "SELECT COUNT(*) FROM compras_items").await, 300);
    let huerfanos: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM compras_items i LEFT JOIN compras c ON c.id = i.compra_id WHERE c.id IS NULL",
    ).fetch_one(&pool).await.unwrap();
    assert_eq!(huerfanos, 0);

    // Stock exacto: 300 recepciones × 2 unidades (solo suma, jamás resta).
    let fila = sqlx::query("SELECT stock, vendido FROM productos WHERE id = ?")
        .bind(pid).fetch_one(&pool).await.unwrap();
    assert_eq!(fila.get::<f64, _>("stock"), 600.0);
    // Sin corte abierto: las 300 quedaron pendientes, cero movimientos.
    assert_eq!(escalar_i64(&pool, "SELECT COUNT(*) FROM movimientos_caja").await, 0);
    let pendientes: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM compras WHERE movimiento_id IS NULL AND monto_pagado > 0",
    ).fetch_one(&pool).await.unwrap();
    assert_eq!(pendientes, 300);
}

#[tokio::test]
async fn treinta_recepciones_concurrentes_sin_perdidas() {
    let pool = db().await;
    let p = prov(&pool).await;

    let mut pids = Vec::new();
    for i in 0..30 {
        pids.push(seed_producto(&pool, &format!("EstresConc{}", i), 0.0, 20.0).await);
    }

    let t0 = std::time::Instant::now();
    let mut handles = Vec::new();
    for (i, pid) in pids.iter().enumerate() {
        let pool_ref = pool.clone();
        let pid = *pid;
        let nombre = format!("EstresConc{i}");
        handles.push(tokio::spawn(async move {
            let (items, monto) = recepcion(pid, &nombre);
            registrar_compra_impl(&pool_ref, 1, p, items, monto, "efectivo".into(), None)
                .await
                .is_ok()
        }));
    }
    let mut oks = 0;
    for h in handles {
        if h.await.unwrap_or(false) {
            oks += 1;
        }
    }
    println!("[estres] 30 compras concurrentes en {} ms", t0.elapsed().as_millis());

    assert_eq!(oks, 30);
    let con_stock_dos: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM productos WHERE stock = 2 AND nombre LIKE 'EstresConc%'",
    ).fetch_one(&pool).await.unwrap();
    assert_eq!(con_stock_dos, 30);
}
