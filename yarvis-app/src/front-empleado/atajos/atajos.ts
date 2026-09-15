// ═══════════════════════════════════════════════════════════════════════════
// ATAJOS · REGISTRO — Atajos de teclado del módulo de empleado.
// Viven a nivel shell (EmployeeDashboard), así funcionan desde CUALQUIER
// pestaña: nueva_venta, inventario, tickets, perfil, etc. Por algo son
// atajos. Mapa actual: F3 → corte de caja (modal X/Z, #17), F5 → cobrar
// (#19), F4 → pago directo al proveedor (#18). F2/F6/F8 (issues #16,
// #20, #21) se cuelgan aquí mismo cuando existan.
//
// Dos mecanismos:
//   · Bloqueo: una pestaña bloquea un atajo mientras su propio modal está
//     abierto (ej: cobro con F5), para no encimar modales ni cambiar de
//     tab perdiendo un borrador.
//   · Acciones pendientes: la tecla puede llegar antes de que la pestaña
//     destino monte (cambio de tab). La acción queda guardada y la
//     pestaña la consume al montar; si ya estaba montada, le llega por
//     suscripción. Quien la atiende la consume (una sola vez).
// ═══════════════════════════════════════════════════════════════════════════

/** Tecla del corte de caja (issue #17). */
export const TECLA_CORTE = "F3";
/** Tecla de cobranza (issue #19). */
export const TECLA_COBRAR = "F5";
/** Tecla de pago directo al proveedor (issue #18). */
export const TECLA_PAGAR = "F4";
/** Tecla de reimpresión de ticket por número (issue #16). */
export const TECLA_REIMPRIMIR = "F2";
/** Tecla del menú de ayuda de atajos (issue #21). */
export const TECLA_AYUDA = "F8";
/** Tecla para ir al buscador de venta listo para escribir. */
export const TECLA_BUSCAR = "F7";

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

/** Acciones que viajan del shell a la pestaña destino. */
export type AccionAtajo = "cobrar" | "pagar-proveedor" | "buscar";

type OyenteAccion = (a: AccionAtajo) => void;

const oyentes = new Set<OyenteAccion>();
let pendiente: AccionAtajo | null = null;

/** Pide una acción: avisa a los montados y la deja pendiente para quien monte. */
export function solicitarAccion(a: AccionAtajo): void {
  pendiente = a;
  oyentes.forEach((o) => o(a));
}

/** Suscribe a la pestaña a las acciones (devolver el cleanup al desmontar). */
export function suscribirAccion(o: OyenteAccion): () => void {
  oyentes.add(o);
  return () => {
    oyentes.delete(o);
  };
}

/** Toma la acción pendiente (una sola vez); null si no hay. Con
    `esperada`, solo la toma si coincide (no roba acciones ajenas). */
export function consumirAccion(esperada?: AccionAtajo): AccionAtajo | null {
  if (esperada !== undefined && pendiente !== esperada) return null;
  const p = pendiente;
  pendiente = null;
  return p;
}
