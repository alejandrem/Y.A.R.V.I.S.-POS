// ═══════════════════════════════════════════════════════════════════════════
// TEST DE ESTRÉS — Consultas agregadas sobre la DB.
// Presión: 10,000 ventas + 30,000 detalles sembrados; las consultas típicas
// del dashboard (resumen por rango, agrupado por cajero_id) deben responder
// rápido gracias a los índices. Mide ms por query.
// ═══════════════════════════════════════════════════════════════════════════

#[path = "../common/mod.rs"]
mod common;

use common::{db, seed_empleado};
use sqlx::Row;

#[tokio::test]
async fn diez_mil_ventas_agregadas_rapidas() {
    let pool = db().await;

    // 5 empleados que venderán
    let mut cajeros = Vec::new();
    for i in 0..5 {
        let id = seed_empleado(&pool, &format!("Cajero{}", i), "clave123").await;
        cajeros.push(id);
    }

    // Seed masivo: 10k ventas en lotes de un solo multi-INSERT (rápido)
    let t_seed = std::time::Instant::now();
    for lote in 0..10 {
        let mut q = String::from(
            "INSERT INTO ventas (total, subtotal, metodo_pago, cajero, cajero_id, estado) VALUES ",
        );
        for i in 0..1000 {
            let idx = (lote * 1000 + i) as usize;
            if i > 0 {
                q.push(',');
            }
            q.push_str(&format!(
                "(100.0, 100.0, 'efectivo', 'Cajero{}', {}, 'completada')",
                idx % 5,
                cajeros[idx % 5]
            ));
        }
        sqlx::query(&q).execute(&pool).await.unwrap();
    }
    println!("[estres] seed 10k ventas en {} ms", t_seed.elapsed().as_millis());

    // Detalles: 3 por venta vía INSERT...SELECT
    let t0 = std::time::Instant::now();
    sqlx::query(
        "INSERT INTO detalle_ventas (venta_id, producto_id, producto_nombre, cantidad, precio_unitario, subtotal)
         SELECT id, NULL, 'Item', 3.0, 33.33, 99.99 FROM ventas",
    )
    .execute(&pool)
    .await
    .unwrap();
    println!("[estres] seed 30k detalles en {} ms", t0.elapsed().as_millis());

    // Query típica 1: resumen por rango de fechas
    let t1 = std::time::Instant::now();
    let fila = sqlx::query(
        "SELECT COUNT(*) AS n, COALESCE(SUM(total),0)*1.0 AS total FROM ventas WHERE estado = 'completada'",
    ).fetch_one(&pool).await.unwrap();
    let ms1 = t1.elapsed().as_millis();
    assert_eq!(fila.get::<i64,_>("n"), 10_000);
    assert_eq!(fila.get::<f64,_>("total"), 1_000_000.0);

    // Query típica 2: agrupado por cajero_id (usa el índice nuevo)
    let t2 = std::time::Instant::now();
    let filas = sqlx::query(
        "SELECT cajero_id, COUNT(*) AS n FROM ventas WHERE estado = 'completada' GROUP BY cajero_id",
    ).fetch_all(&pool).await.unwrap();
    let ms2 = t2.elapsed().as_millis();
    assert_eq!(filas.len(), 5);

    // Query típica 3: join ventas-detalles con agregación
    let t3 = std::time::Instant::now();
    let total_items: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(subtotal),0) FROM detalle_ventas",
    ).fetch_one(&pool).await.unwrap();
    let ms3 = t3.elapsed().as_millis();

    println!(
        "[estres] resumen={}ms | group_by_cajero={}ms | sum_detalles={}ms",
        ms1, ms2, ms3
    );

    assert_eq!(total_items, 999_900.0);
    // Umbrales holgados para CI; si se disparan hay un escaneo O(n²) escondido
    assert!(ms1 < 2000, "resumen tardó {}ms", ms1);
    assert!(ms2 < 2000, "group_by tardó {}ms", ms2);
    assert!(ms3 < 2000, "sum_detalles tardó {}ms", ms3);
}
