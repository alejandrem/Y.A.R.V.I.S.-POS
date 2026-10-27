use super::parsear_items_con_descuentos;
use crate::cerebro::analizador_tickets::{MapeoColumnas, TotalesTicket};
use crate::cerebro::parseador_masivo::items::resolver_totales_venta;

fn mapeo() -> MapeoColumnas {
    MapeoColumnas {
        cantidad: Some(0),
        producto: Some(vec![1, 4]),
        precio_unitario: Some(5),
        total: Some(6),
        descuento: None,
    }
}

#[test]
fn descuento_separado_se_asigna_al_articulo_anterior() {
    let lineas = vec![
        "1 Leche Lala Entera 1L 29.00 29.00".to_string(),
        "Descuento -2.00".to_string(),
    ];
    let items = parsear_items_con_descuentos(&lineas, &mapeo(), 7);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].descuento, Some(2.0));
    assert_eq!(
        resolver_totales_venta(&items, &TotalesTicket::default()),
        (29.0, 0.0, 27.0)
    );
}

#[test]
fn descuento_en_linea_monetaria_conserva_subtotal_bruto() {
    let lineas = vec!["2 COCA $25.00 10% $45.00".to_string()];
    let items = parsear_items_con_descuentos(&lineas, &mapeo(), 6);
    assert_eq!(items[0].total, 50.0);
    assert_eq!(
        resolver_totales_venta(&items, &TotalesTicket::default()),
        (50.0, 0.0, 45.0)
    );
}
