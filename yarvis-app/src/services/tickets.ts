// ═══════════════════════════════════════════════════════════════════════════
// SERVICIO DE TICKETS — Lectura paginada del historial de ventas.
// ═══════════════════════════════════════════════════════════════════════════

import { invokeTauri } from "./tauri";

export interface TicketDb {
  id: number;
  folio_ticket: string | null;
  fecha: string;
  total: number;
  metodo_pago: string;
}

export const PAGE_SIZE_TICKETS = 100;

export const obtenerTickets = (limit = PAGE_SIZE_TICKETS, offset = 0) =>
  invokeTauri<TicketDb[]>("get_tickets", { limit, offset });

export const obtenerTotalTickets = () =>
  invokeTauri<number>("get_tickets_total");
