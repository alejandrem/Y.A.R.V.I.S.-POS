use crate::cerebro::analizador_tickets::{descuento_separado, parsear_linea, Item, MapeoColumnas};

pub(super) fn parsear_items_con_descuentos(
    lineas: &[String],
    mapeo: &MapeoColumnas,
    total_cols: usize,
) -> Vec<Item> {
    let mut items = Vec::new();
    for linea in lineas {
        if let Some(descuento) = descuento_separado(linea) {
            aplicar_descuento(&mut items, descuento);
        } else if let Some(mut item) = parsear_linea(linea, mapeo, total_cols) {
            normalizar_total_bruto(&mut item);
            items.push(item);
        }
    }
    items
}

fn aplicar_descuento(items: &mut [Item], descuento: f64) {
    let Some(item) = items.last_mut() else { return };
    let acumulado = item.descuento.unwrap_or(0.0) + descuento;
    item.descuento = Some((acumulado * 100.0).round() / 100.0);
}

fn normalizar_total_bruto(item: &mut Item) {
    let Some(descuento) = item.descuento else {
        return;
    };
    let neto = item.cantidad * item.precio_unitario - descuento;
    if (item.total - neto).abs() <= 0.02 {
        item.total = ((item.total + descuento) * 100.0).round() / 100.0;
    }
}

#[cfg(test)]
mod tests;
