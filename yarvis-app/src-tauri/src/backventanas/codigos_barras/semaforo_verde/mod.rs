// ============================================================
// semaforo_verde — Auto-asignacion 100% segura (issue #10).
// Re-exports: la ruta canonica es
// `codigos_barras::semaforo_verde::*`.
// ============================================================

pub mod ean;
pub mod presentacion;
pub mod verde;

// Glob a proposito: `#[tauri::command]` genera items ocultos (`__cmd__*`)
// junto a cada funcion y solo el glob los re-exporta (mismo patron que
// `admininventory::inventory`). La ruta `semaforo_verde::verde_*` queda
// intacta para `lib.rs`.
pub use ean::validar_ean;
pub use presentacion::{
    extraer_presentacion, misma_presentacion, normalizar_unidad, presentacion_de_catalogo,
    quitar_presentacion,
};
pub use verde::*;
