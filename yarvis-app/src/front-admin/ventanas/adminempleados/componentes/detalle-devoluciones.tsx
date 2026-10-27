import { MorphIcon } from "morphicons/react";
import { ICONO_DOCUMENTO } from "../../../../components/ui";
import { formatMoney } from "../utilidades/helpers";
import type { DevolucionRegistrada } from "../../../../services/devoluciones";

interface DetalleDevolucionesProps {
  devoluciones: DevolucionRegistrada[];
}

export const DetalleDevoluciones = ({ devoluciones }: DetalleDevolucionesProps) => (
  <div className="bg-white rounded-[2.5rem] border border-neutral-200 p-6 sm:p-8 space-y-5">
    <div className="flex items-center gap-3">
      <div className="w-10 h-10 bg-neutral-950 text-neutral-50 rounded-xl flex items-center justify-center">
        <MorphIcon icon={ICONO_DOCUMENTO} size={16} strokeWidth={2.2} spring="smooth" />
      </div>
      <div>
        <h4 className="text-xs font-black text-neutral-900 uppercase tracking-widest">Devoluciones</h4>
        <p className="text-[9px] text-neutral-400 font-bold uppercase tracking-wider">{devoluciones.length} tickets devueltos</p>
      </div>
    </div>
    <div className="space-y-2.5 max-h-72 overflow-y-auto custom-scrollbar pr-1">
      {devoluciones.length ? devoluciones.map((d) => (
        <article key={d.id} className="p-3.5 bg-neutral-50 rounded-2xl flex items-center justify-between gap-3">
          <div className="min-w-0">
            <p className="text-[10px] font-black text-neutral-900 uppercase truncate">{d.folio_ticket}</p>
            <p className="text-[9px] font-bold text-neutral-400 mt-1">{d.creado_en}</p>
            <span className={`inline-block mt-1 text-[8px] font-black uppercase ${d.estado_movimiento === "pendiente" ? "text-amber-600" : "text-emerald-600"}`}>
              {d.estado_movimiento === "pendiente" ? "Egreso pendiente de corte" : "Egreso registrado"}
            </span>
          </div>
          <span className="text-xs font-black text-neutral-900 shrink-0">{formatMoney(d.monto)}</span>
        </article>
      )) : (
        <p className="py-10 text-center text-[10px] font-black text-neutral-300 uppercase tracking-widest italic">Sin devoluciones registradas</p>
      )}
    </div>
  </div>
);
