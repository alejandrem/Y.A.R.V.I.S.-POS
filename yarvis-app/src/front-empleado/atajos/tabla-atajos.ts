// ═══════════════════════════════════════════════════════════════════════════
// ATAJOS · TABLA ÚNICA — Fuente de verdad de los atajos de caja.
// Topbar, menú F8 y ejecución salen de aquí: imposible desincronizarlos.
// `enTopbar` = sale arriba; `enMenu` = sale en la rejilla F8;
// `listo` = funciona (si no, se ve apagado como "próximamente").
// ═══════════════════════════════════════════════════════════════════════════

import type { IconInput } from "morphicons/react";
import {
  ICONO_BILLETE, ICONO_CAJA, ICONO_BUSCAR, ICONO_AYUDA, ICONO_DOCUMENTO,
} from "../../components/ui";

export type AccionAtajoMenu = "reimprimir" | "corte" | "pagar" | "cobrar" | "buscar" | "menu";

export interface EntradaAtajo {
  tecla: string;
  /** Etiqueta corta del cuadrito F8. */
  etiqueta: string;
  /** Texto del topbar (si sale ahí). */
  labelTopbar: string;
  descripcion: string;
  icono: IconInput;
  accion: AccionAtajoMenu | null;
  listo: boolean;
  enTopbar: boolean;
  enMenu: boolean;
}

export const TABLA_ATAJOS: EntradaAtajo[] = [
  { tecla: "F2", etiqueta: "TICKET", labelTopbar: "Ticket", descripcion: "Reimprimir ticket por número", icono: ICONO_DOCUMENTO, accion: "reimprimir", listo: true, enTopbar: false, enMenu: true },
  { tecla: "F3", etiqueta: "CORTE", labelTopbar: "Corte", descripcion: "Corte de caja X / Z", icono: ICONO_CAJA, accion: "corte", listo: true, enTopbar: true, enMenu: true },
  { tecla: "F4", etiqueta: "PAGAR", labelTopbar: "Pagar", descripcion: "Pago directo al proveedor", icono: ICONO_BILLETE, accion: "pagar", listo: true, enTopbar: false, enMenu: true },
  { tecla: "F5", etiqueta: "COBRAR", labelTopbar: "Cobrar", descripcion: "Cobrar la venta", icono: ICONO_BILLETE, accion: "cobrar", listo: true, enTopbar: true, enMenu: true },
  { tecla: "F6", etiqueta: "CAJÓN", labelTopbar: "Caja", descripcion: "Abrir el cajón (próximamente)", icono: ICONO_CAJA, accion: null, listo: false, enTopbar: true, enMenu: true },
  { tecla: "F7", etiqueta: "BUSCAR", labelTopbar: "Buscar", descripcion: "Ir al buscador de venta", icono: ICONO_BUSCAR, accion: "buscar", listo: true, enTopbar: true, enMenu: true },
  { tecla: "F8", etiqueta: "ATAJOS", labelTopbar: "Atajos", descripcion: "Este menú", icono: ICONO_AYUDA, accion: "menu", listo: true, enTopbar: true, enMenu: true },
];
