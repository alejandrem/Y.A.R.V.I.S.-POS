// ============================================================
// familia_c — Validación por CONSISTENCIA DE PRECIO UNITARIO.
//
// En el formato "CANT PRODUCTO IMPORTE" no hay precio por línea que
// multiplicar: la única fuente de verdad es que el MISMO producto cobra
// (casi) siempre el mismo precio unitario. Sobre un lote de meses los
// productos se repiten cientos de veces, así que esta señal es FUERTE.
//
// Se tolera el historial de precios (el plátano pudo subir de $22 a $25):
// por producto se toma el precio DOMINANTE y se cuentan como sólidas solo
// las líneas a +-3% de él. Con todo eso, solo aceptamos la familia C si:
//   * hay suficientes líneas observables (productos que se repiten),
//   * ≥ 60% de esas líneas son consistentes con el precio dominante.
// ============================================================

use super::super::esquema::resolver_indice;
use super::super::parser::limpiar_precio;
use super::hipotesis::linea_cuadra;

const MIN_REPETIDAS_C: usize = 5;
const MIN_FRACCION_CONSISTENTE_C: f64 = 0.6;

/// Evidencia de familia C: líneas observables (de productos repetidos,
/// las únicas que pueden validar precio) y cuántas son consistentes.
pub(crate) struct EvidenciaC {
    pub(crate) repetidas: usize,
    pub(crate) consistentes: usize,
}

/// Valida la familia C por CONSISTENCIA DE PRECIO UNITARIO y devuelve la
/// evidencia; None si es demasiado débil para confiar (pocos repetidos o
/// fracción consistente < 60%).
pub(crate) fn consistencia_familia_c(
    muestras: &[Vec<String>],
    cant_i: i32,
    total_i: i32,
    producto: &(i32, i32),
) -> Option<EvidenciaC> {
    // Clave: producto (upper) → lista de precios unitarios observados.
    let mut por_producto: std::collections::HashMap<String, Vec<f64>> =
        std::collections::HashMap::new();

    for cols in muestras {
        if linea_cuadra(cols, cant_i, None, total_i, producto) != Some(true) {
            continue;
        }
        let n = cols.len();
        let ci = resolver_indice(Some(cant_i), n)?;
        let ti = resolver_indice(Some(total_i), n)?;
        let p_ini = resolver_indice(Some(producto.0), n)?;
        let p_fin = resolver_indice(Some(producto.1), n)?;
        if p_fin < p_ini {
            continue;
        }
        let cantidad = limpiar_precio(&cols[ci]);
        let total = limpiar_precio(&cols[ti]);
        if cantidad <= 0.0 {
            continue;
        }
        let clave = cols[p_ini..=p_fin].join(" ").to_uppercase();
        por_producto
            .entry(clave)
            .or_default()
            .push(total / cantidad);
    }

    let mut repetidas = 0usize;
    let mut consistentes = 0usize;
    for precios in por_producto.values() {
        if precios.len() < 2 {
            continue;
        }
        repetidas += precios.len();
        // Precio dominante: el cubo de 2 decimales con más líneas.
        let mut cubos: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for p in precios {
            *cubos.entry(format!("{p:.2}")).or_default() += 1;
        }
        let dominante: f64 = cubos
            .iter()
            .max_by_key(|(_, c)| *c)
            .map(|(k, _)| k.parse().unwrap_or(0.0))
            .unwrap_or(0.0);
        consistentes += precios
            .iter()
            .filter(|p| {
                let dif = (*p - dominante).abs();
                dif <= 0.05_f64.max(dominante.abs() * 0.03)
            })
            .count();
    }

    if repetidas < MIN_REPETIDAS_C {
        return None;
    }
    let fraccion = consistentes as f64 / repetidas as f64;
    (fraccion >= MIN_FRACCION_CONSISTENTE_C).then_some(EvidenciaC {
        repetidas,
        consistentes,
    })
}
