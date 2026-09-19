// ============================================================
// FORMATOS BASE — Las tres familias estructurales + descuentos.
//
// El detector ensaya estas formas en cada carpeta (hipotesis A/B/C).
// Son renglones sueltos verificados, sin ticket completo: lo que
// importa aqui es la ECUACION cantidad x precio - descuento ~= total.
//
// Familia A (cantidad primero):  "3 COCA 25.00 75.00 5.00"
// Familia B (producto primero):  "COCA 2 25.00 50.00"
// Familia C (un solo importe):   "CAJA DE 24 CERVEZAS MODELO $540.00"
// Descuento %:                   "2 Rockaleta $6.00 10% $10.80"
// Descuento $:                   "3 COCA 25.00 75.00 5.00" (columna)
// Sin descuento:                 "1 Heineken 473ml $28.00 - $28.00"
// ============================================================

use crate::cerebro::analizador_tickets::{parsear_linea, MapeoColumnas};

fn mapeo(cantidad: i32, producto: i32, precio: i32, total: i32) -> MapeoColumnas {
    MapeoColumnas {
        cantidad: Some(cantidad),
        producto: Some(vec![producto]),
        precio_unitario: Some(precio),
        total: Some(total),
        descuento: None,
    }
}

#[test]
fn familia_a_con_descuento_en_columna() {
    // 3 x 25 = 75 - 5 (columna) = 70... el total impreso manda ($75.00
    // es bruto en este formato; el descuento se guarda aparte).
    let m = MapeoColumnas {
        cantidad: Some(0),
        producto: Some(vec![1]),
        precio_unitario: Some(2),
        total: Some(3),
        descuento: Some(4),
    };
    let item = parsear_linea("3 COCA 25.00 75.00 5.00", &m, 5).unwrap();
    assert_eq!(item.producto, "COCA");
    assert_eq!(item.descuento, Some(5.0));
}

#[test]
fn familia_a_con_porcentaje() {
    let item = parsear_linea("2 Rockaleta $6.00 10% $10.80", &mapeo(0, 1, 2, -1), 5).unwrap();
    assert_eq!(item.producto, "ROCKALETA");
    assert_eq!(item.descuento, Some(1.2));
    assert_eq!(item.total, 10.8);
}

#[test]
fn familia_c_un_solo_importe() {
    // Sin precio unitario ni cantidad explicita: todo es producto
    // menos el ultimo importe (total).
    let m = MapeoColumnas {
        cantidad: None,
        producto: Some(vec![0, 1, 2, 3, 4]),
        precio_unitario: None,
        total: Some(-1),
        descuento: None,
    };
    let item = parsear_linea("CAJA DE 24 CERVEZAS MODELO $540.00", &m, 5).unwrap();
    assert_eq!(item.producto, "CAJA DE 24 CERVEZAS MODELO");
    assert_eq!(item.total, 540.0);
}

#[test]
fn guion_significa_sin_descuento() {
    let item = parsear_linea("1 Heineken 473ml $28.00 - $28.00", &mapeo(0, 1, 2, -1), 6).unwrap();
    assert_eq!(item.producto, "HEINEKEN 473ML");
    assert_eq!(item.descuento, None);
    assert_eq!(item.total, 28.0);
}
