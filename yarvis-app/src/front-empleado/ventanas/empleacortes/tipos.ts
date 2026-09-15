// ═══════════════════════════════════════════════════════════════════════════
// EMPLEACORTES · TIPOS — Contratos del backend backcortes (corte_x_reporte,
// corte_z_cierre). El empleado es el único que crea cortes X/Z; el admin
// solo los consulta desde Finanzas → Cortes.
// ═══════════════════════════════════════════════════════════════════════════

export interface TicketResumen {
  venta_id: number;
  folio: string;
  fecha: string;
  total: number;
  metodo_pago: string;
}

export interface TotalesVentana {
  total_ventas: number;
  total_efectivo: number;
  total_tarjeta: number;
  total_transferencia: number;
  num_tickets: number;
}

export interface ProductoAgregado {
  producto_nombre: string;
  cantidad: number;
  monto: number;
}

export interface CorteXReporte {
  corte_id: number;
  cajero_id: number;
  ancla: string;
  cierre: string;
  tickets: TicketResumen[];
  totales: TotalesVentana;
}

export interface CorteZReporte extends CorteXReporte {
  productos: ProductoAgregado[];
}

export type TipoCorte = "X" | "Z";
