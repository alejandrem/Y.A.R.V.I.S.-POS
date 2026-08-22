// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Módulo VENTAS (emplea_new_venta).
// Prueba completar_venta_impl: validaciones, persistencia de venta+items,
// descuento de stock, vinculación por cajero_id y detección de método de pago.
// ═══════════════════════════════════════════════════════════════════════════

#[path = "../common/mod.rs"]
mod common;

use common::{db, escalar_i64, seed_producto};
use sqlx::Row;
use yarvis_app_lib::backventanas::backempleado::emplea_new_venta::new_venta::completar_venta_impl;
use yarvis_app_lib::models::{CartItemRequest, VentaRequest};

fn venta(items: Vec<CartItemRequest>, total: f64, efectivo: f64) -> VentaRequest {
    VentaRequest {
        items,
        total,
        subtotal: total,
        descuento: 0.0,
        monto_efectivo: efectivo,
        monto_tarjeta: 0.0,
        monto_transferencia: 0.0,
        cliente_id: None,
    }
}

#[tokio::test]
async fn venta_valida_inserta_descuenta_y_vincula_cajero() {
    let pool = db().await;
    let p1 = seed_producto(&pool, "Coca-Cola", 10.0, 18.0).await;

    let v = venta(
        vec![CartItemRequest { id: Some(p1 as i32), nombre: "Coca-Cola".into(), precio_venta: 18.0, cantidad: 3.0 }],
        54.0,
        54.0,
    );
    let resp = completar_venta_impl(&pool, &v, "Peter".into(), 77).await.unwrap();

    assert!(resp.venta_id > 0);
    // Stock descontado y vendido acumulado
    let fila = sqlx::query("SELECT stock, vendido FROM productos WHERE id = ?")
        .bind(p1)
        .fetch_one(&pool).await.unwrap();
    let stock: f64 = fila.get("stock");
    let vendido: f64 = fila.get("vendido");
    assert_eq!(stock, 7.0);
    assert_eq!(vendido, 3.0);
    // Item persistido y vinculado a la venta
    let items = escalar_i64(&pool, "SELECT COUNT(*) FROM detalle_ventas").await;
    assert_eq!(items, 1);
    // Vinculación canónica por cajero_id
    let cajero_id: Option<i64> = sqlx::query_scalar("SELECT cajero_id FROM ventas WHERE id = ?")
        .bind(resp.venta_id).fetch_one(&pool).await.unwrap();
    assert_eq!(cajero_id, Some(77));
}

#[tokio::test]
async fn venta_sin_items_rechazada() {
    let pool = db().await;
    let r = completar_venta_impl(&pool, &venta(vec![], 0.0, 0.0), "x".into(), 1).await;
    assert!(r.is_err());
    assert_eq!(escalar_i64(&pool, "SELECT COUNT(*) FROM ventas").await, 0);
}

#[tokio::test]
async fn pago_menor_al_total_rechazado() {
    let pool = db().await;
    let p = seed_producto(&pool, "Pan", 5.0, 20.0).await;
    let v = venta(
        vec![CartItemRequest { id: Some(p as i32), nombre: "Pan".into(), precio_venta: 20.0, cantidad: 2.0 }],
        40.0,
        30.0,
    );
    let r = completar_venta_impl(&pool, &v, "x".into(), 1).await;
    assert!(r.is_err());
    // El stock NO debe haberse tocado
    let stock: f64 = sqlx::query_scalar("SELECT stock FROM productos WHERE id = ?")
        .bind(p).fetch_one(&pool).await.unwrap();
    assert_eq!(stock, 5.0);
}

#[tokio::test]
async fn item_sin_producto_no_rompe_la_venta() {
    let pool = db().await;
    let v = venta(
        vec![CartItemRequest { id: None, nombre: "Producto suelto".into(), precio_venta: 15.0, cantidad: 1.0 }],
        15.0,
        20.0,
    );
    let resp = completar_venta_impl(&pool, &v, "x".into(), 1).await.unwrap();
    let items = escalar_i64(&pool, "SELECT COUNT(*) FROM detalle_ventas").await;
    assert_eq!(items, 1);
    assert_eq!(resp.mensaje.len() > 0, true);
}

#[tokio::test]
async fn metodo_pago_mixto_detectado() {
    let pool = db().await;
    let p = seed_producto(&pool, "Refresco", 9.0, 25.0).await;
    let mut v = venta(
        vec![CartItemRequest { id: Some(p as i32), nombre: "Refresco".into(), precio_venta: 25.0, cantidad: 1.0 }],
        25.0,
        10.0,
    );
    v.monto_tarjeta = 15.0;
    completar_venta_impl(&pool, &v, "x".into(), 1).await.unwrap();
    let metodo: String = sqlx::query_scalar("SELECT metodo_pago FROM ventas LIMIT 1")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(metodo, "efectivo/tarjeta");
}
