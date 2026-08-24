// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Módulo INVENTARIO (admininventory).
// Prueba add/update_inventory_item_impl, unicidad de codigo_barras (índice
// parcial: duplicado rechazado, NULL repetido permitido) y el borrado.
// ═══════════════════════════════════════════════════════════════════════════

#[path = "../common/mod.rs"]
mod common;

use common::{db, escalar_i64};
use yarvis_app_lib::backventanas::backadmin::admininventory::inventory::{
    add_inventory_item_impl, update_inventory_item_impl,
};
use yarvis_app_lib::models::InventoryItem;

fn item(nombre: &str) -> InventoryItem {
    InventoryItem {
        id: None,
        nombre: nombre.into(),
        descripcion: None,
        precio_costo: 10.0,
        precio_venta: 20.0,
        stock: 5.0,
        stock_minimo: 1.0,
        vendido: 0.0,
        codigo_barras: None,
        categoria: None,
    }
}

#[tokio::test]
async fn alta_de_producto_persiste_campos() {
    let pool = db().await;
    let mut it = item("Coca-Cola");
    it.codigo_barras = Some("7501234567890".into());
    let id = add_inventory_item_impl(&pool, &it).await.unwrap();

    let (nombre, cb): (String, Option<String>) =
        sqlx::query_as("SELECT nombre, codigo_barras FROM productos WHERE id = ?")
            .bind(id).fetch_one(&pool).await.unwrap();
    assert_eq!(nombre, "Coca-Cola");
    assert_eq!(cb.as_deref(), Some("7501234567890"));
}

#[tokio::test]
async fn edicion_modifica_stock_y_precios() {
    let pool = db().await;
    let mut it = item("Pan Bimbo");
    let id = add_inventory_item_impl(&pool, &it).await.unwrap();

    it.id = Some(id);
    it.stock = 50.0;
    it.precio_venta = 42.5;
    update_inventory_item_impl(&pool, &it).await.unwrap();

    let (stock, precio): (f64, i64) =
        sqlx::query_as("SELECT stock, precio_venta FROM productos WHERE id = ?")
            .bind(id).fetch_one(&pool).await.unwrap();
    assert_eq!(stock, 50.0);
    assert_eq!(precio, 4_250);
}

#[tokio::test]
async fn edicion_sin_id_rechazada() {
    let pool = db().await;
    assert!(update_inventory_item_impl(&pool, &item("Sin ID")).await.is_err());
}

#[tokio::test]
async fn codigo_de_barras_duplicado_rechazado_por_la_db() {
    let pool = db().await;
    let mut a = item("Original");
    a.codigo_barras = Some("UNICO123".into());
    add_inventory_item_impl(&pool, &a).await.unwrap();

    let mut b = item("Clon");
    b.codigo_barras = Some("UNICO123".into());
    // La restricción UNIQUE parcial debe rechazar el clon
    assert!(add_inventory_item_impl(&pool, &b).await.is_err());
}

#[tokio::test]
async fn productos_sin_codigo_barras_pueden_coexistir() {
    let pool = db().await;
    add_inventory_item_impl(&pool, &item("Sin código A")).await.unwrap();
    add_inventory_item_impl(&pool, &item("Sin código B")).await.unwrap();
    assert_eq!(
        escalar_i64(&pool, "SELECT COUNT(*) FROM productos WHERE codigo_barras IS NULL").await,
        2
    );
}
