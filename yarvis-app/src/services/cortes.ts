// ═══════════════════════════════════════════════════════════════════════════
// SERVICIO DE CORTES IMPORTADOS — Parseo histórico X/Z.
// El backend clasifica, importa con idempotencia por hash y expone
// historial + detalle con verificación recalculada.
// ═══════════════════════════════════════════════════════════════════════════

import { invokeTauri } from "./tauri";

export interface ResumenCortes {
  archivos: number;
  cortes_x: number;
  cortes_z: number;
  omitidos_no_corte: number;
  omitidos_duplicados: number;
  errores: string[];
  productos_vinculados: number;
  productos_nuevos: number;
}

export interface CorteImportadoRow {
  id: number;
  tipo: string;
  folio: string | null;
  estacion: string | null;
  cajero: string;
  fecha: string | null;
  total_caja: number;
  total_ventas: number;
  clientes_atendidos: number;
  verificado: boolean;
}

export interface ItemImportado {
  kind: string;
  nombre: string;
  cantidad: number | null;
  precio_unitario: number;
  subtotal: number;
}

export interface CorteImportadoDetalle {
  id: number;
  tipo: string;
  folio: string | null;
  estacion: string | null;
  cajero: string;
  empresa: string | null;
  moneda: string;
  fecha: string | null;
  total_ingresos: number;
  total_egresos: number;
  total_caja: number;
  total_ventas: number;
  ventas_gravadas: number;
  impuesto: number;
  ventas_no_gravadas: number;
  redondeos: number;
  ventas_credito: number;
  total_unidades: number;
  clientes_atendidos: number;
  caja_ok: boolean;
  ventas_ok: boolean;
  items: ItemImportado[];
}

export const PAGE_SIZE_CORTES = 100;

export const importarCarpetaCortes = (carpeta: string) =>
  invokeTauri<ResumenCortes>("importar_carpeta_cortes", { carpeta });

export const obtenerCortesImportados = (limit = PAGE_SIZE_CORTES, offset = 0) =>
  invokeTauri<CorteImportadoRow[]>("get_cortes_importados", { limit, offset });

export const obtenerCorteImportadoDetalle = (corteId: number) =>
  // Tauri matchea args por nombre exacto: el comando espera `corte_id`.
  invokeTauri<CorteImportadoDetalle>("get_corte_importado_detalle", { corte_id: corteId });
