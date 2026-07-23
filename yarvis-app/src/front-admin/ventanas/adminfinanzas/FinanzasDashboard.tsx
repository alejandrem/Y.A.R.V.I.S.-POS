import { useState, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { 
  useMetricas, useGraficas, useAlertas,
  type ResumenPeriodo, type PuntoEquilibrio, type DatoGraficaPL, type AlertaFinanciera,
  type FiltrosPeriodo
} from './hooks';
import { 
  formatMXN, formatPct, getFechaInicioPeriodo, getFechaHoy,
  COLORES_CATEGORIAS, COLORES_SEVERIDAD
} from './utils';
import {
  LineChart, Line, AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer,
  PieChart, Pie, Cell, BarChart, Bar
} from 'recharts';

export const FinanzasDashboard = () => {
  const [periodo, setPeriodo] = useState<'semana' | 'mes' | 'trimestre' | 'año'>('mes');
  const [fechaFin] = useState(getFechaHoy());
  const fechaInicio = getFechaInicioPeriodo(periodo, fechaFin);

  const { resumen, puntoEquilibrio, loading: loadingMetricas, cargarResumen } = useMetricas();
  const { datosPL, loading: loadingGraficas, cargarPL } = useGraficas();
  const { alertas, loading: loadingAlertas } = useAlertas();

  useEffect(() => {
    cargarResumen(fechaInicio, fechaFin);
    cargarPL(fechaInicio, fechaFin, periodo === 'semana' ? 'semana' : periodo === 'mes' ? 'mes' : 'mes');
  }, [fechaInicio, fechaFin, periodo, cargarResumen, cargarPL]);

  const alertasCriticas = useMemo(() => 
    alertas.filter(a => !a.leida && (a.severidad === 'rojo' || a.severidad === 'amarillo')).slice(0, 3),
  [alertas]);

  const kpis = useMemo(() => [
    {
      label: 'UTILIDAD BRUTA',
      value: resumen ? formatMXN(resumen.total_utilidad_bruta) : '$0.00',
      subtitle: 'Ventas - COGS',
      color: 'bg-blue-500',
      icon: '📈',
      trend: resumen && resumen.total_utilidad_bruta > 0 ? 'up' : 'down',
    },
    {
      label: 'UTILIDAD OPERATIVA',
      value: resumen ? formatMXN(resumen.total_utilidad_operativa) : '$0.00',
      subtitle: 'Bruta - Gastos Op.',
      color: 'bg-green-500',
      icon: '🏢',
      trend: resumen && resumen.total_utilidad_operativa > 0 ? 'up' : 'down',
    },
    {
      label: 'UTILIDAD NETA',
      value: resumen ? formatMXN(resumen.total_utilidad_neta) : '$0.00',
      subtitle: 'Operativa - Impuestos',
      color: 'bg-purple-500',
      icon: '💰',
      trend: resumen && resumen.total_utilidad_neta > 0 ? 'up' : 'down',
    },
    {
      label: 'MARGEN NETO %',
      value: resumen ? formatPct(resumen.margen_promedio_pct) : '0.00%',
      subtitle: '(Neta / Ventas) × 100',
      color: resumen && resumen.margen_promedio_pct > 15 ? 'bg-emerald-500' : 'bg-amber-500',
      icon: '📊',
      trend: resumen && resumen.margen_promedio_pct > 15 ? 'up' : 'down',
    },
  ], [resumen]);

  return (
    <div className="space-y-8 animate-in fade-in slide-in-from-bottom-2 duration-500">
      {/* Header con selector de período */}
      <div className="flex flex-col sm:flex-row sm:items-end sm:justify-between gap-4">
        <div>
          <h2 className="text-2xl font-black text-neutral-900 uppercase tracking-tight">Dashboard Financiero</h2>
          <p className="text-xs text-neutral-400 uppercase tracking-widest">Visión general del rendimiento del negocio</p>
        </div>
        <div className="flex flex-wrap gap-2">
          {['semana', 'mes', 'trimestre', 'año'].map(p => (
            <button
              key={p}
              onClick={() => setPeriodo(p as any)}
              className={`px-4 py-2 text-xs font-black uppercase tracking-widest rounded-xl transition-all ${
                periodo === p 
                  ? 'bg-neutral-900 text-white shadow-lg shadow-neutral-200' 
                  : 'bg-neutral-100 text-neutral-500 hover:bg-neutral-200 hover:text-neutral-700'
              }`}
            >
              {p.toUpperCase()}
            </button>
          ))}
        </div>
      </div>

      {/* KPIs Cards */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        {kpis.map((kpi, i) => (
          <KPICard key={i} {...kpi} />
        ))}
      </div>

      {/* Grid principal: Gráfica P&L + Punto Equilibrio + Alertas */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Gráfica P&L Evolution */}
        <div className="lg:col-span-2 bg-white rounded-[2.5rem] border border-neutral-200 shadow-sm overflow-hidden">
          <div className="p-6 border-b border-neutral-100">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-black text-neutral-900 uppercase tracking-tight">EVOLUCIÓN P&L (PÉRDIDAS Y GANANCIAS)</h3>
              <div className="flex gap-2">
                {['dia', 'semana', 'mes'].map(g => (
                  <button
                    key={g}
                    className={`px-3 py-1 text-[10px] font-black rounded-lg transition-all ${
                      (periodo === 'semana' && g === 'semana') || (periodo === 'mes' && g === 'mes') || (periodo === 'trimestre' && g === 'mes')
                        ? 'bg-neutral-900 text-white' 
                        : 'bg-neutral-100 text-neutral-500 hover:bg-neutral-200'
                    }`}
                  >
                    {g.toUpperCase()}
                  </button>
                ))}
              </div>
            </div>
            <div className="h-[350px]">
              <ResponsiveContainer width="100%" height="100%">
                <ComposedChart data={datosPL} margin={{ top: 10, right: 30, left: 0, bottom: 0 }}>
                  <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" vertical={false} />
                  <XAxis 
                    dataKey="fecha" 
                    tick={{ fontSize: 10, fill: '#9ca3af' }} 
                    axisLine={{ stroke: '#e5e7eb' }}
                    tickLine={false}
                  />
                  <YAxis 
                    tick={{ fontSize: 10, fill: '#9ca3af' }} 
                    axisLine={false}
                    tickLine={false}
                    tickFormatter={v => formatMXN(v).replace('$', '') + 'k'}
                  />
                  <Tooltip 
                    content={<CustomTooltip />}
                    formatter={v => [formatMXN(v), '']}
                    labelFormatter={fecha => new Date(fecha).toLocaleDateString('es-MX', { weekday: 'short', day: 'numeric', month: 'short' })}
                  />
                  <Legend layout="horizontal" align="center" verticalAlign="bottom" iconType="line" wrapperStyle={{ paddingTop: 10 }} />
                  <Area type="monotone" dataKey="ingresos" name="Ingresos Totales" stroke="#22c55e" fill="#22c55e" fillOpacity={0.1} strokeWidth={2} />
                  <Area type="monotone" dataKey="gastos" name="Gastos Totales" stroke="#ef4444" fill="#ef4444" fillOpacity={0.1} strokeWidth={2} />
                  <Line type="monotone" dataKey="utilidad_neta" name="Utilidad Neta" stroke="#8b5cf6" strokeWidth={3} dot={false} activeDot={{ r: 6 }} />
                </ComposedChart>
              </ResponsiveContainer>
            </div>
          </div>
        </div>

        {/* Panel derecho: Punto Equilibrio + Alertas */}
        <div className="space-y-6">
          {/* Punto de Equilibrio */}
          <div className="bg-white rounded-[2.5rem] border border-neutral-200 shadow-sm overflow-hidden p-6">
            <h3 className="text-lg font-black text-neutral-900 uppercase tracking-tight mb-4 flex items-center gap-2">
              <span className="w-8 h-8 bg-purple-100 text-purple-600 rounded-xl flex items-center justify-center text-sm">⚖️</span>
              PUNTO DE EQUILIBRIO
            </h3>
            {puntoEquilibrio ? (
              <div className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <MetricItem label="GASTOS FIJOS MENSUALES" value={formatMXN(puntoEquilibrio.gastos_fijos_mensuales)} />
                  <MetricItem label="MARGEN CONTRIBUCIÓN" value={formatPct(puntoEquilibrio.margen_contribucion_pct)} />
                  <MetricItem label="VENTAS NECESARIAS" value={formatMXN(puntoEquilibrio.ventas_necesarias)} />
                  <MetricItem label="TICKETS NECESARIOS" value={puntoEquilibrio.tickets_necesarios.toFixed(0)} />
                </div>
                <div className="pt-4 border-t border-neutral-100">
                  <div className="h-4 bg-neutral-100 rounded-full overflow-hidden">
                    const progreso = Math.min(100, (resumen?.total_ventas || 0) / (puntoEquilibrio.ventas_necesarias || 1) * 100);
                    <div className={`h-full rounded-full transition-all duration-500 ${progreso >= 100 ? 'bg-green-500' : progreso >= 70 ? 'bg-amber-500' : 'bg-red-500'}`} style={{ width: `${progreso}%` }}></div>
                  </div>
                  <p className="text-xs text-neutral-500 mt-1 text-center">
                    {progreso >= 100 ? '✅ Punto de equilibrio SUPERADO' : `Faltan ${formatMXN(puntoEquilibrio.ventas_necesarias - (resumen?.total_ventas || 0))} para alcanzar el break-even`}
                  </p>
                </div>
              </div>
            ) : (
              <div className="text-center py-8 text-neutral-400">Calculando...</div>
            )}
          </div>

          {/* Alertas Críticas */}
          <div className="bg-white rounded-[2.5rem] border border-neutral-200 shadow-sm overflow-hidden">
            <div className="p-4 border-b border-neutral-100 flex items-center justify-between">
              <h3 className="text-lg font-black text-neutral-900 uppercase tracking-tight flex items-center gap-2">
                <span className="w-8 h-8 bg-red-100 text-red-600 rounded-xl flex items-center justify-center text-sm">🚨</span>
                ALERTAS CRÍTICAS
              </h3>
              {alertasCriticas.length > 0 && (
                <span className="px-2 py-1 bg-red-500 text-white text-[10px] font-black rounded-lg">{alertasCriticas.length}</span>
              )}
            </div>
            <div className="p-4">
              {alertasCriticas.length === 0 ? (
                <div className="text-center py-8">
                  <div className="w-16 h-16 bg-green-50 rounded-full flex items-center justify-center mx-auto mb-3 text-3xl">✅</div>
                  <p className="text-sm font-medium text-neutral-600">Sin alertas críticas</p>
                  <p className="text-xs text-neutral-400">Todo bajo control</p>
                </div>
              ) : (
                <div className="space-y-3">
                  {alertasCriticas.map(alerta => (
                    <AlertaItem key={alerta.id} alerta={alerta} compact />
                  ))}
                </div>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* Resumen rápido: Ventas por método + Gastos por categoría */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div className="bg-white rounded-[2.5rem] border border-neutral-200 shadow-sm overflow-hidden p-6">
          <h3 className="text-lg font-black text-neutral-900 uppercase tracking-tight mb-4 flex items-center gap-2">
            <span className="w-8 h-8 bg-blue-100 text-blue-600 rounded-xl flex items-center justify-center text-sm">💳</span>
            VENTAS POR MÉTODO DE PAGO
          </h3>
          <VentasMetodoChart />
        </div>
        <div className="bg-white rounded-[2.5rem] border border-neutral-200 shadow-sm overflow-hidden p-6">
          <h3 className="text-lg font-black text-neutral-900 uppercase tracking-tight mb-4 flex items-center gap-2">
            <span className="w-8 h-8 bg-orange-100 text-orange-600 rounded-xl flex items-center justify-center text-sm">🍩</span>
            GASTOS POR CATEGORÍA
          </h3>
          <GastosCategoriaChart />
        </div>
      </div>
    </div>
  );
};

// Componentes auxiliares
const KPICard = ({ label, value, subtitle, color, icon, trend }: any) => (
  <div className="bg-white rounded-[2.5rem] border border-neutral-200 shadow-sm overflow-hidden p-6">
    <div className="flex items-start justify-between">
      <div>
        <p className="text-[10px] font-black text-neutral-400 uppercase tracking-widest mb-1">{label}</p>
        <p className="text-3xl font-black text-neutral-900">{value}</p>
        <p className="text-xs text-neutral-400 mt-1">{subtitle}</p>
      </div>
      <div className={`${color} w-14 h-14 rounded-2xl flex items-center justify-center text-2xl`}>{icon}</div>
    </div>
    <div className="mt-4 pt-4 border-t border-neutral-100 flex items-center justify-between">
      <span className={`text-[10px] font-black ${trend === 'up' ? 'text-green-600' : 'text-red-600'}`}>
        {trend === 'up' ? '↑ Positivo' : '↓ Revisar'}
      </span>
    </div>
  </div>
);

const MetricItem = ({ label, value }: any) => (
  <div>
    <p className="text-[10px] font-black text-neutral-400 uppercase tracking-widest">{label}</p>
    <p className="text-xl font-black text-neutral-900 mt-1">{value}</p>
  </div>
);

const AlertaItem = ({ alerta, compact }: any) => {
  const colores = COLORES_SEVERIDAD[alerta.severidad];
  return (
    <div className={`p-3 rounded-xl border-l-4 ${colores.border} ${colores.bg.replace('500', '50')} flex items-start gap-3`}>
      <span className="text-lg">{colores.icon}</span>
      <div className="flex-1 min-w-0">
        <p className="text-xs font-black text-neutral-900">{alerta.titulo}</p>
        <p className="text-[10px] text-neutral-600 mt-0.5 truncate">{alerta.mensaje}</p>
      </div>
      {!compact && <span className="text-[9px] font-black text-neutral-400">{new Date(alerta.creada_en).toLocaleTimeString('es-MX', { hour: '2-digit', minute: '2-digit' })}</span>}
    </div>
  );
};

const CustomTooltip = ({ active, payload, label }: any) => {
  if (!active) return null;
  return (
    <div className="bg-neutral-900 text-white p-3 rounded-lg shadow-lg border border-neutral-700">
      <p className="text-xs font-medium text-neutral-400 mb-2">{label}</p>
      {payload?.map((entry: any, i: number) => (
        <p key={i} className="text-sm font-bold" style={{ color: entry.color }}>
          {entry.name}: {entry.value !== null ? formatMXN(entry.value) : '—'}
        </p>
      ))}
    </div>
  );
};

// Gráfica de ventas por método (placeholder - se conectará con datos reales)
const VentasMetodoChart = () => {
  const data = [
    { name: 'Efectivo', value: 45, color: '#22c55e' },
    { name: 'Tarjeta', value: 35, color: '#3b82f6' },
    { name: 'Transferencia', value: 20, color: '#8b5cf6' },
  ];
  return (
    <div className="h-[200px]">
      <ResponsiveContainer width="100%" height="100%">
        <PieChart>
          <Pie
            data={data}
            cx="50%" cy="50%"
            innerRadius={50} outerRadius={80}
            dataKey="value" nameKey="name"
            label={({ name, percent }) => `${name} ${(percent * 100).toFixed(0)}%`}
            labelLine={false}
          >
            {data.map((_, i) => <Cell key={i} fill={COLORES_CATEGORIAS[i]} />)}
          </Pie>
          <Tooltip formatter={v => [`${v}%`, '']} />
        </PieChart>
      </ResponsiveContainer>
    </div>
  );
};

// Gráfica de gastos por categoría (placeholder)
const GastosCategoriaChart = () => {
  const data = [
    { name: 'Servicios', value: 40, color: '#ef4444' },
    { name: 'Operativo', value: 30, color: '#f59e0b' },
    { name: 'Nómina', value: 20, color: '#3b82f6' },
    { name: 'Impuestos', value: 10, color: '#8b5cf6' },
  ];
  return (
    <div className="h-[200px]">
      <ResponsiveContainer width="100%" height="100%">
        <PieChart>
          <Pie
            data={data}
            cx="50%" cy="50%"
            innerRadius={50} outerRadius={80}
            dataKey="value" nameKey="name"
            label={({ name, percent }) => `${name} ${(percent * 100).toFixed(0)}%`}
            labelLine={false}
          >
            {data.map((_, i) => <Cell key={i} fill={COLORES_CATEGORIAS[i]} />)}
          </Pie>
          <Tooltip formatter={v => [`${v}%`, '']} />
        </PieChart>
      </ResponsiveContainer>
    </div>
  );
};

import { useEffect, useMemo } from 'react';
import { formatMXN, formatPct, getFechaInicioPeriodo, getFechaHoy } from './utils';
import { ComposedChart } from 'recharts';

// Helper functions
function formatMXN(monto: number): string {
  return new Intl.NumberFormat('es-MX', { style: 'currency', currency: 'MXN', minimumFractionDigits: 2 }).format(monto);
}

function formatPct(valor: number): string {
  return `${valor.toFixed(2)}%`;
}

function getFechaHoy(): string {
  return new Date().toISOString().split('T')[0];
}

function getFechaInicioPeriodo(periodo: string, fechaFin: string): string {
  const fin = new Date(fechaFin);
  switch (periodo) {
    case 'semana': fin.setDate(fin.getDate() - 7); break;
    case 'mes': fin.setMonth(fin.getMonth() - 1); break;
    case 'trimestre': fin.setMonth(fin.getMonth() - 3); break;
    case 'año': fin.setFullYear(fin.getFullYear() - 1); break;
  }
  return fin.toISOString().split('T')[0];
}

export default FinanzasDashboard;