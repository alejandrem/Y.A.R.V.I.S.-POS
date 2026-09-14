// ═══════════════════════════════════════════════════════════════════════════
// SERVICIO DE PROVEEDORES — Única fuente de verdad para alta, compras,
// sugerencia de pago e historial. Los componentes consumen estas
// funciones en lugar de invocar `invoke` directamente.
// ═══════════════════════════════════════════════════════════════════════════

import { invokeTauri } from "./tauri";

export interface Proveedor {
  id: number;
  nombre: string;
  telefono: string | null;
  correo: string | null;
  total_compras: number;
  total_pagado: number;
}

export interface SugerenciaPago {
  sugerido: number | null;
  precio_costo: number;
}

export interface ItemCompraEnvio {
  producto_id?: number | null;
  nombre: string;
  presentacion: "unidad" | "paquete";
  cantidad: number;
  piezasPorPaquete?: number | null;
  paquetes?: number | null;
}

export interface CompraRegistrada {
  compra_id: number;
  sugerido: number;
  pagado: number;
  movimiento_id: number | null;
  movimiento_pendiente: boolean;
}

export interface CompraRow {
  id: number;
  proveedor: string;
  fecha: string;
  pagado: number;
  sugerido: number;
  metodo_pago: string;
  items: number;
  movimiento_pendiente: boolean;
  rectifica_a: number | null;
  rectificada: boolean;
}

export interface ItemCompraRow {
  nombre: string;
  presentacion: string;
  cantidad: number;
  precio_sugerido: number;
  producto_id: number | null;
  piezas_por_paquete: number | null;
  paquetes: number | null;
}

export interface CompraDetalle {
  id: number;
  proveedor: string;
  fecha: string;
  pagado: number;
  sugerido: number;
  metodo_pago: string;
  comentario: string | null;
  movimiento_id: number | null;
  rectifica_a: number | null;
  rectificada_por: number[];
  items: ItemCompraRow[];
}

export const guardarProveedor = (nombre: string, telefono?: string | null, correo?: string | null) =>
  invokeTauri<number>("guardar_proveedor", { nombre, telefono: telefono ?? null, correo: correo ?? null });

export const crearProveedorGenerico = () =>
  invokeTauri<Proveedor>("crear_proveedor_generico");

export const listarProveedores = () =>
  invokeTauri<Proveedor[]>("listar_proveedores");

export const sugerirPago = (nombreProducto: string, cantidad: number) =>
  invokeTauri<SugerenciaPago>("sugerir_pago", { nombreProducto, cantidad });

export const registrarCompra = (args: {
  proveedorId: number;
  items: ItemCompraEnvio[];
  montoPagado: number;
  metodoPago?: string;
  comentario?: string | null;
}) => invokeTauri<CompraRegistrada>("registrar_compra", args);

export const rectificarCompra = (args: {
  compraOriginalId: number;
  items: ItemCompraEnvio[];
  montoPagado: number;
  metodoPago?: string;
  comentario?: string | null;
}) => invokeTauri<CompraRegistrada>("rectificar_compra", args);

export const historialCompras = (proveedorId?: number | null, limit = 100, offset = 0) =>
  invokeTauri<CompraRow[]>("historial_compras", { proveedorId: proveedorId ?? null, limit, offset });

export const obtenerCompraDetalle = (compraId: number) =>
  invokeTauri<CompraDetalle>("get_compra_detalle", { compraId });
