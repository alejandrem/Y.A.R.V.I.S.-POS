// ═══════════════════════════════════════════════════════════════════════════
// ATAJOS · HOOK — Listener global de atajos del empleado.
// Se monta UNA vez en el shell (EmployeeDashboard), no por pestaña.
// F3 abre el corte, F5 va a cobrar y F4 al pago directo, estés en la
// pestaña que estés. F5/F4/F3 se suprimen siempre (en un POS esas teclas
// son de la caja: un refresh del navegador vaciaría el carrito en memoria).
// ═══════════════════════════════════════════════════════════════════════════

import { useEffect } from "react";
import {
  TECLA_CORTE, TECLA_COBRAR, TECLA_PAGAR, TECLA_REIMPRIMIR, TECLA_AYUDA, TECLA_BUSCAR,
  atajoBloqueado,
} from "./atajos";

interface AtajosEmpleadosOpts {
  onCorte: () => void;
  onCobrar?: () => void;
  onPagar?: () => void;
  onReimprimir?: () => void;
  onAyuda?: () => void;
  onBuscar?: () => void;
  /** El shell lo pone en true mientras un modal de shell está abierto. */
  deshabilitado?: boolean;
}

export function useAtajosEmpleados({ onCorte, onCobrar, onPagar, onReimprimir, onAyuda, onBuscar, deshabilitado = false }: AtajosEmpleadosOpts): void {
  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (e.key !== TECLA_CORTE && e.key !== TECLA_COBRAR && e.key !== TECLA_PAGAR && e.key !== TECLA_REIMPRIMIR && e.key !== TECLA_AYUDA && e.key !== TECLA_BUSCAR) return;
      e.preventDefault();
      if (deshabilitado) return;
      if (e.key === TECLA_CORTE) {
        if (atajoBloqueado(TECLA_CORTE)) return;
        onCorte();
      } else if (e.key === TECLA_COBRAR) {
        if (atajoBloqueado(TECLA_COBRAR)) return;
        onCobrar?.();
      } else if (e.key === TECLA_PAGAR) {
        if (atajoBloqueado(TECLA_PAGAR)) return;
        onPagar?.();
      } else if (e.key === TECLA_REIMPRIMIR) {
        if (atajoBloqueado(TECLA_REIMPRIMIR)) return;
        onReimprimir?.();
      } else if (e.key === TECLA_AYUDA) {
        if (atajoBloqueado(TECLA_AYUDA)) return;
        onAyuda?.();
      } else if (e.key === TECLA_BUSCAR) {
        if (atajoBloqueado(TECLA_BUSCAR)) return;
        onBuscar?.();
      }
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [onCorte, onCobrar, onPagar, onReimprimir, onAyuda, onBuscar, deshabilitado]);
}
