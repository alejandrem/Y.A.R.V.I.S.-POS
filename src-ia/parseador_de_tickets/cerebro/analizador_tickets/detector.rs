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

use serde::Serialize;

use super::esquema::{resolver_indice, MapeoColumnas};
use super::parser::{es_porcentaje, es_token_numero, limpiar_precio, preprocesar_linea};
use super::es_linea_util;

/// Resultado de la detección estadística de columnas.
#[derive(Debug, Clone, Serialize)]
pub struct DeteccionMapeo {
    pub mapeo: MapeoColumnas,
    /// Fracción de líneas de la muestra cuya ecuación cuadró con el
    /// mapeo ganador (0.0..=1.0).
    pub confianza: f64,
    /// Líneas útiles con ≥3 columnas consideradas en la muestra.
    pub lineas_evaluadas: usize,
    /// Líneas donde la hipótesis ganadora cuadró la ecuación.
    pub lineas_validas: usize,
}

/// Mínimo de líneas útiles para que la muestra signifique algo.
const MIN_LINEAS_MUESTRA: usize = 3;
/// Mínimo de líneas que la hipótesis ganadora debe cuadrar.
const MIN_VALIDAS: usize = 3;
/// Techo de líneas a considerar (más no mejora la detección, solo cuesta).
const MAX_LINEAS: usize = 2000;
/// Una cantidad de ticket nunca pasa de esto (protección contra códigos
/// de barras o piezas de fecha confundidas con cantidad).
const CANTIDAD_MAXIMA: f64 = 100_000.0;

// Candidatos de índice (negativos = desde la derecha). El orden de
// recorrido favorece el formato dominante de tickets mexicanos:
// total=-1, precio=-2.
const CANDIDATOS_TOTAL: &[i32] = &[-1, -2, -3];
const CANDIDATOS_PRECIO: &[i32] = &[-1, -2, -3, -4];

/// Una columna fuera de los roles asignados no debe ser letra suelta:
/// solo el ruido típico de ticket está permitido (importe extra de
/// descuento, porcentaje, guion de "sin descuento", código de barras).
fn es_ruido_permitido(token: &str) -> bool {
    token == "-"
        || es_token_numero(token)
        || es_porcentaje(token).is_some()
        || (token.len() >= 6 && token.chars().all(|c| c.is_ascii_digit()))
}

/// Prueba UNA hipótesis sobre UNA línea.
///
/// - `None`  → no evaluable con esta hipótesis (columnas que no resuelven
///             o no son numéricas donde debe); no cuenta ni a favor ni
///             en contra.
/// - `Some(true)`  → CUADRA: cant×precio(−descuento) ≈ total y TODA la
///                   línea queda explicada (regla de cobertura).
/// - `Some(false)` → evaluable pero NO cuadra: en contra.
fn linea_cuadra(
    cols: &[String],
    cant_i: i32,
    precio_i: i32,
    total_i: i32,
    producto: &(i32, i32),
) -> Option<bool> {
    let n = cols.len();
    let ci = resolver_indice(Some(cant_i), n)?;
    let pi = resolver_indice(Some(precio_i), n)?;
    let ti = resolver_indice(Some(total_i), n)?;
    if ci == pi || ci == ti || pi == ti {
        return None;
    }

    // Rango del producto
    let p_ini = resolver_indice(Some(producto.0), n)?;
    let p_fin = resolver_indice(Some(producto.1), n)?;
    if p_ini > p_fin {
        return None;
    }
    let rango_producto: Vec<usize> = (p_ini..=p_fin).collect();
    if rango_producto.contains(&ci) || rango_producto.contains(&pi) || rango_producto.contains(&ti)
    {
        return None;
    }

    // El producto debe tener al menos una letra.
    if !rango_producto
        .iter()
        .any(|&i| cols[i].chars().any(|c| c.is_ascii_alphabetic()))
    {
        return Some(false);
    }

    // Cobertura: ninguna columna sin explicar.
    for (i, token) in cols.iter().enumerate() {
        if i == ci || i == pi || i == ti || rango_producto.contains(&i) {
            continue;
        }
        if !es_ruido_permitido(token) {
            return Some(false);
        }
    }

    let cantidad = if es_token_numero(&cols[ci]) {
        limpiar_precio(&cols[ci])
    } else {
        return None;
    };
    let precio = if es_token_numero(&cols[pi]) {
        limpiar_precio(&cols[pi])
    } else {
        return None;
    };
    let total = if es_token_numero(&cols[ti]) {
        limpiar_precio(&cols[ti])
    } else {
        return None;
    };

    if cantidad <= 0.0 || cantidad > CANTIDAD_MAXIMA || precio <= 0.0 || total <= 0.0 {
        return Some(false);
    }

    // Tolerancia: 5 centavos o 1% (redondeos de la impresora).
    let base = cantidad * precio;
    let tolerancia = 0.05_f64.max(base.abs() * 0.01);
    if (base - total).abs() <= tolerancia {
        return Some(true);
    }

    // Descuento porcentual ENTRE precio y total ("10%" en 2 X $6 10% $10.80).
    let (lo, hi) = if pi < ti { (pi, ti) } else { (ti, pi) };
    for token in &cols[lo + 1..hi] {
        if let Some(pct) = es_porcentaje(token) {
            let con_descuento = base * (100.0 - pct) / 100.0;
            if (con_descuento - total).abs() <= tolerancia {
                return true.into();
            }
        }
    }

    // Descuento como COLUMNA numérica: "2 COCA 25.00 2.00 48.00" →
    // cant×precio − descuento ≈ total (los importes intermedios se suman).
    let descuentos: f64 = cols[lo + 1..hi]
        .iter()
        .filter(|t| es_token_numero(t))
        .map(|t| limpiar_precio(t))
        .sum();
    if descuentos > 0.0 && (base - descuentos - total).abs() <= tolerancia {
        return Some(true);
    }
    Some(false)
}

/// Detecta estadísticamente el mapeo de columnas de un conjunto de líneas
/// de tickets. Devuelve `None` si la muestra es muy chica o ninguna
/// hipótesis cuadra (formato de un solo importe, texto libre, etc.).
pub fn detectar_mapeo(lineas: &[&str]) -> Option<DeteccionMapeo> {
    let muestras: Vec<Vec<String>> = lineas
        .iter()
        .map(|l| l.trim())
        .filter(|l| es_linea_util(l))
        .take(MAX_LINEAS)
        .map(preprocesar_linea)
        .map(|l| l.split_whitespace().map(String::from).collect::<Vec<_>>())
        // Sin ≥3 columnas no hay (cantidad, producto, ≥1 importe) detectable.
        .filter(|cols| cols.len() >= 3)
        .collect();

    if muestras.len() < MIN_LINEAS_MUESTRA {
        return None;
    }

    // Recorre todas las hipótesis y se queda con la que MÁS líneas cuadra
    // (en empate gana la primera, que es el formato más dominante según
    // el orden de `hipotesis()`).
    let mut mejor: Option<(usize, i32, i32, i32, (i32, i32))> = None;
    for (cant_i, precio_i, total_i, producto) in hipotesis() {
        let mut validas = 0usize;
        for cols in &muestras {
            if linea_cuadra(cols, cant_i, precio_i, total_i, &producto) == Some(true) {
                validas += 1;
            }
        }
        if validas >= MIN_VALIDAS
            && mejor
                .as_ref()
                .map(|(v, _, _, _, _)| validas > *v)
                .unwrap_or(true)
        {
            mejor = Some((validas, cant_i, precio_i, total_i, producto));
        }
    }
    let (validas, cant_i, precio_i, total_i, producto) = mejor?;

    Some(DeteccionMapeo {
        mapeo: MapeoColumnas {
            cantidad: Some(cant_i),
            producto: Some(vec![producto.0, producto.1]),
            precio_unitario: Some(precio_i),
            total: Some(total_i),
            descuento: None,
        },
        confianza: (validas as f64 / muestras.len() as f64 * 1000.0).round() / 1000.0,
        lineas_evaluadas: muestras.len(),
        lineas_validas: validas,
    })
}

/// Candidatos de cantidad por FAMILIA. En la familia A (cantidad primero)
/// convienen índices desde la izquierda (el anclaje típico); en la B
/// (producto primero) convienen NEGATIVOS para tolerar nombres largos:
/// "COCA 2 25 50" y "FANTA NARANJA 600ML 2 15 30" comparten cantidad=-3.
const CANDIDATOS_CANTIDAD_A: &[i32] = &[0, 1, 2, -3, -4];
const CANDIDATOS_CANTIDAD_B: &[i32] = &[-3, -4, -5, 1, 2, 3];

/// Genera todas las hipótesis viables, las dos familias incluidas.
/// Devuelve (cant_i, precio_i, total_i, (prod_ini, prod_fin)).
fn hipotesis() -> Vec<(i32, i32, i32, (i32, i32))> {
    let mut out = Vec::new();
    for &total_i in CANDIDATOS_TOTAL {
        for &precio_i in CANDIDATOS_PRECIO {
            for &cant_i in CANDIDATOS_CANTIDAD_A {
                // Familia A: CANT PRODUCTO PRECIO TOTAL
                out.push((cant_i, precio_i, total_i, (cant_i + 1, precio_i - 1)));
            }
            for &cant_i in CANDIDATOS_CANTIDAD_B {
                // Familia B: PRODUCTO CANT PRECIO TOTAL
                out.push((cant_i, precio_i, total_i, (0, cant_i - 1)));
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn detectar(lineas: &[&str]) -> DeteccionMapeo {
        detectar_mapeo(lineas).expect("debió detectar un mapeo")
    }

    // ---------- Formatos básicos ----------

    #[test]
    fn formato_clasico_cantidad_producto_precio_total() {
        let lineas = [
            "2 COCA 25.00 50.00",
            "5 PAN 10.00 50.00",
            "1 LECHE 22.50 22.50",
            "3 JABON 15.00 45.00",
        ];
        let d = detectar(&lineas);
        assert_eq!(d.mapeo.cantidad, Some(0));
        assert_eq!(d.mapeo.precio_unitario, Some(-2));
        assert_eq!(d.mapeo.total, Some(-1));
        assert_eq!(d.mapeo.producto, Some(vec![1, -3]));
        assert_eq!(d.confianza, 1.0);
    }

    #[test]
    fn formato_con_dolar_y_producto_de_largo_variable() {
        let lineas = [
            "2 FANTA NARANJA 600ML $15.50 $31.00",
            "1 COCA $25.00 $25.00",
            "10 SABRITAS ORIGINAL $12.00 $120.00",
            "3 GALLETAS EMPERADOR CHOCOLATE $8.50 $25.50",
        ];
        let d = detectar(&lineas);
        assert_eq!(d.mapeo.cantidad, Some(0));
        assert_eq!(d.mapeo.precio_unitario, Some(-2));
        assert_eq!(d.mapeo.total, Some(-1));
        assert_eq!(d.confianza, 1.0);
    }

    #[test]
    fn cantidad_compacta_nx() {
        let lineas = [
            "2x COCA $25.00 $50.00",
            "5x PAN $10.00 $50.00",
            "1x LECHE $22.50 $22.50",
        ];
        let d = detectar(&lineas);
        assert_eq!(d.mapeo.cantidad, Some(0));
        assert_eq!(d.confianza, 1.0);
    }

    #[test]
    fn dolar_separado_del_numero_se_preprocesa() {
        let lineas = [
            "2 COCA $ 25.00 $ 50.00",
            "1 PAN $ 10.00 $ 10.00",
            "6 HUEVO $ 4.50 $ 27.00",
        ];
        let d = detectar(&lineas);
        assert_eq!(d.mapeo.cantidad, Some(0));
        assert_eq!(d.mapeo.total, Some(-1));
        assert_eq!(d.confianza, 1.0);
    }

    // ---------- Formatos alternativos ----------

    #[test]
    fn producto_primero_cantidad_en_medio() {
        // Incluye un producto de 2 palabras para exigir índices negativos.
        let lineas = [
            "COCA 2 25.00 50.00",
            "PAN BIMBO 5 10.00 50.00",
            "LECHE 1 22.50 22.50",
        ];
        let d = detectar(&lineas);
        assert_eq!(d.mapeo.cantidad, Some(-3));
        assert_eq!(d.mapeo.precio_unitario, Some(-2));
        assert_eq!(d.mapeo.total, Some(-1));
        assert_eq!(d.confianza, 1.0);
    }

    #[test]
    fn codigo_de_barras_como_primera_columna() {
        let lineas = [
            "7501034567890 1 COCA 25.00 25.00",
            "7501034567891 2 PAN 10.00 20.00",
            "7501034567892 3 LECHE 22.50 67.50",
        ];
        let d = detectar(&lineas);
        assert_eq!(d.mapeo.cantidad, Some(1));
        assert_eq!(d.confianza, 1.0);
    }

    #[test]
    fn columna_de_descuento_numerico() {
        // CANT PROD PRECIO DESCUENTO TOTAL
        let lineas = [
            "2 COCA 25.00 2.00 48.00",
            "5 PAN 10.00 5.00 45.00",
            "1 LECHE 22.50 0.00 22.50",
            "3 JABON 15.00 3.00 42.00",
        ];
        let d = detectar(&lineas);
        assert_eq!(d.mapeo.total, Some(-1));
        assert!(d.confianza >= 0.5);
    }

    #[test]
    fn descuento_porcentual() {
        let lineas = [
            "2 Rockaleta $6.00 10% $10.80",
            "1 Heineken 473ml $28.00 - $28.00",
            "1 Tocino 200g $48.00 - $48.00",
            "4 Gomitas $5.00 10% $18.00",
        ];
        let d = detectar(&lineas);
        assert_eq!(d.mapeo.cantidad, Some(0));
        assert_eq!(d.mapeo.precio_unitario, Some(-3));
        assert_eq!(d.mapeo.total, Some(-1));
        assert_eq!(d.confianza, 1.0);
    }

    // ---------- Robustez ----------

    #[test]
    fn lineas_basura_no_tumban_la_deteccion() {
        let lineas = [
            "GRACIAS POR SU COMPRA",
            "2 COCA 25.00 50.00",
            "------------------------",
            "5 PAN 10.00 50.00",
            "TOTAL $100.00",
            "1 LECHE 22.50 22.50",
            "3 JABON 15.00 45.00",
        ];
        let d = detectar(&lineas);
        assert_eq!(d.confianza, 1.0);
        assert_eq!(d.lineas_evaluadas, 4);
    }

    #[test]
    fn nombre_largo_no_cuela_letra_fuera_del_producto() {
        // Si el detector eligiera producto=[1,1] para "FANTA NARANJA 600ML",
        // la regla de cobertura lo tira: quedan letras sin explicar.
        let lineas = [
            "2 FANTA NARANJA 600ML 15.00 30.00",
            "5 PAN BLANCO GRANDE 10.00 50.00",
            "1 LECHE ENTERA 22.50 22.50",
        ];
        let d = detectar(&lineas);
        assert_eq!(d.mapeo.producto, Some(vec![1, -3]));
        assert_eq!(d.confianza, 1.0);
    }

    #[test]
    fn linea_atipica_baja_la_confianza_no_el_resultado() {
        let lineas = [
            "2 COCA 25.00 50.00",
            "5 PAN 10.00 50.00",
            "1 LECHE 22.50 22.50",
            "3 JABON 15.00 45.00",
            "PROMO DEL DIA HOY 99.99 88.88", // no cuadra: baja confianza
        ];
        let d = detectar(&lineas);
        assert_eq!(d.lineas_validas, 4);
        assert!(d.confianza < 1.0);
        assert!(d.confianza >= 0.79);
    }

    #[test]
    fn mapeo_detectado_parsea_cada_linea_sin_alucinaciones() {
        let lineas = [
            "2 COCA 25.00 50.00",
            "7 PAN 10.00 70.00",
            "3 LECHE 22.50 67.50",
            "4 JABON 15.00 60.00",
        ];
        let d = detectar(&lineas);
        for linea in &lineas {
            let item = crate::cerebro::analizador_tickets::parsear_linea(
                linea,
                &d.mapeo,
                linea.split_whitespace().count(),
            )
            .expect("cada línea debe parsear con el mapeo detectado");
            let esperado = item.cantidad * item.precio_unitario;
            assert!(
                (esperado - item.total).abs() <= 0.06,
                "{linea} no cuadró tras el mapeo: {item:?}"
            );
        }
        assert_eq!(d.lineas_validas, 4);
    }

    #[test]
    fn formato_de_un_solo_importe_no_se_detecta() {
        let lineas = ["2 COCA 50.00", "5 PAN 50.00", "1 LECHE 22.50"];
        assert!(detectar_mapeo(&lineas).is_none());
    }

    #[test]
    fn texto_libre_sin_estructura_no_se_detecta() {
        let lineas = ["hoy fue un buen dia", "manana compro mas", "el proveedor no vino"];
        assert!(detectar_mapeo(&lineas).is_none());
    }

    #[test]
    fn muestra_demasiado_chica_no_detecta() {
        assert!(detectar_mapeo(&["2 COCA 25.00 50.00", "1 PAN 10.00 10.00"]).is_none());
        assert!(detectar_mapeo(&[]).is_none());
    }
}
