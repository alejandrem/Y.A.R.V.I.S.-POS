use crate::cerebro::analizador_tickets::{Item, TotalesTicket};

pub(super) fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

/// Frontera pesos → centavos: la DB almacena el dinero en INTEGER centavos
/// desde la migración f64→centavos, pero TODO este módulo opera en pesos
/// (dominio del texto del ticket). Solo [`super::almacen`] debe llamar esto,
/// justo antes de parametrizar los INSERT. `.round()` absorbe el ruido de
/// punto flotante del parseo (ej. 10.80 * 100 = 1079.9999… → 1080).
pub(super) fn a_centavos(pesos: f64) -> i64 {
    (pesos * 100.0).round() as i64
}

/// Suma de totales de línea. Espejo de `sum(i.get("total",0) or cant*precio)`.
pub(super) fn calcular_subtotal(items: &[Item]) -> f64 {
    items
        .iter()
        .map(|i| {
            if i.total != 0.0 {
                i.total
            } else {
                i.cantidad * i.precio_unitario
            }
        })
        .sum()
}

/// Resuelve los totales que se guardan en la venta.
///
/// REGLA DEL NEGOCIO: los precios de los productos en los tickets ya vienen
/// CON IVA incluido (igual que en la compra masiva de catálogos). Por tanto:
///
///   * `subtotal_calc` = suma de totales de línea (ya con IVA).
///   * `total_calc`    = `subtotal_calc` (NO se multiplica × 1.16).
///   * `iva_calc`      = 0.0  (no se agrega IVA encima; ya está incluido).
///
/// Si el ticket imprime sus propios SUBTOTAL/IVA/TOTAL reales, se respetan
/// esos valores. Si el ticket trae TOTAL pero no SUBTOTAL/IVA, se asume que
/// el total ya incluye IVA y se guardan `subtotal` y `iva` en 0 para no
/// duplicar el impuesto.
///
/// Si el total real difiere >±0.5% del calculado, se prefiere el del ticket
/// y se loguea.
///
/// NOTA (migración centavos): esta función NO cambió. Su dominio son pesos
/// f64 (texto del ticket) y la tolerancia ±0.5% es relativa, así que sigue
/// siendo consistente; la conversión a centavos ocurre solo en `almacen.rs`
/// al momento de escribir en la DB.
pub(super) fn resolver_totales_venta(items: &[Item], reales: &TotalesTicket) -> (f64, f64, f64) {
    let subtotal_calc = calcular_subtotal(items);
    // Los precios ya incluyen IVA: el total = subtotal, sin sumar 16% extra.
    let total_calc = round2(subtotal_calc);
    let iva_calc = 0.0;

    let subtotal = reales.subtotal.unwrap_or(subtotal_calc);
    let iva = reales.iva.unwrap_or(iva_calc);
    let total = reales.total.unwrap_or(total_calc);

    if let Some(real_total) = reales.total {
        let dif = (real_total - total_calc).abs();
        let umbral = total_calc * 0.005;
        if real_total > 0.0 && dif > umbral {
            let porciento = if total_calc > 0.0 {
                dif / total_calc * 100.0
            } else {
                0.0
            };
            tracing::warn!(
                "[YARVIS] Total real del ticket ({real_total:.2}) difiere {porciento:.2}% del calculado ({total_calc:.2}); se usa el del ticket"
            );
        }
    }

    (round2(subtotal), round2(iva), round2(total))
}
