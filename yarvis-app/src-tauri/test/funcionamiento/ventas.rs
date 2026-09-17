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
        vec![CartItemRequest { id: Some(p1), nombre: "Coca-Cola".into(), precio_venta: 18.0, cantidad: 3.0, descuento: 0.0 }],
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
        vec![CartItemRequest { id: Some(p), nombre: "Pan".into(), precio_venta: 20.0, cantidad: 2.0, descuento: 0.0 }],
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
        vec![CartItemRequest { id: None, nombre: "Producto suelto".into(), precio_venta: 15.0, cantidad: 1.0, descuento: 0.0 }],
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
        vec![CartItemRequest { id: Some(p), nombre: "Refresco".into(), precio_venta: 25.0, cantidad: 1.0, descuento: 0.0 }],
        25.0,
        10.0,
    );
    v.monto_tarjeta = 15.0;
    completar_venta_impl(&pool, &v, "x".into(), 1).await.unwrap();
    let metodo: String = sqlx::query_scalar("SELECT metodo_pago FROM ventas LIMIT 1")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(metodo, "efectivo/tarjeta");
}

#[tokio::test]
async fn sobreventa_permitida_deja_stock_negativo() {
    let pool = db().await;
    // Quedan 2 unidades; se venden 5 → stock queda en -3 (sobreventa
    // intencional: el físico puede tener más que el sistema).
    let p = seed_producto(&pool, "Último refresco", 2.0, 18.0).await;
    let v = venta(
        vec![CartItemRequest { id: Some(p), nombre: "Último refresco".into(), precio_venta: 18.0, cantidad: 5.0, descuento: 0.0 }],
        90.0,
        90.0,
    );

    let r = completar_venta_impl(&pool, &v, "Peter".into(), 1).await;
    assert!(r.is_ok(), "la sobreventa debe permitirse, error: {:?}", r.err());

    // Venta + items SÍ quedaron escritos y el stock quedó negativo.
    assert_eq!(escalar_i64(&pool, "SELECT COUNT(*) FROM ventas").await, 1);
    assert_eq!(escalar_i64(&pool, "SELECT COUNT(*) FROM detalle_ventas").await, 1);
    let fila = sqlx::query("SELECT stock, vendido FROM productos WHERE id = ?")
        .bind(p).fetch_one(&pool).await.unwrap();
    let stock: f64 = Row::get(&fila, "stock");
    let vendido: f64 = Row::get(&fila, "vendido");
    assert_eq!(stock, -3.0, "el stock debe quedar en negativo para conciliar");
    assert_eq!(vendido, 5.0);
}

#[tokio::test]
async fn venta_multi_item_permite_sobreventa_en_un_item() {
    let pool = db().await;
    let p_ok = seed_producto(&pool, "Alcanza", 10.0, 20.0).await;
    let p_no = seed_producto(&pool, "Sobreventa", 1.0, 30.0).await;

    // El segundo item supera su stock (vende 2 teniendo 1): con sobreventa
    // permitida la venta entera entra y ese producto queda en -1.
    let v = VentaRequest {
        items: vec![
            CartItemRequest { id: Some(p_ok), nombre: "Alcanza".into(), precio_venta: 20.0, cantidad: 3.0, descuento: 0.0 },
            CartItemRequest { id: Some(p_no), nombre: "Sobreventa".into(), precio_venta: 30.0, cantidad: 2.0, descuento: 0.0 },
        ],
        total: 120.0,
        subtotal: 120.0,
        descuento: 0.0,
        monto_efectivo: 120.0,
        monto_tarjeta: 0.0,
        monto_transferencia: 0.0,
        cliente_id: None,
    };

    let r = completar_venta_impl(&pool, &v, "Peter".into(), 1).await;
    assert!(r.is_ok(), "la sobreventa multi-item debe entrar, error: {:?}", r.err());

    let stock_ok: f64 = sqlx::query_scalar("SELECT stock FROM productos WHERE id = ?")
        .bind(p_ok).fetch_one(&pool).await.unwrap();
    let stock_no: f64 = sqlx::query_scalar("SELECT stock FROM productos WHERE id = ?")
        .bind(p_no).fetch_one(&pool).await.unwrap();
    assert_eq!(stock_ok, 7.0);
    assert_eq!(stock_no, -1.0, "el item en sobreventa queda en negativo");
}

#[tokio::test]
async fn fallo_a_mitad_de_venta_revierte_todo_transaccion() {
    let pool = db().await;
    let p_real = seed_producto(&pool, "Producto bueno", 10.0, 18.0).await;

    // El 2º item apunta a un producto inexistente: detalle_ventas.producto_id
    // tiene FK → productos(id), así que la inserción falla A MITAD de la venta.
    let v = VentaRequest {
        items: vec![
            CartItemRequest { id: Some(p_real), nombre: "Bueno".into(), precio_venta: 18.0, cantidad: 2.0, descuento: 0.0 },
            CartItemRequest { id: Some(999_999), nombre: "Fantasma".into(), precio_venta: 50.0, cantidad: 1.0, descuento: 0.0 },
        ],
        total: 86.0,
        subtotal: 86.0,
        descuento: 0.0,
        monto_efectivo: 100.0,
        monto_tarjeta: 0.0,
        monto_transferencia: 0.0,
        cliente_id: None,
    };

    let r = completar_venta_impl(&pool, &v, "Peter".into(), 1).await;
    assert!(r.is_err(), "la venta con producto fantasma debe fallar");

    // ATOMICIDAD: no quedó NADA — ni venta, ni items, ni stock tocado.
    assert_eq!(escalar_i64(&pool, "SELECT COUNT(*) FROM ventas").await, 0);
    assert_eq!(escalar_i64(&pool, "SELECT COUNT(*) FROM detalle_ventas").await, 0);
    let fila = sqlx::query("SELECT stock, vendido FROM productos WHERE id = ?")
        .bind(p_real).fetch_one(&pool).await.unwrap();
    let stock: f64 = Row::get(&fila, "stock");
    let vendido: f64 = Row::get(&fila, "vendido");
    assert_eq!(stock, 10.0, "el stock del producto bueno quedó intacto");
    assert_eq!(vendido, 0.0);
}

#[tokio::test]
async fn total_del_frontend_se_ignora_se_persiste_el_recalculado() {
    // Issue #5: el frontend manda total/subtotal mentirosos (ruido IEEE-754
    // o invoke manipulado). El backend cobra y persiste SU recálculo en
    // centavos; rechazar por inconsistencia rompería el ruido legítimo
    // (19.99*3 = 59.970000000000006), así que se tolera y se corrige.
    let pool = db().await;
    let p = seed_producto(&pool, "Jabón", 10.0, 12.0).await;
    let mut v = venta(
        vec![CartItemRequest { id: Some(p), nombre: "Jabón".into(), precio_venta: 12.0, cantidad: 2.0, descuento: 0.0 }],
        59.970000000000006,
        24.0,
    );
    v.subtotal = 0.01;
    completar_venta_impl(&pool, &v, "x".into(), 1).await.unwrap();
    let total: i64 = sqlx::query_scalar("SELECT total FROM ventas LIMIT 1")
        .fetch_one(&pool).await.unwrap();
    let subtotal: i64 = sqlx::query_scalar("SELECT subtotal FROM ventas LIMIT 1")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(subtotal, 2400);
    assert_eq!(total, 2400);
}

#[tokio::test]
async fn descuento_por_linea_guarda_neto_y_total_cuadra() {
    // 2x Coca 18 = 36 con desc 6 + 1x Pan 20 sin desc, global 0.
    // subtotal=5600, descuento=600, total=5000, detalle neto 3000/2000.
    let pool = db().await;
    let p1 = seed_producto(&pool, "Coca-Cola", 10.0, 18.0).await;
    let p2 = seed_producto(&pool, "Pan", 10.0, 20.0).await;
    let v = VentaRequest {
        items: vec![
            CartItemRequest {
                id: Some(p1),
                nombre: "Coca-Cola".into(),
                precio_venta: 18.0,
                cantidad: 2.0,
                descuento: 6.0,
            },
            CartItemRequest {
                id: Some(p2),
                nombre: "Pan".into(),
                precio_venta: 20.0,
                cantidad: 1.0,
                descuento: 0.0,
            },
        ],
        total: 0.0,
        subtotal: 0.0,
        descuento: 0.0,
        monto_efectivo: 50.0,
        monto_tarjeta: 0.0,
        monto_transferencia: 0.0,
        cliente_id: None,
    };
    completar_venta_impl(&pool, &v, "x".into(), 1)
        .await
        .unwrap();
    let (subtotal, descuento, total): (i64, i64, i64) =
        sqlx::query_as("SELECT subtotal, descuento, total FROM ventas LIMIT 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!((subtotal, descuento, total), (5600, 600, 5000));
    let filas: Vec<(i64, i64)> =
        sqlx::query_as("SELECT descuento, subtotal FROM detalle_ventas ORDER BY id")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(filas, vec![(600, 3000), (0, 2000)]);
}

#[tokio::test]
async fn descuento_linea_mas_global_se_suman() {
    // Bruto 100, linea 10 + global 5 => total 85, descuento 15.
    let pool = db().await;
    let p = seed_producto(&pool, "Leche", 10.0, 100.0).await;
    let v = VentaRequest {
        items: vec![CartItemRequest {
            id: Some(p),
            nombre: "Leche".into(),
            precio_venta: 100.0,
            cantidad: 1.0,
            descuento: 10.0,
        }],
        total: 0.0,
        subtotal: 0.0,
        descuento: 5.0,
        monto_efectivo: 85.0,
        monto_tarjeta: 0.0,
        monto_transferencia: 0.0,
        cliente_id: None,
    };
    completar_venta_impl(&pool, &v, "x".into(), 1)
        .await
        .unwrap();
    let (descuento, total): (i64, i64) =
        sqlx::query_as("SELECT descuento, total FROM ventas LIMIT 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!((descuento, total), (1500, 8500));
}

#[tokio::test]
async fn descuento_linea_mayor_a_su_bruto_se_rechaza() {
    let pool = db().await;
    let p = seed_producto(&pool, "Pan", 10.0, 20.0).await;
    let v = VentaRequest {
        items: vec![CartItemRequest {
            id: Some(p),
            nombre: "Pan".into(),
            precio_venta: 20.0,
            cantidad: 1.0,
            descuento: 25.0,
        }],
        total: 0.0,
        subtotal: 0.0,
        descuento: 0.0,
        monto_efectivo: 0.0,
        monto_tarjeta: 0.0,
        monto_transferencia: 0.0,
        cliente_id: None,
    };
    let r = completar_venta_impl(&pool, &v, "x".into(), 1).await;
    assert!(
        r.is_err(),
        "descuento de linea mayor a su bruto debe fallar"
    );
    assert_eq!(escalar_i64(&pool, "SELECT COUNT(*) FROM ventas").await, 0);
}

#[tokio::test]
async fn descuento_mayor_al_subtotal_se_rechaza() {
    let pool = db().await;
    let p = seed_producto(&pool, "Pan", 10.0, 20.0).await;
    let mut v = venta(
        vec![CartItemRequest { id: Some(p), nombre: "Pan".into(), precio_venta: 20.0, cantidad: 1.0, descuento: 0.0 }],
        0.0,
        0.0,
    );
    v.descuento = 50.0;
    let r = completar_venta_impl(&pool, &v, "x".into(), 1).await;
    assert!(r.is_err(), "descuento mayor al subtotal debe fallar");
    assert_eq!(escalar_i64(&pool, "SELECT COUNT(*) FROM ventas").await, 0);
}
