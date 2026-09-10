// ═══════════════════════════════════════════════════════════════════════════
// SERVICIO DE IMPRESORA — Fase 1 (spooler Windows RAW) + Fase 2 (`escpos`).
// Única fuente de verdad para los comandos de impresión térmica.
// Los componentes consumen estas funciones, nunca `invoke` crudo.
// ═══════════════════════════════════════════════════════════════════════════

import { invoke } from "@tauri-apps/api/core";

export interface ImpresoraInfo {
  nombre: string;
  predeterminada: boolean;
}

export interface FilaConciliacionPrint {
  nombre: string;
  fisico: number;
  sistema: number;
  precio_venta: number;
}

/** Impresoras instaladas vistas por el spooler de Windows. */
export async function listarImpresoras(): Promise<ImpresoraInfo[]> {
  return invoke<ImpresoraInfo[]>("listar_impresoras");
}

/** Manda bytes ESC/POS ya armados a una impresora (lo usará Fase 2). */
export async function imprimirBytesRaw(
  nombreImpresora: string,
  bytes: number[],
): Promise<string> {
  return invoke<string>("imprimir_bytes_raw", { nombreImpresora, bytes });
}

/** Destino Fase 2: spooler local (Fase 1) o térmica en red (TCP 9100). */
export type DestinoPrint =
  | { Spooler: { nombre: string } }
  | { Red: { ip: string; puerto: number } };

export interface LineaVentaPrint {
  nombre: string;
  cantidad: number;
  precio_unitario: number;
}

export interface PagoPrint {
  metodo: string;
  monto: number;
}

export interface TicketVentaPrint {
  tienda: string;
  ubicacion?: string | null;
  folio: string;
  fecha?: string | null;
  lineas: LineaVentaPrint[];
  total: number;
  pagos: PagoPrint[];
  qr?: string | null;
}

/** Chequeo TCP sin gastar papel (típico 9100). */
export async function probarRed(ip: string, puerto: number): Promise<string> {
  return invoke<string>("probar_red", { ip, puerto });
}

/** La que usa el modal de venta: ticket `escpos` con QR al destino elegido. */
export async function imprimirTicketVenta(
  destino: DestinoPrint,
  ticket: TicketVentaPrint,
): Promise<string> {
  return invoke<string>("imprimir_ticket_venta", { destino, ticket });
}

/** La que usa el botón "Imprimir Lista": arma el ticket 80mm en Rust y lo manda RAW. */
export async function imprimirListaConciliacion(
  nombreImpresora: string,
  filas: FilaConciliacionPrint[],
  tienda?: string,
): Promise<string> {
  return invoke<string>("imprimir_lista_conciliacion", {
    nombreImpresora,
    tienda: tienda ?? null,
    filas,
  });
}
