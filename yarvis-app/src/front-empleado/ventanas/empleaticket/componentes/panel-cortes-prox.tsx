// ═══════════════════════════════════════════════════════════════════════════
// PANEL CORTES PRÓXIMAMENTE — Placeholder con la misma Tarjeta que la
// lista de tickets para no romper el grid de 2 columnas. Trae su propio
// botón de rango (default Todos) para ir igual que los demás bloques;
// cuando exista el comando operator de cortes, aquí va la lista real
// (solo lectura) filtrada por ese rango.
// ═══════════════════════════════════════════════════════════════════════════

import { useState } from "react";
import { RangoDropdown } from "../../../../front-admin/ventanas/adminventas/graficas/controles";
import { RANGOS_MIS, type RangoSel } from "./kpis-mis-tickets";

const PanelCortesProx = () => {
  const [rango, setRangoRaw] = useState<RangoSel>({ v: "todos" });
  const setRango = (v: string, c?: number) => setRangoRaw({ v, c });

  return (
    <div className="bg-white rounded-[2.5rem] border border-neutral-200 p-4 sm:p-8 shadow-sm">
      <h3 className="text-xs font-black text-neutral-900 uppercase tracking-widest flex items-center gap-2 mb-8">
        <div className="w-1.5 h-4 bg-neutral-900 rounded-full"></div>
        Mis cortes
        <span className="ml-auto">
          <RangoDropdown valor={rango.v} opciones={RANGOS_MIS} onChange={setRango} />
        </span>
      </h3>
      <div className="h-64 flex flex-col items-center justify-center border-2 border-dashed border-neutral-100 rounded-3xl bg-neutral-50/50">
        <div className="w-12 h-12 bg-white rounded-2xl flex items-center justify-center shadow-sm text-lg mb-4">💰</div>
        <p className="text-[10px] font-black text-neutral-900 uppercase tracking-widest">Próximamente</p>
        <p className="text-[10px] font-bold text-neutral-400 mt-2">Aquí verás tus cortes</p>
      </div>
      <p className="text-[10px] text-neutral-300 font-bold mt-4">Solo lectura · crear y cerrar sigue en Finanzas (admin)</p>
    </div>
  );
};

export default PanelCortesProx;
