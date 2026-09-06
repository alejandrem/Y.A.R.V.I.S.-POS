// ============================================================
// cortes — Cortes X y Z de caja: apertura, cierre y movimientos.
// ============================================================
//
// Organización (un tema por archivo):
//   * lecturas.rs    → lista de cortes + detalle completo
//   * creacion.rs    → apertura de cortes X y Z
//   * cierre.rs      → cierre con recálculo antisabotaje del servidor
//   * movimientos.rs → entradas/retiros manuales + cortes por cajero
//
// Las rutas `adminfinanzas::cortes::X` no cambian: este archivo
// re-exporta todo igual que antes.

mod cierre;
mod creacion;
mod lecturas;
mod movimientos;

// Glob a propósito: `#[tauri::command]` genera items ocultos (`__cmd__*`)
// junto a cada función y solo el glob los re-exporta.
pub use cierre::*;
pub use creacion::*;
pub use lecturas::*;
pub use movimientos::*;
