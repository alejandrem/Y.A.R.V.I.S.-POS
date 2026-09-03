// ============================================================
// analizador_tickets — Segmentación y parseo línea-a-línea de
// tickets (regex puro, sin modelos). Port de analizador.py.
//
//   * fechas.rs      → fecha/hora del ticket
//   * pagos.rs       → método de pago
//   * esquema.rs     → MapeoColumnas + Item (contrato frontend↔núcleo)
//   * parser.rs      → parsear_linea (orquesta limpiador + mapeo)
//   * encabezado.rs  → cajero y metadatos del encabezado
//   * segmentador.rs → corte 1 archivo → N tickets (un ticket por bloque)
//
// Conexión con `filtrador`: `parsear_linea` llama a
// `filtrador::limpiar_producto`.
// ============================================================

mod detector;
mod encabezado;
mod esquema;
mod fechas;
mod pagos;
mod parser;
mod segmentador;
mod totales;

pub use detector::{detectar_mapeo, DeteccionMapeo};
pub use encabezado::extraer_cajero;
pub use esquema::{resolver_indice, Item, MapeoColumnas};
pub use fechas::{extraer_fecha_hora_regex, tiene_fecha};
pub use pagos::extraer_metodo_pago;
pub use parser::{es_linea_util, limpiar_precio, parsear_linea, PRECIO_MAXIMO};
pub use segmentador::{segmentar, TicketSegmento};
pub use totales::{extraer_totales, TotalesTicket};

#[cfg(test)]
mod tests {
    use super::*;

    fn mapeo(cantidad: i32, producto: i32, precio: i32, total: i32) -> MapeoColumnas {
        MapeoColumnas {
            cantidad: Some(cantidad),
            producto: Some(vec![producto]),
            precio_unitario: Some(precio),
            total: Some(total),
            descuento: None,
        }
    }

    // ---------- es_linea_util (test_analizador.py) ----------

    #[test]
    fn productos_legitimos_no_se_descartan() {
        let productos = [
            "GATORADE TOTAL 600ML $35.00",
            "CAJA DE MADERA 30X30 $120.00",
            "CAJA DE 24 CERVEZAS $540.00",
            "CANTIMPLORA LITRO $45.00",
            "PRECIOS JUSTOS COMBO $99.00",
            "OLIVA EXTRA VIRGEN 500ML $185.00",
            "COLONIA 900 PERFUME $150.00",
            "COCA-COLA 600ML $25.00",
            "TOTALMAX $18.00",
            "PAN WHITE 680GR $22.00",
            "DIVA PLATINUM $65.00",
        ];
        for p in productos {
            assert!(es_linea_util(p), "Producto perdido: {p}");
        }
    }

    #[test]
    fn cabeceras_se_descartan() {
        let cabeceras = [
            "TOTAL ---- $1,234.56",
            "EFECTIVO $500.00",
            "IVA 16%",
            "SUBTOTAL $1,064.28",
            "GRACIAS POR SU COMPRA",
            "METODO DE PAGO: TARJETA",
            "CFDI: 4D8F2A1",
            "CAJA: 3",
        ];
        for c in cabeceras {
            assert!(!es_linea_util(c), "Cabecera como producto: {c}");
        }
    }

    #[test]
    fn linea_vacia_no_es_util() {
        assert!(!es_linea_util(""));
        assert!(!es_linea_util("   "));
    }

    #[test]
    fn multiples_productos_por_linea() {
        assert!(es_linea_util("2 TAZAS $60.00 $120.00"));
        assert!(es_linea_util("Coca-Cola 600ML $25 $18"));
    }

    #[test]
    fn cabeceras_con_dos_puntos_se_descartan() {
        for c in [
            "TOTAL: $1,234.56",
            "CAJA: 3",
            "FECHA: 12/05/2026",
            "ATENDIO: MARIA",
            "METODO DE PAGO: EFECTIVO",
        ] {
            assert!(!es_linea_util(c), "{c}");
        }
    }

    #[test]
    fn productos_ambiguos_con_numeros_no_se_descartan() {
        for p in [
            "CAJA DE 24 CERVEZAS MODELO $540.00",
            "BEBIDA CAJA TETRA 1L $19.00",
            "ABARROTES VARIOS $5.00",
        ] {
            assert!(es_linea_util(p), "{p}");
        }
    }

    #[test]
    fn productos_con_unidades_de_medida_no_se_descartan() {
        for p in [
            "COCA-COLA 600ML $25.00 $25.00",
            "AGUA CIEL 1.5L $22.00",
            "CABLE HDMI 1.5M $120.00",
            "LAMINA GALVANIZADA 15MM $85.00",
            "HARINA DE TRIGO 1KG $32.00",
            "MADERA 30X30 $120.00",
        ] {
            assert!(es_linea_util(p), "Producto con unidad perdido: {p}");
        }
    }

    #[test]
    fn producto_con_unidad_y_palabra_de_cabecera_no_se_descarta() {
        // La unidad (32GB) cuenta como dato de producto: la línea no se
        // descarta aunque el primer token sea de la lista PRIMERAS_CABECERAS.
        assert!(es_linea_util("TARJETA MEMORIA 32GB $250.00 $250.00"));
        assert!(es_linea_util("PAGO FACIL RECARGAS 1KG $400.00 $400.00"));
    }

    #[test]
    fn linea_solo_separadores() {
        assert!(!es_linea_util("----------------"));
        assert!(!es_linea_util("===================="));
        assert!(!es_linea_util("~~~~~~~"));
    }

    #[test]
    fn saludo_breve_no_crashea() {
        let res = es_linea_util("HOLA");
        assert!(res);
    }

    // ---------- parsear_linea (verificado contra Python) ----------

    #[test]
    fn parsea_linea_tipica() {
        let item = parsear_linea("2 COCA 25.00 50.00", &mapeo(0, 1, 2, 3), 4).unwrap();
        assert_eq!(item.producto, "COCA");
        assert_eq!(item.cantidad, 2.0);
        assert_eq!(item.precio_unitario, 25.0);
        assert_eq!(item.total, 50.0);
        assert_eq!(item.descuento, None);
    }

    #[test]
    fn parsea_linea_con_indice_negativo() {
        let m = MapeoColumnas {
            cantidad: Some(1),
            producto: Some(vec![0]),
            precio_unitario: Some(2),
            total: Some(-1),
            descuento: None,
        };
        let item = parsear_linea("COCA-COLA 2 25.00 50.00", &m, 4).unwrap();
        assert_eq!(item.producto, "COCA-COLA");
        assert_eq!(item.cantidad, 2.0);
        assert_eq!(item.total, 50.0);
    }

    #[test]
    fn parsea_rango_de_producto() {
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
    fn parsea_con_descuento() {
        // 3 COCA $25.00 $75.00 $5.00  (última columna = descuento)
        let m = MapeoColumnas {
            cantidad: Some(0),
            producto: Some(vec![1]),
            precio_unitario: Some(2),
            total: Some(3),
            descuento: Some(4),
        };
        let item = parsear_linea("3 COCA 25.00 75.00 5.00", &m, 5).unwrap();
        assert_eq!(item.descuento, Some(5.0));
    }

    #[test]
    fn linea_no_util_devuelve_none() {
        assert!(parsear_linea("GRACIAS POR SU COMPRA", &mapeo(0, 1, 2, 3), 4).is_none());
        assert!(parsear_linea("", &mapeo(0, 1, 2, 3), 4).is_none());
    }

    #[test]
    fn parsea_ticket_real_con_producto_variable_y_descuento_porcentual() {
        let m = MapeoColumnas {
            cantidad: Some(0),
            producto: Some(vec![1]),
            precio_unitario: Some(2),
            total: Some(-1),
            descuento: None,
        };

        let item = parsear_linea("2 Rockaleta $6.00 10% $10.80", &m, 5).unwrap();
        assert_eq!(item.producto, "ROCKALETA");
        assert_eq!(item.cantidad, 2.0);
        assert_eq!(item.precio_unitario, 6.0);
        assert_eq!(item.total, 10.8);
        assert_eq!(item.descuento, Some(1.2));

        let item = parsear_linea("1 Heineken 473ml $28.00 - $28.00", &m, 6).unwrap();
        assert_eq!(item.producto, "HEINEKEN 473ML");
        assert_eq!(item.precio_unitario, 28.0);
        assert_eq!(item.total, 28.0);
    }

    // ---------- extraer_fecha_hora_regex (verificado contra Python) ----------

    #[test]
    fn fecha_dd_mm_yyyy_y_hora() {
        let (f, h) = extraer_fecha_hora_regex("Fecha: 15/03/2024\nCompra: 14:32\n");
        assert_eq!(f.as_deref(), Some("2024-03-15"));
        assert_eq!(h.as_deref(), Some("14:32"));
    }

    #[test]
    fn fecha_iso_y_hora_pm() {
        let (f, h) = extraer_fecha_hora_regex("2024-03-15\nHora: 2:32 PM\n");
        assert_eq!(f.as_deref(), Some("2024-03-15"));
        assert_eq!(h.as_deref(), Some("14:32"));
    }

    #[test]
    fn fecha_con_mes_en_texto() {
        let (f, h) = extraer_fecha_hora_regex("15 de marzo de 2024\n14:32:05\n");
        assert_eq!(f.as_deref(), Some("2024-03-15"));
        assert_eq!(h.as_deref(), Some("14:32"));
    }

    #[test]
    fn fecha_generica_sin_encabezado() {
        let (f, _) = extraer_fecha_hora_regex("Compra\n2024-03-15\n");
        assert_eq!(f.as_deref(), Some("2024-03-15"));
    }

    // ---------- extraer_metodo_pago (verificado contra Python) ----------

    #[test]
    fn pago_tarjeta_debito() {
        let texto = "TOTAL $100.00\nMETODO DE PAGO: TARJETA DEBITO\n";
        assert_eq!(extraer_metodo_pago(texto), "tarjeta");
    }

    #[test]
    fn pago_tarjeta_sola_es_tarjeta() {
        let texto = "TOTAL $100.00\nMETODO DE PAGO: TARJETA\n";
        assert_eq!(extraer_metodo_pago(texto), "tarjeta");
    }

    #[test]
    fn pago_efectivo_linea_con_monto() {
        let texto = "EFECTIVO........... $1,234.56\n";
        assert_eq!(extraer_metodo_pago(texto), "efectivo");
    }

    #[test]
    fn pago_transferencia() {
        let texto = "TOTAL $500.00\nFORMA DE PAGO: TRANSFERENCIA\n";
        assert_eq!(extraer_metodo_pago(texto), "transferencia");
    }

    #[test]
    fn pago_solo_ultimas_25_lineas() {
        let mut texto = String::from("TRANSFERENCIA $500\n");
        for _ in 0..30 {
            texto.push_str("ITEM DE PRUEBA $1.00\n");
        }
        // Método de pago fuera de las últimas 25 líneas → no se detecta.
        assert_eq!(extraer_metodo_pago(&texto), "efectivo");
    }
}
