// ═══════════════════════════════════════════════════════════════════════════
// TEST DE ESTRÉS — Módulo VENTAS.
// Presión: 500 cobros secuenciales con verificación de integridad exacta
// (total == suma de subtotales, stock jamás negativo) y 50 ventas
// concurrentes sobre productos independientes. Mide tiempo total.
// ═══════════════════════════════════════════════════════════════════════════

#[path = "../common/mod.rs"]
mod common;

use common::{db, escalar_i64, seed_producto};
use sqlx::Row;
use yarvis_app_lib::backventanas::backempleado::emplea_new_venta::new_venta::completar_venta_impl;
use yarvis_app_lib::models::{CartItemRequest, VentaRequest};

fn venta_de(pid: i32, precio: f64) -> VentaRequest {
    VentaRequest {
        items: vec![CartItemRequest {
            id: Some(pid),
            nombre: "Producto estrés".into(),
            precio_venta: precio,
            cantidad: 1.0,
        }],
        total: precio,
        subtotal: precio,
        descuento: 0.0,
        monto_efectivo: precio,
        monto_tarjeta: 0.0,
        monto_transferencia: 0.0,
        cliente_id: None,
    }
}

#[tokio::test]
async fn quinientas_ventas_secuenciales_integridad_exacta() {
    let pool = db().await;
    let p = seed_producto(&pool, "Masivo", 10000.0, 13.5).await;
    let t0 = std::time::Instant::now();

    for i in 0..500 {
        let r = completar_venta_impl(&pool, &venta_de(p as i32, 13.5), "cajero".into(), 1).await;
        assert!(r.is_ok(), "venta #{} falló: {:?}", i, r.err());
    }
    let ms = t0.elapsed().as_millis();
    println!("[estres] 500 ventas secuenciales en {} ms ({} ms/venta)", ms, ms / 500);

    // Integridad exacta: cada venta cuadra contra sus detalles
    let descuadradas: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM ventas v WHERE ABS(v.total - (SELECT COALESCE(SUM(subtotal),0) FROM detalle_ventas d WHERE d.venta_id = v.id)) > 0.01",
    ).fetch_one(&pool).await.unwrap();
    assert_eq!(descuadradas, 0);

    // Vendido y stock exactos tras 500 cobros de 1 pieza
    let fila = sqlx::query("SELECT stock, vendido FROM productos WHERE id = ?")
        .bind(p).fetch_one(&pool).await.unwrap();
    assert_eq!(fila.get::<f64,_>("vendido"), 500.0);
    assert_eq!(fila.get::<f64,_>("stock"), 9500.0);
}

#[tokio::test]
async fn cincuenta_ventas_concurrentes_sin_perdidas() {
    let pool = db().await;

    let mut productos = Vec::new();
    for i in 0..50 {
        productos.push(seed_producto(&pool, &format!("Conc{}", i), 10.0, 20.0).await);
    }

    let t0 = std::time::Instant::now();
    let mut handles = Vec::new();
    for pid in &productos {
        let pool_ref = pool.clone();
        let pid = *pid as i32;
        handles.push(tokio::spawn(async move {
            completar_venta_impl(&pool_ref, &venta_de(pid, 20.0), "c".into(), 1).await.is_ok()
        }));
    }
    let mut oks = 0;
    for h in handles {
        if h.await.unwrap_or(false) {
            oks += 1;
        }
    }
    println!("[estres] 50 ventas concurrentes en {} ms", t0.elapsed().as_millis());

    assert_eq!(oks, 50);
    let con_una_venta: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM productos WHERE vendido = 1 AND nombre LIKE 'Conc%'",
    ).fetch_one(&pool).await.unwrap();
    assert_eq!(con_una_venta, 50);
}
