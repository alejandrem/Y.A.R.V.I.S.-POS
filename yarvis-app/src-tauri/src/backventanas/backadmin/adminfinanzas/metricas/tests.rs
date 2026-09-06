// Tests de métricas: la utilidad neta sale de tablas vivas, no de caché.

use super::*;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::Executor;

#[tokio::test]
async fn utilidad_viva_no_depende_de_resumen_materializado() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();

    pool.execute(
        "CREATE TABLE ventas (id INTEGER PRIMARY KEY, fecha TEXT, total INTEGER, iva INTEGER, estado TEXT)",
    )
    .await
    .unwrap();
    pool.execute(
        "CREATE TABLE detalle_ventas (venta_id INTEGER, producto_id INTEGER, cantidad REAL)",
    )
    .await
    .unwrap();
    pool.execute("CREATE TABLE productos (id INTEGER PRIMARY KEY, precio_costo INTEGER)")
        .await
        .unwrap();
    pool.execute("CREATE TABLE pagos_gastos (fecha_pago TEXT, monto_pagado INTEGER)")
        .await
        .unwrap();

    // Montos en CENTAVOS: $100.00 - $60.00 - $50.00 - $16.00
    sqlx::query("INSERT INTO ventas VALUES (1, '2026-08-10 12:00:00', 10000, 1600, 'completada')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO productos VALUES (1, 6000)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO detalle_ventas VALUES (1, 1, 1)")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO pagos_gastos VALUES ('2026-08-10', 5000)")
        .execute(&pool)
        .await
        .unwrap();

    // 100 - 60 - 50 - 16 = -26 (en pesos). No se crea resumen_financiero_diario:
    // el cálculo debe depender de las tablas transaccionales vivas.
    let utilidad = calcular_utilidad_neta_periodo(&pool, "2026-08-01", "2026-08-31")
        .await
        .unwrap();
    assert!((utilidad + 26.0).abs() < f64::EPSILON);
}
