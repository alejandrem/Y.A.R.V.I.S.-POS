//! lector_txt — Port de `yarvis-IA/parseador_de_tickets/formatos/lector_txt.py`
//!
//! Parser de catálogos en formato visual (texto plano .txt):
//!   - Producto -- $VENTA $COSTO
//!   - Producto - $VENTA - $COSTO
//!   - Producto = $VENTA $COSTO
//!   - Producto VENTA COSTO (sin $)
//!   - Múltiples productos por línea con |
//!   - Cantidad al inicio: 10Producto $10 $5
//!   - Formato de tabla: Producto  CANT  $VTA  $CST (sin separador)
//!
//! Organización (un tema por archivo):
//!   * patrones.rs → regex del lector (+ nota del lookbehind→clase negada)
//!   * linea.rs    → parseo de UNA línea (7 patrones en orden)
//!   * visual.rs   → catálogo completo (detección CSV vs visual)
//!   * tests.rs    → suite del lector (espejo de test_lector_txt.py)

mod linea;
mod patrones;
#[cfg(test)]
mod tests;
mod visual;

pub use linea::parsear_linea_catalogo;
pub use visual::parsear_catalogo_visual;
