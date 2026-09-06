// ============================================================
// carpeta — Modo síncrono: agrega los resultados por archivo en
// estadísticas totales de la carpeta.
// ============================================================

use std::collections::HashSet;

use super::super::resumen::{ArchivoResultado, EstadisticasCarpeta, TicketFallido};
use super::stream::procesar_archivos;
use crate::cerebro::analizador_tickets::MapeoColumnas;

pub fn procesar_carpeta_impl(
    archivos: Vec<String>,
    mapeo: MapeoColumnas,
    db_path: String,
) -> EstadisticasCarpeta {
    let mut stats = EstadisticasCarpeta {
        total_archivos: archivos.len(),
        ..Default::default()
    };

    let mut nombres_nuevos_vistos: HashSet<String> = HashSet::new();
    let (tx, rx) = std::sync::mpsc::channel::<ArchivoResultado>();

    procesar_archivos(&archivos, &mapeo, &db_path, &tx);
    drop(tx);

    for res in rx {
        stats.procesados += 1;
        if res.ok {
            stats.exitosos += 1;
            stats.ventas_creadas += res.ventas;
            stats.ventas_omitidas += res.ventas_omitidas;
            stats.archivos_formato_distinto += res.formato_distinto as usize;
            stats.items_insertados += res.items;
            stats.duplicados_detectados += res.duplicados;
            stats.productos_existentes += res.existentes;
            stats.productos_nuevos += res.nuevos.len();
            for nuevo in res.nuevos {
                if nombres_nuevos_vistos.insert(nuevo.nombre.clone()) {
                    stats.productos_nuevos_lista.push(nuevo);
                }
            }
            stats.resumen_ventas.extend(res.ventas_info);
        } else {
            stats.errores += 1;
            stats.tickets_fallidos.push(TicketFallido {
                archivo: res.archivo,
                motivo: res.motivo,
            });
        }
    }

    stats.productos_nuevos_lista.truncate(100);
    stats.tickets_fallidos.truncate(500);
    stats
}
