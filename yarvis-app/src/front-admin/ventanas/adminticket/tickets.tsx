import { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import Graficas from "./graficas";

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
  const [selectedRange, setSelectedRange] = useState("1 AÑO");
  
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
      "TODOS": 99999,
      "7 DÍAS": 7,
      "15 DÍAS": 15,
      "1 MES": 30,
      "6 MESES": 180,
      "1 AÑO": 365,
    };

    const days = daysMap[selectedRange] ?? 99999;
    const cutoff = new Date();
    cutoff.setDate(cutoff.getDate() - days);

    if (selectedRange === "TODOS") return tickets;
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
      "TODOS": 99999,
      "7 DÍAS": 7,
      "15 DÍAS": 15,
      "1 MES": 30,
      "6 MESES": 180,
      "1 AÑO": 365,
    };

    const days = daysMap[selectedRange] ?? 99999;
    const cutoff = new Date();
    cutoff.setDate(cutoff.getDate() - days);

    if (selectedRange === "TODOS") return cortes;
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
      <div className="grid grid-cols-1 sm:grid-cols-3 gap-6">
        {[
          { label: "Promedio Venta", value: `$ ${displayMetrics.avgTicket.toFixed(2)}`, info: filteredTickets.length === 0 ? "Esperando ventas..." : `${filteredTickets.length} ventas`, color: "bg-neutral-50" },
          { label: "Tickets / Día", value: `${displayMetrics.ticketsPerDay} TICKETS`, info: filteredTickets.length === 0 ? "Sin actividad" : "Promedio diario", color: "bg-neutral-50" },
          { label: "Promedio Corte", value: `$ ${displayMetrics.avgCorte.toFixed(2)}`, info: filteredCortes.length === 0 ? "Esperando cortes..." : `${filteredCortes.length} cortes`, color: "bg-neutral-900 text-white shadow-2xl shadow-neutral-200" }
        ].map((stat, i) => (
          <div key={i} className={`${stat.color} p-4 sm:p-8 rounded-2xl sm:rounded-[2rem] border border-neutral-100 relative overflow-hidden group hover:scale-[1.02] transition-all duration-300`}>
            <p className={`text-[9px] sm:text-[10px] font-black uppercase tracking-widest mb-2 ${stat.color.includes('black') ? 'text-neutral-400' : 'text-neutral-400'}`}>{stat.label}</p>
            <h3 className="text-xl sm:text-2xl font-black">{stat.value}</h3>
            <span className={`absolute top-4 sm:top-8 right-4 sm:right-8 text-[7px] sm:text-[8px] font-black px-1.5 sm:px-2 py-0.5 sm:py-1 rounded-lg uppercase ${stat.color.includes('black') ? 'bg-white/10 text-neutral-400' : 'bg-neutral-100 text-neutral-400'}`}>
              {stat.info}
            </span>
          </div>
        ))}
      </div>

      {/* HISTORIALES */}
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 sm:gap-8">
        <div className="bg-white rounded-2xl sm:rounded-[2.5rem] border border-neutral-200 p-4 sm:p-8 shadow-sm">
          <h3 className="text-xs font-black text-neutral-900 uppercase tracking-widest flex items-center gap-2 mb-4 sm:mb-8">
            <div className="w-1.5 h-4 bg-neutral-900 rounded-full"></div>
            Historial de Tickets
          </h3>
          <div className="h-48 sm:h-64 flex flex-col items-center justify-center border-2 border-dashed border-neutral-100 rounded-2xl sm:rounded-3xl bg-neutral-50/50">
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

        <div className="bg-white rounded-2xl sm:rounded-[2.5rem] border border-neutral-200 p-4 sm:p-8 shadow-sm">
          <h3 className="text-xs font-black text-neutral-900 uppercase tracking-widest flex items-center gap-2 mb-4 sm:mb-8">
            <div className="w-1.5 h-4 bg-neutral-900 rounded-full"></div>
            Historial de Cortes
          </h3>
          <div className="h-48 sm:h-64 flex flex-col items-center justify-center border-2 border-dashed border-neutral-100 rounded-2xl sm:rounded-3xl bg-neutral-50/50">
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

      {/* SECCIÓN DE GRÁFICAS (Rendimiento, Cortes, Predicción) */}
      <div className="flex items-center justify-between mb-6">
        <h3 className="text-xl font-black text-neutral-900 uppercase tracking-tight">Análisis de Rendimiento</h3>
        <div className="flex bg-neutral-100 p-1 rounded-xl flex-wrap gap-1">
          {["TODOS", "7 DÍAS", "15 DÍAS", "1 MES", "6 MESES", "1 AÑO", "PERSONALIZADO"].map(r => (
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
        <div className="flex items-center gap-6 bg-neutral-900 p-6 rounded-3xl mb-8 self-end shadow-2xl shadow-neutral-200">
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

      <Graficas filteredTickets={filteredTickets} filteredCortes={filteredCortes} />
    </div>
  );
};

export default Tickets;
