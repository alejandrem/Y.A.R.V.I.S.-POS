// ============================================================
// FORMATO: Minisuper "El Trebol" (multi-renglon + puntos + off).
//
// FORMA COMPLETA DEL TICKET:
// --------------------------------
// ********************************
//      MINISUPER "EL TREBOL"
//    Av. Reforma 210, Local 3
//       Cholula, Puebla
//     C.P. 72760 · Tel: 222-987-6543
// ********************************
// No. Venta: 000001
// 17/09/2025          14:28 hrs
// --------------------------------
// 1) Coca-Cola 600ml
//    2 pza x $18.00 ......... $36.00
//
// 2) Sabritas Original 45g
//    1 pza x $18.00 (10% off) $16.20
//
// 3) Corona 473ml
//    3 pza x $22.00 ......... $66.00
// --------------------------------
// Subtotal ................ $118.20
// Descuento ................. -$1.80
// --------------------------------
// TOTAL .................... $116.40
//
// Pago: EFECTIVO
// Recibido ................. $200.00
// Cambio .................... $83.60
// ********************************
//     ¡Gracias por su compra!
//       Vuelva pronto :)
// ********************************
//
// PARTICULARIDADES:
// - Cada producto en DOS renglones (`N) Nombre` + `cant pza x $P..$T`).
// - Puntos de relleno pegando precio y total.
// - Descuento con muletilla: `(10% off)` (a veces `(15% off).$T` pegado).
// - Folio con etiqueta `No. Venta:`.
// ============================================================

use crate::cerebro::analizador_tickets::{
    extraer_metodo_pago, parsear_linea, segmentar, unir_lineas_multirenglon, MapeoColumnas,
};

fn mapeo_a() -> MapeoColumnas {
    MapeoColumnas {
        cantidad: Some(0),
        producto: Some(vec![1]),
        precio_unitario: Some(2),
        total: Some(-1),
        descuento: None,
    }
}

const TICKET: &str = "No. Venta: 000001\n17/09/2025          14:28 hrs\n1) Coca-Cola 600ml\n   2 pza x $18.00 ......... $36.00\n2) Sabritas Original 45g\n   1 pza x $18.00 (10% off) $16.20\n3) Corona 473ml\n   3 pza x $22.00 ......... $66.00\nTOTAL .................... $116.40\nPago: EFECTIVO\n";

#[test]
fn folio_y_fecha_del_trebol() {
    let segs = segmentar(TICKET);
    assert_eq!(segs.len(), 1);
    assert_eq!(segs[0].folio.as_deref(), Some("000001"));
    assert_eq!(segs[0].fecha_hora.as_deref(), Some("2025-09-17 14:28:00"));
}

#[test]
fn item_normal_multi_renglon() {
    let unido = unir_lineas_multirenglon("1) Coca-Cola 600ml\n   2 pza x $18.00 ......... $36.00");
    let item = parsear_linea(&unido, &mapeo_a(), 5).unwrap();
    assert_eq!(item.producto, "COCA-COLA 600ML");
    assert_eq!(item.cantidad, 2.0);
    assert_eq!(item.precio_unitario, 18.0);
    assert_eq!(item.total, 36.0);
    assert_eq!(item.descuento, None);
}

#[test]
fn item_con_off_parentesis() {
    // `(10% off)` pegado o separado: descuento $1.80, total neto.
    let unido =
        unir_lineas_multirenglon("2) Sabritas Original 45g\n   1 pza x $18.00 (10% off) $16.20");
    let item = parsear_linea(&unido, &mapeo_a(), 6).unwrap();
    assert_eq!(item.producto, "SABRITAS ORIGINAL 45G");
    assert_eq!(item.descuento, Some(1.8));
    assert_eq!(item.total, 16.2);
}

#[test]
fn item_con_off_pegado_al_total() {
    // Variante real (1418 archivos): `(15% off).$34.00` con el total pegado.
    let unido =
        unir_lineas_multirenglon("1) Churrumais 45g\n   2 pza x $20.00 (15% off).$34.00");
    let item = parsear_linea(&unido, &mapeo_a(), 6).unwrap();
    assert_eq!(item.descuento, Some(6.0));
    assert_eq!(item.total, 34.0);
}

#[test]
fn pago_efectivo_del_trebol() {
    assert_eq!(extraer_metodo_pago("Pago: EFECTIVO\n"), "efectivo");
}
