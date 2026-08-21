export interface ColumnMapping {
  cantidad: number | null;
  producto: number[] | null;
  precio_unitario: number | null;
  total: number | null;
  descuento: number | null;
}

export interface LLMAnalysis {
  status: string;
  mapeo: {
    formato_detectado: string;
    columnas: ColumnMapping;
    delimitador: string;
    moneda: string;
    total_columnas: number;
    tiene_descuento: boolean;
    tiene_iva: boolean;
  };
  fecha_ticket: string | null;
  hora_ticket: string | null;
  ejemplo_parseado: any[];
  confianza: number;
  notas: string;
}

// ── FINANZAS ──────────────────────────────────────────────────────────────

export interface GastoRecurrente {
  id: number;
  nombre: string;
  tipo: string;
  categoria: string;
  monto_proyectado: number;
  monto_real: number;
  frecuencia: string;
  dia_pago: number | null;
  intervalo_dias: number | null;
  fecha_inicio: string;
  fecha_fin: string | null;
  estado_pago: string;
  proxima_fecha_pago: string | null;
  dias_para_vencer: number | null;
  folio_comprobante: string | null;
  notas: string | null;
  creado_en: string;
}

export interface CrearGastoRequest {
  nombre: string;
  tipo: string;
  categoria: string;
  monto_proyectado: number;
  frecuencia: string;
  dia_pago: number | null;
  intervalo_dias: number | null;
  fecha_inicio: string;
  fecha_fin: string | null;
  folio_comprobante: string | null;
  notas: string | null;
}

export interface PagoGasto {
  id: number;
  gasto_id: number;
  fecha_pago: string;
  monto_pagado: number;
  metodo_pago: string | null;
  folio_comprobante: string | null;
  notas: string | null;
  creado_en: string;
}

export interface RegistrarPagoRequest {
  gasto_id: number;
  fecha_pago: string;
  monto_pagado: number;
  metodo_pago: string;
  folio_comprobante: string | null;
  notas: string | null;
}

export interface CorteCaja {
  id: number;
  fecha_apertura: string;
  fecha_cierre: string | null;
  monto_inicial: number;
  total_ventas: number;
  total_efectivo: number;
  total_tarjeta: number;
  total_transferencia: number;
  entradas_manuales: number;
  retiros_manuales: number;
  diferencia: number;
  usuario_id: number;
  usuario_nombre: string | null;
  estado: string;
  tipo_corte: string;
  turno: string | null;
  observaciones: string | null;
}

export interface MovimientoCaja {
  id: number;
  corte_id: number;
  tipo: string;
  concepto: string;
  monto: number;
  metodo_pago: string | null;
  creado_en: string;
}

export interface MetricasUtilidad {
  fecha: string;
  ventas_totales: number;
  costo_ventas: number;
  utilidad_bruta: number;
  gastos_operativos: number;
  utilidad_operativa: number;
  impuestos_comisiones: number;
  utilidad_neta: number;
  margen_neto_pct: number;
}

export interface ResumenPeriodo {
  periodo_inicio: string;
  periodo_fin: string;
  total_ventas: number;
  total_costo_ventas: number;
  total_utilidad_bruta: number;
  total_gastos_operativos: number;
  total_utilidad_operativa: number;
  total_impuestos_comisiones: number;
  total_utilidad_neta: number;
  margen_promedio_pct: number;
  punto_equilibrio_ventas: number;
}

export interface DatoGraficaPL {
  fecha: string;
  ingresos: number;
  gastos: number;
  utilidad_neta: number;
}

export interface DatoGraficaGastosCategoria {
  categoria: string;
  monto: number;
  porcentaje: number;
}

export interface DatoGraficaCortesZ {
  fecha: string;
  turno: string;
  total_ventas: number;
  diferencia: number;
  cajero: string;
}

export interface PuntoEquilibrio {
  gastos_fijos_mensuales: number;
  margen_contribucion_pct: number;
  ventas_necesarias: number;
  tickets_promedio: number;
  tickets_necesarios: number;
}

export interface AlertaFinanciera {
  id: number;
  tipo: string;
  severidad: string;
  titulo: string;
  mensaje: string;
  entidad_id: number | null;
  entidad_tipo: string | null;
  fecha_vencimiento: string | null;
  leida: number;
  creada_en: string;
}

export interface FiltrosCortes {
  cajero_id: number | null;
  fecha_inicio: string | null;
  fecha_fin: string | null;
  turno: string | null;
  tipo_corte: string | null;
  estado: string | null;
}

