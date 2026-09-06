// Parser TXT/Visual en Rust.
// Catálogo, mapeo, carpetas y procesamiento masivo: todo sin LLM. La
// estructura de columnas la detecta el detector estadístico
// (`src_ia::cerebro::analizador_tickets::detectar_mapeo`).
//
// Organización (un tema por archivo):
//   * archivos.rs  → exploración de carpetas y lectura cruda
//   * catalogo.rs  → parseo de catálogos visuales
//   * deteccion.rs → detección estadística del mapeo (sin IA)
//   * lote.rs      → parseo con mapeo: uno, carpeta o streaming
//
// Las rutas `adminparser::parser_txt::X` no cambian: este archivo
// re-exporta todo igual que antes.

mod archivos;
mod catalogo;
mod deteccion;
mod lote;

// Glob a propósito: `#[tauri::command]` genera items ocultos (`__cmd__*`)
// junto a cada función y solo el glob los re-exporta.
pub use archivos::*;
pub use catalogo::*;
pub use deteccion::*;
pub use lote::*;
