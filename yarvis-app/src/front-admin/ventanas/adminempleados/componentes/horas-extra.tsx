// Sección "Horas Extras Indefinidas": historial completo de días
// trabajados fuera de horario, con filas expandibles por día.
import { MorphIcon } from "morphicons/react";
import { MiniBarraDia, type DiaExtra } from "../../../../components/turno";
import { ICONO_RELOJ } from "../../../../components/ui";

// Minutos extra a formato legible "Xh Ym".
const fmtMinExtra = (m: number) => `${Math.floor(m / 60)}h ${m % 60}m`;

interface HorasExtraProps {
  extrasDetalle: DiaExtra[];
  expandidasAdmin: Set<string>;
  onToggle: (fecha: string) => void;
}

export const HorasExtra = ({ extrasDetalle, expandidasAdmin, onToggle }: HorasExtraProps) => {
  return (
    <div className="bg-white rounded-[2.5rem] border border-neutral-200 p-6 sm:p-8 shadow-sm">
      <div className="flex items-center gap-3 mb-6">
        <div className="w-10 h-10 bg-neutral-950 rounded-2xl flex items-center justify-center shadow-md shrink-0">
          <MorphIcon icon={ICONO_RELOJ} size={16} strokeWidth={2.2} spring="smooth" />
        </div>
        <div>
          <h4 className="text-xs font-black text-neutral-900 uppercase tracking-widest">Horas Extras Indefinidas</h4>
          <p className="text-[9px] text-neutral-400 font-bold uppercase tracking-wider">Cada día que trabajó fuera de su horario</p>
        </div>
        <span className="ml-auto px-3 py-1 bg-neutral-950 text-white text-[9px] font-black rounded-lg uppercase tracking-widest">
          {extrasDetalle.length} {extrasDetalle.length === 1 ? "DÍA" : "DÍAS"}
        </span>
      </div>

      <div className={`space-y-2 pr-1 custom-scrollbar ${extrasDetalle.length > 8 ? "max-h-[420px] overflow-y-auto" : ""}`}>
        {extrasDetalle.map((d) => {
          const totalExtra = d.extra_pre_min + d.extra_post_min;
          const abierta = expandidasAdmin.has(d.fecha);
          return (
            <div key={d.fecha} className={`rounded-2xl border transition-all ${abierta ? "border-neutral-300 bg-neutral-50/60" : "border-neutral-100 hover:border-neutral-200"}`}>
              <button
                onClick={() => onToggle(d.fecha)}
                className="w-full flex items-center gap-3 p-3.5 text-left"
              >
                <div className="shrink-0 w-24">
                  <p className="text-[11px] font-black text-neutral-900 uppercase leading-none">{d.dia_label}</p>
                  <p className="text-[9px] font-bold text-neutral-400 mt-1">{d.fecha.slice(5).split("-").reverse().join("/")}</p>
                </div>

                <div className="flex-1 flex items-center gap-3 min-w-0" title={`Entrada oficial ${d.entrada_oficial} · Salida ${d.salida_oficial}`}>
                  <span className="text-[9px] font-black uppercase tracking-widest whitespace-nowrap text-neutral-500">{d.primer_login}</span>
                  <MiniBarraDia d={d} />
                  <span className="text-[9px] font-black uppercase tracking-widest whitespace-nowrap text-emerald-600">{d.ultimo_login}</span>
                </div>

                <span className="px-2 py-0.5 bg-emerald-100 text-emerald-600 rounded-lg text-[8px] font-black uppercase tracking-widest whitespace-nowrap shrink-0">
                  +{fmtMinExtra(totalExtra)}
                </span>

                <MorphIcon
                  icon={ICONO_RELOJ}
                  size={13}
                  strokeWidth={2.4}
                  spring="snappy"
                  reducedMotion="user"
                  className={`shrink-0 text-neutral-300 transition-transform duration-200 ${abierta ? "rotate-180" : ""}`}
                />
              </button>

              {abierta && (
                <div className="px-4 pb-4 animate-in fade-in slide-in-from-top-1 duration-200">
                  <div className="bg-white rounded-xl border border-neutral-200 p-4 text-[10px] font-bold text-neutral-500 grid grid-cols-1 sm:grid-cols-2 gap-x-6 gap-y-2">
                    <p><span className="font-black text-neutral-400 uppercase tracking-wider">Llegó:</span> <span className="text-neutral-900 font-black">{d.primer_login}</span>{d.extra_pre_min > 0 && <> · <span className="text-emerald-600 font-black">{fmtMinExtra(d.extra_pre_min)} antes de su entrada</span></>}</p>
                    <p><span className="font-black text-neutral-400 uppercase tracking-wider">Se fue:</span> <span className="text-neutral-900 font-black">{d.ultimo_login}</span>{d.extra_post_min > 0 && <> · <span className="text-emerald-600 font-black">{fmtMinExtra(d.extra_post_min)} después de su salida</span></>}</p>
                    <p><span className="font-black text-neutral-400 uppercase tracking-wider">Horario ese día:</span> <span className="text-neutral-900 font-black">{d.entrada_oficial} — {d.salida_oficial}</span></p>
                    <p><span className="font-black text-neutral-400 uppercase tracking-wider">Total trabajado:</span> <span className="text-neutral-900 font-black">{fmtMinExtra(d.trabajo_min)}</span> · <span className="text-emerald-600 font-black">Extra total: {fmtMinExtra(totalExtra)}</span></p>
                  </div>
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
};
