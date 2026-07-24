export function formatMXN(monto: number): string {
  return new Intl.NumberFormat('es-MX', { 
    style: 'currency', 
    currency: 'MXN', 
    minimumFractionDigits: 2 
  }).format(monto);
}

export function formatPct(valor: number): string {
  return `${valor.toFixed(2)}%`;
}

export function getFechaHoy(): string {
  return new Date().toISOString().split('T')[0];
}

export function getFechaInicioPeriodo(periodo: string, fechaFin: string): string {
  const fin = new Date(fechaFin);
  switch (periodo) {
    case 'semana': fin.setDate(fin.getDate() - 7); break;
    case 'mes': fin.setMonth(fin.getMonth() - 1); break;
    case 'trimestre': fin.setMonth(fin.getMonth() - 3); break;
    case 'año': fin.setFullYear(fin.getFullYear() - 1); break;
  }
  return fin.toISOString().split('T')[0];
}

export function formatFechaCorta(fecha: string): string {
  return new Date(fecha).toLocaleDateString('es-MX', { 
    weekday: 'short', day: 'numeric', month: 'short' 
  });
}

export function formatFechaCompleta(fecha: string): string {
  return new Date(fecha).toLocaleDateString('es-MX', { 
    year: 'numeric', month: 'long', day: 'numeric' 
  });
}

export function getColorSeveridad(severidad: 'rojo' | 'amarillo' | 'verde'): string {
  const colores: Record<string, string> = {
    rojo: 'bg-red-500',
    amarillo: 'bg-neutral-500',
    verde: 'bg-neutral-800',
  };
  return colores[severidad] || 'bg-gray-500';
}

export function getIconoSeveridad(severidad: 'rojo' | 'amarillo' | 'verde'): string {
  const iconos: Record<string, string> = {
    rojo: '🔴',
    amarillo: '🟡',
    verde: '⚪',
  };
  return iconos[severidad] || '⚪';
}

export const COLORES_CATEGORIAS = [
  '#000000', '#171717', '#262626', '#404040', '#525252',
  '#737373', '#a3a3a3', '#d4d4d4', '#e5e5e5', '#ef4444',
];

export const COLORES_SEVERIDAD: Record<'rojo' | 'amarillo' | 'verde', { 
  bg: string; text: string; border: string; icon: string 
}> = {
  rojo: { bg: 'bg-red-500', text: 'text-white', border: 'border-red-200', icon: '🔴' },
  amarillo: { bg: 'bg-neutral-500', text: 'text-white', border: 'border-neutral-200', icon: '🟡' },
  verde: { bg: 'bg-neutral-800', text: 'text-white', border: 'border-neutral-600', icon: '⚪' },
};

export const COLORES_ESTADO_PAGO: Record<string, { bg: string; text: string; dot: string }> = {
  pendiente: { bg: 'bg-neutral-50', text: 'text-neutral-700', dot: 'bg-neutral-500' },
  pagado: { bg: 'bg-neutral-50', text: 'text-neutral-700', dot: 'bg-neutral-500' },
  proximo_vencer: { bg: 'bg-neutral-100', text: 'text-neutral-800', dot: 'bg-neutral-600' },
  vencido: { bg: 'bg-red-50', text: 'text-red-700', dot: 'bg-red-500' },
};

export const COLORES_TIPO_CORTE: Record<string, { bg: string; text: string }> = {
  X: { bg: 'bg-neutral-100', text: 'text-neutral-700' },
  Z: { bg: 'bg-neutral-800', text: 'text-white' },
};