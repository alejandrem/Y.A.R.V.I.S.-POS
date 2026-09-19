// ============================================================
// FORMATO: Abarrotes "La Esquina Feliz" (un renglon + % suelto).
//
// FORMA COMPLETA DEL TICKET:
// ------------------------------------------------
// ================================================
//           ABARROTES "LA ESQUINA FELIZ"
//          Calle Morelos #45, Col. Centro
//                 Atlixco, Puebla
//                    C.P. 74200
//                Tel: 244-123-4567
// ================================================
// Ticket: TCK-000006
// Fecha: 17/09/2025    Hora: 15:37:25
// ------------------------------------------------
// CANT PRODUCTO              P.UNIT  DESC  IMPORTE
// ------------------------------------------------
// 3    Boing Guayaba         $10.00     -   $30.00
// 1    Yogurt Yoplait 150g   $18.00     -   $18.00
// 5    Mazapan De la Rosa     $8.00     -   $40.00
// 3    Plumas Bic punta me.  $18.00     -   $54.00
// 4    Papas Ruffles Queso   $20.00     -   $80.00
// 4    Crema Lala 200g       $22.00   15%   $74.80
// ------------------------------------------------
// SUBTOTAL:                          $310.00
// DESCUENTO:                         -$13.20
// TOTAL:                             $296.80
//
// Forma de pago: TRANSFERENCIA
// Monto cobrado:                     $296.80
// ================================================
//             GRACIAS POR SU COMPRA!
//                 Vuelva pronto :)
// ================================================
//
// PARTICULARIDADES:
// - Un renglon por producto (familia A directa, sin pegado).
// - Columna DESC con `-` (sin descuento) o `15%` (porcentaje suelto).
// - Folio con etiqueta `Ticket:` + fecha y hora separadas.
// - 6100 archivos reales importados de este formato.
// ============================================================

use crate::cerebro::analizador_tickets::{
    extraer_metodo_pago, parsear_linea, segmentar, MapeoColumnas,
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

const TICKET: &str = "Ticket: TCK-000006\nFecha: 17/09/2025    Hora: 15:37:25\n3    Boing Guayaba         $10.00     -   $30.00\n4    Crema Lala 200g       $22.00   15%   $74.80\nTOTAL:                             $296.80\nForma de pago: TRANSFERENCIA\n";

#[test]
fn folio_y_fecha_esquina() {
    let segs = segmentar(TICKET);
    assert_eq!(segs.len(), 1);
    assert_eq!(segs[0].folio.as_deref(), Some("TCK-000006"));
    assert_eq!(segs[0].fecha_hora.as_deref(), Some("2025-09-17 15:37:00"));
}

#[test]
fn item_con_guion_sin_descuento() {
    let item = parsear_linea("3    Boing Guayaba         $10.00     -   $30.00", &mapeo_a(), 5).unwrap();
    assert_eq!(item.producto, "BOING GUAYABA");
    assert_eq!(item.cantidad, 3.0);
    assert_eq!(item.total, 30.0);
    assert_eq!(item.descuento, None);
}

#[test]
fn item_con_porcentaje_suelto() {
    // 4 x $22 = $88 - 15% ($13.20) = $74.80.
    let item =
        parsear_linea("4    Crema Lala 200g       $22.00   15%   $74.80", &mapeo_a(), 6).unwrap();
    assert_eq!(item.producto, "CREMA LALA 200G");
    assert_eq!(item.descuento, Some(13.2));
    assert_eq!(item.total, 74.8);
}

#[test]
fn pago_transferencia() {
    assert_eq!(extraer_metodo_pago("Forma de pago: TRANSFERENCIA\n"), "transferencia");
}
