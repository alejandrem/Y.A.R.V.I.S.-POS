// ============================================================
// totales — Los 13 totales del corte en centavos.
//
// Cada etiqueta se busca SOLO dentro de su sección: así "Impuesto"
// pelado no se confunde con "Impuesto 16%"/"Impuesto 10%", ni
// "Total de ventas" con "Total ventas del dia".
// ============================================================

use regex::Regex;
use std::sync::LazyLock;

use super::marcadores::{seccion, Marca};
use crate::parseador_de_cortes::valores::montos::limpiar_monto;

/// Todos los totales del corte (faltantes quedan en None).
#[derive(Debug, Default)]
pub struct TotalesCorte {
    pub ingresos: Option<i64>,
    pub egresos: Option<i64>,
    pub caja: Option<i64>,
    pub ventas_16: Option<i64>,
    pub impuesto_16: Option<i64>,
    pub ventas_10: Option<i64>,
    pub impuesto_10: Option<i64>,
    pub gravadas: Option<i64>,
    pub impuesto: Option<i64>,
    pub no_gravadas: Option<i64>,
    pub redondeos: Option<i64>,
    pub total_ventas: Option<i64>,
    pub ventas_credito: Option<i64>,
    pub unidades: Option<f64>,
    pub clientes: Option<i64>,
}

/// Monto al final de la primera línea que contenga la etiqueta.
/// La línea debe TERMINAR en el monto; los porcentajes ("16%") se
/// buscan tras los ':' para no tomarlos como valor.
fn monto_tras(lineas: &[String], etiqueta: &str) -> Option<i64> {
    static RE_FIN: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)\$?\s*([\d,]+\.\d{2}|\.\d{2}|\d+)\s*$").expect("regex monto final")
    });
    let et = etiqueta.to_lowercase();
    for l in lineas {
        if !l.to_lowercase().contains(&et) {
            continue;
        }
        let cola = l.rsplit(':').next().unwrap_or(l);
        if let Some(c) = RE_FIN.captures(cola) {
            if let Some(v) = limpiar_monto(&c[1]) {
                return Some(v);
            }
        }
    }
    None
}

/// Etiqueta anclada al inicio (`^impuesto\s*:`) para no confundirla
/// con `Impuesto 16%:` / `Impuesto 10%:`.
fn monto_anclado(lineas: &[String], etiqueta: &str) -> Option<i64> {
    let patron = format!(r"(?i)^\**\s*{etiqueta}\s*:");
    let re = Regex::new(&patron).ok()?;
    for l in lineas {
        if re.is_match(l) {
            let cola = l.rsplit(':').next().unwrap_or(l);
            static RE_FIN: LazyLock<Regex> = LazyLock::new(|| {
                Regex::new(r"\$?\s*([\d,]+\.\d{2}|\.\d{2}|\d+)\s*$").expect("regex monto final 2")
            });
            if let Some(c) = RE_FIN.captures(cola) {
                if let Some(v) = limpiar_monto(&c[1]) {
                    return Some(v);
                }
            }
        }
    }
    None
}

pub fn extraer_totales(partes: &[(Marca, Vec<String>)]) -> TotalesCorte {
    let ing = seccion(partes, Marca::Ingresos);
    let egr = seccion(partes, Marca::Egresos);
    let vc = seccion(partes, Marca::VentasCorte);
    let todas: Vec<String> = partes.iter().flat_map(|(_, ls)| ls.clone()).collect();
    let mut unidades: Option<f64> = None;
    let mut clientes: Option<i64> = None;
    for l in todas.iter() {
        let low = l.to_lowercase();
        if unidades.is_none() && low.contains("total venta en unidades") {
            let cola = l.rsplit(':').next().unwrap_or(l);
            let num: String = cola.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
            unidades = num.parse::<f64>().ok();
        }
        if clientes.is_none() && low.contains("clientes atendidos") {
            let num: String = l.chars().filter(|c| c.is_ascii_digit()).collect();
            clientes = num.parse::<i64>().ok();
        }
    }
    TotalesCorte {
        ingresos: monto_tras(ing, "total de ingresos"),
        egresos: monto_tras(egr, "total de egresos"),
        caja: monto_tras(ing, "total en caja")
            .or_else(|| monto_tras(egr, "total en caja"))
            .or_else(|| monto_tras(&todas, "total en caja")),
        ventas_16: monto_tras(vc, "ventas 16%"),
        impuesto_16: monto_tras(vc, "impuesto 16%"),
        ventas_10: monto_tras(vc, "ventas 10%"),
        impuesto_10: monto_tras(vc, "impuesto 10%"),
        gravadas: monto_anclado(vc, "ventas gravadas"),
        impuesto: monto_anclado(vc, "impuesto"),
        no_gravadas: monto_anclado(vc, "ventas no gravadas"),
        redondeos: monto_tras(vc, "redondeos"),
        total_ventas: monto_tras(vc, "total de ventas")
            .or_else(|| monto_tras(&todas, "total ventas del dia")),
        ventas_credito: monto_tras(vc, "ventas credito"),
        unidades,
        clientes,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn impuesto_pelado_no_confunde_tasas() {
        let vc = vec![
            "Impuesto 16%      :       $.00".to_string(),
            "Ventas gravadas   :       $.00".to_string(),
            "Impuesto          :       $.00".to_string(),
        ];
        assert_eq!(monto_anclado(&vc, "impuesto"), Some(0));
        assert_eq!(monto_tras(&vc, "impuesto 16%"), Some(0));
    }

    #[test]
    fn total_de_ventas_no_confunde_total_del_dia() {
        let vc = vec!["Total de ventas   :  $1,000.00".to_string()];
        let otras = vec!["**Total ventas del dia   :   1,000.00".to_string()];
        assert_eq!(monto_tras(&vc, "total de ventas"), Some(100000));
        // ...pero sirve de respaldo cuando la sección no lo trae.
        assert_eq!(monto_tras(&otras, "total ventas del dia"), Some(100000));
    }
}
