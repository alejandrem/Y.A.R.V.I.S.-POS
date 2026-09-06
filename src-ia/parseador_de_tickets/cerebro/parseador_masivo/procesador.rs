// ============================================================
// procesador — Orquestación del lote: stream por canal y modo síncrono.
// ============================================================
//
// Organización:
//   * stream.rs  → `procesar_archivos` (por archivo, con canal)
//   * carpeta.rs → `procesar_carpeta_impl` (agrega estadísticas totales)
//
// Las rutas `parseador_masivo::procesador::X` no cambian.

mod carpeta;
mod stream;

pub use carpeta::procesar_carpeta_impl;
pub use stream::procesar_archivos;
