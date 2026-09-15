// ═══════════════════════════════════════════════════════════════════════════
// TURNO-EXTRA · ÍNDICE — Todo lo de la barra de asistencia y horas extra
// sale de aquí: tipos, geometría y las 2 pistas visuales.
// ═══════════════════════════════════════════════════════════════════════════

export type { BloqueHoy, MiTurno, BarraTurno, DiaExtra } from "./tipos";
export { minsDe, fmtHM, geometriaBarra, etiquetaEntrada } from "./geometria";
export { MiniBarraDia, geometriaMiniBarra, ESCALA_NOCTURNA_MIN } from "./MiniBarraDia";
export type { GeometriaMini } from "./MiniBarraDia";
export { BarraTurnoNormal } from "./BarraTurnoNormal";
export { BarraExtra } from "./BarraExtra";
