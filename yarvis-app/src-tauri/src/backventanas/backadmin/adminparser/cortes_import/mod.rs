// Cortes importados (parseo histórico X/Z): lectura, vinculación al
// catálogo, importación e historial. Cada archivo una tarea;
// `cortes_caja` no se toca (es solo para operativos en vivo).
pub mod historial;
pub mod importacion;
pub mod lectura;
pub mod vinculacion;
