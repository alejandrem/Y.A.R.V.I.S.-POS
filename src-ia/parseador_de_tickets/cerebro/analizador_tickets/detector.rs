// ============================================================
// detector — Detección ESTADÍSTICA del mapeo de columnas, sin LLM.
//
// Reemplaza al análisis con el modelo local (Qwen 1.7B) en la
// calibración de la importación masiva. En vez de "preguntarle" a un
// modelo qué columna es cada cosa, se ensayan hipótesis de mapeo y se
// ponen a prueba contra la ecuación
//
//      cantidad × precio_unitario  −  descuento  ≈  total
//
// sobre cientos de líneas reales del lote. El mapeo ganador no es una
// opinión ni una "confianza" reportada por el modelo: es el que
// VERIFICA matemáticamente contra los datos.
//
// Cubre formatos heterogéneos:
//   * CANT PRODUCTO PRECIO TOTAL          ("2 COCA 25.00 50.00")
//   * con $ y número separado             ("2 COCA $ 25 $ 50")
//   * cantidad compacta Nx                ("2x COCA $25 $50")
//   * producto primero, cantidad en medio ("COCA 2 25.00 50.00")
//   * código de barras como 1ª columna    ("75012345 1 COCA 25.00 25.00")
//   * descuento porcentual entre precio y total ("2 X $6 10% $10.80")
//   * columna de descuento numérico       ("2 X 25.00 2.00 48.00")
//   * productos de largo variable (los índices NEGATIVOS del mapeo
//     resultante toleran "COCA" y "FANTA NARANJA 600ML" a la vez).
//
// La regla de COBERTURA es la que da la confianza: TODA columna de cada
// línea debe estar explicada por la hipótesis (cantidad, producto,
// precio, total) o ser ruido conocido (porcentaje, "-", importe suelto).
// Así, productos de largo variable no engañan al detector: si una letra
// queda fuera del rango del producto, la hipótesis pierde la línea.
//
// Limitación honesta: formatos con UN solo importe por línea
// (ej. "2 COCA 50.00") no permiten distinguir precio de total por
// matemática → se devuelve None y la UI lo comunica.
// ============================================================
//
// Organización (un tema por archivo):
//   * muestra.rs     → filtrado de líneas útiles (`muestrear`)
//   * hipotesis.rs   → ensayo de mapeos (`linea_cuadra`, `hipotesis`)
//   * familia_c.rs   → validación por consistencia de precios
//   * diagnostico.rs → explicación accionable del fallo (para la UI)
//   * tests.rs       → suite del detector
// Este archivo solo orquesta: `detectar_mapeo` + tipos públicos.

mod diagnostico;
mod familia_c;
mod hipotesis;
mod muestra;
#[cfg(test)]
mod tests;

pub use diagnostico::{diagnosticar_muestra, DiagnosticoMuestra};
pub(crate) use familia_c::consistencia_familia_c;
pub(crate) use hipotesis::{hipotesis, linea_cuadra, MIN_VALIDAS};
pub(crate) use muestra::muestrear;

use super::esquema::MapeoColumnas;
use serde::Serialize;

/// Resultado de la detección estadística de columnas.
#[derive(Debug, Clone, Serialize)]
pub struct DeteccionMapeo {
    pub mapeo: MapeoColumnas,
    /// Fracción de líneas que cuadraron con el mapeo ganador (0.0..=1.0).
    /// En familia C (un solo importe) el denominador son solo las líneas
    /// de productos repetidos —las únicas observables— no toda la muestra.
    pub confianza: f64,
    /// Líneas consideradas (toda la muestra, o solo repetidas en familia C).
    pub lineas_evaluadas: usize,
    /// Líneas donde la hipótesis ganadora cuadró la ecuación.
    pub lineas_validas: usize,
}

/// Mínimo de líneas útiles para que la muestra signifique algo.
const MIN_LINEAS_MUESTRA: usize = 3;

/// Detecta estadísticamente el mapeo de columnas de un conjunto de líneas
/// de tickets. Devuelve `None` si la muestra es muy chica o ninguna
/// hipótesis cuadra (formato de un solo importe sin repetidos, texto
/// libre, etc.). Para saber POR QUÉ falló, ver `diagnosticar_muestra`.
pub fn detectar_mapeo(lineas: &[&str]) -> Option<DeteccionMapeo> {
    let muestras = muestrear(lineas);

    if muestras.len() < MIN_LINEAS_MUESTRA {
        return None;
    }

    // Recorre todas las hipótesis y se queda con la que MÁS líneas cuadra.
    // Desempate: las hipótesis CON columna de precio (donde la ecuación es
    // verificable línea a línea) ganan sobre la familia C (un solo importe,
    // validada solo por consistencia entre tickets).
    //
    // La confianza sale del denominador correcto de cada familia: en la A/B
    // es cuadradas/muestra; en la C es consistentes/repetidas (los productos
    // que aparecen una sola vez no son observables: ni confirman ni niegan).
    let mut mejor: Option<(usize, usize, bool, i32, Option<i32>, i32, (i32, i32))> = None;
    for (cant_i, precio_i, total_i, producto) in hipotesis() {
        let mut validas = 0usize;
        for cols in &muestras {
            if linea_cuadra(cols, cant_i, precio_i, total_i, &producto) == Some(true) {
                validas += 1;
            }
        }
        if validas < MIN_VALIDAS {
            continue;
        }
        // Denominador de la confianza (toda la muestra, salvo familia C).
        let mut evaluadas = muestras.len();
        if precio_i.is_none() {
            // Familia C: estructura no basta; exigimos consistencia de
            // precios entre productos repetidos como validación matemática.
            let Some(ev) = consistencia_familia_c(&muestras, cant_i, total_i, &producto)
            else {
                continue;
            };
            validas = ev.consistentes;
            evaluadas = ev.repetidas;
            if validas < MIN_VALIDAS {
                continue;
            }
        }
        let tiene_precio = precio_i.is_some();
        let es_mejor = match &mejor {
            None => true,
            Some((v, _, tp, _, _, _, _)) => {
                validas > *v || (validas == *v && tiene_precio && !*tp)
            }
        };
        if es_mejor {
            mejor = Some((validas, evaluadas, tiene_precio, cant_i, precio_i, total_i, producto));
        }
    }
    let (validas, evaluadas, _tiene_precio, cant_i, precio_i, total_i, producto) = mejor?;

    Some(DeteccionMapeo {
        mapeo: MapeoColumnas {
            cantidad: Some(cant_i),
            producto: Some(vec![producto.0, producto.1]),
            precio_unitario: precio_i,
            total: Some(total_i),
            descuento: None,
        },
        confianza: (validas as f64 / evaluadas as f64 * 1000.0).round() / 1000.0,
        lineas_evaluadas: evaluadas,
        lineas_validas: validas,
    })
}
