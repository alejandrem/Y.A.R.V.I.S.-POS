// ═══════════════════════════════════════════════════════════════════════════
// PANEL MIS CORTES — Historial de cortes X/Z del operador actual.
// Gemelo a ListaMisTickets: misma tarjeta, mismo RangoDropdown, datos de
// `get_mis_cortes` (operator-scoped por session.user_id). Solo lectura:
// crear y cerrar siguen en el modal de corte (F3) y Finanzas (admin).
// ═══════════════════════════════════════════════════════════════════════════

import { useEffect, useState } from "react";
import {
  moneda, RangoDropdown,
} from "../../../../front-admin/ventanas/adminventas/graficas/controles";
import { RANGOS_MIS, rangoMisADias, type RangoSel } from "./kpis-mis-tickets";
import { obtenerMisCortes, type MiCorte } from "../../../../services/misventas";
import { reportarError } from "../../../../services/tauri";

const etiquetaCorte = (c: MiCorte) =>
  `${c.tipo_corte || "Z"} · ${c.estado}`;

const PanelMisCortes = () => {
  const [rango, setRangoRaw] = useState<RangoSel>({ v: "todos" });
  const setRango = (v: string, c?: number) => setRangoRaw({ v, c });
  const dias = rangoMisADias(rango.v, rango.c);
  const [cortes, setCortes] = useState<MiCorte[] | null>(null);

  useEffect(() => {
    let vivo = true;
    setCortes(null);
    obtenerMisCortes(dias)
      .then((c) => {
        if (vivo) setCortes(c);
      })
      .catch((e) => reportarError("No se pudieron cargar tus cortes", e));
    return () => {
      vivo = false;
    };
  }, [dias]);

  return (
    <div className="bg-white rounded-[2.5rem] border border-neutral-200 p-4 sm:p-8 shadow-sm">
      <h3 className="text-xs font-black text-neutral-900 uppercase tracking-widest flex items-center gap-2 mb-8">
        <div className="w-1.5 h-4 bg-neutral-900 rounded-full"></div>
        Mis cortes
        {cortes !== null && (
          <span className="px-2 py-1 bg-neutral-100 text-neutral-500 text-[9px] font-black rounded-lg">
            {cortes.length}
          </span>
        )}
        <span className="ml-auto">
          <RangoDropdown valor={rango.v} opciones={RANGOS_MIS} onChange={setRango} />
        </span>
      </h3>
      <div className="h-64 flex flex-col border-2 border-dashed border-neutral-100 rounded-3xl bg-neutral-50/50 overflow-hidden">
        {cortes === null ? (
          <div className="flex-1 flex items-center justify-center">
            <div className="w-8 h-8 rounded-full border-2 border-neutral-900 border-t-transparent animate-spin" />
          </div>
        ) : cortes.length === 0 ? (
          <div className="flex-1 flex flex-col items-center justify-center">
            <div className="w-12 h-12 bg-white rounded-2xl flex items-center justify-center shadow-sm text-lg mb-4">💰</div>
            <p className="text-[10px] font-black text-neutral-300 uppercase tracking-widest italic">
              No tienes cortes en este periodo
            </p>
          </div>
        ) : (
          <div className="flex-1 px-4 py-4 space-y-2 overflow-y-auto custom-scrollbar">
            {cortes.map((c) => (
              <div
                key={c.id}
                className="w-full flex items-center justify-between p-3 bg-white rounded-xl border border-neutral-100 shadow-sm text-left"
              >
                <div className="text-left min-w-0">
                  <p className="text-[10px] font-black text-neutral-900 uppercase truncate">
                    Corte #{c.id} — {etiquetaCorte(c)}
                  </p>
                  <p className="text-[8px] text-neutral-400 font-bold uppercase truncate">
                    {c.fecha_cierre ?? c.fecha_apertura ?? "sin fecha"}
                  </p>
                </div>
                <span className="text-xs font-black text-neutral-900 shrink-0">
                  {moneda(c.total_ventas)}
                </span>
              </div>
            ))}
          </div>
        )}
      </div>
      <p className="text-[10px] text-neutral-300 font-bold mt-4">Solo lectura · crear y cerrar sigue en el modal de corte (F3)</p>
    </div>
  );
};

export default PanelMisCortes;
