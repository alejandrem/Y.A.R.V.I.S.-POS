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

/** Nombres típicos de impresoras virtuales (PDF/XPS/OneNote/Fax): aceptan
 * el trabajo RAW pero no imprimen nada (Print to PDF guarda un .pdf vacío
 * de 0 bytes). El POS manda ESC/POS de térmica: solo sirve una física. */
const NOMBRES_VIRTUALES = ["print to pdf", "xps", "onenote", "fax", " pdf"];

/** true si el nombre parece impresora virtual (no térmica). */
export function esImpresoraVirtual(nombre: string): boolean {
  const n = nombre.trim().toLowerCase();
  return NOMBRES_VIRTUALES.some((v) => n.includes(v));
}

/** Frena en seco con mensaje de tendero si el destino es virtual.
 * Se llama en cada servicio de impresión/cajón: un solo lugar cubre
 * ticket, listas, facturas y F6. */
export function exigirImpresoraFisica(nombre: string): void {
  if (esImpresoraVirtual(nombre)) {
    throw new Error(
      `"${nombre.trim()}" es una impresora virtual: no imprime tickets. Elige tu térmica (80/58mm) en el selector.`,
    );
  }
}

/** Manda bytes ESC/POS ya armados a una impresora (lo usará Fase 2). */
export async function imprimirBytesRaw(
  nombreImpresora: string,
  bytes: number[],
): Promise<string> {
  exigirImpresoraFisica(nombreImpresora);
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
  /** Descuento en pesos de esta línea (monto, no %). Opcional = 0. */
  descuento?: number | null;
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
  /** Descuento global en pesos (fuera de las líneas). Opcional = 0. */
  descuento_global?: number | null;
  /** Ancho de papel en mm (80/58). Opcional = 80mm. */
  ancho_mm?: number | null;
  total: number;
  pagos: PagoPrint[];
  qr?: string | null;
}

/** Anchos de papel soportados (térmica 80mm/48cols o 58mm/32cols). */
export const ANCHOS_PAPEL = [
  { mm: 80, label: "80mm" },
  { mm: 58, label: "58mm" },
] as const;

/** Chequeo TCP sin gastar papel (típico 9100). */
export async function probarRed(ip: string, puerto: number): Promise<string> {
  return invoke<string>("probar_red", { ip, puerto });
}

/** La que usa el modal de venta: ticket `escpos` con QR al destino elegido. */
export async function imprimirTicketVenta(
  destino: DestinoPrint,
  ticket: TicketVentaPrint,
): Promise<string> {
  if ("Spooler" in destino) exigirImpresoraFisica(destino.Spooler.nombre);
  return invoke<string>("imprimir_ticket_venta", { destino, ticket });
}

/** Pulso de apertura de cajón (ESC p) por spooler o red. */
export async function abrirCajon(destino: DestinoPrint): Promise<string> {
  if ("Spooler" in destino) exigirImpresoraFisica(destino.Spooler.nombre);
  return invoke<string>("abrir_cajon", { destino });
}

/** Abre el cajón con la impresora predeterminada del spooler (F6).
 * El cajón va conectado a la térmica: el pulso sale por esa vía.
 * Salta las virtuales (PDF/XPS): mandarles RAW solo genera archivos
 * vacíos. Sin física instalada lanza error en lenguaje de tendero. */
export async function abrirCajonPredeterminado(): Promise<string> {
  const lista = await listarImpresoras();
  const fisicas = lista.filter((i) => !esImpresoraVirtual(i.nombre));
  const def = fisicas.find((i) => i.predeterminada) ?? fisicas[0];
  if (!def) throw new Error("Solo hay impresoras virtuales (PDF/XPS): conecta tu térmica 80/58mm");
  return abrirCajon({ Spooler: { nombre: def.nombre } });
}

/** La que usa el botón "Imprimir Lista": arma el ticket en Rust y lo manda RAW. */
export async function imprimirListaConciliacion(
  nombreImpresora: string,
  filas: FilaConciliacionPrint[],
  tienda?: string,
  anchoMm?: number | null,
): Promise<string> {
  exigirImpresoraFisica(nombreImpresora);
  return invoke<string>("imprimir_lista_conciliacion", {
    nombreImpresora,
    tienda: tienda ?? null,
    filas,
    ancho_mm: anchoMm ?? null,
  });
}

export interface FilaStockBajoPrint {
  nombre: string;
  stock: number;
  minimo: number;
}

/** La que usa el botón "Imprimir" de la Alerta de Stock Bajo. */
export async function imprimirListaStockBajo(
  nombreImpresora: string,
  filas: FilaStockBajoPrint[],
  tienda?: string,
  anchoMm?: number | null,
): Promise<string> {
  exigirImpresoraFisica(nombreImpresora);
  return invoke<string>("imprimir_lista_stock_bajo", {
    nombreImpresora,
    tienda: tienda ?? null,
    filas,
    ancho_mm: anchoMm ?? null,
  });
}
