// ============================================================
// corte_x — Reporte X del empleado.
//
// REGLA: el corte X se puede hacer cuando quiera, las veces que
// quiera, y NO afecta en nada al turno. Es solo una foto de lo
// vendido desde el ancla (último Z / primer login) hasta ahora.
//
// Se GUARDA en `cortes_caja` como tipo 'X' ya cerrado, para que
// pueda reimprimirse cuando se desee. Lleva los tickets del
// periodo (folio + total por ticket).
// ============================================================

mod snapshot;

pub use snapshot::*;
