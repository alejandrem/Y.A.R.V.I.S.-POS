// ============================================================
// parseador_de_cortes — Parseo determinista de cortes de caja X/Z.
//
// Misma filosofía que el parseador de tickets: 100% reglas, sin IA.
// El TIPO (X/Z) se detecta por el título `*** CORTE X|Z ...`; cada
// sección (`**Ingresos**`, `VENTAS DEL CORTE`, `Ventas por artículo`,
// `Ventas por ticket`...) se extrae por marcadores y los números se
// VERIFICAN con matemática exacta en centavos:
//
//   total_caja    == total_ingresos − total_egresos
//   total_ventas  == Σ subtotales (artículos o tickets)
//
// Los datasets mezclan tickets y cortes en la misma carpeta: usen
// `clasificar_archivo` para enrutar (los tickets no son cortes).
// ============================================================

pub mod fechas;
pub mod montos;
pub mod parser;
pub mod secciones;
pub mod tipos;

// Datasets reales de ejemplo (solo tests).
#[cfg(test)]
mod fixtures;

pub use parser::parse_corte;
pub use secciones::clasificar as clasificar_archivo;
pub use tipos::{ClaseArchivo, CorteParseado, ItemCorte, TipoCorte, Verificacion};
