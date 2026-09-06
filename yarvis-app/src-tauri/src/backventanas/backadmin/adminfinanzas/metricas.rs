// ============================================================
// metricas — KPIs financieros: ventas, ganancias y punto de equilibrio.
// ============================================================
//
// Organización (un tema por archivo):
//   * componentes.rs → piezas de cálculo (fuente única de utilidad neta)
//   * diarias.rs     → métricas día por día
//   * resumen.rs     → resumen del periodo + caché materializada
//   * equilibrio.rs  → punto de equilibrio (break-even)
//   * tests.rs       → suite de métricas
//
// Las rutas `adminfinanzas::metricas::X` no cambian: este archivo
// re-exporta todo igual que antes (alertas.rs usa dos helpers).

mod componentes;
mod diarias;
mod equilibrio;
mod resumen;
#[cfg(test)]
mod tests;

pub(crate) use componentes::calcular_utilidad_neta_periodo;
pub(crate) use resumen::sincronizar_resumen_mes_actual;
// Glob a propósito: `#[tauri::command]` genera items ocultos (`__cmd__*`)
// junto a cada función y solo el glob los re-exporta.
pub use diarias::*;
pub use equilibrio::*;
pub use resumen::*;
