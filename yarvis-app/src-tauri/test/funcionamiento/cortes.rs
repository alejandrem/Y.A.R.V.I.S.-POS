// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Módulo CORTES DE CAJA (adminfinanzas/cortes).
// Prueba cerrar_corte_impl: el servidor RECALCULA los totales desde la tabla
// ventas (los valores del cliente se ignoran), la diferencia de caja es
// exacta en centavos, solo cierra cortes abiertos y respeta la ventana
// [apertura, cierre] con el estado de las ventas.
// ═══════════════════════════════════════════════════════════════════════════

#[path = "../common/mod.rs"]
mod common;

use common::{db, seed_producto};
use sqlx::Row;
use yarvis_app_lib::backventanas::backadmin::adminfinanzas::cortes::cerrar_corte_impl;
use yarvis_app_lib::dinero::a_centavos;

/// Inserta un corte ABIERTO con ventana fija y devuelve su id.
async fn seed_corte(pool: &sqlx::SqlitePool, apertura: &str) -> i64 {
    let r = sqlx::query("INSERT INTO cortes_caja (fecha_apertura, estado) VALUES (?, 'abierto')")
        .bind(apertura)
        .execute(pool)
        .await
        .unwrap();
    r.last_insert_rowid()
}

/// Inserta una venta completada con fecha explícita.
async fn seed_venta(
    pool: &sqlx::SqlitePool,
    producto: i64,
    fecha: &str,
    metodo: &str,
    total_pesos: f64,
) {
    let total = a_centavos(total_pesos);
    let r = sqlx::query(
        "INSERT INTO ventas (fecha, total, subtotal, metodo_pago, cajero, estado) VALUES (?, ?, ?, ?, 'test', 'completada')",
    )
    .bind(fecha)
    .bind(total)
    .bind(total)
    .bind(metodo)
    .execute(pool)
    .await
    .unwrap();
    let venta_id = r.last_insert_rowid();
    sqlx::query(
        "INSERT INTO detalle_ventas (venta_id, producto_id, producto_nombre, cantidad, precio_unitario, subtotal) VALUES (?, ?, 'x', 1, ?, ?)",
    )
    .bind(venta_id)
    .bind(producto)
    .bind(total)
    .bind(total)
    .execute(pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn cierre_recalcula_totales_desde_ventas_ignorando_al_cliente() {
    let pool = db().await;
    let p = seed_producto(&pool, "Prod", 100.0, 10.0).await;
    let corte_id = seed_corte(&pool, "2026-08-20 00:00:00").await;

    // Ventas dentro de la ventana: $100 ef + $50 tj + $25 tr = $175
    seed_venta(&pool, p, "2026-08-21 10:00:00", "efectivo", 100.0).await;
    seed_venta(&pool, p, "2026-08-21 11:00:00", "tarjeta", 50.0).await;
    seed_venta(&pool, p, "2026-08-22 09:30:00", "transferencia", 25.0).await;
    // FUERA de la ventana: antes de la apertura — NO debe contar
    seed_venta(&pool, p, "2026-08-19 23:59:00", "efectivo", 999.0).await;
    // Cancelada dentro de la ventana — NO debe contar
    sqlx::query("INSERT INTO ventas (fecha, total, subtotal, metodo_pago, cajero, estado) VALUES ('2026-08-21 12:00:00', 50000, 50000, 'efectivo', 'test', 'cancelada')")
        .execute(&pool).await.unwrap();

    // Cierre SIN entradas ni retiros: diferencia = contado − vendido = 0.
    let resumen = cerrar_corte_impl(&pool, corte_id, 0.0, 0.0).await.unwrap();

    assert_eq!(resumen.total_efectivo, 100.0);
    assert_eq!(resumen.total_tarjeta, 50.0);
    assert_eq!(resumen.total_transferencia, 25.0);
    assert_eq!(resumen.total_ventas, 175.0);
    assert_eq!(resumen.diferencia, 0.0);

    // La DB quedó consistente (en centavos) y cerrada.
    let fila =
        sqlx::query("SELECT total_ventas, total_efectivo, diferencia, estado FROM cortes_caja WHERE id = ?")
            .bind(corte_id).fetch_one(&pool).await.unwrap();
    assert_eq!(fila.get::<i64, _>("total_ventas"), 17_500);
    assert_eq!(fila.get::<i64, _>("total_efectivo"), 10_000);
    assert_eq!(fila.get::<i64, _>("diferencia"), 0);
    assert_eq!(fila.get::<String, _>("estado"), "cerrado");
}

#[tokio::test]
async fn diferencia_refleja_entradas_y_retiros_en_centavos_exactos() {
    let pool = db().await;
    let p = seed_producto(&pool, "Prod2", 100.0, 5.0).await;
    let corte_id = seed_corte(&pool, "2026-08-20 00:00:00").await;

    // Vendido $33.33 en efectivo. El cajero retira $10 manualmente:
    // calculado = 33.33 − 10 = 23.33 → diferencia = −$10.00 exacta.
    seed_venta(&pool, p, "2026-08-21 10:00:00", "efectivo", 33.33).await;

    let resumen = cerrar_corte_impl(&pool, corte_id, 0.0, 10.0).await.unwrap();

    assert_eq!(resumen.total_ventas, 33.33);
    assert!((resumen.diferencia + 10.0).abs() < f64::EPSILON,
        "la diferencia debe ser exactamente -10.00, fue {}", resumen.diferencia);
}

#[tokio::test]
async fn no_permite_doble_cierre() {
    let pool = db().await;
    let corte_id = seed_corte(&pool, "2026-08-20 00:00:00").await;

    cerrar_corte_impl(&pool, corte_id, 0.0, 0.0).await.unwrap();
    let segundo = cerrar_corte_impl(&pool, corte_id, 0.0, 0.0).await;

    assert!(segundo.is_err(), "cerrar dos veces debe fallar");
}
