// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Importación histórica de cortes X/Z.
// Carpeta mezclada (cortes + un ticket suelto): clasifica, importa,
// omite no-cortes, es idempotente por hash y expone historial/detalle.
// Los casos son ADREDE distintos a los del parser (otra estación,
// otro cajero, 24h) para probar generalidad, no memorización.
// ═══════════════════════════════════════════════════════════════════════════

#[path = "../common/mod.rs"]
mod common;

use common::db;
use std::sync::atomic::{AtomicU64, Ordering};
use yarvis_app_lib::backventanas::backadmin::adminparser::cortes_import::importacion::importar_carpeta_cortes_impl;

static SEQ: AtomicU64 = AtomicU64::new(0);

const CORTE_Z_MINI: &str = "*** CORTE Z EN MONEDA:MXN***
TIENDA PRUEBA

*** Corte Z 7
EST01 10/05/2025 20:00:00
**Ingresos**

EFE Ventas  $1,000.00
-----------------------------------
  Total de Ingresos:  $1,000.00

**Egresos**

-----------------------------------
   Total de Egresos:       $.00
-----------------------------------
   Total en caja:     $1,000.00
-----------------------------------
*********VENTAS DEL CORTE**********
Ventas 16%        :       $.00
Impuesto 16%      :       $.00
-----------------------------------
Ventas gravadas   :       $.00
Impuesto          :       $.00
Ventas no gravadas:  $1,000.00
-----------------------------------
Redondeos         :       $.00
Total de ventas   :  $1,000.00
-----------------------------------
Ventas credito    :       $.00
-----------------------------------

**Ventas por artículo**

PROD A -       2 -   $1,000.00
**Total ventas del dia   :   1,000.00
**Total venta en unidades:      2.00
-----------------------------
Clientes atendidos: 1";

const CORTE_X_MINI: &str = "*** CORTE X EN MONEDA:MXN***
TIENDA PRUEBA

Cajero: LUIS

Corte X 3
EST02 11/05/2025 09:00:00
**Ingresos**

EFE Ventas    $500.00
-----------------------------------
  Total de Ingresos:    $500.00

**Egresos**

-----------------------------------
   Total de Egresos:       $.00
-----------------------------------
   Total en caja:       $500.00
-----------------------------------
*********VENTAS DEL CORTE**********
Ventas 16%        :       $.00
Impuesto 16%      :       $.00
-----------------------------------
Ventas gravadas   :       $.00
Impuesto          :       $.00
Ventas no gravadas:    $500.00
-----------------------------------
Redondeos         :       $.00
Total de ventas   :    $500.00
-----------------------------------
Ventas credito    :       $.00
-----------------------------------

**Ventas por ticket**

T-1                  300.00
T-2                  200.00
**Total ventas del dia :     500.00
-----------------------------
Clientes atendidos: 2";

const TICKET_SUELTO: &str = "2 Pan Bimbo 42.00 84.00\nTOTAL: $84.00\n";

fn carpeta_mezclada() -> String {
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("cortes_test_{}_{}", std::process::id(), n));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("a_z.txt"), CORTE_Z_MINI).unwrap();
    std::fs::write(dir.join("b_x.txt"), CORTE_X_MINI).unwrap();
    std::fs::write(dir.join("c_ticket.txt"), TICKET_SUELTO).unwrap();
    dir.to_string_lossy().to_string()
}

#[tokio::test]
async fn carpeta_mezclada_clasifica_e_importa() {
    let pool = db().await;
    let carpeta = carpeta_mezclada();
    let r = importar_carpeta_cortes_impl(&pool, &carpeta).await.unwrap();
    assert_eq!(r.archivos, 3);
    assert_eq!(r.cortes_z, 1);
    assert_eq!(r.cortes_x, 1);
    assert_eq!(r.omitidos_no_corte, 1);
    assert_eq!(r.omitidos_duplicados, 0);
    assert!(r.errores.is_empty());
}

#[tokio::test]
async fn reimportar_es_idempotente_por_hash() {
    let pool = db().await;
    let carpeta = carpeta_mezclada();
    importar_carpeta_cortes_impl(&pool, &carpeta).await.unwrap();
    let r2 = importar_carpeta_cortes_impl(&pool, &carpeta).await.unwrap();
    assert_eq!(r2.cortes_x, 0);
    assert_eq!(r2.cortes_z, 0);
    assert_eq!(r2.omitidos_duplicados, 2);
    assert_eq!(r2.omitidos_no_corte, 1);
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM cortes_importados")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(n, 2);
}

#[tokio::test]
async fn vincula_catalogo_y_no_toca_stock() {
    use common::seed_producto;
    let pool = db().await;
    // Catálogo maestro ya importado (paso 01): coincide exacto.
    seed_producto(&pool, "PROD A", 10.0, 500.0).await;

    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("cortes_vinc_{}_{}", std::process::id(), n));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("z.txt"),
        CORTE_Z_MINI.replace("PROD A -       2 -   $1,000.00", "PROD A -       2 -   $1,000.00\nCOSA RARA NUEVA XYZ -       1 -   $50.00"),
    )
    .unwrap();
    // OJO: el reemplazo rompe la suma (1050 != 1000) a propósito: la
    // importación guarda igual y lo reporta (ventas_ok=false), lo que aquí
    // se prueba es el vínculo, no la verificación.
    let r = importar_carpeta_cortes_impl(&pool, &dir.to_string_lossy()).await.unwrap();
    assert_eq!(r.cortes_z, 1);
    assert_eq!(r.productos_vinculados, 1);
    assert_eq!(r.productos_nuevos, 1);

    // Ambos items con producto_id; el histórico NO movió el stock.
    let vinculados: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM cortes_importados_items WHERE producto_id IS NOT NULL",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(vinculados, 2);
    let (stock, vendido): (f64, f64) =
        sqlx::query_as("SELECT stock, vendido FROM productos WHERE nombre = 'PROD A'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(stock, 10.0);
    assert_eq!(vendido, 2.0);
    let stock_nuevo: f64 =
        sqlx::query_scalar("SELECT stock FROM productos WHERE nombre = 'COSA RARA NUEVA XYZ'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(stock_nuevo, 0.0);
}

#[tokio::test]
async fn detalle_trae_totales_items_y_verificacion() {
    let pool = db().await;
    let carpeta = carpeta_mezclada();
    importar_carpeta_cortes_impl(&pool, &carpeta).await.unwrap();

    let id: i64 = sqlx::query_scalar("SELECT id FROM cortes_importados WHERE tipo = 'Z'")
        .fetch_one(&pool)
        .await
        .unwrap();
    // El detalle exige sesión admin; aquí se valida la persistencia
    // directa que el comando lee (mismos SELECTs, sin Tauri).
    let (tipo, caja, ventas, items): (String, i64, i64, i64) = sqlx::query_as(
        "SELECT c.tipo, c.total_caja, c.total_ventas, COUNT(i.id)
         FROM cortes_importados c LEFT JOIN cortes_importados_items i ON i.corte_id = c.id
         WHERE c.id = ? GROUP BY c.id",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(tipo, "Z");
    assert_eq!(caja, 100000);
    assert_eq!(ventas, 100000);
    // 1 ingreso + 1 artículo.
    assert_eq!(items, 2);
}
