// ============================================================
// parseador_de_cortes — Parseo determinista de cortes de caja X/Z.
//
// Misma filosofía que el parseador de tickets: 100% reglas, sin IA.
// El TIPO (X/Z) se detecta por el título; cada sección se extrae por
// marcadores y los números se VERIFICAN con matemática exacta en
// centavos: caja == ingresos − egresos, ventas == Σ renglones.
//
// Los datasets mezclan tickets y cortes en la misma carpeta: usen
// `clasificar_archivo` para enrutar (los tickets no son cortes).
//
// Estructura por tareas (1 archivo = 1 tarea, loners en la raíz):
//   tipos.rs            contrato con el backend (loner)
//   parser.rs           orquesta la extracción (loner)
//   valores/            limpieza de primitivas impresas
//   secciones/          encabezado, partición, totales y renglones
// ============================================================

pub mod parser;
pub mod secciones;
pub mod tipos;
pub mod valores;

pub use parser::parse_corte;
pub use secciones::clasificar as clasificar_archivo;
pub use tipos::{ClaseArchivo, CorteParseado, ItemCorte, TipoCorte, Verificacion};
