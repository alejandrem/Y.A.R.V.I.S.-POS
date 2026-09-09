// ============================================================
// parser — Orquesta la extracción y verifica la matemática.
//
// Falla solo si el archivo no es un corte X/Z. Los números
// faltantes quedan en cero/None y la verificación lo reporta en
// `advertencias` (un corte real a medias se guarda, no se tira).
// ============================================================

use super::secciones::{
    clasificar, extraer_cajero, extraer_empresa, extraer_estacion_fecha, extraer_folio,
    extraer_moneda, extraer_totales, linea_articulo, linea_pago, linea_ticket, partir,
    seccion, Marca,
};
use super::tipos::{ClaseArchivo, CorteParseado, ItemCorte, TipoCorte, Verificacion};

/// Parsea el texto completo de un corte. Err si no es X/Z.
pub fn parse_corte(texto: &str) -> Result<CorteParseado, String> {
    let tipo = match clasificar(texto) {
        ClaseArchivo::CorteX => TipoCorte::X,
        ClaseArchivo::CorteZ => TipoCorte::Z,
        ClaseArchivo::NoEsCorte => return Err("el archivo no es un corte de caja X/Z".to_string()),
    };

    let partes = partir(texto);
    let tot = extraer_totales(&partes);
    let (estacion, fecha) = extraer_estacion_fecha(texto);

    let mut items: Vec<ItemCorte> = Vec::new();
    for l in seccion(&partes, Marca::Ingresos) {
        if let Some((c, m)) = linea_pago(l) {
            items.push(ItemCorte {
                kind: "INGRESO".into(),
                nombre: c,
                cantidad: None,
                precio_unitario: m,
                subtotal: m,
            });
        }
    }
    for l in seccion(&partes, Marca::Egresos) {
        if let Some((c, m)) = linea_pago(l) {
            items.push(ItemCorte {
                kind: "EGRESO".into(),
                nombre: c,
                cantidad: None,
                precio_unitario: m,
                subtotal: m,
            });
        }
    }
    for l in seccion(&partes, Marca::PorArticulo) {
        if let Some((n, cant, sub)) = linea_articulo(l) {
            let unit = if cant > 0.0 { (sub as f64 / cant).round() as i64 } else { sub };
            items.push(ItemCorte {
                kind: "ARTICULO".into(),
                nombre: n,
                cantidad: Some(cant),
                precio_unitario: unit,
                subtotal: sub,
            });
        }
    }
    for l in seccion(&partes, Marca::PorTicket) {
        if let Some((folio, m)) = linea_ticket(l) {
            items.push(ItemCorte {
                kind: "TICKET".into(),
                nombre: folio,
                cantidad: None,
                precio_unitario: m,
                subtotal: m,
            });
        }
    }

    let mut adv = Vec::new();
    let caja_ok = match (tot.ingresos, tot.egresos, tot.caja) {
        (Some(i), Some(e), Some(c)) => {
            let ok = c == i - e;
            if !ok {
                adv.push(format!("caja {c} != ingresos {i} - egresos {e}"));
            }
            ok
        }
        _ => {
            adv.push("faltan ingresos/egresos/caja para verificar".to_string());
            false
        }
    };
    let suma_items: i64 = items
        .iter()
        .filter(|it| it.kind == "ARTICULO" || it.kind == "TICKET")
        .map(|it| it.subtotal)
        .sum();
    let hay_vendibles = items.iter().any(|it| it.kind == "ARTICULO" || it.kind == "TICKET");
    let ventas_ok = match (tot.total_ventas, hay_vendibles) {
        (Some(v), true) => {
            let ok = v == suma_items;
            if !ok {
                adv.push(format!("total ventas {v} != suma renglones {suma_items}"));
            }
            ok
        }
        (None, _) => {
            adv.push("falta el total de ventas para verificar".to_string());
            false
        }
        (Some(_), false) => {
            adv.push("sin renglones de artículo/ticket que sumen".to_string());
            false
        }
    };

    Ok(CorteParseado {
        tipo,
        folio: extraer_folio(texto),
        estacion,
        fecha,
        cajero: extraer_cajero(texto),
        empresa: extraer_empresa(texto),
        moneda: extraer_moneda(texto),
        total_ingresos: tot.ingresos.unwrap_or(0),
        total_egresos: tot.egresos.unwrap_or(0),
        total_caja: tot.caja.unwrap_or(0),
        total_ventas: tot.total_ventas.unwrap_or(0),
        ventas_gravadas: tot.gravadas.unwrap_or(0),
        impuesto: tot.impuesto.or(tot.impuesto_16).unwrap_or(0),
        ventas_no_gravadas: tot.no_gravadas.unwrap_or(0),
        redondeos: tot.redondeos.unwrap_or(0),
        ventas_credito: tot.ventas_credito.unwrap_or(0),
        total_unidades: tot.unidades.unwrap_or(0.0),
        clientes_atendidos: tot.clientes.unwrap_or(0),
        items,
        verificacion: Verificacion { caja_ok, ventas_ok, advertencias: adv },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Z sintético pero con la forma del estándar (artículos + 12h am).
    const MINI_Z: &str = "*** CORTE Z EN MONEDA:MXN***\nTIENDA X\n*** Corte Z 9\nCAJA1 05/06/2024 10:15:00 a. m.\n**Ingresos**\nEFE Ventas  $1,000.00\nTotal de Ingresos: $1,000.00\n**Egresos**\nTotal de Egresos: $.00\nTotal en caja: $1,000.00\n*********VENTAS DEL CORTE**********\nVentas no gravadas: $1,000.00\nTotal de ventas: $1,000.00\n**Ventas por artículo**\nPROD A - 2 - $600.00\nPROD B - 1 - $400.00\n**Total venta en unidades: 3.00\nClientes atendidos: 2";

    /// X sintético con la variante `CORTE DE CAJA` y hora 24h.
    const MINI_X: &str = "CORTE DE CAJA X EN MONEDA:MXN\nTIENDA X\nCajero: ANA\nCorte de caja X 4\nCAJA2 06/06/2024 21:00:00\n**Ingresos**\nEFE Ventas $700.00\nTotal de Ingresos: $700.00\n**Egresos**\nTotal de Egresos: $.00\nTotal en caja: $700.00\n*********VENTAS DEL CORTE**********\nVentas no gravadas: $700.00\nTotal de ventas: $700.00\n**Ventas por ticket**\nF-1  500.00\nF-2  200.00\nClientes atendidos: 5";

    #[test]
    fn z_minimo_parsea_y_verifica() {
        let c = parse_corte(MINI_Z).expect("el Z mínimo debe parsear");
        assert_eq!(c.tipo, TipoCorte::Z);
        assert_eq!(c.folio, Some("9".into()));
        assert_eq!(c.estacion, Some("CAJA1".into()));
        assert_eq!(c.fecha, Some("2024-06-05 10:15:00".into()));
        assert_eq!(c.total_ingresos, 100000);
        assert_eq!(c.total_caja, 100000);
        assert_eq!(c.total_ventas, 100000);
        assert_eq!(c.total_unidades, 3.0);
        assert_eq!(c.clientes_atendidos, 2);
        assert_eq!(c.items.iter().filter(|i| i.kind == "ARTICULO").count(), 2);
        assert!(c.verificacion.caja_ok && c.verificacion.ventas_ok);
    }

    #[test]
    fn x_con_variante_de_titulo_parsea_y_verifica() {
        let c = parse_corte(MINI_X).expect("el X mínimo debe parsear");
        assert_eq!(c.tipo, TipoCorte::X);
        assert_eq!(c.folio, Some("4".into()));
        assert_eq!(c.fecha, Some("2024-06-06 21:00:00".into()));
        assert_eq!(c.cajero, "ANA");
        assert_eq!(c.total_ventas, 70000);
        assert_eq!(c.items.iter().filter(|i| i.kind == "TICKET").count(), 2);
        assert!(c.verificacion.caja_ok && c.verificacion.ventas_ok);
    }

    #[test]
    fn descuadre_se_reporta_no_se_tira() {
        let roto = MINI_Z.replace("Total en caja: $1,000.00", "Total en caja: $999.00");
        let c = parse_corte(&roto).expect("un descuadre no debe tumbar el parseo");
        assert!(!c.verificacion.caja_ok);
        assert_eq!(c.verificacion.advertencias.len(), 1);
        assert_eq!(c.total_caja, 99900);
    }

    #[test]
    fn ticket_normal_no_es_corte() {
        assert!(parse_corte("2 Pan Bimbo 42.00 84.00\nTOTAL: $84.00").is_err());
    }

    /// Formato variante: estación minúscula, hora 24h, egresos reales,
    /// X con artículos en vez de tickets y sin línea de empresa.
    /// Prueba que el parser no está memorizando el estándar.
    const VARIANTE: &str = "*** CORTE X EN MONEDA:MXN***\nCorte X 2\ncaja_norte 15/07/2024 14:05:09\n**Ingresos**\nEFE Ventas $900.00\nTotal de Ingresos: $900.00\n**Egresos**\nRetiro socio $100.00\nTotal de Egresos: $100.00\nTotal en caja: $800.00\n*********VENTAS DEL CORTE**********\nVentas no gravadas: $800.00\nTotal de ventas: $800.00\n**Ventas por artículo**\nREFRESCO - 4 - $800.00\nClientes atendidos: 3";

    #[test]
    fn variante_con_egresos_y_sin_empresa_verifica() {
        let c = parse_corte(VARIANTE).expect("la variante debe parsear");
        assert_eq!(c.tipo, TipoCorte::X);
        assert_eq!(c.folio, Some("2".into()));
        assert_eq!(c.estacion, Some("caja_norte".into()));
        assert_eq!(c.fecha, Some("2024-07-15 14:05:09".into()));
        assert_eq!(c.empresa, None);
        assert_eq!(c.total_ingresos, 90000);
        assert_eq!(c.total_egresos, 10000);
        assert_eq!(c.total_caja, 80000);
        assert_eq!(c.total_ventas, 80000);
        assert_eq!(c.items.iter().filter(|i| i.kind == "EGRESO").count(), 1);
        assert!(c.verificacion.caja_ok && c.verificacion.ventas_ok);
        assert!(c.verificacion.advertencias.is_empty());
    }
}
