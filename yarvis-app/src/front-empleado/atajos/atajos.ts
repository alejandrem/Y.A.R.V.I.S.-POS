// ═══════════════════════════════════════════════════════════════════════════
// ATAJOS · REGISTRO — Atajos de teclado del módulo de empleado.
// Viven a nivel shell (EmployeeDashboard), así funcionan desde CUALQUIER
// pestaña: nueva_venta, inventario, tickets, perfil, etc. Por algo son
// atajos. Mapa actual: F3 → corte de caja (modal X/Z). F2/F4/F5/F6/F8
// (issues #16, #18, #19, #20, #21) se cuelgan aquí mismo cuando existan.
//
// Bloqueo: una pestaña puede bloquear un atajo mientras su propio modal
// está abierto (ej: cobro con F5), para no encimar modales.
// ═══════════════════════════════════════════════════════════════════════════

/** Tecla del corte de caja (issue #17). */
export const TECLA_CORTE = "F3";

const bloqueos = new Map<string, boolean>();

/** Bloquea/desbloquea un atajo (llamar en cleanup para liberar). */
export function fijarBloqueoAtajo(tecla: string, bloqueado: boolean): void {
  if (bloqueado) bloqueos.set(tecla, true);
  else bloqueos.delete(tecla);
}

/** ¿El atajo está bloqueado por algún modal local? */
export function atajoBloqueado(tecla: string): boolean {
  return bloqueos.get(tecla) ?? false;
}
