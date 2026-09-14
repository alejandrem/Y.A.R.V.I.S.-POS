// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Proveedores del empleado (empleaproveedores).
// Alta con validación y duplicado amigable, sugerencia desde costo,
// compra todo-o-nada (stock + egreso o pendiente), validaciones y
// el dilema del no-registrado (FK lo hace imposible en disco).
// ═══════════════════════════════════════════════════════════════════════════

#[path = "../common/mod.rs"]
mod common;

use common::{db, seed_empleado, seed_producto};
use yarvis_app_lib::backventanas::backempleado::empleaproveedores::compras::{
    get_compra_detalle_impl, historial_compras_impl, rectificar_compra_impl, registrar_compra_impl,
    ItemCompraRequest,
};

#[tokio::test]
async fn rectificar_crea_nueva_revierte_stock_y_conserva_original() {
    let pool = db().await;
    let prov = proveedor(&pool, "Recti").await;
    let pid = seed_producto(&pool, "Leche", 10.0, 20.0).await;

    let orig = registrar_compra_impl(&pool, 1, prov, vec![item(Some(pid), "Leche", 4.0)], 40.0, "efectivo".into(), None)
        .await
        .unwrap();

    // Rectificativa: 2 unidades y $20. Misma factura, otros números.
    let rect = rectificar_compra_impl(&pool, 1, orig.compra_id, vec![item(Some(pid), "Leche", 2.0)], 20.0, "efectivo".into(), Some("me equivoqué".into()))
        .await
        .unwrap();
    assert_ne!(rect.compra_id, orig.compra_id);

    // Stock neto: 10 +4 −4 +2 = 12 (reversa exacta, sin duplicar).
    let stock: f64 = sqlx::query_scalar("SELECT stock FROM productos WHERE id = ?")
        .bind(pid).fetch_one(&pool).await.unwrap();
    assert_eq!(stock, 12.0);

    // Original intacta; nueva apunta a ella; historial marca ambas caras.
    let det_orig = get_compra_detalle_impl(&pool, orig.compra_id).await.unwrap();
    assert_eq!(det_orig.pagado, 40.0);
    let det_new = get_compra_detalle_impl(&pool, rect.compra_id).await.unwrap();
    assert_eq!(det_new.rectifica_a, Some(orig.compra_id));
    assert_eq!(det_new.comentario.as_deref(), Some("me equivoqué"));
    let hist = historial_compras_impl(&pool, Some(prov), 100, 0).await.unwrap();
    assert_eq!(hist.len(), 2);
    let h_orig = hist.iter().find(|h| h.id == orig.compra_id).unwrap();
    let h_new = hist.iter().find(|h| h.id == rect.compra_id).unwrap();
    assert!(h_orig.rectificada);
    assert!(!h_new.rectificada);
    assert_eq!(h_new.rectifica_a, Some(orig.compra_id));

    // Rectificar inexistente: error, nada creado.
    assert!(rectificar_compra_impl(&pool, 1, 999999, vec![item(Some(pid), "Leche", 1.0)], 10.0, "efectivo".into(), None).await.is_err());
    assert!(rectificar_compra_impl(&pool, 1, orig.compra_id, vec![], 10.0, "efectivo".into(), None).await.is_err());
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM compras").fetch_one(&pool).await.unwrap();
    assert_eq!(n, 2);
}
use yarvis_app_lib::backventanas::backempleado::empleaproveedores::proveedores::{
    crear_proveedor_generico_impl, guardar_proveedor_impl, listar_proveedores_impl,
};
use yarvis_app_lib::backventanas::backempleado::empleaproveedores::sugerencia::sugerir_pago_impl;
use yarvis_app_lib::dinero::a_centavos;

async fn proveedor(pool: &sqlx::SqlitePool, nombre: &str) -> i64 {
    guardar_proveedor_impl(pool, nombre.into(), None, None).await.unwrap()
}

async fn corte_abierto(pool: &sqlx::SqlitePool) -> (i64, i64) {
    // usuario_id tiene FK real: se crea el cajero de verdad.
    let cajero = seed_empleado(pool, "Cajero", "x1234567").await;
    let corte = sqlx::query("INSERT INTO cortes_caja (usuario_id, estado) VALUES (?, 'abierto')")
        .bind(cajero)
        .execute(pool)
        .await
        .unwrap()
        .last_insert_rowid();
    (cajero, corte)
}

fn item(pid: Option<i64>, nombre: &str, cantidad: f64) -> ItemCompraRequest {
    ItemCompraRequest {
        producto_id: pid,
        nombre: nombre.into(),
        presentacion: "unidad".into(),
        cantidad,
        piezas_por_paquete: None,
        paquetes: None,
    }
}

#[tokio::test]
async fn generico_autoincrementa_y_rellena_huecos() {
    let pool = db().await;
    let g1 = crear_proveedor_generico_impl(&pool).await.unwrap();
    assert_eq!(g1.nombre, "MOSTRADOR 00001");
    // Hueco ocupado a mano: lo salta.
    guardar_proveedor_impl(&pool, "MOSTRADOR 00002".into(), None, None).await.unwrap();
    let g2 = crear_proveedor_generico_impl(&pool).await.unwrap();
    assert_eq!(g2.nombre, "MOSTRADOR 00003");
    // El genérico sirve para comprar de inmediato (sin corte: pendiente).
    let pid = seed_producto(&pool, "Azúcar", 10.0, 30.0).await;
    let r = registrar_compra_impl(&pool, 1, g2.id, vec![item(Some(pid), "Azúcar", 1.0)], 10.0, "efectivo".into(), None)
        .await
        .unwrap();
    assert!(r.movimiento_pendiente);
    assert_eq!(r.pagado, 10.0);
}

#[tokio::test]
async fn alta_lista_ordena_y_agrega_en_ceros() {
    let pool = db().await;
    proveedor(&pool, "Zeta").await;
    proveedor(&pool, "alfa").await;
    let lista = listar_proveedores_impl(&pool).await.unwrap();
    assert_eq!(lista.len(), 2);
    assert_eq!(lista[0].nombre, "alfa");
    assert_eq!(lista[1].nombre, "Zeta");
    assert_eq!(lista[0].total_compras, 0);
    assert_eq!(lista[0].total_pagado, 0.0);
}

#[tokio::test]
async fn nombre_vacio_y_duplicado_rechazados() {
    let pool = db().await;
    assert!(guardar_proveedor_impl(&pool, "".into(), None, None).await.is_err());
    assert!(guardar_proveedor_impl(&pool, "   ".into(), None, None).await.is_err());
    proveedor(&pool, "Don Chuy").await;
    let dup = guardar_proveedor_impl(&pool, "don chuy ".into(), None, None).await;
    assert_eq!(dup.unwrap_err(), "Ese proveedor ya está registrado.");
    // Teléfono y correo opcionales se guardan y se recortan.
    let id = guardar_proveedor_impl(&pool, "Otro".into(), Some(" 555 ".into()), None)
        .await
        .unwrap();
    assert!(id > 0);
    let lista = listar_proveedores_impl(&pool).await.unwrap();
    assert_eq!(lista.iter().find(|p| p.nombre == "Otro").unwrap().telefono.as_deref(), Some("555"));
}

#[tokio::test]
async fn sugerencia_desde_costo_y_sin_recomendacion() {
    let pool = db().await;
    // seed: precio_venta 20 → costo 10.
    seed_producto(&pool, "Coca-Cola 600", 5.0, 20.0).await;
    let s = sugerir_pago_impl(&pool, "coca-cola 600", 3.0).await.unwrap();
    assert_eq!(s.precio_costo, 10.0);
    assert_eq!(s.sugerido, Some(30.0));
    // Sin costo registrado → None, nunca inventa.
    let nada = sugerir_pago_impl(&pool, "Producto Fantasma", 2.0).await.unwrap();
    assert_eq!(nada.sugerido, None);
    assert_eq!(nada.precio_costo, 0.0);
    assert!(sugerir_pago_impl(&pool, "Coca-Cola 600", 0.0).await.is_err());
    // Nombre ambiguo (dos productos normalizan igual) → None, no inventa.
    seed_producto(&pool, "COCA-COLA  600", 1.0, 99.0).await;
    let amb = sugerir_pago_impl(&pool, "coca cola 600", 1.0).await.unwrap();
    assert_eq!(amb.sugerido, None);
}

#[tokio::test]
async fn compra_full_con_corte_genera_egreso_vinculado() {
    let pool = db().await;
    let prov = proveedor(&pool, " jars").await;
    // OJO: el nombre se recorta al guardar.
    let lista = listar_proveedores_impl(&pool).await.unwrap();
    assert_eq!(lista[0].nombre, "jars");
    let pid = seed_producto(&pool, "Azúcar", 10.0, 30.0).await;
    let (cajero, corte) = corte_abierto(&pool).await;

    let r = registrar_compra_impl(
        &pool, cajero, prov,
        vec![item(Some(pid), "Azúcar", 4.0)],
        55.0, "efectivo".into(), Some("nota".into()),
    )
    .await
    .unwrap();

    // Sugerido = 4 × costo 15 = 60; pagado lo que escribió el empleado.
    assert_eq!(r.sugerido, 60.0);
    assert_eq!(r.pagado, 55.0);
    assert!(!r.movimiento_pendiente);
    let mid = r.movimiento_id.expect("debió crear el egreso");

    // Stock sumó (entrada, no venta) y la compra apunta al movimiento.
    let stock: f64 = sqlx::query_scalar("SELECT stock FROM productos WHERE id = ?")
        .bind(pid).fetch_one(&pool).await.unwrap();
    assert_eq!(stock, 14.0);
    let mov: (String, i64, i64) = sqlx::query_as(
        "SELECT tipo, monto, referencia_id FROM movimientos_caja WHERE id = ?",
    )
    .bind(mid).fetch_one(&pool).await.unwrap();
    assert_eq!(mov.0, "retiro");
    assert_eq!(mov.1, a_centavos(55.0));
    assert_eq!(mov.2, r.compra_id);
    let corte_mov: i64 = sqlx::query_scalar("SELECT corte_id FROM movimientos_caja WHERE id = ?")
        .bind(mid).fetch_one(&pool).await.unwrap();
    assert_eq!(corte_mov, corte);
}

#[tokio::test]
async fn compra_sin_corte_queda_pendiente_y_monto_cero_sin_movimiento() {
    let pool = db().await;
    let prov = proveedor(&pool, "Sur").await;
    let pid = seed_producto(&pool, "Sal", 3.0, 10.0).await;

    // Sin corte abierto: se guarda, movimiento pendiente.
    let r = registrar_compra_impl(&pool, 99, prov, vec![item(Some(pid), "Sal", 1.0)], 5.0, "efectivo".into(), None)
        .await
        .unwrap();
    assert!(r.movimiento_pendiente);
    assert_eq!(r.movimiento_id, None);

    // Monto 0 (fiado total): sin movimiento y sin pendiente.
    let r0 = registrar_compra_impl(&pool, 99, prov, vec![item(None, "Pimienta", 2.0)], 0.0, "efectivo".into(), None)
        .await
        .unwrap();
    assert!(!r0.movimiento_pendiente);
    assert_eq!(r0.movimiento_id, None);
    // Renglón libre sin producto_id: se guarda el nombre, no mueve stock.
    let det = get_compra_detalle_impl(&pool, r0.compra_id).await.unwrap();
    assert_eq!(det.items[0].producto_id, None);
    assert_eq!(det.items[0].nombre, "Pimienta");
}

#[tokio::test]
async fn validaciones_no_dejan_basura() {
    let pool = db().await;
    let prov = proveedor(&pool, "Norte").await;
    let pid = seed_producto(&pool, "Arroz", 5.0, 20.0).await;
    let ok = || item(Some(pid), "Arroz", 1.0);

    // Dilema del no-registrado: imposible, error amigable antes que la FK.
    let r = registrar_compra_impl(&pool, 1, 999999, vec![ok()], 10.0, "efectivo".into(), None).await;
    assert_eq!(r.unwrap_err(), "Registra primero al proveedor.");
    assert!(registrar_compra_impl(&pool, 1, prov, vec![], 10.0, "efectivo".into(), None).await.is_err());
    assert!(registrar_compra_impl(&pool, 1, prov, vec![item(Some(pid), "Arroz", 0.0)], 10.0, "efectivo".into(), None).await.is_err());
    assert!(registrar_compra_impl(&pool, 1, prov, vec![item(Some(pid), "Arroz", -2.0)], 10.0, "efectivo".into(), None).await.is_err());
    let mut mala = ok();
    mala.presentacion = "caja".into();
    assert!(registrar_compra_impl(&pool, 1, prov, vec![mala], 10.0, "efectivo".into(), None).await.is_err());
    assert!(registrar_compra_impl(&pool, 1, prov, vec![ok()], -5.0, "efectivo".into(), None).await.is_err());
    let mut fantasma = ok();
    fantasma.producto_id = Some(424242);
    assert!(registrar_compra_impl(&pool, 1, prov, vec![fantasma], 10.0, "efectivo".into(), None).await.is_err());

    // Nada a medias: cero compras tras todos los rechazos.
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM compras").fetch_one(&pool).await.unwrap();
    assert_eq!(n, 0);
}

#[tokio::test]
async fn paquete_multiplica_piezas_y_guarda_desglose() {
    let pool = db().await;
    let prov = proveedor(&pool, "Paquetero").await;
    let pid = seed_producto(&pool, "Refresco Lata", 0.0, 20.0).await;

    let r = registrar_compra_impl(
        &pool, 1, prov,
        vec![ItemCompraRequest {
            producto_id: Some(pid),
            nombre: "Refresco Lata".into(),
            presentacion: "paquete".into(),
            cantidad: 999.0, // se ignora: manda piezas × paquetes
            piezas_por_paquete: Some(12.0),
            paquetes: Some(3.0),
        }],
        100.0, "efectivo".into(), None,
    )
    .await
    .unwrap();

    // 3 × 12 = 36 unidades al stock y en el renglón.
    let stock: f64 = sqlx::query_scalar("SELECT stock FROM productos WHERE id = ?")
        .bind(pid).fetch_one(&pool).await.unwrap();
    assert_eq!(stock, 36.0);
    let det = get_compra_detalle_impl(&pool, r.compra_id).await.unwrap();
    assert_eq!(det.items[0].cantidad, 36.0);
    assert_eq!(det.items[0].piezas_por_paquete, Some(12.0));
    assert_eq!(det.items[0].paquetes, Some(3.0));

    // Sin piezas o en cero: rechazo amigable, nada guardado.
    for (pz, pq) in [(None, Some(3.0)), (Some(12.0), None), (Some(0.0), Some(3.0)), (Some(12.0), Some(-1.0))] {
        let mala = ItemCompraRequest {
            producto_id: Some(pid),
            nombre: "Refresco Lata".into(),
            presentacion: "paquete".into(),
            cantidad: 1.0,
            piezas_por_paquete: pz,
            paquetes: pq,
        };
        assert!(registrar_compra_impl(&pool, 1, prov, vec![mala], 10.0, "efectivo".into(), None).await.is_err());
    }
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM compras").fetch_one(&pool).await.unwrap();
    assert_eq!(n, 1);
}

#[tokio::test]
async fn paquete_legado_sin_desglose_conserva_total() {
    let pool = db().await;
    let prov = proveedor(&pool, "Legado").await;
    let pid = seed_producto(&pool, "Galleta", 0.0, 10.0).await;

    // Renglón de antes del soporte de paquetes (desglose NULL): entra
    // con el total intacto como piezas × 1, no se bloquea ni se borra.
    let r = registrar_compra_impl(
        &pool, 1, prov,
        vec![ItemCompraRequest {
            producto_id: Some(pid),
            nombre: "Galleta".into(),
            presentacion: "paquete".into(),
            cantidad: 24.0,
            piezas_por_paquete: None,
            paquetes: None,
        }],
        50.0, "efectivo".into(), None,
    )
    .await
    .unwrap();
    let det = get_compra_detalle_impl(&pool, r.compra_id).await.unwrap();
    assert_eq!(det.items[0].cantidad, 24.0);
    assert_eq!(det.items[0].piezas_por_paquete, Some(24.0));
    assert_eq!(det.items[0].paquetes, Some(1.0));
    let stock: f64 = sqlx::query_scalar("SELECT stock FROM productos WHERE id = ?")
        .bind(pid).fetch_one(&pool).await.unwrap();
    assert_eq!(stock, 24.0);
}

#[tokio::test]
async fn historial_filtra_pagina_y_detalle_completo() {
    let pool = db().await;
    let a = proveedor(&pool, "A").await;
    let b = proveedor(&pool, "B").await;
    let pid = seed_producto(&pool, "Frijol", 9.0, 25.0).await;
    for _ in 0..3 {
        registrar_compra_impl(&pool, 1, a, vec![item(Some(pid), "Frijol", 1.0)], 10.0, "efectivo".into(), None).await.unwrap();
    }
    registrar_compra_impl(&pool, 1, b, vec![item(Some(pid), "Frijol", 1.0)], 10.0, "tarjeta".into(), Some("urgente".into())).await.unwrap();

    let todas = historial_compras_impl(&pool, None, 100, 0).await.unwrap();
    assert_eq!(todas.len(), 4);
    let solo_b = historial_compras_impl(&pool, Some(b), 100, 0).await.unwrap();
    assert_eq!(solo_b.len(), 1);
    assert_eq!(solo_b[0].proveedor, "B");
    assert_eq!(solo_b[0].metodo_pago, "tarjeta");
    let pag1 = historial_compras_impl(&pool, None, 3, 0).await.unwrap();
    let pag2 = historial_compras_impl(&pool, None, 3, 3).await.unwrap();
    assert_eq!(pag1.len(), 3);
    assert_eq!(pag2.len(), 1);
    assert_ne!(pag1[0].id, pag2[0].id);

    let det = get_compra_detalle_impl(&pool, solo_b[0].id).await.unwrap();
    assert_eq!(det.comentario.as_deref(), Some("urgente"));
    assert_eq!(det.items.len(), 1);
    assert_eq!(det.items[0].presentacion, "unidad");
    assert!(get_compra_detalle_impl(&pool, 999999).await.is_err());
}
