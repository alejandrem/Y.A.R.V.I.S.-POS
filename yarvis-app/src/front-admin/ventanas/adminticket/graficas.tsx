import { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  LineChart, Line, BarChart, Bar, XAxis, YAxis, CartesianGrid,
  Tooltip, ResponsiveContainer, Area, ComposedChart, Legend,
  PieChart, Pie, Cell,
} from "recharts";

interface TicketDb {
  id: number;
  fecha: string;
  total: number;
  metodo_pago: string;
}

interface CorteDb {
  id: number;
  fecha: string;
  total_ventas: number;
  total_efectivo: number;
}

interface GraficasProps {
  filteredTickets: TicketDb[];
  filteredCortes: CorteDb[];
}

const COLORS = {
  primary: "#171717",
  secondary: "#737373",
  accent: "#22c55e",
  red: "#ef4444",
  blue: "#3b82f6",
  amber: "#f59e0b",
  purple: "#a855f7",
};

const Graficas = ({ filteredTickets, filteredCortes }: GraficasProps) => {
  const [predictionRange, setPredictionRange] = useState("7D");
  const [predictionData, setPredictionData] = useState<any[] | null>(null);
  const [predictionLoading, setPredictionLoading] = useState(false);
  const [predictionError, setPredictionError] = useState("");

  const fetchPrediction = async (days: number) => {
    setPredictionLoading(true);
    setPredictionError("");
    try {
      const result = await invoke<any>("get_predictions", { days });
      if (result?.data) {
        setPredictionData(result.data);
      } else {
        setPredictionError(result?.error || "Sin datos");
        setPredictionData(null);
      }
    } catch (e: any) {
      setPredictionError(String(e));
      setPredictionData(null);
    } finally {
      setPredictionLoading(false);
    }
  };

  const daysMap: Record<string, number> = { "7D": 7, "15D": 15, "6M": 180, "1A": 365 };

  useEffect(() => {
    fetchPrediction(daysMap[predictionRange] || 7);
  }, [predictionRange]);

  const ticketsPorDia = useMemo(() => {
    if (filteredTickets.length === 0) return [];
    const map = new Map<string, { fecha: string; total: number; count: number }>();
    for (const t of filteredTickets) {
      const dia = t.fecha.split(" ")[0];
      const existing = map.get(dia) || { fecha: dia, total: 0, count: 0 };
      existing.total += t.total;
      existing.count += 1;
      map.set(dia, existing);
    }
    return Array.from(map.values()).sort((a, b) => a.fecha.localeCompare(b.fecha));
  }, [filteredTickets]);

  const cortesData = useMemo(() => {
    return filteredCortes
      .map((c) => ({ fecha: c.fecha.split(" ")[0], total: c.total_ventas, efectivo: c.total_efectivo }))
      .sort((a, b) => a.fecha.localeCompare(b.fecha));
  }, [filteredCortes]);

  const pagosData = useMemo(() => {
    const map = new Map<string, number>();
    for (const t of filteredTickets) {
      const mp = t.metodo_pago || "efectivo";
      map.set(mp, (map.get(mp) || 0) + 1);
    }
    return Array.from(map.entries())
      .map(([name, value]) => ({ name: name.charAt(0).toUpperCase() + name.slice(1), value }))
      .sort((a, b) => b.value - a.value);
  }, [filteredTickets]);

  const PAGO_COLORS = ["#171717", "#3b82f6", "#22c55e", "#f59e0b", "#a855f7", "#ef4444", "#737373"];

  return (
    <div className="space-y-8 pt-10 border-t border-neutral-100">
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 sm:gap-8">
        {/* RENDIMIENTO DE TICKETS */}
        <div className="bg-neutral-50 rounded-2xl sm:rounded-[2.5rem] border border-neutral-100 p-4 sm:p-8 h-48 sm:h-80 flex flex-col relative overflow-hidden">
          <div className="absolute top-4 sm:top-6 left-4 sm:left-8 z-10">
            <p className="text-[10px] font-black text-neutral-900 uppercase tracking-widest">Rendimiento de Tickets</p>
            <p className="text-[8px] text-neutral-400 font-bold uppercase mt-1">Mostrando: {filteredTickets.length} tickets</p>
          </div>
          {ticketsPorDia.length > 0 ? (
            <div className="w-full h-full pt-14">
              <ResponsiveContainer width="100%" height="100%">
                <LineChart data={ticketsPorDia}>
                  <CartesianGrid strokeDasharray="3 3" stroke="#e5e5e5" />
                  <XAxis dataKey="fecha" tick={{ fontSize: 9 }} tickLine={false} />
                  <YAxis tick={{ fontSize: 9 }} tickLine={false} />
                  <Tooltip />
                  <Line type="monotone" dataKey="total" stroke={COLORS.primary} strokeWidth={2} dot={false} />
                </LineChart>
              </ResponsiveContainer>
            </div>
          ) : (
            <div className="flex-1 flex flex-col items-center justify-center">
              <div className="w-16 h-16 bg-white rounded-2xl flex items-center justify-center shadow-sm text-xl">📊</div>
              <p className="text-[10px] font-black text-neutral-300 uppercase tracking-[0.2em] mt-4">
                {filteredTickets.length === 0 ? "Esperando tickets..." : "Listo para graficar"}
              </p>
            </div>
          )}
        </div>

        {/* FLUJO DE CORTES DE CAJA */}
        <div className="bg-neutral-50 rounded-2xl sm:rounded-[2.5rem] border border-neutral-100 p-4 sm:p-8 h-48 sm:h-80 flex flex-col relative overflow-hidden">
          <div className="absolute top-4 sm:top-6 left-4 sm:left-8 z-10">
            <p className="text-[10px] font-black text-neutral-900 uppercase tracking-widest">Flujo de Cortes de Caja</p>
            <p className="text-[8px] text-neutral-400 font-bold uppercase mt-1">Mostrando: {filteredCortes.length} cortes</p>
          </div>
          {cortesData.length > 0 ? (
            <div className="w-full h-full pt-14">
              <ResponsiveContainer width="100%" height="100%">
                <BarChart data={cortesData}>
                  <CartesianGrid strokeDasharray="3 3" stroke="#e5e5e5" />
                  <XAxis dataKey="fecha" tick={{ fontSize: 9 }} tickLine={false} />
                  <YAxis tick={{ fontSize: 9 }} tickLine={false} />
                  <Tooltip />
                  <Bar dataKey="total" fill={COLORS.primary} radius={[4, 4, 0, 0]} />
                </BarChart>
              </ResponsiveContainer>
            </div>
          ) : (
            <div className="flex-1 flex flex-col items-center justify-center">
              <div className="w-16 h-16 bg-white rounded-2xl flex items-center justify-center shadow-sm text-xl">📉</div>
              <p className="text-[10px] font-black text-neutral-300 uppercase tracking-[0.2em] mt-4">
                {filteredCortes.length === 0 ? "Esperando cortes..." : "Listo para graficar"}
              </p>
            </div>
          )}
        </div>
      </div>

      {/* PREDICCIÓN DE FLUJO DE CAJA (PROPHET) */}
      <div className="sm:col-span-2 bg-neutral-900 rounded-2xl sm:rounded-[2.5rem] p-5 sm:p-10 shadow-2xl relative overflow-hidden h-[280px] sm:h-[450px] flex flex-col">
        <div className="relative z-10 flex flex-col gap-3 sm:gap-6">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="text-lg font-black text-white uppercase tracking-tight">Predicción de Flujo de Caja</h3>
              <p className="text-[10px] text-neutral-500 font-black uppercase tracking-[0.2em] mt-1">Modelo Prophet (Próximos días)</p>
            </div>
            <div className="flex bg-white/5 p-1 rounded-xl gap-1">
              {["7D", "15D", "6M", "1A"].map((r) => (
                <button
                  key={r}
                  onClick={() => setPredictionRange(r)}
                  className={`px-3 py-1.5 text-[8px] font-black rounded-lg transition-all ${predictionRange === r ? "bg-white text-neutral-900" : "text-neutral-500 hover:text-white"}`}
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
                  <XAxis dataKey="fecha" tick={{ fontSize: 9, fill: "#999" }} tickLine={false} />
                  <YAxis tick={{ fontSize: 9, fill: "#999" }} tickLine={false} />
                  <Tooltip contentStyle={{ backgroundColor: "#171717", border: "1px solid #333", borderRadius: "12px", color: "#fff" }} />
                  <Legend />
                  <Area type="monotone" dataKey="maximo" fill="#22c55e" fillOpacity={0.1} stroke="none" />
                  <Area type="monotone" dataKey="minimo" fill="#22c55e" fillOpacity={0.1} stroke="none" />
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
                  {predictionError || "Sin datos para predecir"}
                </p>
              </div>
            </>
          )}
        </div>

        <div className="absolute -bottom-10 -right-10 w-64 h-64 bg-white/5 rounded-full blur-3xl pointer-events-none"></div>
      </div>

      {/* FORMAS DE PAGO + ACCIONES RÁPIDAS */}
      <div className="grid grid-cols-1 sm:grid-cols-3 gap-4 sm:gap-8 pb-12">
        <div className="sm:col-span-2 bg-neutral-50 rounded-2xl sm:rounded-[2.5rem] border border-neutral-100 p-4 sm:p-8 h-48 sm:h-64 flex flex-col relative overflow-hidden">
          <div className="absolute top-4 sm:top-6 left-4 sm:left-8 z-10">
            <p className="text-[10px] font-black text-neutral-900 uppercase tracking-widest">Formas de Pago</p>
            <p className="text-[8px] text-neutral-400 font-bold uppercase mt-1">{filteredTickets.length} tickets</p>
          </div>
          {pagosData.length > 0 ? (
            <div className="flex-1 flex flex-row items-center pt-14">
              <div className="w-1/2 h-full">
                <ResponsiveContainer width="100%" height="100%">
                  <PieChart>
                    <Pie
                      data={pagosData}
                      cx="50%"
                      cy="50%"
                      innerRadius={45}
                      outerRadius={70}
                      dataKey="value"
                    >
                      {pagosData.map((_, i) => (
                        <Cell key={i} fill={PAGO_COLORS[i % PAGO_COLORS.length]} />
                      ))}
                    </Pie>
                    <Tooltip />
                  </PieChart>
                </ResponsiveContainer>
              </div>
              <div className="w-1/2 flex flex-col justify-center space-y-2 pr-2">
                {pagosData.map((p, i) => (
                  <div key={p.name} className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <div className="w-2.5 h-2.5 rounded-full" style={{ backgroundColor: PAGO_COLORS[i % PAGO_COLORS.length] }} />
                      <span className="text-[10px] font-black text-neutral-600 uppercase">{p.name}</span>
                    </div>
                    <span className="text-[10px] font-black text-neutral-900">{p.value} ({Math.round(p.value / pagosData.reduce((a, b) => a + b.value, 0) * 100)}%)</span>
                  </div>
                ))}
              </div>
            </div>
          ) : (
            <div className="flex-1 flex flex-col items-center justify-center">
              <div className="w-16 h-16 bg-white rounded-2xl flex items-center justify-center shadow-sm text-xl">💳</div>
              <p className="text-[10px] font-black text-neutral-300 uppercase tracking-[0.2em] mt-4">Sin datos de pago</p>
            </div>
          )}
        </div>

        <div className="bg-neutral-50 rounded-2xl sm:rounded-[2rem] border border-neutral-100 p-4 sm:p-8 space-y-3 sm:space-y-4">
          <h4 className="text-[9px] sm:text-[10px] font-black text-neutral-900 uppercase tracking-widest">Acciones Rápidas</h4>
          <button className="w-full py-3 sm:py-4 bg-white border border-neutral-200 rounded-xl sm:rounded-2xl text-[9px] sm:text-[10px] font-black text-neutral-900 uppercase tracking-widest hover:bg-neutral-900 hover:text-white transition-all shadow-sm flex items-center justify-center gap-3">
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"><path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/><polyline points="14 2 14 8 20 8"/></svg>
            Generar Factura
          </button>
          <button className="w-full py-3 sm:py-4 bg-white border border-neutral-200 rounded-xl sm:rounded-2xl text-[9px] sm:text-[10px] font-black text-neutral-900 uppercase tracking-widest hover:bg-neutral-900 hover:text-white transition-all shadow-sm flex items-center justify-center gap-3">
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" x2="12" y1="15" y2="3"/></svg>
            Reporte Gral. PDF
          </button>
        </div>
      </div>
    </div>
  );
};

export default Graficas;
