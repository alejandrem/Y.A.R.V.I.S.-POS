// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Módulo TICKETS (admintickets).
// Prueba guardar_ticket_parseado_impl: atomicidad de la importación
// (venta + detalles + inventario en una sola transacción), reporte visible
// de items sin vincular y ausencia de errores silenciados.
// ═══════════════════════════════════════════════════════════════════════════

#[path = "../common/mod.rs"]
mod common;

use common::{db, escalar_i64, seed_producto};
use sqlx::Row;
use yarvis_app_lib::backventanas::backadmin::admintickets::tickets::guardar_ticket_parseado_impl;
use yarvis_app_lib::models::TicketItem;

fn item(producto: &str, cantidad: f64, precio: f64) -> TicketItem {
    TicketItem {
        producto: producto.into(),
        cantidad,
        precio,
        total: precio * cantidad,
    }
}

#[tokio::test]
async fn ticket_importa_y_vincula_inventario_por_nombre() {
    let pool = db().await;
    seed_producto(&pool, "Coca-Cola 600ml", 10.0, 18.0).await;

    let msg = guardar_ticket_parseado_impl(
        &pool,
        vec![item("coca-cola 600ML", 3.0, 18.0)],
        54.0,
        Some("2026-08-01".into()),
        Some("13:30".into()),
        Some("efectivo".into()),
    )
    .await
    .unwrap();

    assert!(msg.contains("correctamente"), "mensaje inesperado: {msg}");
    // La venta quedó registrada como IMPORTADOR con su fecha
    let (total, fecha): (i64, Option<String>) =
        sqlx::query_as("SELECT total, fecha FROM ventas WHERE cajero = 'IMPORTADOR'")
            .fetch_one(&pool).await.unwrap();
    assert_eq!(total, 5_400);
    assert_eq!(fecha.as_deref(), Some("2026-08-01 13:30:00"));
    // El item quedó en detalle_ventas vinculado a la venta
    assert_eq!(escalar_i64(&pool, "SELECT COUNT(*) FROM detalle_ventas").await, 1);
    // Stock y vendido ajustados (match case-insensitive)
    let fila = sqlx::query("SELECT stock, vendido FROM productos WHERE nombre = 'Coca-Cola 600ml'")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(fila.get::<f64, _>("stock"), 7.0);
    assert_eq!(fila.get::<f64, _>("vendido"), 3.0);
}

#[tokio::test]
async fn producto_sin_coincidencia_se_reporta_no_se_silencia() {
    let pool = db().await;
    seed_producto(&pool, "Pan Bimbo", 5.0, 42.0).await;

    // "Leche Lala" NO existe en inventario: antes esto se tragaba con `let _ =`.
    let msg = guardar_ticket_parseado_impl(
        &pool,
        vec![
            item("Pan Bimbo", 1.0, 42.0),
            item("Leche Lala", 2.0, 25.0),
        ],
        92.0,
        None,
        None,
        None,
    )
    .await
    .unwrap();

    // El resultado debe REPORTAR la no-vinculación, no fingir éxito total.
    assert!(
        msg.contains("sin coincidencia"),
        "la falta de vinculación debe ser visible al usuario: {msg}"
    );
    // Pan Bimbo sí se ajustó; Leche Lala no existe así que nadie más cambió
    let fila = sqlx::query("SELECT stock, vendido FROM productos WHERE nombre = 'Pan Bimbo'")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(fila.get::<f64, _>("stock"), 4.0);
    assert_eq!(fila.get::<f64, _>("vendido"), 1.0);
}

#[tokio::test]
async fn importacion_es_atomica_todo_o_nada() {
    let pool = db().await;

    // Un item con precio NaN fuerza el fallo del INSERT a mitad del lote
    // (SQLite rechaza NULL/NaN en columnas NOT NULL / reales).
    let malo = TicketItem {
        producto: "Producto maldito".into(),
        cantidad: f64::NAN,
        precio: 10.0,
        total: 10.0,
    };

    let r = guardar_ticket_parseado_impl(
        &pool,
        vec![item("Alcanza", 1.0, 20.0), malo],
        30.0,
        None,
        None,
        None,
    )
    .await;

    assert!(r.is_err(), "un item corrupto debe abortar la importación");
    // ATOMICIDAD: no quedó venta ni detalles a medias.
    assert_eq!(escalar_i64(&pool, "SELECT COUNT(*) FROM ventas").await, 0);
    assert_eq!(escalar_i64(&pool, "SELECT COUNT(*) FROM detalle_ventas").await, 0);
}
