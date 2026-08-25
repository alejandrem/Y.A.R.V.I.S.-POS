// Card "Cortes de caja" del detalle: listado scrollable de los
// últimos cortes con montos y estado.
import { MorphIcon } from "morphicons/react";
import { ICONO_CALENDARIO } from "../../../../components/ui";
import { formatDate, formatMoney, type CorteEmpleado } from "../utilidades/helpers";

interface DetalleCortesProps {
  cortes: CorteEmpleado[];
}

export const DetalleCortes = ({ cortes }: DetalleCortesProps) => {
  return (
    <div className="bg-white rounded-[2.5rem] border border-neutral-200 p-6 sm:p-8 space-y-5">
      <div className="flex items-center gap-3">
        <div className="w-10 h-10 bg-neutral-950 text-neutral-50 rounded-xl flex items-center justify-center">
          <MorphIcon icon={ICONO_CALENDARIO} size={16} strokeWidth={2.2} spring="smooth" />
        </div>
        <div>
          <h4 className="text-xs font-black text-neutral-900 uppercase tracking-widest">Cortes de caja</h4>
          <p className="text-[9px] text-neutral-400 font-bold uppercase tracking-wider">Últimos {cortes.length} cortes</p>
        </div>
      </div>

      <div className="space-y-2.5 max-h-72 overflow-y-auto custom-scrollbar pr-1">
        {cortes.length > 0 ? (
          cortes.map((c, idx) => (
            <div key={idx} className="p-3.5 bg-neutral-50 rounded-2xl flex items-center justify-between">
              <div>
                <p className="text-[10px] font-black text-neutral-900 uppercase">{formatDate(c.fecha_apertura)}</p>
                <p className="text-[9px] font-bold text-neutral-400 mt-0.5">Inicial: {formatMoney(c.monto_inicial)}</p>
              </div>
              <div className="text-right">
                <p className="text-xs font-black text-neutral-900">{formatMoney(c.total_ventas)}</p>
                <span
                  className={`inline-block mt-1 px-2 py-0.5 text-[8px] font-black uppercase rounded-md ${
                    c.estado === "abierto" ? "bg-emerald-50 text-emerald-600" : "bg-neutral-200 text-neutral-500"
                  }`}
                >
                  {c.estado}
                </span>
              </div>
            </div>
          ))
        ) : (
          <p className="py-10 text-center text-[10px] font-black text-neutral-300 uppercase tracking-widest italic">
            Sin cortes registrados
          </p>
        )}
      </div>
    </div>
  );
};
