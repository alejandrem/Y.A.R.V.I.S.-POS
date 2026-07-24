import { useState, useEffect, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Area, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer,
  PieChart, Pie, Cell, ComposedChart,
} from 'recharts';
import { useMetricas, useGraficas, useAlertas } from './hooks';
import {
  formatMXN, formatPct, getFechaInicioPeriodo, getFechaHoy,
  COLORES_CATEGORIAS, COLORES_SEVERIDAD,
} from './utils';

export const FinanzasDashboard = () => {
  const [periodo, setPeriodo] = useState<'semana' | 'mes' | 'trimestre' | 'año'>('mes');
  const [fechaFin] = useState(getFechaHoy());
  const fechaInicio = getFechaInicioPeriodo(periodo, fechaFin);

  const { resumen, puntoEquilibrio, loading: loadingMetricas, cargarResumen } = useMetricas();
  const { datosPL, loading: loadingGraficas, cargarPL } = useGraficas();
  const { alertas, cargarAlertas } = useAlertas();

  useEffect(() => {
    cargarResumen(fechaInicio, fechaFin);
    cargarPL(fechaInicio, fechaFin, periodo === 'semana' ? 'semana' : 'mes');
    cargarAlertas(true);
  }, [fechaInicio, fechaFin, periodo, cargarResumen, cargarPL, cargarAlertas]);

  const alertasCriticas = useMemo(() =>
    alertas.filter(a => !a.leida && (a.severidad === 'rojo' || a.severidad === 'amarillo')).slice(0, 3),
    [alertas]);

  const kpis = useMemo(() => [
    {
      label: 'UTILIDAD BRUTA',
      value: resumen ? formatMXN(resumen.total_utilidad_bruta) : '—',
      subtitle: 'Ventas − COGS',
      icon: '📈',
      trend: resumen && resumen.total_utilidad_bruta > 0 ? 'up' : 'down',
    },
    {
      label: 'UTILIDAD OPERATIVA',
      value: resumen ? formatMXN(resumen.total_utilidad_operativa) : '—',
      subtitle: 'Bruta − Gastos Op.',
      icon: '🏢',
      trend: resumen && resumen.total_utilidad_operativa > 0 ? 'up' : 'down',
    },
    {
      label: 'UTILIDAD NETA',
      value: resumen ? formatMXN(resumen.total_utilidad_neta) : '—',
      subtitle: 'Operativa − Impuestos',
      icon: '💰',
      trend: resumen && resumen.total_utilidad_neta > 0 ? 'up' : 'down',
    },
    {
      label: 'MARGEN NETO %',
      value: resumen ? formatPct(resumen.margen_promedio_pct) : '—',
      subtitle: '(Neta / Ventas) × 100',
      icon: '📊',
      trend: resumen && resumen.margen_promedio_pct > 15 ? 'up' : 'down',
    },
  ], [resumen]);

  const progresoEquilibrio = useMemo(() => {
    if (!puntoEquilibrio || !resumen || puntoEquilibrio.ventas_necesarias <= 0) return 0;
    return Math.min(100, (resumen.total_ventas / puntoEquilibrio.ventas_necesarias) * 100);
  }, [puntoEquilibrio, resumen]);

  return (
    <div className="space-y-10 animate-in fade-in duration-500">

      <div className="flex flex-col sm:flex-row sm:items-end sm:justify-between gap-4">
        <div>
          <h3 className="text-xl font-black text-neutral-900 uppercase tracking-tight">Dashboard Financiero</h3>
          <p className="text-[10px] text-neutral-400 font-black uppercase tracking-widest mt-0.5">Visión general del rendimiento del negocio</p>
        </div>
        <div className="flex bg-neutral-100 p-1 rounded-xl flex-wrap gap-1">
          {(['semana', 'mes', 'trimestre', 'año'] as const).map(p => (
            <button
              key={p}
              onClick={() => setPeriodo(p)}
              className={`px-3 py-2 text-[8px] font-black rounded-lg transition-all ${periodo === p
                ? 'bg-white shadow-sm text-neutral-900'
                : 'text-neutral-400 hover:text-neutral-600'
                }`}
            >
              {p === 'semana' ? '7D' : p === 'mes' ? '1M' : p === 'trimestre' ? '3M' : '1A'}
            </button>
          ))}
        </div>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6">
        {loadingMetricas
          ? Array.from({ length: 4 }).map((_, i) => (
            <div key={i} className="bg-neutral-50 rounded-2xl border border-neutral-100 p-6 h-[148px] animate-pulse" />
          ))
          : kpis.map((kpi, i) => <KPICard key={i} {...kpi} />)}
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">

        <div className="lg:col-span-2 bg-neutral-50 rounded-2xl sm:rounded-[2.5rem] border border-neutral-100 p-4 sm:p-8 h-64 sm:h-80 flex flex-col relative overflow-hidden">
          <div className="absolute top-4 sm:top-6 left-4 sm:left-8 z-10">
            <p className="text-[10px] font-black text-neutral-900 uppercase tracking-widest">Evolución P&L</p>
            <p className="text-[8px] text-neutral-400 font-bold uppercase mt-1">Pérdidas y Ganancias</p>
          </div>
          {loadingGraficas ? (
            <div className="flex-1 flex items-center justify-center">
              <div className="w-8 h-8 border-2 border-neutral-900 border-t-transparent rounded-full animate-spin" />
            </div>
          ) : datosPL.length === 0 ? (
            <div className="flex-1 flex flex-col items-center justify-center">
              <div className="w-16 h-16 bg-white rounded-2xl flex items-center justify-center shadow-sm text-xl">📈</div>
              <p className="text-[10px] font-black text-neutral-300 uppercase tracking-[0.2em] mt-4">Sin datos de ventas en este período</p>
            </div>
          ) : (
            <div className="w-full h-full pt-14">
              <ResponsiveContainer width="100%" height="100%">
                <ComposedChart data={datosPL} margin={{ top: 10, right: 20, left: 0, bottom: 0 }}>
                  <CartesianGrid strokeDasharray="3 3" stroke="#e5e5e5" vertical={false} />
                  <XAxis dataKey="fecha" tick={{ fontSize: 9, fill: '#9ca3af' }} axisLine={{ stroke: '#e5e7eb' }} tickLine={false} />
                  <YAxis tick={{ fontSize: 9, fill: '#9ca3af' }} axisLine={false} tickLine={false} tickFormatter={v => `$${(v / 1000).toFixed(0)}k`} />
                  <Tooltip content={<CustomTooltip />} />
                  <Legend layout="horizontal" align="center" verticalAlign="bottom" iconType="line" wrapperStyle={{ paddingTop: 10 }} />
                  <Area type="monotone" dataKey="ingresos" name="Ingresos" stroke="#171717" fill="#171717" fillOpacity={0.1} strokeWidth={2} />
                  <Area type="monotone" dataKey="gastos" name="Gastos" stroke="#ef4444" fill="#ef4444" fillOpacity={0.1} strokeWidth={2} />
                  <Area type="monotone" dataKey="utilidad_neta" name="Utilidad Neta" stroke="#22c55e" fill="#22c55e" fillOpacity={0.05} strokeWidth={3} />
                </ComposedChart>
              </ResponsiveContainer>
            </div>
          )}
        </div>

        <div className="space-y-6">

          <div className="bg-neutral-50 rounded-2xl sm:rounded-[2.5rem] border border-neutral-100 p-4 sm:p-8 h-48 sm:h-80 flex flex-col relative overflow-hidden">
            <div className="absolute top-4 sm:top-6 left-4 sm:left-8 z-10">
              <p className="text-[10px] font-black text-neutral-900 uppercase tracking-widest flex items-center gap-2">
                <span className="w-1.5 h-4 bg-neutral-900 rounded-full inline-block" />
                BREAK-EVEN
              </p>
            </div>
            {puntoEquilibrio ? (
              <div className="flex-1 flex flex-col justify-center pt-6 space-y-4">
                <div className="grid grid-cols-2 gap-3">
                  <MetricItem label="GASTOS FIJOS" value={formatMXN(puntoEquilibrio.gastos_fijos_mensuales)} />
                  <MetricItem label="MARGEN CONTRIB." value={formatPct(puntoEquilibrio.margen_contribucion_pct)} />
                  <MetricItem label="VENTAS NECES." value={formatMXN(puntoEquilibrio.ventas_necesarias)} />
                  <MetricItem label="TICKETS NECES." value={puntoEquilibrio.tickets_necesarios.toFixed(0)} />
                </div>
                <div className="pt-3 border-t border-neutral-200">
                  <div className="h-3 bg-neutral-200 rounded-full overflow-hidden">
                    <div
                      className={`h-full rounded-full transition-all duration-700 ${progresoEquilibrio >= 100 ? 'bg-neutral-800' : progresoEquilibrio >= 70 ? 'bg-neutral-500' : 'bg-red-500'}`}
                      style={{ width: `${progresoEquilibrio}%` }}
                    />
                  </div>
                  <p className="text-[10px] text-neutral-500 mt-1.5 text-center font-bold">
                    {progresoEquilibrio >= 100
                      ? '✅ Break-even superado'
                      : `Faltan ${formatMXN(Math.max(0, puntoEquilibrio.ventas_necesarias - (resumen?.total_ventas ?? 0)))} para el break-even`}
                  </p>
                </div>
              </div>
            ) : (
              <div className="flex-1 flex flex-col items-center justify-center">
                <div className="w-16 h-16 bg-white rounded-2xl flex items-center justify-center shadow-sm text-xl">⚖️</div>
                <p className="text-[10px] font-black text-neutral-300 uppercase tracking-[0.2em] mt-4">
                  {loadingMetricas ? 'Calculando...' : 'Sin datos suficientes'}
                </p>
              </div>
            )}
          </div>

          <div className="bg-neutral-50 rounded-2xl sm:rounded-[2.5rem] border border-neutral-100 p-4 sm:p-8 h-48 sm:h-80 flex flex-col relative overflow-hidden">
            <div className="absolute top-4 sm:top-6 left-4 sm:left-8 z-10 flex items-center justify-between w-[calc(100%-2rem)] sm:w-[calc(100%-4rem)]">
              <p className="text-[10px] font-black text-neutral-900 uppercase tracking-widest flex items-center gap-2">
                <span className="w-1.5 h-4 bg-red-500 rounded-full inline-block" />
                ALERTAS
              </p>
              {alertasCriticas.length > 0 && (
                <span className="px-2 py-0.5 bg-red-500 text-white text-[8px] font-black rounded-lg">{alertasCriticas.length}</span>
              )}
            </div>
            <div className="flex-1 flex flex-col justify-center pt-6">
              {alertasCriticas.length === 0 ? (
                <div className="flex flex-col items-center justify-center">
                  <div className="w-16 h-16 bg-white rounded-2xl flex items-center justify-center shadow-sm text-xl">⚪</div>
                  <p className="text-[10px] font-black text-neutral-300 uppercase tracking-[0.2em] mt-4">Sin alertas críticas</p>
                  <p className="text-[8px] text-neutral-300 mt-1">Todo bajo control</p>
                </div>
              ) : (
                <div className="space-y-2">
                  {alertasCriticas.map(a => <AlertaItem key={a.id} alerta={a} compact />)}
                </div>
              )}
            </div>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 gap-8 pb-12">
        <div className="bg-neutral-50 rounded-2xl sm:rounded-[2.5rem] border border-neutral-100 p-4 sm:p-8 h-48 sm:h-64 flex flex-col relative overflow-hidden">
          <div className="absolute top-4 sm:top-6 left-4 sm:left-8 z-10">
            <p className="text-[10px] font-black text-neutral-900 uppercase tracking-widest">Ventas por Método</p>
            <p className="text-[8px] text-neutral-400 font-bold uppercase mt-1">Distribución de cobro</p>
          </div>
          <VentasMetodoChart fechaInicio={fechaInicio} fechaFin={fechaFin} />
        </div>
        <div className="bg-neutral-50 rounded-2xl sm:rounded-[2.5rem] border border-neutral-100 p-4 sm:p-8 h-48 sm:h-64 flex flex-col relative overflow-hidden">
          <div className="absolute top-4 sm:top-6 left-4 sm:left-8 z-10">
            <p className="text-[10px] font-black text-neutral-900 uppercase tracking-widest">Gastos por Categoría</p>
            <p className="text-[8px] text-neutral-400 font-bold uppercase mt-1">Distribución de egresos</p>
          </div>
          <GastosCategoriaChart fechaInicio={fechaInicio} fechaFin={fechaFin} />
        </div>
      </div>

    </div>
  );
};

const KPICard = ({ label, value, subtitle, trend }: any) => (
  <div className={`p-4 sm:p-8 rounded-2xl sm:rounded-[2rem] border border-neutral-100 relative overflow-hidden group hover:scale-[1.02] transition-all duration-300 ${
    trend === 'up' ? 'bg-neutral-900 text-white shadow-2xl shadow-neutral-200' : 'bg-neutral-50'
  }`}>
    <p className={`text-[9px] sm:text-[10px] font-black uppercase tracking-widest mb-2 ${trend === 'up' ? 'text-neutral-400' : 'text-neutral-400'}`}>{label}</p>
    <h3 className="text-xl sm:text-2xl font-black">{value}</h3>
    <span className={`absolute top-4 sm:top-8 right-4 sm:right-8 text-[7px] sm:text-[8px] font-black px-1.5 sm:px-2 py-0.5 sm:py-1 rounded-lg uppercase ${trend === 'up' ? 'bg-white/10 text-neutral-400' : 'bg-neutral-100 text-neutral-400'}`}>
      {subtitle}
    </span>
  </div>
);

const MetricItem = ({ label, value }: { label: string; value: string }) => (
  <div>
    <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest">{label}</p>
    <p className="text-base font-black text-neutral-900 mt-0.5">{value}</p>
  </div>
);

const AlertaItem = ({ alerta, compact }: { alerta: any; compact?: boolean }) => {
  const colores = COLORES_SEVERIDAD[alerta.severidad as keyof typeof COLORES_SEVERIDAD] ?? COLORES_SEVERIDAD.rojo;
  return (
    <div className={`p-3 rounded-xl border-l-4 ${colores.border} bg-white flex items-start gap-3`}>
      <span className="text-base leading-none mt-0.5">{colores.icon}</span>
      <div className="flex-1 min-w-0">
        <p className="text-xs font-black text-neutral-900">{alerta.titulo}</p>
        <p className="text-[10px] text-neutral-500 mt-0.5 truncate">{alerta.mensaje}</p>
      </div>
      {!compact && (
        <span className="text-[9px] font-black text-neutral-400 shrink-0">
          {new Date(alerta.creada_en).toLocaleTimeString('es-MX', { hour: '2-digit', minute: '2-digit' })}
        </span>
      )}
    </div>
  );
};

const CustomTooltip = ({ active, payload, label }: any) => {
  if (!active || !payload?.length) return null;
  return (
    <div className="bg-neutral-900 text-white p-3 rounded-xl shadow-2xl border border-neutral-700 text-xs">
      <p className="text-neutral-400 font-medium mb-2">{label}</p>
      {payload.map((e: any, i: number) => (
        <p key={i} className="font-bold" style={{ color: e.color }}>
          {e.name}: {formatMXN(e.value)}
        </p>
      ))}
    </div>
  );
};

const VentasMetodoChart = ({ fechaInicio, fechaFin }: { fechaInicio: string; fechaFin: string }) => {
  const [data, setData] = useState<{ name: string; value: number }[]>([]);

  useEffect(() => {
    invoke<any[]>('get_cortes_caja', { filtros: { fecha_inicio: fechaInicio, fecha_fin: fechaFin } })
      .then(cortes => {
        const efe = cortes.reduce((a, c) => a + (c.total_efectivo || 0), 0);
        const tar = cortes.reduce((a, c) => a + (c.total_tarjeta || 0), 0);
        const tra = cortes.reduce((a, c) => a + (c.total_transferencia || 0), 0);
        setData(
          [{ name: 'Efectivo', value: efe }, { name: 'Tarjeta', value: tar }, { name: 'Transferencia', value: tra }]
            .filter(d => d.value > 0)
            .map(d => ({ ...d, value: Math.round(d.value * 100) / 100 }))
        );
      })
      .catch(() => setData([]));
  }, [fechaInicio, fechaFin]);

  if (data.length === 0) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center">
        <div className="w-16 h-16 bg-white rounded-2xl flex items-center justify-center shadow-sm text-xl">💳</div>
        <p className="text-[10px] font-black text-neutral-300 uppercase tracking-[0.2em] mt-4">Sin cortes registrados</p>
      </div>
    );
  }

  return (
    <div className="flex-1 flex flex-row items-center pt-14">
      <div className="w-1/2 h-full">
        <ResponsiveContainer width="100%" height="100%">
          <PieChart>
            <Pie data={data} cx="50%" cy="50%" innerRadius={45} outerRadius={70} dataKey="value" nameKey="name" paddingAngle={2}>
              {data.map((_, i) => <Cell key={i} fill={COLORES_CATEGORIAS[i % COLORES_CATEGORIAS.length]} />)}
            </Pie>
            <Tooltip formatter={(v: any) => [formatMXN(v), '']} />
          </PieChart>
        </ResponsiveContainer>
      </div>
      <div className="w-1/2 flex flex-col justify-center space-y-2 pr-2">
        {data.map((d, i) => (
          <div key={d.name} className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <div className="w-2.5 h-2.5 rounded-full" style={{ backgroundColor: COLORES_CATEGORIAS[i % COLORES_CATEGORIAS.length] }} />
              <span className="text-[10px] font-black text-neutral-600 uppercase">{d.name}</span>
            </div>
            <span className="text-[10px] font-black text-neutral-900">{formatMXN(d.value)}</span>
          </div>
        ))}
      </div>
    </div>
  );
};

const GastosCategoriaChart = ({ fechaInicio, fechaFin }: { fechaInicio: string; fechaFin: string }) => {
  const [data, setData] = useState<{ name: string; value: number }[]>([]);

  useEffect(() => {
    invoke<any[]>('get_gastos_por_categoria', { fecha_inicio: fechaInicio, fecha_fin: fechaFin })
      .then(cats => setData(cats.filter(c => c.monto > 0).map(c => ({ name: c.categoria, value: c.monto }))))
      .catch(() => setData([]));
  }, [fechaInicio, fechaFin]);

  if (data.length === 0) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center">
        <div className="w-16 h-16 bg-white rounded-2xl flex items-center justify-center shadow-sm text-xl">🍩</div>
        <p className="text-[10px] font-black text-neutral-300 uppercase tracking-[0.2em] mt-4">Sin gastos registrados</p>
      </div>
    );
  }

  return (
    <div className="flex-1 flex flex-row items-center pt-14">
      <div className="w-1/2 h-full">
        <ResponsiveContainer width="100%" height="100%">
          <PieChart>
            <Pie data={data} cx="50%" cy="50%" innerRadius={45} outerRadius={70} dataKey="value" nameKey="name" paddingAngle={2}>
              {data.map((_, i) => <Cell key={i} fill={COLORES_CATEGORIAS[i % COLORES_CATEGORIAS.length]} />)}
            </Pie>
            <Tooltip formatter={(v: any) => [formatMXN(v), '']} />
          </PieChart>
        </ResponsiveContainer>
      </div>
      <div className="w-1/2 flex flex-col justify-center space-y-2 pr-2">
        {data.map((d, i) => (
          <div key={d.name} className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <div className="w-2.5 h-2.5 rounded-full" style={{ backgroundColor: COLORES_CATEGORIAS[i % COLORES_CATEGORIAS.length] }} />
              <span className="text-[10px] font-black text-neutral-600 uppercase">{d.name}</span>
            </div>
            <span className="text-[10px] font-black text-neutral-900">{formatMXN(d.value)}</span>
          </div>
        ))}
      </div>
    </div>
  );
};

export { AlertaItem };
export default FinanzasDashboard;
