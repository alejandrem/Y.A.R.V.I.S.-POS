// ═══════════════════════════════════════════════════════════════════════════
// TURNO — SHIM de compatibilidad (DEPRECADO).
// Toda la lógica de la barra de asistencia y horas extra vive en
// ./turno-extra/ (tipos, geometría, MiniBarraDia, BarraTurnoNormal,
// BarraExtra). Este archivo solo re-exporta para no romper los
// importadores viejos (admin + tests). Código nuevo: importar de
// ./turno-extra.
// ═══════════════════════════════════════════════════════════════════════════

export * from "./turno-extra/index";
