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

// ═══════════════════════════════════════════════════════════════════════════
// VISTA VENTAS — KPIs, desglose por empleado, top productos y pronóstico.
// ═══════════════════════════════════════════════════════════════════════════

use yarvis_app_lib::backventanas::backadmin::admintickets::tickets::{
    get_kpis_ventas_impl, get_top_productos_impl, get_ventas_con_pronostico_impl,
    get_ventas_por_empleado_dia_impl,
};

async fn seed_venta(pool: &sqlx::SqlitePool, fecha: &str, total_centavos: i64, cajero: &str) -> i64 {
    let r = sqlx::query("INSERT INTO ventas (fecha, total, estado, cajero) VALUES (?, ?, 'completada', ?)")
        .bind(fecha)
        .bind(total_centavos)
        .bind(cajero)
        .execute(pool)
        .await
        .unwrap();
    r.last_insert_rowid()
}

#[tokio::test]
async fn kpis_hoy_vs_ayer_suman_y_promedian() {
    let pool = db().await;
    let hoy: String = sqlx::query_scalar("SELECT date('now','localtime')")
        .fetch_one(&pool).await.unwrap();
    let ayer: String = sqlx::query_scalar("SELECT date('now','localtime','-1 day')")
        .fetch_one(&pool).await.unwrap();

    seed_venta(&pool, &format!("{hoy} 10:00:00"), 10_000, "MARIA").await;
    seed_venta(&pool, &format!("{hoy} 18:00:00"), 20_000, "JUAN").await;
    seed_venta(&pool, &format!("{ayer} 12:00:00"), 5_000, "MARIA").await;

    let v = get_kpis_ventas_impl(&pool, 1, 0).await.unwrap();
    assert_eq!(v["actual"]["total"], 300.0);
    assert_eq!(v["actual"]["tickets"], 2);
    assert_eq!(v["actual"]["ticket_promedio"], 150.0);
    assert_eq!(v["anterior"]["total"], 50.0);
    assert_eq!(v["anterior"]["tickets"], 1);
    assert!(v["actual"]["margen_pct"].is_number(), "falta margen");
}

#[tokio::test]
async fn ventas_por_empleado_agrupa_y_marca_sin_asignar() {
    let pool = db().await;
    let hoy: String = sqlx::query_scalar("SELECT date('now','localtime')")
        .fetch_one(&pool).await.unwrap();

    seed_venta(&pool, &format!("{hoy} 09:00:00"), 10_000, "MARIA").await;
    seed_venta(&pool, &format!("{hoy} 10:00:00"), 20_000, "JUAN").await;
    seed_venta(&pool, &format!("{hoy} 11:00:00"), 5_000, "").await;

    let filas = get_ventas_por_empleado_dia_impl(&pool, 7).await.unwrap();
    assert_eq!(filas.len(), 3);
    let sin = filas.iter().find(|f| f.cajero == "SIN ASIGNAR").expect("falta SIN ASIGNAR");
    assert_eq!(sin.total, 50.0);
    let juan = filas.iter().find(|f| f.cajero == "JUAN").unwrap();
    assert_eq!(juan.total, 200.0);
}

#[tokio::test]
async fn top_productos_ordena_por_ingreso_y_recorta_a_5() {
    let pool = db().await;
    let hoy: String = sqlx::query_scalar("SELECT date('now','localtime')")
        .fetch_one(&pool).await.unwrap();

    // 6 productos; debe volver el top 5 ordenado por subtotal.
    for (i, (nombre, subtotal_c)) in
        [("F", 1_000), ("E", 2_000), ("D", 3_000), ("C", 4_000), ("B", 5_000), ("A", 6_000)]
            .iter()
            .enumerate()
    {
        let vid = seed_venta(&pool, &format!("{hoy} 10:0{i}:00"), *subtotal_c, "MARIA").await;
        sqlx::query("INSERT INTO detalle_ventas (venta_id, producto_nombre, cantidad, precio_unitario, subtotal) VALUES (?, ?, 1, ?, ?)")
            .bind(vid)
            .bind(nombre)
            .bind(*subtotal_c)
            .bind(*subtotal_c)
            .execute(&pool)
            .await
            .unwrap();
    }

    let top = get_top_productos_impl(&pool, 30).await.unwrap();
    assert_eq!(top.len(), 5);
    assert_eq!(top[0].nombre, "A");
    assert_eq!(top[0].total, 60.0);
    assert_eq!(top[4].nombre, "E");
}

#[tokio::test]
async fn ventas_con_pronostico_engancha_historia_y_futuro() {
    // La DB de pruebas del pool es sqlx (async); el pronóstico usa rusqlite
    // por archivo, así que se siembra un temporal aparte.
    let path = std::env::temp_dir().join(format!(
        "yarvis_pron_{}.db",
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    {
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute_batch("CREATE TABLE ventas (id INTEGER PRIMARY KEY, fecha TEXT, total INTEGER, estado TEXT);")
            .unwrap();
        for i in 0..10 {
            conn.execute(
                "INSERT INTO ventas (fecha, total, estado) VALUES (?1, ?2, 'completada')",
                rusqlite::params![format!("2026-08-{:02} 12:00:00", 1 + i), 10_000 + i * 1_000],
            )
            .unwrap();
        }
    }

    let (historial, pronostico) =
        get_ventas_con_pronostico_impl(path.clone(), 7).await.unwrap();
    assert_eq!(historial.len(), 10);
    assert_eq!(historial.first().unwrap().fecha, "2026-08-01");
    assert_eq!(historial.last().unwrap().fecha, "2026-08-10");
    assert_eq!(pronostico.len(), 7);
    assert_eq!(pronostico.first().unwrap().fecha, "2026-08-11");
    for p in &pronostico {
        assert!(p.minimo <= p.prediccion + 1e-9 && p.prediccion - 1e-9 <= p.maximo);
    }
    let _ = std::fs::remove_file(&path);
}
