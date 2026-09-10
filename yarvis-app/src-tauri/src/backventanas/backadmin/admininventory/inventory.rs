// ============================================================
// inventory — Comandos Tauri de inventario (CRUD, catálogos, semántica).
// ============================================================
//
// Organización (un tema por archivo):
//   * catalogo.rs   → huella SHA256 e historial de catálogos importados
//   * crud.rs       → altas, bajas y cambios de productos
//   * importar.rs   → importación masiva de catálogos (todo-o-nada)
//   * semantica.rs  → búsqueda por similitud y backfill de embeddings
//   * historial.rs  → lecturas del historial para la UI
//
// Los códigos de barras viven en el módulo compartido
// `backventanas::codigos_barras` (no es backadmin ni backempleado).
// Aquí se re-exporta el comando para no romper la ruta
// `admininventory::inventory::get_product_by_barcode`.
//
// Las rutas `admininventory::inventory::X` no cambian: este archivo
// re-exporta todo igual que antes.

mod catalogo;
mod crud;
mod historial;
mod importar;
mod semantica;

pub use catalogo::CatalogoImportado;
// Glob a propósito: `#[tauri::command]` genera items ocultos (`__cmd__*`)
// junto a cada función y solo el glob los re-exporta. Las rutas
// `admininventory::inventory::X` quedan intactas.
pub use crate::backventanas::codigos_barras::get_product_by_barcode;
pub use crud::*;
pub use historial::*;
pub use importar::*;
pub use semantica::*;
