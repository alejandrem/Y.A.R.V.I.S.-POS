// Secciones del corte: encabezado, partición, totales y renglones.
// Cada archivo una tarea; el parser los orquesta.
pub mod encabezado;
pub mod marcadores;
pub mod renglones;
pub mod totales;

pub use encabezado::{
    clasificar, extraer_cajero, extraer_empresa, extraer_estacion_fecha, extraer_folio,
    extraer_moneda,
};
pub use marcadores::{partir, seccion, Marca};
pub use renglones::{linea_articulo, linea_pago, linea_ticket};
pub use totales::{extraer_totales, TotalesCorte};
