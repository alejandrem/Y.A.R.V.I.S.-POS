// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Mis tickets del empleado (empleatickets).
// Todo operator-scoped por cajero_id: aislamiento entre empleados,
// ventana de días, paginación, KPIs, agrupado por día y propiedad
// del detalle. Las canceladas nunca cuentan.
// ═══════════════════════════════════════════════════════════════════════════

#[path = "../common/mod.rs"]
mod common;

use common::db;
use yarvis_app_lib::backventanas::backempleado::empleatickets::mis_tickets::{
    mi_ticket_detalle_impl, mis_kpis_impl, mis_tickets_impl, mis_ventas_por_dia_impl, DIAS_TODOS,
};
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

/// Venta directa con fecha relativa ("0" = hoy, "-2" = hace 2 días).
async fn venta(
    pool: &sqlx::SqlitePool,
    cajero_id: i64,
    cajero: &str,
    total_pesos: f64,
    hace_dias: i64,
    estado: &str,
) -> i64 {
    let fecha = if hace_dias == 0 {
        "datetime('now','localtime')".to_string()
    } else {
        format!("datetime('now','localtime','-{hace_dias} days')")
    };
    let sql = format!(
        "INSERT INTO ventas (fecha, total, subtotal, descuento, metodo_pago, cajero, cajero_id, estado)
         VALUES ({fecha}, ?, ?, 0, 'efectivo', ?, ?, ?)"
    );
    sqlx::query(&sql)
        .bind(a_centavos(total_pesos))
        .bind(a_centavos(total_pesos))
        .bind(cajero)
        .bind(cajero_id)
        .bind(estado)
        .execute(pool)
        .await
        .unwrap()
        .last_insert_rowid()
}

async fn detalle(pool: &sqlx::SqlitePool, venta_id: i64) {
    sqlx::query(
        "INSERT INTO detalle_ventas (venta_id, producto_id, producto_nombre, cantidad, precio_unitario, subtotal)
         VALUES (?, NULL, 'Coca', 2, ?, ?)",
    )
    .bind(venta_id)
    .bind(a_centavos(20.0))
    .bind(a_centavos(40.0))
    .execute(pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn mis_tickets_solo_trae_los_mios_y_respeta_dias() {
    let pool = db().await;
    let yo = empleado(&pool, "Pedro").await;
    let otro = empleado(&pool, "Maria").await;
    venta(&pool, yo, "Pedro", 100.0, 0, "completada").await;
    venta(&pool, yo, "Pedro", 200.0, 10, "completada").await;
    venta(&pool, otro, "Maria", 999.0, 0, "completada").await;

    // dias=1 → solo la de hoy mía.
    let hoy = mis_tickets_impl(&pool, yo, "Pedro", 100, 0, 1).await.unwrap();
    assert_eq!(hoy.len(), 1);
    assert_eq!(hoy[0].total, 100.0);

    // dias=30 → las dos mías, nunca la de María.
    let mes = mis_tickets_impl(&pool, yo, "Pedro", 100, 0, 30).await.unwrap();
    assert_eq!(mes.len(), 2);
    assert!(mes.iter().all(|t| t.total != 999.0));
}

#[tokio::test]
async fn mis_tickets_excluye_canceladas_y_pagina() {
    let pool = db().await;
    let yo = empleado(&pool, "Pedro").await;
    for i in 0..3 {
        venta(&pool, yo, "Pedro", 50.0 + i as f64, 0, "completada").await;
    }
    venta(&pool, yo, "Pedro", 5000.0, 0, "cancelada").await;

    let todos = mis_tickets_impl(&pool, yo, "Pedro", 100, 0, 1).await.unwrap();
    assert_eq!(todos.len(), 3);

    // Paginación: 2 + 1, sin solaparse.
    let p1 = mis_tickets_impl(&pool, yo, "Pedro", 2, 0, 1).await.unwrap();
    let p2 = mis_tickets_impl(&pool, yo, "Pedro", 2, 2, 1).await.unwrap();
    assert_eq!(p1.len(), 2);
    assert_eq!(p2.len(), 1);
    assert_ne!(p1[0].id, p2[0].id);

    // Clamp: limit 0 → 1, dias 0 → 1 (hoy).
    let clamp = mis_tickets_impl(&pool, yo, "Pedro", 0, -5, 0).await.unwrap();
    assert_eq!(clamp.len(), 1);
}

#[tokio::test]
async fn mis_kpis_suman_y_promedian_solo_mias() {
    let pool = db().await;
    let yo = empleado(&pool, "Pedro").await;
    let otro = empleado(&pool, "Maria").await;
    venta(&pool, yo, "Pedro", 100.0, 0, "completada").await;
    venta(&pool, yo, "Pedro", 300.0, 0, "completada").await;
    venta(&pool, otro, "Maria", 1000.0, 0, "completada").await;

    let k = mis_kpis_impl(&pool, yo, "Pedro", 1).await.unwrap();
    assert_eq!(k.total, 400.0);
    assert_eq!(k.tickets, 2);
    assert_eq!(k.ticket_promedio, 200.0);

    // Sin ventas → ceros, no NaN.
    let vacio = mis_kpis_impl(&pool, otro, "Maria", 1).await.unwrap();
    let _ = venta(&pool, otro, "Maria", 1.0, 40, "completada").await;
    let _ = vacio;
    let solo_hoy_otro = mis_kpis_impl(&pool, otro, "Maria", 1).await.unwrap();
    assert_eq!(solo_hoy_otro.total, 1000.0);
}

#[tokio::test]
async fn por_dia_agrupa_y_detalle_exige_propiedad() {
    let pool = db().await;
    let yo = empleado(&pool, "Pedro").await;
    let otro = empleado(&pool, "Maria").await;
    let v1 = venta(&pool, yo, "Pedro", 100.0, 0, "completada").await;
    venta(&pool, yo, "Pedro", 50.0, 0, "completada").await;
    venta(&pool, yo, "Pedro", 70.0, 2, "completada").await;
    detalle(&pool, v1).await;

    let dias = mis_ventas_por_dia_impl(&pool, yo, "Pedro", 7).await.unwrap();
    assert_eq!(dias.len(), 2);
    // Orden ascendente: el más viejo primero.
    assert!(dias[0].total == 70.0 && dias[1].total == 150.0);

    // Detalle propio OK con items en pesos.
    let d = mi_ticket_detalle_impl(&pool, yo, "Pedro", v1).await.unwrap();
    assert_eq!(d.total, 100.0);
    assert_eq!(d.items.len(), 1);
    assert_eq!(d.items[0].precio_unitario, 20.0);

    // Otro empleado no puede verlo (mismo error que inexistente).
    let ajeno = mi_ticket_detalle_impl(&pool, otro, "Maria", v1).await;
    assert_eq!(ajeno.unwrap_err(), "Ticket no encontrado");
    let in9 = mi_ticket_detalle_impl(&pool, yo, "Pedro", 999999).await;
    assert!(in9.is_err());
}

#[tokio::test]
async fn fallback_nombre_rescata_ventas_nunca_vinculadas() {
    let pool = db().await;
    // Usuario creado manual DESPUÉS de las ventas: cajero_id quedó NULL
    // pero la etiqueta coincide con su nombre.
    sqlx::query(
        "INSERT INTO ventas (fecha, total, subtotal, descuento, metodo_pago, cajero, cajero_id, estado)
         VALUES (datetime('now','localtime'), ?, ?, 0, 'efectivo', 'Pedro', NULL, 'completada')",
    )
    .bind(a_centavos(250.0))
    .bind(a_centavos(250.0))
    .execute(&pool)
    .await
    .unwrap();
    // IMPORTADOR nunca es de nadie.
    sqlx::query(
        "INSERT INTO ventas (fecha, total, subtotal, descuento, metodo_pago, cajero, cajero_id, estado)
         VALUES (datetime('now','localtime'), ?, ?, 0, 'efectivo', 'IMPORTADOR', NULL, 'completada')",
    )
    .bind(a_centavos(9999.0))
    .bind(a_centavos(9999.0))
    .execute(&pool)
    .await
    .unwrap();
    let yo = empleado(&pool, "Pedro").await;

    let lista = mis_tickets_impl(&pool, yo, "Pedro", 100, 0, 1).await.unwrap();
    assert_eq!(lista.len(), 1);
    assert_eq!(lista[0].total, 250.0);

    let k = mis_kpis_impl(&pool, yo, "Pedro", 1).await.unwrap();
    assert_eq!(k.total, 250.0);
    assert_eq!(k.tickets, 1);

    let dias = mis_ventas_por_dia_impl(&pool, yo, "Pedro", 7).await.unwrap();
    assert_eq!(dias.len(), 1);
    assert_eq!(dias[0].total, 250.0);

    let d = mi_ticket_detalle_impl(&pool, yo, "Pedro", lista[0].id)
        .await
        .unwrap();
    assert_eq!(d.total, 250.0);
}

#[tokio::test]
async fn todos_trae_historial_viejo_que_30_dias_excluye() {
    let pool = db().await;
    let yo = empleado(&pool, "Pedro").await;
    venta(&pool, yo, "Pedro", 77.0, 400, "completada").await;
    venta(&pool, yo, "Pedro", 10.0, 0, "completada").await;

    let todo = mis_tickets_impl(&pool, yo, "Pedro", 100, 0, DIAS_TODOS)
        .await
        .unwrap();
    assert_eq!(todo.len(), 2);

    let mes = mis_tickets_impl(&pool, yo, "Pedro", 100, 0, 30)
        .await
        .unwrap();
    assert_eq!(mes.len(), 1);
    assert_eq!(mes[0].total, 10.0);

    let k = mis_kpis_impl(&pool, yo, "Pedro", DIAS_TODOS).await.unwrap();
    assert_eq!(k.total, 87.0);
    assert_eq!(k.tickets, 2);
}
