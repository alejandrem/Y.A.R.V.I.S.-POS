// ============================================================
// backcortes — TODA la lógica de cortes de caja X/Z en un solo
// lugar (compartido admin + empleado).
//
// * comun.rs       → ancla de turno + consultas por ventana
// * apertura.rs    → apertura manual de cortes (flujo admin legacy)
// * cierre.rs      → cierre genérico por id con recálculo antisabotaje
// * lecturas.rs    → historial + detalle de cortes
// * movimientos.rs → entradas/retiros manuales + cortes por cajero
// * corte_x/       → reporte X: snapshot libre, no afecta al turno
// * corte_z/       → cierre Z: cierre definitivo del turno del empleado
//
// NOTA SQL: no se necesitó migración nueva. `cortes_caja` +
// `movimientos_caja` + `ventas`/`detalle_ventas` (filtro por
// `cajero_id` + ventana de fechas) + `asistencias` (primer_login)
// ya cubren X y Z. `ventas` queda append-only: el "reinicio a 0"
// del Z es LÓGICO (la próxima ventana parte de su fecha_cierre),
// no se borra ni se reasigna nada.
// ============================================================

pub mod apertura;
pub mod cierre;
pub mod comun;
pub mod corte_x;
pub mod corte_z;
pub mod impresion;
pub mod lecturas;
pub mod movimientos;
