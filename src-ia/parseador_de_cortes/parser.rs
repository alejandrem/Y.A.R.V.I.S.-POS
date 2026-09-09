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
    let tot = extraer_totales(&partes, texto);
    let (estacion, fecha) = extraer_estacion_fecha(texto);

    let mut items: Vec<ItemCorte> = Vec::new();
    for l in seccion(&partes, Marca::Ingresos) {
        if let Some((c, m)) = linea_pago(l, "total de ingresos") {
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
        if let Some((c, m)) = linea_pago(l, "total de egresos") {
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
    use super::super::fixtures::{CORTE_X_EJEMPLO, CORTE_Z_EJEMPLO};
    use super::*;

    #[test]
    fn z_estandar_parsea_y_verifica() {
        let c = parse_corte(CORTE_Z_EJEMPLO).expect("el Z estándar debe parsear");
        assert_eq!(c.tipo, TipoCorte::Z);
        assert_eq!(c.folio, Some("54".into()));
        assert_eq!(c.estacion, Some("ESTACION01".into()));
        assert_eq!(c.fecha, Some("2025-01-01 11:25:57".into()));
        assert_eq!(c.cajero, "SISTEMA");
        assert_eq!(c.total_ingresos, 246280);
        assert_eq!(c.total_egresos, 0);
        assert_eq!(c.total_caja, 246280);
        assert_eq!(c.total_ventas, 246280);
        assert_eq!(c.ventas_no_gravadas, 246280);
        assert_eq!(c.total_unidades, 20.0);
        assert_eq!(c.clientes_atendidos, 9);
        let arts: Vec<_> = c.items.iter().filter(|i| i.kind == "ARTICULO").collect();
        assert_eq!(arts.len(), 12);
        assert_eq!(c.verificacion.caja_ok, true);
        assert_eq!(c.verificacion.ventas_ok, true);
        assert!(c.verificacion.advertencias.is_empty());
    }

    #[test]
    fn x_estandar_parsea_y_verifica() {
        let c = parse_corte(CORTE_X_EJEMPLO).expect("el X estándar debe parsear");
        assert_eq!(c.tipo, TipoCorte::X);
        assert_eq!(c.folio, Some("1".into()));
        assert_eq!(c.fecha, Some("2026-03-31 21:35:08".into()));
        assert_eq!(c.cajero, "GENERAL");
        assert_eq!(c.total_ingresos, 70200);
        assert_eq!(c.total_caja, 70200);
        assert_eq!(c.total_ventas, 70200);
        assert_eq!(c.total_unidades, 19.0);
        assert_eq!(c.clientes_atendidos, 8);
        let tix: Vec<_> = c.items.iter().filter(|i| i.kind == "TICKET").collect();
        assert_eq!(tix.len(), 8);
        assert_eq!(c.verificacion.caja_ok, true);
        assert_eq!(c.verificacion.ventas_ok, true);
    }

    #[test]
    fn ticket_normal_no_es_corte() {
        assert!(parse_corte("2 Pan Bimbo 42.00 84.00\nTOTAL: $84.00").is_err());
    }
}
