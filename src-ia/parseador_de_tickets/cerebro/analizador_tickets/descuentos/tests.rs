use super::descuento_separado;

#[test]
fn reconoce_descuento_con_importe_negativo() {
    assert_eq!(descuento_separado(" Descuento   -2.00 "), Some(2.0));
}

#[test]
fn no_confunde_nombres_con_etiqueta_de_descuento() {
    assert_eq!(descuento_separado("Descuento Comercial 5.00"), Some(5.0));
    assert_eq!(descuento_separado("Producto Descuento 5.00"), None);
}
