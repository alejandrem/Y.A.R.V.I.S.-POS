// ============================================================
// muestra — Filtrado de líneas útiles para la detección.
// ============================================================
//
// De todo el texto crudo se queda solo lo que puede ser un renglón de
// producto con (cantidad, producto, ≥1 importe): el resto (encabezados,
// totales, agradecimientos) no vota en la detección.

use super::super::{es_linea_util, parser::preprocesar_linea};

/// Techo de líneas a considerar (más no mejora la detección, solo cuesta).
pub(crate) const MAX_LINEAS: usize = 2000;

pub(crate) fn muestrear(lineas: &[&str]) -> Vec<Vec<String>> {
    lineas
        .iter()
        .map(|l| l.trim())
        .filter(|l| es_linea_util(l))
        .take(MAX_LINEAS)
        .map(preprocesar_linea)
        .map(|l| l.split_whitespace().map(String::from).collect::<Vec<_>>())
        // Sin ≥3 columnas no hay (cantidad, producto, ≥1 importe) detectable.
        .filter(|cols| cols.len() >= 3)
        .collect()
}
