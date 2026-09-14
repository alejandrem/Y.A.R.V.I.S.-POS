// ============================================================
// semaforo_amarillo — Sugerencia con confirmacion humana.
// (issue #11, el dificil). Re-exports con glob: `#[tauri::command]`
// genera items ocultos que solo el glob re-exporta (patron del repo).
// ============================================================

pub mod candidatos;
pub mod comandos;
pub mod confirmar;
pub mod fuzzy;
pub mod rechazos;
pub mod score;
pub mod sugerencias;
pub mod tipos;
pub mod umbral;

pub use candidatos::*;
pub use comandos::*;
pub use confirmar::*;
pub use rechazos::*;
pub use score::*;
pub use sugerencias::*;
pub use tipos::*;
pub use umbral::*;
