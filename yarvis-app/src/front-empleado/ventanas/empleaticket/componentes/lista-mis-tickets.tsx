// ═══════════════════════════════════════════════════════════════════════════
// LISTA MIS TICKETS — Historial propio del periodo (click → modal).
// Tarjeta gemela al historial de adminticket/tickets.tsx pero filtrada
// por cajero_id = sesión. Cada renglón es un <button> accesible.
// ═══════════════════════════════════════════════════════════════════════════

import { useEffect, useState } from "react";
import {
  moneda, RangoDropdown,
} from "../../../../front-admin/ventanas/adminventas/graficas/controles";
import { RANGOS_MIS, rangoMisADias, type RangoSel } from "./kpis-mis-tickets";
import { obtenerMisTickets, type MiTicket } from "../../../../services/misventas";
import { reportarError } from "../../../../services/tauri";

interface ListaMisTicketsProps {
  onVer: (t: MiTicket) => void;
}

// Rango propio: solo afecta a esta lista (igual que en adminventas).
// Arranca en Todos: el historial completo es lo que siempre se ve.
const ListaMisTickets = ({ onVer }: ListaMisTicketsProps) => {
  const [rango, setRangoRaw] = useState<RangoSel>({ v: "todos" });
  const setRango = (v: string, c?: number) => setRangoRaw({ v, c });
  const dias = rangoMisADias(rango.v, rango.c);
  const [tickets, setTickets] = useState<MiTicket[] | null>(null);

  useEffect(() => {
    let viva = true;
    setTickets(null);
    obtenerMisTickets(dias, 100, 0)
      .then((t) => {
        if (viva) setTickets(t);
      })
      .catch((e) => reportarError("No se pudieron cargar tus tickets", e));
    return () => {
      viva = false;
    };
  }, [dias]);

  return (
    <div className="bg-white rounded-[2.5rem] border border-neutral-200 p-4 sm:p-8 shadow-sm">
      <h3 className="text-xs font-black text-neutral-900 uppercase tracking-widest flex items-center gap-2 mb-8">
        <div className="w-1.5 h-4 bg-neutral-900 rounded-full"></div>
        Mis tickets
        {tickets !== null && (
          <span className="px-2 py-1 bg-neutral-100 text-neutral-500 text-[9px] font-black rounded-lg">
            {tickets.length}
          </span>
        )}
        <span className="ml-auto">
          <RangoDropdown valor={rango.v} opciones={RANGOS_MIS} onChange={setRango} />
        </span>
      </h3>
      <div className="h-64 flex flex-col border-2 border-dashed border-neutral-100 rounded-3xl bg-neutral-50/50 overflow-hidden">
        {tickets === null ? (
          <div className="flex-1 flex items-center justify-center">
            <div className="w-8 h-8 rounded-full border-2 border-neutral-900 border-t-transparent animate-spin" />
          </div>
        ) : tickets.length === 0 ? (
          <div className="flex-1 flex flex-col items-center justify-center">
            <div className="w-12 h-12 bg-white rounded-2xl flex items-center justify-center shadow-sm text-lg mb-4">🎫</div>
            <p className="text-[10px] font-black text-neutral-300 uppercase tracking-widest italic">
              No tienes tickets en este periodo
            </p>
          </div>
        ) : (
          <div className="flex-1 px-4 py-4 space-y-2 overflow-y-auto custom-scrollbar">
            {tickets.map((t) => (
              <button
                key={t.id}
                onClick={() => onVer(t)}
                className="w-full flex items-center justify-between p-3 bg-white rounded-xl border border-neutral-100 shadow-sm hover:border-neutral-900 hover:shadow-md transition-all text-left"
              >
                <div className="text-left min-w-0">
                  <p className="text-[10px] font-black text-neutral-900 uppercase truncate">
                    Ticket {t.folio_ticket || `#${t.id}`}
                  </p>
                  <p className="text-[8px] text-neutral-400 font-bold uppercase truncate">
                    {t.fecha} · {t.metodo_pago}
                  </p>
                </div>
                <span className="flex items-center gap-2 shrink-0">
                  <span className="text-xs font-black text-neutral-900">{moneda(t.total)}</span>
                  <span className="text-neutral-300">›</span>
                </span>
              </button>
            ))}
          </div>
        )}
      </div>
      <p className="text-[10px] text-neutral-400 font-bold mt-4">Toca un ticket para ver su detalle</p>
    </div>
  );
};

export default ListaMisTickets;
