// ═══════════════════════════════════════════════════════════════════════════
// SERVICIO MIS TICKETS — Lecturas operator-scoped del empleado actual.
// El backend resuelve el cajero por session.user_id: aquí nunca se manda
// nombre ni id (evita suplantar a otro empleado).
// ═══════════════════════════════════════════════════════════════════════════

import { invokeTauri } from "./tauri";

export interface MiTicket {
  id: number;
  folio_ticket: string | null;
  fecha: string;
  total: number;
  metodo_pago: string;
}

export interface MisKpis {
  total: number;
  tickets: number;
  ticket_promedio: number;
}

export interface VentaDia {
  fecha: string; // YYYY-MM-DD
  total: number;
}

export interface MiTicketItem {
  producto_nombre: string;
  cantidad: number;
  precio_unitario: number;
  subtotal: number;
}

export interface MiTicketDetalle {
  id: number;
  folio_ticket: string | null;
  fecha: string;
  total: number;
  subtotal: number;
  descuento: number;
  metodo_pago: string;
  items: MiTicketItem[];
}

export const obtenerMisTickets = (dias = 1, limit = 100, offset = 0) =>
  invokeTauri<MiTicket[]>("get_mis_tickets", { limit, offset, dias });

export const obtenerMisKpis = (dias = 1) =>
  invokeTauri<MisKpis>("get_mis_kpis", { dias });

export const obtenerMisVentasPorDia = (dias = 7) =>
  invokeTauri<VentaDia[]>("get_mis_ventas_por_dia", { dias });

export const obtenerMiTicketDetalle = (ventaId: number) =>
  // Tauri matchea args por nombre exacto: el comando espera `venta_id`.
  invokeTauri<MiTicketDetalle>("get_mi_ticket_detalle", { venta_id: ventaId });
