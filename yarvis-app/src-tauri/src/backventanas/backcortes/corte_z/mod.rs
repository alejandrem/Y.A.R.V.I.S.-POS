// ============================================================
// corte_z — Cierre definitivo del turno del empleado.
//
// REGLA: el corte Z se hace al cerrar turno (el empleado se va a
// su casa). REINICIA el conteo: todo lo vendido después de su
// fecha_cierre pertenece al siguiente turno (reinicio LÓGICO,
// ver `comun::ancla_turno`). Se GUARDA en `cortes_caja` como tipo
// 'Z' cerrado, para reimprimirse cuando se desee.
//
// Lleva TODOS los productos vendidos del periodo (nombre +
// cantidad + monto, concordando con detalle_ventas y tickets),
// más los tickets y los totales por método.
// ============================================================

mod cierre;

pub use cierre::*;
