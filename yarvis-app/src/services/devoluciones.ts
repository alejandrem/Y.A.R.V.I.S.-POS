// Consultas y devoluciones de ticket completo.
import { invokeTauri } from "./tauri";

export interface TicketDevolucion {
  id: number;
  folio_ticket: string | null;
  fecha: string;
  total: number;
  metodo_pago: string;
}

export interface ItemTicketDevolucion {
  producto_nombre: string;
  cantidad: number;
  precio_unitario: number;
  subtotal: number;
}

export interface DetalleTicketDevolucion extends TicketDevolucion {
  items: ItemTicketDevolucion[];
}

export interface ResultadoDevolucion {
  devolucion_id: number;
  venta_id: number;
  folio_ticket: string;
  monto: number;
  egreso_registrado: boolean;
}

export interface DevolucionRegistrada {
  id: number;
  venta_id: number;
  folio_ticket: string;
  fecha_ticket: string;
  creado_en: string;
  monto: number;
  empleado_nombre: string;
  estado_movimiento: "registrado" | "pendiente";
}

export const buscarTicketsDevolucion = (termino: string) =>
  invokeTauri<TicketDevolucion[]>("buscar_tickets_devolucion", { termino });

export const obtenerTicketDevolucion = (ventaId: number) =>
  invokeTauri<DetalleTicketDevolucion>("get_ticket_devolucion", { ventaId });

export const devolverTicketCompleto = (ventaId: number) =>
  invokeTauri<ResultadoDevolucion>("devolver_ticket_completo", { ventaId });

export const obtenerMisDevoluciones = () =>
  invokeTauri<DevolucionRegistrada[]>("get_mis_devoluciones");

export const obtenerDevolucionesEmpleado = (empleadoId: number) =>
  invokeTauri<DevolucionRegistrada[]>("get_devoluciones_empleado", { empleadoId });
