// ============================================================
// parseador_masivo — Procesamiento masivo de carpetas de tickets
// .txt con transacción propia por archivo. Port de lote.py.
//
//   * archivos.rs    → descubrimiento y lectura de archivos .txt
//   * items.rs       → sumas y redondeo sobre items parseados
//   * almacen.rs     → escritura en SQLite (venta + detalle + stock)
//   * resumen.rs     → modelos de resultado/estadísticas
//   * procesador.rs  → orquestación (stream por canal y modo síncrono)
//   * tests.rs       → suite de integración (DB temporal por test)
//
// Sin HTTP: expone funciones puras consumibles desde Tauri.
// ============================================================

pub(crate) mod almacen;
mod archivos;
mod items;
mod procesador;
mod resumen;
#[cfg(test)]
mod tests;

pub use archivos::{obtener_archivos_txt, ordenar_archivos_cronologicamente};
pub use procesador::{procesar_archivos, procesar_carpeta_impl};
pub use resumen::{
    ArchivoResultado, EstadisticasCarpeta, ProductoNuevo, ResumenVenta, TicketFallido,
};

// Imports que los tests del módulo usan vía `use super::*`.
#[cfg(test)]
use crate::cerebro::analizador_tickets::MapeoColumnas;
#[cfg(test)]
use rusqlite::{params, Connection};
#[cfg(test)]
use std::path::Path;
