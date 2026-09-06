// ============================================================
// hipotesis — Ensayo de mapeos de columnas sobre UNA línea.
//
// Cada hipótesis dice "la cantidad está en la columna X, el precio en la
// Y, el total en la Z, el producto entre A y B" y `linea_cuadra` la pone
// a prueba contra la ecuación cantidad × precio − descuento ≈ total,
// exigiendo además COBERTURA total (toda columna explicada o ruido
// conocido). Cubre las familias A (cantidad primero) y B (producto
// primero); la C (un solo importe) vive en `familia_c.rs`.
// ============================================================

use super::super::esquema::resolver_indice;
use super::super::parser::{es_porcentaje, es_token_numero, limpiar_precio};

/// Mínimo de líneas que la hipótesis ganadora debe cuadrar.
pub(crate) const MIN_VALIDAS: usize = 3;
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
///                 Para la familia C (un solo importe) CUADRA significa
///                 "estructura sana"; la matemática la valida después la
///                   consistencia de precios entre productos repetidos.
/// - `Some(false)` → evaluable pero NO cuadra: en contra.
pub(crate) fn linea_cuadra(
    cols: &[String],
    cant_i: i32,
    precio_i: Option<i32>,
    total_i: i32,
    producto: &(i32, i32),
) -> Option<bool> {
    let n = cols.len();
    let ci = resolver_indice(Some(cant_i), n)?;
    let pi = resolver_indice(precio_i, n);
    let ti = resolver_indice(Some(total_i), n)?;
    if ci == ti {
        return None;
    }
    if let Some(pi) = pi {
        if ci == pi || pi == ti {
            return None;
        }
    }

    // Rango del producto
    let p_ini = resolver_indice(Some(producto.0), n)?;
    let p_fin = resolver_indice(Some(producto.1), n)?;
    if p_ini > p_fin {
        return None;
    }
    let rango_producto: Vec<usize> = (p_ini..=p_fin).collect();
    if rango_producto.contains(&ci)
        || rango_producto.contains(&ti)
        || pi.is_some_and(|pi| rango_producto.contains(&pi))
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
        if i == ci || i == ti || pi == Some(i) || rango_producto.contains(&i) {
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
    let total = if es_token_numero(&cols[ti]) {
        limpiar_precio(&cols[ti])
    } else {
        return None;
    };

    // Familia C: UN solo importe ("CANT PRODUCTO IMPORTE"). No hay precio
    // unitario que multiplicar: la línea cuadra si la estructura es sana y
    // la confianza REAL la da la consistencia de precios entre tickets.
    let Some(pi) = pi else {
        return Some(cantidad > 0.0 && cantidad <= CANTIDAD_MAXIMA && total > 0.0);
    };

    let precio = if es_token_numero(&cols[pi]) {
        limpiar_precio(&cols[pi])
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

/// Candidatos de cantidad por FAMILIA. En la familia A (cantidad primero)
/// convienen índices desde la izquierda (el anclaje típico); en la B
/// (producto primero) convienen NEGATIVOS para tolerar nombres largos:
/// "COCA 2 25 50" y "FANTA NARANJA 600ML 2 15 30" comparten cantidad=-3.
const CANDIDATOS_CANTIDAD_A: &[i32] = &[0, 1, 2, -3, -4];
const CANDIDATOS_CANTIDAD_B: &[i32] = &[-3, -4, -5, 1, 2, 3];
/// Familia C (un solo importe): "CANT PRODUCTO IMPORTE".
const CANDIDATOS_CANTIDAD_C: &[i32] = &[0, 1];

/// Genera todas las hipótesis viables, las TRES familias incluidas.
/// Devuelve (cant_i, precio_i, total_i, (prod_ini, prod_fin)); en la
/// familia C `precio_i` es `None` (solo hay un importe por línea).
pub(crate) fn hipotesis() -> Vec<(i32, Option<i32>, i32, (i32, i32))> {
    let mut out = Vec::new();
    for &total_i in CANDIDATOS_TOTAL {
        for &precio_i in CANDIDATOS_PRECIO {
            for &cant_i in CANDIDATOS_CANTIDAD_A {
                // Familia A: CANT PRODUCTO PRECIO TOTAL
                out.push((cant_i, Some(precio_i), total_i, (cant_i + 1, precio_i - 1)));
            }
            for &cant_i in CANDIDATOS_CANTIDAD_B {
                // Familia B: PRODUCTO CANT PRECIO TOTAL
                out.push((cant_i, Some(precio_i), total_i, (0, cant_i - 1)));
            }
        }
        // Familia C: CANT PRODUCTO IMPORTE (precio = total/cantidad).
        for &cant_i in CANDIDATOS_CANTIDAD_C {
            out.push((cant_i, None, total_i, (cant_i + 1, total_i - 1)));
        }
    }
    out
}
