export type FrecuenciaGasto = 'semanal' | 'quincenal' | 'mensual' | 'trimestral' | 'personalizado';
export type TipoGasto = 'luz' | 'agua' | 'renta' | 'internet' | 'nomina' | 'mantenimiento' | 'proveedores' | 'impuestos' | 'varios';
export type CategoriaGasto = 'servicios' | 'operativo' | 'nomina' | 'impuestos' | 'varios';
export type EstadoPago = 'pendiente' | 'pagado' | 'proximo_vencer' | 'vencido';
export type TipoCorte = 'X' | 'Z';
export type TurnoCorte = 'matutino' | 'vespertino' | 'nocturno';
export type SeveridadAlerta = 'verde' | 'amarillo' | 'rojo';
export type EstadoCorte = 'abierto' | 'cerrado' | 'parcial';

export interface GastoRecurrente {
  id: number;
  nombre: string;
  tipo: TipoGasto;
  categoria: CategoriaGasto;
  monto_proyectado: number;
  monto_real: number;
  frecuencia: FrecuenciaGasto;
  dia_pago: number | null;
  intervalo_dias: number | null;
  fecha_inicio: string;
  fecha_fin: string | null;
  estado_pago: EstadoPago;
  folio_comprobante: string | null;
  comprobante_url: string | null;
  notas: string | null;
  creado_en: string;
  actualizado_en: string;
  proxima_fecha_pago: string | null;
  dias_para_vencer: number | null;
}

export interface CrearGastoRequest {
  nombre: string;
  tipo: TipoGasto;
  categoria: CategoriaGasto;
  monto_proyectado: number;
  frecuencia: FrecuenciaGasto;
  dia_pago?: number;
  intervalo_dias?: number;
  fecha_inicio: string;
  fecha_fin?: string;
  folio_comprobante?: string;
  notas?: string;
}

export interface PagoGasto {
  id: number;
  gasto_id: number;
  fecha_pago: string;
  monto_pagado: number;
  metodo_pago: string | null;
  folio_comprobante: string | null;
  comprobante_url: string | null;
  notas: string | null;
  creado_en: string;
}

export interface RegistrarPagoRequest {
  gasto_id: number;
  fecha_pago: string;
  monto_pagado: number;
  metodo_pago?: string;
  folio_comprobante?: string;
  notas?: string;
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
  usuario_nombre: string;
  estado: EstadoCorte;
  tipo_corte: TipoCorte;
  turno: TurnoCorte | null;
  observaciones: string | null;
}

export interface MovimientoCaja {
  id: number;
  corte_id: number;
  tipo: 'entrada' | 'retiro' | 'venta' | 'devolucion';
  concepto: string;
  monto: number;
  metodo_pago: string | null;
  referencia_id: number | null;
  creado_en: string;
}

export interface CrearCorteRequest {
  monto_inicial: number;
  tipo_corte: TipoCorte;
  turno?: TurnoCorte;
  observaciones?: string;
}

export interface CerrarCorteRequest {
  corte_id: number;
  total_ventas: number;
  total_efectivo: number;
  total_tarjeta: number;
  total_transferencia: number;
  entradas_manuales: number;
  retiros_manuales: number;
}

export interface CorteDetalle {
  corte: CorteCaja;
  movimientos: MovimientoCaja[];
  ventas_por_metodo: [string, number, number][];
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
  severidad: SeveridadAlerta;
  titulo: string;
  mensaje: string;
  entidad_id: number | null;
  entidad_tipo: string | null;
  fecha_vencimiento: string | null;
  leida: boolean;
  creada_en: string;
}

export interface FiltrosCortes {
  cajero_id?: number;
  fecha_inicio?: string;
  fecha_fin?: string;
  turno?: TurnoCorte;
  tipo_corte?: TipoCorte;
  estado?: EstadoCorte;
}

export interface FiltrosPeriodo {
  fecha_inicio: string;
  fecha_fin: string;
}

export interface TabFinanzas {
  id: 'dashboard' | 'gastos' | 'cortes' | 'graficas' | 'alertas';
  label: string;
  icon: React.ReactNode;
}

export const TIPOS_GASTO: { value: TipoGasto; label: string; icon: string }[] = [
  { value: 'luz', label: 'Luz / Electricidad', icon: '⚡' },
  { value: 'agua', label: 'Agua', icon: '💧' },
  { value: 'renta', label: 'Renta / Alquiler', icon: '🏠' },
  { value: 'internet', label: 'Internet / Teléfono', icon: '🌐' },
  { value: 'nomina', label: 'Nómina', icon: '👥' },
  { value: 'mantenimiento', label: 'Mantenimiento', icon: '🔧' },
  { value: 'proveedores', label: 'Proveedores', icon: '📦' },
  { value: 'impuestos', label: 'Impuestos', icon: '📋' },
  { value: 'varios', label: 'Varios', icon: '📝' },
];

export const CATEGORIAS_GASTO: { value: CategoriaGasto; label: string }[] = [
  { value: 'servicios', label: 'Servicios Básicos' },
  { value: 'operativo', label: 'Gastos Operativos' },
  { value: 'nomina', label: 'Nómina' },
  { value: 'impuestos', label: 'Impuestos' },
  { value: 'varios', label: 'Varios' },
];

export const FRECUENCIAS_GASTO: { value: FrecuenciaGasto; label: string; config: 'dia_semana' | 'dia_mes' | 'intervalo' }[] = [
  { value: 'semanal', label: 'Semanal (cada 7 días)', config: 'dia_semana' },
  { value: 'quincenal', label: 'Quincenal (días 1 y 15)', config: 'dia_mes' },
  { value: 'mensual', label: 'Mensual (día específico)', config: 'dia_mes' },
  { value: 'trimestral', label: 'Trimestral (cada 3 meses)', config: 'dia_mes' },
  { value: 'personalizado', label: 'Personalizado (cada N días)', config: 'intervalo' },
];

export const DIAS_SEMANA = [
  { value: 0, label: 'Domingo' },
  { value: 1, label: 'Lunes' },
  { value: 2, label: 'Martes' },
  { value: 3, label: 'Miércoles' },
  { value: 4, label: 'Jueves' },
  { value: 5, label: 'Viernes' },
  { value: 6, label: 'Sábado' },
];

export const TURNOS_CORTE: { value: TurnoCorte; label: string }[] = [
  { value: 'matutino', label: 'Matutino' },
  { value: 'vespertino', label: 'Vespertino' },
  { value: 'nocturno', label: 'Nocturno' },
];

export const COLORES_CATEGORIAS = [
  '#ef4444', '#3b82f6', '#22c55e', '#f59e0b', '#8b5cf6',
  '#ec4899', '#06b6d4', '#84cc16', '#f97316', '#6366f1',
];

export const COLORES_SEVERIDAD: Record<SeveridadAlerta, { bg: string; text: string; border: string; icon: string }> = {
  rojo: { bg: 'bg-red-500', text: 'text-white', border: 'border-red-200', icon: '🔴' },
  amarillo: { bg: 'bg-amber-500', text: 'text-white', border: 'border-amber-200', icon: '🟡' },
  verde: { bg: 'bg-green-500', text: 'text-white', border: 'border-green-200', icon: '🟢' },
};

export const COLORES_ESTADO_PAGO: Record<EstadoPago, { bg: string; text: string; dot: string }> = {
  pendiente: { bg: 'bg-amber-50', text: 'text-amber-700', dot: 'bg-amber-500' },
  pagado: { bg: 'bg-green-50', text: 'text-green-700', dot: 'bg-green-500' },
  proximo_vencer: { bg: 'bg-orange-50', text: 'text-orange-700', dot: 'bg-orange-500' },
  vencido: { bg: 'bg-red-50', text: 'text-red-700', dot: 'bg-red-500' },
};

export const COLORES_TIPO_CORTE: Record<TipoCorte, { bg: string; text: string }> = {
  X: { bg: 'bg-blue-50', text: 'text-blue-700' },
  Z: { bg: 'bg-purple-50', text: 'text-purple-700' },
};