// ============================================================
// unir — Pegado de productos multi-renglon (familia "El Trebol").
//
// Algunas impresoras parten cada producto en DOS renglones:
//
//   1) Pelon Pelo Rico
//      2 pza x $8.00..........$16.00
//
// El detector y el parser trabajan renglon por renglon exigiendo
// `cantidad x precio ~= total` en UNO SOLO: la mitad-nombre no tiene
// numeros y la mitad-detalle no tiene nombre, asi que 0% cuadraba.
//
// Este modulo fusiona esos pares ANTES de segmentar/detectar:
//
//   2 Pelon Pelo Rico $8.00..........$16.00
//
// (los puntos se separan despues en `preprocesar_linea`).
// Solo se pega si el renglon N es cabecera `N) Nombre` y el N+1 es
// detalle `cant UNIDAD x $...`. Las lineas sueltas pasan intactas,
// asi que los formatos viejos de un renglon no cambian nada.
// ============================================================

use regex::Regex;
use std::sync::LazyLock;

/// Cabecera de item: `1) Pelon Pelo Rico`. Se captura el nombre.
static RE_CABECERA: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\s*\d+\)\s*(\S.*)$").expect("regex cabecera multi-renglon")
});

/// Detalle de item: `2 pza x $8.00..........$16.00`.
/// Unidades tipicas de tickets mexicanos; el `x` + `$` son obligatorios
/// para no confundir un producto que se llame "Caja" (`2 Caja $50 $100`
/// NO pega porque le falta el `x $`).
static RE_DETALLE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)^\s*(\d+(?:[.,]\d+)?)\s+(?:pzas?\.?|pza\.?|piezas?|paq(?:uete)?s?\.?|cajas?|bolsas?|botellas?|latas?|kg|kgs?|grs?|g|l|lt|ml|m)\b\s*x\s*(\$.*)$",
    )
    .expect("regex detalle multi-renglon")
});

/// Nombre de la cabecera (`1) Pelon` -> `Pelon`), None si no es cabecera.
fn nombre_cabecera(linea: &str) -> Option<String> {
    let c = RE_CABECERA.captures(linea)?;
    let nombre = c[1].trim().to_string();
    (!nombre.is_empty()).then_some(nombre)
}

/// (cantidad, cuerpo-desde-$) del detalle, None si no es detalle.
fn detalle_cantidad(linea: &str) -> Option<(String, String)> {
    let c = RE_DETALLE.captures(linea)?;
    let cuerpo = c[2].trim().to_string();
    (!cuerpo.is_empty()).then_some((c[1].to_string(), cuerpo))
}

/// Fusiona pares cabecera+detalle de todo el texto. Idempotente en la
/// practica: una linea ya pegada (`2 Pelon $8.00 $16.00`) no es cabecera
/// (`N)`) asi que no se re-pega.
pub fn unir_lineas_multirenglon(texto: &str) -> String {
    let lineas: Vec<&str> = texto.lines().collect();
    let mut out: Vec<String> = Vec::with_capacity(lineas.len());
    let mut i = 0;
    while i < lineas.len() {
        if i + 1 < lineas.len() {
            if let (Some(nombre), Some((cant, cuerpo))) = (
                nombre_cabecera(lineas[i]),
                detalle_cantidad(lineas[i + 1]),
            ) {
                out.push(format!("{cant} {nombre} {cuerpo}"));
                i += 2;
                continue;
            }
        }
        out.push(lineas[i].to_string());
        i += 1;
    }
    out.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pega_par_cabecera_detalle() {
        let texto = "1) Pelon Pelo Rico\n   2 pza x $8.00..........$16.00\nTOTAL $16.00";
        let unido = unir_lineas_multirenglon(texto);
        assert!(unido.contains("2 Pelon Pelo Rico $8.00..........$16.00"));
        assert!(!unido.contains("1) Pelon"));
    }

    #[test]
    fn lineas_sueltas_no_se_tocan() {
        // Formato viejo de un renglon: intacto.
        let texto = "2 Rockaleta $6.00 10% $10.80\n4 Gomitas $5.00 $20.00";
        assert_eq!(unir_lineas_multirenglon(texto), texto);
        // Cabecera sin detalle y detalle sin cabecera: intactos.
        let texto = "1) Pelon Pelo Rico\nTOTAL $16.00";
        assert_eq!(unir_lineas_multirenglon(texto), texto);
        let texto = "2 pza x $8.00..........$16.00";
        assert_eq!(unir_lineas_multirenglon(texto), texto);
    }

    #[test]
    fn producto_llamado_caja_no_se_confunde() {
        // `2 Caja $50.00 $100.00` no es detalle (falta `x $`): no pega.
        let texto = "7) Algo\n2 Caja $50.00 $100.00";
        assert_eq!(unir_lineas_multirenglon(texto), texto);
    }

    #[test]
    fn cantidad_decimal_y_unidades_variadas() {
        let texto = "3) Tortilla kg\n   1.5 kg x $18.00..........$27.00";
        let unido = unir_lineas_multirenglon(texto);
        assert!(unido.contains("1.5 Tortilla kg $18.00..........$27.00"));
    }
}
