// ═══════════════════════════════════════════════════════════════════════════
// SERVICIO DE VENTA — Cobro del operador y búsqueda por similitud.
// ═══════════════════════════════════════════════════════════════════════════

import { invokeTauri } from "./tauri";

export interface ItemVenta {
  id: number | null;
  nombre: string;
  precio_venta: number;
  cantidad: number;
}

export type VentaRequest = {
  items: ItemVenta[];
  total: number;
  subtotal: number;
  descuento: number;
  monto_efectivo: number;
  monto_tarjeta: number;
  monto_transferencia: number;
  cliente_id: number | null;
};

export interface VentaResponse {
  venta_id: number;
  ticket_number: number;
}

export interface SimilarHit {
  id: number;
  contenido: string;
  score: number;
}

export const completarVenta = (venta: VentaRequest) =>
  invokeTauri<VentaResponse>("completar_venta", { venta });

export const buscarProductoSimilar = (query: string, topK = 5) =>
  invokeTauri<SimilarHit[]>("buscar_producto_similar", { query, topK });
