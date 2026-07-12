import { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";

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

const Tickets = () => {
  const [selectedRange, setSelectedRange] = useState("7 DÍAS");
  const [predictionRange, setPredictionRange] = useState("7 DÍAS");
  
  const [tickets, setTickets] = useState<TicketDb[]>([]);
  const [cortes, setCortes] = useState<CorteDb[]>([]);
  
  const [histStartDate, setHistStartDate] = useState("");
  const [histEndDate, setHistEndDate] = useState("");

  const fetchData = async () => {
    try {
      const resTickets = await invoke("get_tickets");
      setTickets(resTickets as TicketDb[]);
      
      const resCortes = await invoke("get_cortes");
      setCortes(resCortes as CorteDb[]);
    } catch (error) {
      console.error("Error al cargar datos:", error);
    }
  };

  useEffect(() => {
    fetchData();
  }, []);

  // Filtrado real por rango de fechas
  const filteredTickets = useMemo(() => {
    if (selectedRange === "PERSONALIZADO") {
      if (!histStartDate || !histEndDate) return tickets;
      const start = new Date(histStartDate);
      const end = new Date(histEndDate);
      return tickets.filter(t => {
        const d = new Date(t.fecha);
        return d >= start && d <= end;
      });
    }

    const daysMap: Record<string, number> = {
      "7 DÍAS": 7,
      "15 DÍAS": 15,
      "1 MES": 30,
      "6 MESES": 180,
      "1 AÑO": 365,
    };

    const days = daysMap[selectedRange] ?? 7;
    const cutoff = new Date();
    cutoff.setDate(cutoff.getDate() - days);

    return tickets.filter(t => new Date(t.fecha) >= cutoff);
  }, [tickets, selectedRange, histStartDate, histEndDate]);

  const filteredCortes = useMemo(() => {
    if (selectedRange === "PERSONALIZADO") {
      if (!histStartDate || !histEndDate) return cortes;
      const start = new Date(histStartDate);
      const end = new Date(histEndDate);
      return cortes.filter(c => {
        const d = new Date(c.fecha);
        return d >= start && d <= end;
      });
    }

    const daysMap: Record<string, number> = {
      "7 DÍAS": 7,
      "15 DÍAS": 15,
      "1 MES": 30,
      "6 MESES": 180,
      "1 AÑO": 365,
    };

    const days = daysMap[selectedRange] ?? 7;
    const cutoff = new Date();
    cutoff.setDate(cutoff.getDate() - days);

    return cortes.filter(c => new Date(c.fecha) >= cutoff);
  }, [cortes, selectedRange, histStartDate, histEndDate]);

  // Métricas basadas en datos filtrados
  const displayMetrics = useMemo(() => {
    const avgTicket = filteredTickets.length > 0
      ? filteredTickets.reduce((sum, t) => sum + t.total, 0) / filteredTickets.length
      : 0;

    const uniqueDates = new Set(filteredTickets.map(t => t.fecha.split(' ')[0]));
    const ticketsPerDay = uniqueDates.size > 0
      ? Math.round(filteredTickets.length / uniqueDates.size)
      : 0;

    const avgCorte = filteredCortes.length > 0
      ? filteredCortes.reduce((sum, c) => sum + c.total_ventas, 0) / filteredCortes.length
      : 0;

    return { avgTicket, ticketsPerDay, avgCorte };
  }, [filteredTickets, filteredCortes]);

  return (
    <div className="max-w-6xl mx-auto space-y-10 animate-in fade-in duration-500">
      {/* HEADER DE SECCIÓN */}
      <header className="flex justify-between items-end border-b border-neutral-100 pb-8">
        <div>
          <h2 className="text-3xl font-black text-neutral-900 uppercase tracking-tight mb-1">Tickets y Facturación</h2>
          <p className="text-[10px] font-black text-neutral-400 uppercase tracking-[0.3em]">Control de Impresión y Flujo de Caja</p>
        </div>
        <div className="flex gap-4">
          <div className="text-right">
            <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest leading-none mb-1">Total Tickets</p>
            <span className="text-lg font-black text-neutral-900">{filteredTickets.length}</span>
          </div>
        </div>
      </header>

      {/* MÉTRICAS PRINCIPALES */}
      <div className="grid grid-cols-3 gap-6">
        {[
          { label: "Promedio Venta", value: `$ ${displayMetrics.avgTicket.toFixed(2)}`, info: filteredTickets.length === 0 ? "Esperando ventas..." : `${filteredTickets.length} ventas`, color: "bg-neutral-50" },
          { label: "Tickets / Día", value: `${displayMetrics.ticketsPerDay} TICKETS`, info: filteredTickets.length === 0 ? "Sin actividad" : "Promedio diario", color: "bg-neutral-50" },
          { label: "Promedio Corte", value: `$ ${displayMetrics.avgCorte.toFixed(2)}`, info: filteredCortes.length === 0 ? "Esperando cortes..." : `${filteredCortes.length} cortes`, color: "bg-neutral-900 text-white shadow-2xl shadow-neutral-200" }
        ].map((stat, i) => (
          <div key={i} className={`${stat.color} p-8 rounded-[2rem] border border-neutral-100 relative overflow-hidden group hover:scale-[1.02] transition-all duration-300`}>
            <p className={`text-[10px] font-black uppercase tracking-widest mb-2 ${stat.color.includes('black') ? 'text-neutral-400' : 'text-neutral-400'}`}>{stat.label}</p>
            <h3 className="text-2xl font-black">{stat.value}</h3>
            <span className={`absolute top-8 right-8 text-[8px] font-black px-2 py-1 rounded-lg uppercase ${stat.color.includes('black') ? 'bg-white/10 text-neutral-400' : 'bg-neutral-100 text-neutral-400'}`}>
              {stat.info}
            </span>
          </div>
        ))}
      </div>

      {/* HISTORIALES */}
      <div className="grid grid-cols-2 gap-8">
        <div className="bg-white rounded-[2.5rem] border border-neutral-200 p-8 shadow-sm">
          <h3 className="text-xs font-black text-neutral-900 uppercase tracking-widest flex items-center gap-2 mb-8">
            <div className="w-1.5 h-4 bg-neutral-900 rounded-full"></div>
            Historial de Tickets
          </h3>
          <div className="h-64 flex flex-col items-center justify-center border-2 border-dashed border-neutral-100 rounded-3xl bg-neutral-50/50">
            {filteredTickets.length > 0 ? (
              <div className="w-full px-4 space-y-2 overflow-y-auto">
                {filteredTickets.map((t) => (
                  <div key={t.id} className="flex items-center justify-between p-3 bg-white rounded-xl border border-neutral-100 shadow-sm">
                    <div className="text-left">
                      <p className="text-[10px] font-black text-neutral-900 uppercase">TICKET #{t.id}</p>
                      <p className="text-[8px] text-neutral-400 font-bold uppercase">{t.fecha}</p>
                    </div>
                    <p className="text-xs font-black text-neutral-900">${t.total.toFixed(2)}</p>
                  </div>
                ))}
              </div>
            ) : (
              <>
                <div className="w-12 h-12 bg-white rounded-2xl flex items-center justify-center shadow-sm text-lg mb-4">🎫</div>
                <p className="text-[10px] font-black text-neutral-300 uppercase tracking-widest italic">No hay tickets registrados aún</p>
              </>
            )}
          </div>
        </div>

        <div className="bg-white rounded-[2.5rem] border border-neutral-200 p-8 shadow-sm">
          <h3 className="text-xs font-black text-neutral-900 uppercase tracking-widest flex items-center gap-2 mb-8">
            <div className="w-1.5 h-4 bg-neutral-900 rounded-full"></div>
            Historial de Cortes
          </h3>
          <div className="h-64 flex flex-col items-center justify-center border-2 border-dashed border-neutral-100 rounded-3xl bg-neutral-50/50">
            {filteredCortes.length > 0 ? (
              <div className="w-full px-4 space-y-2 overflow-y-auto">
                {filteredCortes.map((c) => (
                  <div key={c.id} className="flex items-center justify-between p-3 bg-white rounded-xl border border-neutral-100 shadow-sm">
                    <div className="text-left">
                      <p className="text-[10px] font-black text-neutral-900 uppercase">CORTE #{c.id}</p>
                      <p className="text-[8px] text-neutral-400 font-bold uppercase">{c.fecha}</p>
                    </div>
                    <p className="text-xs font-black text-neutral-900">${c.total_ventas.toFixed(2)}</p>
                  </div>
                ))}
              </div>
            ) : (
              <>
                <div className="w-12 h-12 bg-white rounded-2xl flex items-center justify-center shadow-sm text-lg mb-4">💰</div>
                <p className="text-[10px] font-black text-neutral-300 uppercase tracking-widest italic">No hay cortes realizados aún</p>
              </>
            )}
          </div>
        </div>
      </div>

      {/* SECCIÓN DE GRÁFICAS HISTÓRICAS */}
      <div className="space-y-8 pt-10 border-t border-neutral-100">
        <div className="flex flex-col gap-6">
          <div className="flex items-center justify-between">
            <h3 className="text-xl font-black text-neutral-900 uppercase tracking-tight">Análisis de Rendimiento</h3>
            <div className="flex bg-neutral-100 p-1 rounded-xl flex-wrap gap-1">
              {["7 DÍAS", "15 DÍAS", "1 MES", "6 MESES", "1 AÑO", "PERSONALIZADO"].map(r => (
                <button 
                  key={r} 
                  onClick={() => setSelectedRange(r)}
                  className={`px-3 py-2 text-[8px] font-black rounded-lg transition-all ${selectedRange === r ? 'bg-white shadow-sm text-neutral-900' : 'text-neutral-400 hover:text-neutral-600'}`}
                >
                  {r}
                </button>
              ))}
            </div>
          </div>

          {selectedRange === "PERSONALIZADO" && (
            <div className="flex items-center gap-6 bg-neutral-900 p-6 rounded-3xl self-end shadow-2xl shadow-neutral-200">
              <div className="flex flex-col gap-1.5">
                <label className="text-[9px] font-black text-neutral-500 uppercase tracking-widest ml-1">Fecha Inicial</label>
                <input 
                  type="date"
                  value={histStartDate}
                  onChange={(e) => setHistStartDate(e.target.value)}
                  className="bg-neutral-800 border border-neutral-700 text-white rounded-xl px-4 py-2.5 text-xs font-bold focus:outline-none focus:border-white w-40 transition-all"
                />
              </div>
              <div className="h-8 w-px bg-neutral-800 mt-4"></div>
              <div className="flex flex-col gap-1.5">
                <label className="text-[9px] font-black text-neutral-500 uppercase tracking-widest ml-1">Fecha Final</label>
                <input 
                  type="date"
                  value={histEndDate}
                  onChange={(e) => setHistEndDate(e.target.value)}
                  className="bg-neutral-800 border border-neutral-700 text-white rounded-xl px-4 py-2.5 text-xs font-bold focus:outline-none focus:border-white w-40 transition-all"
                />
              </div>
            </div>
          )}
        </div>

        <div className="grid grid-cols-2 gap-8">
          <div className="bg-neutral-50 rounded-[2.5rem] border border-neutral-100 p-8 h-80 flex flex-col items-center justify-center relative overflow-hidden">
             <div className="absolute top-6 left-8">
                <p className="text-[10px] font-black text-neutral-900 uppercase tracking-widest">Rendimiento de Tickets</p>
                <p className="text-[8px] text-neutral-400 font-bold uppercase mt-1">Mostrando: {filteredTickets.length} tickets</p>
             </div>
             <div className="text-center space-y-4">
                <div className="w-16 h-16 bg-white rounded-2xl flex items-center justify-center mx-auto shadow-sm text-xl">📊</div>
                <p className="text-[10px] font-black text-neutral-300 uppercase tracking-[0.2em]">{filteredTickets.length > 0 ? "Listo para graficar" : "Esperando tickets..."}</p>
             </div>
          </div>

          <div className="bg-neutral-50 rounded-[2.5rem] border border-neutral-100 p-8 h-80 flex flex-col items-center justify-center relative overflow-hidden">
             <div className="absolute top-6 left-8">
                <p className="text-[10px] font-black text-neutral-900 uppercase tracking-widest">Flujo de Cortes de Caja</p>
                <p className="text-[8px] text-neutral-400 font-bold uppercase mt-1">Mostrando: {filteredCortes.length} cortes</p>
             </div>
             <div className="text-center space-y-4">
                <div className="w-16 h-16 bg-white rounded-2xl flex items-center justify-center mx-auto shadow-sm text-xl">📉</div>
                <p className="text-[10px] font-black text-neutral-300 uppercase tracking-[0.2em]">{filteredCortes.length > 0 ? "Listo para graficar" : "Esperando cortes..."}</p>
             </div>
          </div>
        </div>
      </div>

      {/* PREDICCIÓN Y ACCIONES */}
      <div className="grid grid-cols-3 gap-8 pb-12">
        <div className="col-span-2 bg-neutral-900 rounded-[2.5rem] p-10 shadow-2xl relative overflow-hidden h-[450px] flex flex-col">
          <div className="relative z-10 flex flex-col gap-6">
            <div className="flex items-center justify-between">
              <div>
                <h3 className="text-lg font-black text-white uppercase tracking-tight">Predicción de Flujo de Caja</h3>
                <p className="text-[10px] text-neutral-500 font-black uppercase tracking-[0.2em] mt-1">Modelo Prophet (Próximos días)</p>
              </div>
              <div className="flex bg-white/5 p-1 rounded-xl gap-1">
                {["7D", "15D", "6M", "1A"].map(r => (
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
             <div className="w-full h-px bg-white/5 absolute top-1/2"></div>
             <div className="text-center space-y-4">
                <div className="w-20 h-20 bg-white/5 rounded-[2rem] flex items-center justify-center mx-auto text-3xl opacity-20">🔮</div>
                <p className="text-[11px] font-black text-white uppercase tracking-[0.3em] opacity-40">Sin datos para predecir</p>
             </div>
          </div>
          
          <div className="absolute -bottom-10 -right-10 w-64 h-64 bg-white/5 rounded-full blur-3xl pointer-events-none"></div>
        </div>

        <div className="space-y-6">
          <div className="bg-white rounded-[2rem] border border-neutral-100 p-8 h-56 relative overflow-hidden flex flex-col">
             <h4 className="text-[10px] font-black text-neutral-400 uppercase tracking-widest mb-4">Formas de Pago</h4>
             <div className="flex-1 flex items-center justify-center relative">
                <div className="w-24 h-24 rounded-full border-4 border-neutral-50 flex items-center justify-center">
                   <div className="w-16 h-16 rounded-full border-2 border-neutral-50 border-t-neutral-100"></div>
                </div>
                <p className="absolute text-[8px] font-black text-neutral-300 uppercase tracking-tighter">Sin datos</p>
             </div>
          </div>

          <div className="bg-neutral-50 rounded-[2rem] border border-neutral-100 p-8 space-y-4">
             <h4 className="text-[10px] font-black text-neutral-900 uppercase tracking-widest mb-2">Acciones Rápidas</h4>
             <button className="w-full py-4 bg-white border border-neutral-200 rounded-2xl text-[10px] font-black text-neutral-900 uppercase tracking-widest hover:bg-neutral-900 hover:text-white transition-all shadow-sm flex items-center justify-center gap-3">
                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"><path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/><polyline points="14 2 14 8 20 8"/></svg>
                Generar Factura
             </button>
             <button className="w-full py-4 bg-white border border-neutral-200 rounded-2xl text-[10px] font-black text-neutral-900 uppercase tracking-widest hover:bg-neutral-900 hover:text-white transition-all shadow-sm flex items-center justify-center gap-3">
                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" x2="12" y1="15" y2="3"/></svg>
                Reporte Gral. PDF
             </button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Tickets;
