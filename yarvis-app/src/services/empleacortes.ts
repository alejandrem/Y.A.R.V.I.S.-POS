// ═══════════════════════════════════════════════════════════════════════════
// SERVICIO EMPLEACORTES — Comandos del backend backcortes que usa el
// empleado (botón CORTE / F3 en NUEVA VENTA). Impresión directa a la
// térmica por backend, sin diálogos del sistema.
// ═══════════════════════════════════════════════════════════════════════════

import { invoke } from "@tauri-apps/api/core";
import { invokeTauri } from "./tauri";
import type { DestinoPrint } from "./impresora";
import type { CorteXReporte, CorteZReporte } from "../front-empleado/ventanas/empleacortes/tipos";

/** Foto del turno: se puede pedir N veces, no afecta nada. */
export async function pedirCorteX(observaciones: string | null = null): Promise<CorteXReporte> {
  return invokeTauri<CorteXReporte>("corte_x_reporte", { observaciones });
}

/** Cierre definitivo del turno: reinicia el conteo y estampa la salida. */
export async function pedirCorteZ(observaciones: string | null = null): Promise<CorteZReporte> {
  return invokeTauri<CorteZReporte>("corte_z_cierre", { observaciones });
}

/** Manda un corte ya guardado a la térmica (spooler o red). */
export async function imprimirCorte(
  destino: DestinoPrint,
  corteId: number,
  anchoMm?: number | null,
): Promise<string> {
  // Tauri matchea args por nombre exacto: el comando espera `corte_id`.
  return invoke<string>("imprimir_corte", { destino, corte_id: corteId, ancho_mm: anchoMm ?? null });
}
