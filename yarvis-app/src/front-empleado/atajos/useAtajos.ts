// ═══════════════════════════════════════════════════════════════════════════
// ATAJOS · HOOK — Listener global de atajos del empleado.
// Se monta UNA vez en el shell (EmployeeDashboard), no por pestaña.
// F3 abre el modal de corte aunque estés en perfil, tickets, etc.
// ═══════════════════════════════════════════════════════════════════════════

import { useEffect } from "react";
import { TECLA_CORTE, atajoBloqueado } from "./atajos";

interface AtajosEmpleadosOpts {
  onCorte: () => void;
  /** El shell lo pone en true mientras el propio modal de corte está abierto. */
  deshabilitado?: boolean;
}

export function useAtajosEmpleados({ onCorte, deshabilitado = false }: AtajosEmpleadosOpts): void {
  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (deshabilitado) return;
      if (e.key === TECLA_CORTE) {
        if (atajoBloqueado(TECLA_CORTE)) return;
        // F3 en el navegador abre "buscar": se suprime para usarlo de caja.
        e.preventDefault();
        onCorte();
      }
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [onCorte, deshabilitado]);
}
