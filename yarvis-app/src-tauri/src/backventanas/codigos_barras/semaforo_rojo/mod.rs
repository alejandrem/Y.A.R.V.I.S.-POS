// ============================================================
// semaforo_rojo — Captura manual que alimenta el aprendizaje.
// (issue #12). Re-exports: `codigos_barras::semaforo_rojo::*`.
// ============================================================

pub mod comandos;
pub mod rojo;

// Glob a proposito: `#[tauri::command]` genera items ocultos (`__cmd__*`)
// junto a cada funcion y solo el glob los re-exporta (mismo patron que
// `admininventory::inventory`). La ruta `semaforo_rojo::rojo_*` queda
// intacta para `lib.rs`.
pub use comandos::*;
pub use rojo::*;
