// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Automatizador de códigos (semaforo verde + rojo).
// Cubre la migración 0015 + el backend de
// `backventanas/codigos_barras/{semaforo_verde,semaforo_rojo}` con filas
// REALES del dataset (EANs que sí pasan checksum):
//   * verde asigna cuando todo cuadra y rechaza con motivo cuando no.
//   * importar_catalogo guarda el espejo + auto-asigna al gemelo.
//   * rojo registra idempotente, resuelve por alta y por asignación, y
//     el aprendizaje (vinculo manual-rojo) queda para el futuro amarillo.
// ═══════════════════════════════════════════════════════════════════════════

#[path = "../common/mod.rs"]
mod common;

use common::{db, escalar_i64};
use yarvis_app_lib::backventanas::codigos_barras::semaforo_rojo::{
    buscar_aprendizaje_impl, contar_pendientes_impl, listar_pendientes_impl,
    registrar_pendiente_impl, resolver_asignando_impl, resolver_con_alta_impl,
};
use yarvis_app_lib::backventanas::codigos_barras::semaforo_verde::{
    contar_verde_impl, importar_catalogo_barras_impl, intentar_verde_impl, CatalogoRow,
};

fn fila(ean: &str, nombre: &str, marca: &str, cantidad: f64, unidad: &str) -> CatalogoRow {
    CatalogoRow {
        ean: ean.into(),
        nombre: nombre.into(),
        marca: Some(marca.into()),
        cantidad,
        unidad: unidad.into(),
        categoria: None,
    }
}

/// Producto de tienda con presentación canónica (lo que el verde exige).
async fn seed_con_presentacion(
    pool: &sqlx::SqlitePool,
    nombre: &str,
    marca: &str,
    cantidad: f64,
    unidad: &str,
) -> i64 {
    let r = sqlx::query(
        "INSERT INTO productos (nombre, precio_costo, precio_venta, stock, stock_minimo, vendido, marca, cantidad_presentacion, unidad_presentacion) VALUES (?, 0, 0, 0, 0, 0, ?, ?, ?)",
    )
    .bind(nombre)
    .bind(marca)
    .bind(cantidad)
    .bind(unidad)
    .execute(pool)
    .await
    .unwrap();
    r.last_insert_rowid()
}

#[tokio::test]
async fn verde_asigna_cuando_todo_cuadra() {
    let pool = db().await;
    // Gemelo de dataset: SABRITAS ORIGINAL 42g (ean real 7501011101456).
    let id = seed_con_presentacion(&pool, "SABRITAS ORIGINAL", "Sabritas", 42.0, "g").await;

    let v = intentar_verde_impl(&pool, "SABRITAS ORIGINAL 42g", Some("Sabritas"), "7501011101456", id, None)
        .await
        .unwrap();
    assert!(v.asignado, "debió asignar, motivo: {}", v.motivo);

    let cb: Option<String> =
        sqlx::query_scalar("SELECT codigo_barras FROM productos WHERE id = ?")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(cb.as_deref(), Some("7501011101456"));
    assert_eq!(
        escalar_i64(&pool, "SELECT COUNT(*) FROM vinculos_codigos WHERE origen = 'auto-verde'").await,
        1
    );
}

#[tokio::test]
async fn verde_rechaza_medida_distinta_aunque_nombre_parezca() {
    let pool = db().await;
    // Tienda tiene COCA 3L; el ticket/candidato trae ean de COCA 600ml.
    let id = seed_con_presentacion(&pool, "COCA COLA ORIGINAL", "Coca-Cola", 3.0, "l").await;

    // ean real de COCA 600ml del dataset.
    let v = intentar_verde_impl(&pool, "COCA COLA ORIGINAL 600ml", Some("Coca-Cola"), "0000075007614", id, None)
        .await
        .unwrap();
    assert!(!v.asignado, "la medida manda: 600ml != 3L");
    // Nada se escribió.
    assert_eq!(
        escalar_i64(&pool, "SELECT COUNT(*) FROM vinculos_codigos").await,
        0
    );
}

#[tokio::test]
async fn verde_no_pisa_codigo_ajeno_ni_duplicado() {
    let pool = db().await;
    let a = seed_con_presentacion(&pool, "SABRITAS ORIGINAL", "Sabritas", 42.0, "g").await;
    let b = seed_con_presentacion(&pool, "SABRITAS ORIGINAL", "Sabritas", 42.0, "g").await;

    intentar_verde_impl(&pool, "SABRITAS ORIGINAL 42g", Some("Sabritas"), "7501011101456", a, None)
        .await
        .unwrap();
    // El mismo ean para otro producto -> rechazado, no pisado.
    let v = intentar_verde_impl(&pool, "SABRITAS ORIGINAL 42g", Some("Sabritas"), "7501011101456", b, None)
        .await
        .unwrap();
    assert!(!v.asignado);
    let cb: Option<String> =
        sqlx::query_scalar("SELECT codigo_barras FROM productos WHERE id = ?")
            .bind(b)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(cb, None);
}

#[tokio::test]
async fn importar_dataset_guarda_espejo_y_autoasigna_gemelo() {
    let pool = db().await;
    let id = seed_con_presentacion(&pool, "LECHE SANTA CLARA CAFE", "Santa Clara", 250.0, "ml").await;

    let res = importar_catalogo_barras_impl(
        &pool,
        &[
            fila("7501055370276", "LECHE SANTA CLARA CAFE", "Santa Clara", 250.0, "ml"),
            fila("7501011101456", "SABRITAS ORIGINAL", "Sabritas", 42.0, "g"),
        ],
        None,
    )
    .await
    .unwrap();
    assert_eq!(res.catalogo_upserts, 2);
    assert_eq!(res.verde_asignados, 1, "solo la leche tiene gemelo en tienda");
    assert_eq!(res.sin_match, 1);
    assert!(res.errores.is_empty(), "errores: {:?}", res.errores);

    let cb: Option<String> =
        sqlx::query_scalar("SELECT codigo_barras FROM productos WHERE id = ?")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(cb.as_deref(), Some("7501055370276"));

    // Re-importar es idempotente: espejo hace upsert, verde no duplica.
    let res2 = importar_catalogo_barras_impl(
        &pool,
        &[fila("7501055370276", "LECHE SANTA CLARA CAFE", "Santa Clara", 250.0, "ml")],
        None,
    )
    .await
    .unwrap();
    assert_eq!(res2.verde_asignados, 0);
    assert_eq!(
        escalar_i64(&pool, "SELECT COUNT(*) FROM vinculos_codigos").await,
        1
    );
}

#[tokio::test]
async fn rojo_registra_idempotente_y_resuelve_asignando() {
    let pool = db().await;
    let prod = seed_con_presentacion(&pool, "PAN DULCE SURTIDO", "Bimbo", 500.0, "g").await;

    let p1 = registrar_pendiente_impl(&pool, Some("7501000111206"), "PAN DULCE SURTIDO")
        .await
        .unwrap();
    let p2 = registrar_pendiente_impl(&pool, Some("7501000111206"), "PAN DULCE SURTIDO")
        .await
        .unwrap();
    assert_eq!(p1, p2, "mismo (nombre, ean) = mismo pendiente");
    let veces: i64 = sqlx::query_scalar("SELECT veces_visto FROM pendientes_codigos WHERE id = ?")
        .bind(p1)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(veces, 2);

    // Loop de oro en caja: pitaste desconocido -> lo asignas -> aprende.
    resolver_asignando_impl(&pool, p1, prod, None, None).await.unwrap();

    let estado: String =
        sqlx::query_scalar("SELECT estado FROM pendientes_codigos WHERE id = ?")
            .bind(p1)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(estado, "resuelto");
    let cb: Option<String> =
        sqlx::query_scalar("SELECT codigo_barras FROM productos WHERE id = ?")
            .bind(prod)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(cb.as_deref(), Some("7501000111206"));
    // El aprendizaje queda para el futuro amarillo.
    let aprendido =
        buscar_aprendizaje_impl(&pool, &src_ia::embeddings::normalizar("PAN DULCE SURTIDO"))
            .await
            .unwrap();
    assert_eq!(aprendido, Some(prod));
}

#[tokio::test]
async fn rojo_resuelve_con_alta_y_cuenta_cola() {
    let pool = db().await;
    let p = registrar_pendiente_impl(&pool, None, "PRODUCTO NUEVO RARO").await.unwrap();

    let nuevo = resolver_con_alta_impl(
        &pool,
        p,
        "PRODUCTO NUEVO RARO",
        Some("7509990000111"),
        Some("abarrotes"),
        Some("Bimbo"),
        Some(100.0),
        Some("g"),
        None,
    )
    .await
    .unwrap();
    assert!(nuevo > 0);

    let c = contar_pendientes_impl(&pool).await.unwrap();
    assert_eq!(c.resuelto, 1);
    let pendientes = listar_pendientes_impl(&pool, None, 10).await.unwrap();
    assert!(pendientes.is_empty(), "lo resuelto no vuelve a la cola");
}

#[tokio::test]
async fn contar_verde_hoy_solo_cuenta_los_de_hoy() {
    let pool = db().await;
    let id = seed_con_presentacion(&pool, "SABRITAS ORIGINAL", "Sabritas", 42.0, "g").await;
    intentar_verde_impl(&pool, "SABRITAS ORIGINAL 42g", Some("Sabritas"), "7501011101456", id, None)
        .await
        .unwrap();
    // Un vinculo de ayer no entra en el contador de hoy.
    sqlx::query("INSERT INTO vinculos_codigos (ean, producto_id, origen, creado_en) VALUES ('7501011101463', ?, 'auto-verde', datetime('now', '-1 day'))")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();
    let c = contar_verde_impl(&pool).await.unwrap();
    assert_eq!(c.verdes_hoy, 1);
    assert_eq!(c.total_vinculos, 2);
}
