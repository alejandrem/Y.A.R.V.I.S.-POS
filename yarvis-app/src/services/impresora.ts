// ═══════════════════════════════════════════════════════════════════════════
// SERVICIO DE IMPRESORA — Camino A Fase 1 (spooler Windows RAW).
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
