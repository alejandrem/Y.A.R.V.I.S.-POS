import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  ComposedChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer,
  PieChart, Pie, Cell, BarChart, Bar, Line,
} from 'recharts';
import { DatoGraficaPL, DatoGraficaGastosCategoria, DatoGraficaCortesZ } from './types';
import { formatMXN, getFechaHoy, getFechaInicioPeriodo, COLORES_CATEGORIAS } from './utils';

type Periodo = 'semana' | 'mes' | 'trimestre' | 'año';

interface PrediccionDia {
  fecha: string;
  prediccion: number;
  minimo: number;
  maximo: number;
}

const GraficasPanel = () => {
  const [periodo, setPeriodo] = useState<Periodo>('mes');
  const [fechaFin] = useState(getFechaHoy);
  const fechaInicio = getFechaInicioPeriodo(periodo, fechaFin);
  const granularidad = periodo === 'semana' ? 'dia' : periodo === 'mes' ? 'dia' : 'mes';

  const [datosPL, setDatosPL] = useState<DatoGraficaPL[]>([]);
  const [gastosCat, setGastosCat] = useState<DatoGraficaGastosCategoria[]>([]);
  const [tendenciaZ, setTendenciaZ] = useState<DatoGraficaCortesZ[]>([]);
  const [ventasMensual, setVentasMensual] = useState<DatoGraficaPL[]>([]);
  const [loading, setLoading] = useState(false);

  const [predictionRange, setPredictionRange] = useState('7D');
  const [predictionData, setPredictionData] = useState<PrediccionDia[] | null>(null);
  const [predictionLoading, setPredictionLoading] = useState(false);
  const [predictionError, setPredictionError] = useState('');

  useEffect(() => {
    setLoading(true);
    Promise.all([
      invoke<DatoGraficaPL[]>('get_datos_grafica_pl', { fecha_inicio: fechaInicio, fecha_fin: fechaFin, granularidad }),
      invoke<DatoGraficaGastosCategoria[]>('get_gastos_por_categoria', { fecha_inicio: fechaInicio, fecha_fin: fechaFin }),
      invoke<DatoGraficaCortesZ[]>('get_tendencia_cortes_z', { fecha_inicio: fechaInicio, fecha_fin: fechaFin }),
      invoke<DatoGraficaPL[]>('get_ventas_vs_gastos_mensual', { meses: 6 }),
    ])
      .then(([pl, cat, z, mensual]) => {
        setDatosPL(pl);
        setGastosCat(cat);
        setTendenciaZ(z);
        setVentasMensual(mensual);
      })
      .catch(console.error)
      .finally(() => setLoading(false));
  }, [fechaInicio, fechaFin, granularidad]);

  const daysMap: Record<string, number> = { '7D': 7, '15D': 15, '6M': 180, '1A': 365 };

  const fetchPrediction = async (days: number) => {
    setPredictionLoading(true);
    setPredictionError('');
    try {
      const result = await invoke<any>('get_predicciones_financieras', { days });
      if (result?.data) {
        setPredictionData(result.data);
      } else {
        setPredictionError(result?.error || 'Sin datos para predecir');
        setPredictionData(null);
      }
    } catch (e: any) {
      setPredictionError(String(e));
      setPredictionData(null);
    } finally {
      setPredictionLoading(false);
    }
  };

  useEffect(() => {
    fetchPrediction(daysMap[predictionRange] || 7);
  }, [predictionRange]);

  return (
    <div className="space-y-10 animate-in fade-in duration-500">

      <div className="flex flex-col sm:flex-row sm:items-end sm:justify-between gap-4">
        <div>
          <h3 className="text-xl font-black text-neutral-900 uppercase tracking-tight">Gráficas Financieras</h3>
          <p className="text-[10px] text-neutral-400 font-black uppercase tracking-widest mt-0.5">Visualizaciones y predicciones con Prophet</p>
        </div>
        <div className="flex bg-neutral-100 p-1 rounded-xl flex-wrap gap-1">
          {(['semana', 'mes', 'trimestre', 'año'] as Periodo[]).map(p => (
            <button
              key={p}
              onClick={() => setPeriodo(p)}
              className={`px-3 py-2 text-[8px] font-black rounded-lg transition-all ${
                periodo === p ? 'bg-white shadow-sm text-neutral-900' : 'text-neutral-400 hover:text-neutral-600'
              }`}
            >
              {p === 'semana' ? '7D' : p === 'mes' ? '1M' : p === 'trimestre' ? '3M' : '1A'}
            </button>
          ))}
        </div>
      </div>

      {loading && (
        <div className="flex items-center gap-3 text-[10px] font-black text-neutral-400 uppercase tracking-widest">
          <div className="w-4 h-4 border-2 border-neutral-400 border-t-transparent rounded-full animate-spin" />
          Cargando datos...
        </div>
      )}

      <div className="grid grid-cols-1 sm:grid-cols-2 gap-8">
        <div className="bg-neutral-50 rounded-2xl sm:rounded-[2.5rem] border border-neutral-100 p-4 sm:p-8 h-48 sm:h-80 flex flex-col relative overflow-hidden">
          <div className="absolute top-4 sm:top-6 left-4 sm:left-8 z-10">
            <p className="text-[10px] font-black text-neutral-900 uppercase tracking-widest">Evolución P&L</p>
            <p className="text-[8px] text-neutral-400 font-bold uppercase mt-1">Ingresos, Gastos y Utilidad Neta</p>
          </div>
          {datosPL.length === 0 ? (
            <div className="flex-1 flex flex-col items-center justify-center">
              <div className="w-16 h-16 bg-white rounded-2xl flex items-center justify-center shadow-sm text-xl">📈</div>
              <p className="text-[10px] font-black text-neutral-300 uppercase tracking-[0.2em] mt-4">Sin datos en este período</p>
            </div>
          ) : (
            <div className="w-full h-full pt-14">
              <ResponsiveContainer width="100%" height="100%">
                <ComposedChart data={datosPL} margin={{ top: 10, right: 20, left: 0, bottom: 0 }}>
                  <CartesianGrid strokeDasharray="3 3" stroke="#e5e5e5" vertical={false} />
                  <XAxis dataKey="fecha" tick={{ fontSize: 9, fill: '#9ca3af' }} axisLine={{ stroke: '#e5e7eb' }} tickLine={false} />
                  <YAxis tick={{ fontSize: 9, fill: '#9ca3af' }} axisLine={false} tickLine={false} tickFormatter={v => `$${(v / 1000).toFixed(0)}k`} />
                  <Tooltip content={<CustomTooltip />} />
                  <Legend layout="horizontal" align="center" verticalAlign="bottom" wrapperStyle={{ paddingTop: 12 }} />
                  <Area type="monotone" dataKey="ingresos" name="Ingresos" stroke="#171717" fill="#171717" fillOpacity={0.1} strokeWidth={2} />
                  <Area type="monotone" dataKey="gastos" name="Gastos" stroke="#ef4444" fill="#ef4444" fillOpacity={0.1} strokeWidth={2} />
                  <Area type="monotone" dataKey="utilidad_neta" name="Utilidad Neta" stroke="#22c55e" fill="#22c55e" fillOpacity={0.08} strokeWidth={3} />
                </ComposedChart>
              </ResponsiveContainer>
            </div>
          )}
        </div>

        <div className="bg-neutral-50 rounded-2xl sm:rounded-[2.5rem] border border-neutral-100 p-4 sm:p-8 h-48 sm:h-80 flex flex-col relative overflow-hidden">
          <div className="absolute top-4 sm:top-6 left-4 sm:left-8 z-10">
            <p className="text-[10px] font-black text-neutral-900 uppercase tracking-widest">Tendencia Cortes Z</p>
            <p className="text-[8px] text-neutral-400 font-bold uppercase mt-1">Ventas y diferencias por corte</p>
          </div>
          {tendenciaZ.length === 0 ? (
            <div className="flex-1 flex flex-col items-center justify-center">
              <div className="w-16 h-16 bg-white rounded-2xl flex items-center justify-center shadow-sm text-xl">📊</div>
              <p className="text-[10px] font-black text-neutral-300 uppercase tracking-[0.2em] mt-4">Sin datos en este período</p>
            </div>
          ) : (
            <div className="w-full h-full pt-14">
              <ResponsiveContainer width="100%" height="100%">
                <BarChart data={tendenciaZ} margin={{ top: 10, right: 10, left: 0, bottom: 0 }}>
                  <CartesianGrid strokeDasharray="3 3" stroke="#e5e5e5" vertical={false} />
                  <XAxis dataKey="fecha" tick={{ fontSize: 9, fill: '#9ca3af' }} axisLine={{ stroke: '#e5e7eb' }} tickLine={false} />
                  <YAxis tick={{ fontSize: 9, fill: '#9ca3af' }} axisLine={false} tickLine={false} tickFormatter={v => `$${(v / 1000).toFixed(0)}k`} />
                  <Tooltip content={<CustomTooltip />} />
                  <Legend layout="horizontal" align="center" verticalAlign="bottom" wrapperStyle={{ paddingTop: 12 }} />
                  <Bar dataKey="total_ventas" name="Ventas" fill="#171717" radius={[4, 4, 0, 0]} />
                  <Bar dataKey="diferencia" name="Diferencia" fill="#737373" radius={[4, 4, 0, 0]} />
                </BarChart>
              </ResponsiveContainer>
            </div>
          )}
        </div>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 gap-8">
        <div className="bg-neutral-50 rounded-2xl sm:rounded-[2.5rem] border border-neutral-100 p-4 sm:p-8 h-48 sm:h-80 flex flex-col relative overflow-hidden">
          <div className="absolute top-4 sm:top-6 left-4 sm:left-8 z-10">
            <p className="text-[10px] font-black text-neutral-900 uppercase tracking-widest">Gastos por Categoría</p>
            <p className="text-[8px] text-neutral-400 font-bold uppercase mt-1">Distribución de egresos</p>
          </div>
          {gastosCat.length === 0 ? (
            <div className="flex-1 flex flex-col items-center justify-center">
              <div className="w-16 h-16 bg-white rounded-2xl flex items-center justify-center shadow-sm text-xl">🍩</div>
              <p className="text-[10px] font-black text-neutral-300 uppercase tracking-[0.2em] mt-4">Sin datos en este período</p>
            </div>
          ) : (
            <div className="flex-1 flex flex-row items-center pt-14">
              <div className="w-1/2 h-full">
                <ResponsiveContainer width="100%" height="100%">
                  <PieChart>
                    <Pie data={gastosCat} cx="50%" cy="50%" innerRadius={45} outerRadius={70} dataKey="monto" nameKey="categoria" paddingAngle={2}>
                      {gastosCat.map((_, i) => (
                        <Cell key={i} fill={COLORES_CATEGORIAS[i % COLORES_CATEGORIAS.length]} />
                      ))}
                    </Pie>
                    <Tooltip formatter={(v: any, name: any) => [formatMXN(v), name]} />
                  </PieChart>
                </ResponsiveContainer>
              </div>
              <div className="w-1/2 flex flex-col justify-center space-y-2 pr-2">
                {gastosCat.map((g, i) => (
                  <div key={g.categoria} className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <div className="w-2.5 h-2.5 rounded-full" style={{ backgroundColor: COLORES_CATEGORIAS[i % COLORES_CATEGORIAS.length] }} />
                      <span className="text-[10px] font-black text-neutral-600 uppercase">{g.categoria}</span>
                    </div>
                    <span className="text-[10px] font-black text-neutral-900">{g.porcentaje.toFixed(1)}%</span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>

        <div className="bg-neutral-50 rounded-2xl sm:rounded-[2.5rem] border border-neutral-100 p-4 sm:p-8 h-48 sm:h-80 flex flex-col relative overflow-hidden">
          <div className="absolute top-4 sm:top-6 left-4 sm:left-8 z-10">
            <p className="text-[10px] font-black text-neutral-900 uppercase tracking-widest">Ventas vs Gastos</p>
            <p className="text-[8px] text-neutral-400 font-bold uppercase mt-1">Comparativa últimos 6 meses</p>
          </div>
          {ventasMensual.length === 0 ? (
            <div className="flex-1 flex flex-col items-center justify-center">
              <div className="w-16 h-16 bg-white rounded-2xl flex items-center justify-center shadow-sm text-xl">📅</div>
              <p className="text-[10px] font-black text-neutral-300 uppercase tracking-[0.2em] mt-4">Sin datos en este período</p>
            </div>
          ) : (
            <div className="w-full h-full pt-14">
              <ResponsiveContainer width="100%" height="100%">
                <ComposedChart data={ventasMensual} margin={{ top: 10, right: 20, left: 0, bottom: 0 }}>
                  <CartesianGrid strokeDasharray="3 3" stroke="#e5e5e5" vertical={false} />
                  <XAxis dataKey="fecha" tick={{ fontSize: 9, fill: '#9ca3af' }} axisLine={{ stroke: '#e5e7eb' }} tickLine={false} />
                  <YAxis tick={{ fontSize: 9, fill: '#9ca3af' }} axisLine={false} tickLine={false} tickFormatter={v => `$${(v / 1000).toFixed(0)}k`} />
                  <Tooltip content={<CustomTooltip />} />
                  <Legend layout="horizontal" align="center" verticalAlign="bottom" wrapperStyle={{ paddingTop: 12 }} />
                  <Area type="monotone" dataKey="ingresos" name="Ventas" stroke="#171717" fill="#171717" fillOpacity={0.12} strokeWidth={2} />
                  <Area type="monotone" dataKey="gastos" name="Gastos" stroke="#ef4444" fill="#ef4444" fillOpacity={0.12} strokeWidth={2} />
                </ComposedChart>
              </ResponsiveContainer>
            </div>
          )}
        </div>
      </div>

      {/* PREDICCIÓN DE FLUJO DE CAJA (PROPHET) */}
      <div className="bg-neutral-900 rounded-2xl sm:rounded-[2.5rem] p-5 sm:p-10 shadow-2xl relative overflow-hidden h-[280px] sm:h-[450px] flex flex-col">
        <div className="relative z-10 flex flex-col gap-3 sm:gap-6">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="text-lg font-black text-white uppercase tracking-tight">Predicción de Flujo de Caja</h3>
              <p className="text-[10px] text-neutral-500 font-black uppercase tracking-[0.2em] mt-1">Modelo Prophet (Próximos días)</p>
            </div>
            <div className="flex bg-white/5 p-1 rounded-xl gap-1">
              {['7D', '15D', '6M', '1A'].map((r) => (
                <button
                  key={r}
                  onClick={() => setPredictionRange(r)}
                  className={`px-3 py-1.5 text-[8px] font-black rounded-lg transition-all ${predictionRange === r ? 'bg-white text-neutral-900' : 'text-neutral-500 hover:text-white'}`}
                >
                  {r}
                </button>
              ))}
            </div>
          </div>
        </div>

        <div className="flex-1 flex flex-col items-center justify-center relative z-10">
          {predictionLoading ? (
            <p className="text-[11px] font-black text-white uppercase tracking-[0.3em] opacity-40">Calculando predicción...</p>
          ) : predictionData && predictionData.length > 0 ? (
            <div className="w-full h-full pt-8">
              <ResponsiveContainer width="100%" height="100%">
                <ComposedChart data={predictionData}>
                  <CartesianGrid strokeDasharray="3 3" stroke="#333" />
                  <XAxis dataKey="fecha" tick={{ fontSize: 9, fill: '#999' }} tickLine={false} />
                  <YAxis tick={{ fontSize: 9, fill: '#999' }} tickLine={false} tickFormatter={v => `$${(v / 1000).toFixed(0)}k`} />
                  <Tooltip contentStyle={{ backgroundColor: '#171717', border: '1px solid #333', borderRadius: '12px', color: '#fff' }} formatter={(v: any) => [formatMXN(v), '']} />
                  <Legend />
                  <Area type="monotone" dataKey="maximo" fill="#22c55e" fillOpacity={0.1} stroke="none" name="Máximo" />
                  <Area type="monotone" dataKey="minimo" fill="#22c55e" fillOpacity={0.1} stroke="none" name="Mínimo" />
                  <Line type="monotone" dataKey="prediccion" stroke="#22c55e" strokeWidth={2} dot={false} name="Predicción" />
                </ComposedChart>
              </ResponsiveContainer>
            </div>
          ) : (
            <>
              <div className="w-full h-px bg-white/5 absolute top-1/2"></div>
              <div className="text-center space-y-4">
                <div className="w-20 h-20 bg-white/5 rounded-[2rem] flex items-center justify-center mx-auto text-3xl opacity-20">🔮</div>
                <p className="text-[11px] font-black text-white uppercase tracking-[0.3em] opacity-40">
                  {predictionError || 'Sin datos para predecir'}
                </p>
              </div>
            </>
          )}
        </div>

        <div className="absolute -bottom-10 -right-10 w-64 h-64 bg-white/5 rounded-full blur-3xl pointer-events-none"></div>
      </div>

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

export default GraficasPanel;
