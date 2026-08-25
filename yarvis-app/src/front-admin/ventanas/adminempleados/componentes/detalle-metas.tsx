// Card "Metas y bonos" del detalle: avance contra meta mensual,
// bono actual y ROI individual del empleado.
import { MorphIcon } from "morphicons/react";
import type { EmpleadoProfile } from "../../../../services/empleado";
import { ICONO_TARGET, ICONO_ALERTA, ICONO_CHECK } from "../../../../components/ui";
import { formatMoney, type EmpleadoVentas } from "../utilidades/helpers";

interface DetalleMetasProps {
  empleado: EmpleadoProfile;
  ventasDetalle: EmpleadoVentas;
}

export const DetalleMetas = ({ empleado, ventasDetalle }: DetalleMetasProps) => {
  return (
    <div className="bg-white rounded-[2.5rem] border border-neutral-200 p-6 sm:p-8 space-y-6">
      <div className="flex items-center gap-3">
        <div className="w-10 h-10 bg-neutral-950 text-neutral-50 rounded-xl flex items-center justify-center">
          <MorphIcon icon={ICONO_TARGET} size={16} strokeWidth={2.2} spring="smooth" />
        </div>
        <div>
          <h4 className="text-xs font-black text-neutral-900 uppercase tracking-widest">Metas y bonos</h4>
          <p className="text-[9px] text-neutral-400 font-bold uppercase tracking-wider">Avance contra meta mensual</p>
        </div>
      </div>

      <div className="p-4 bg-neutral-50 rounded-2xl">
        <div className="flex justify-between items-center mb-3">
          <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest">Meta mensual</p>
          <p className="text-[10px] font-black text-neutral-900">{formatMoney(empleado.meta_mensual)}</p>
        </div>
        <div className="h-3 rounded-full bg-neutral-200 overflow-hidden">
          <div
            className={`h-full rounded-full transition-all duration-700 ${
              ventasDetalle.total_ventas >= empleado.meta_mensual && empleado.meta_mensual > 0
                ? "bg-emerald-500"
                : "bg-neutral-950"
            }`}
            style={{
              width: `${Math.min(100, empleado.meta_mensual > 0 ? (ventasDetalle.total_ventas / empleado.meta_mensual) * 100 : 0)}%`,
            }}
          />
        </div>
        <p className="text-[9px] font-bold text-neutral-400 mt-2 uppercase tracking-widest">
          {empleado.meta_mensual > 0
            ? `${Math.round((ventasDetalle.total_ventas / empleado.meta_mensual) * 100)}% de la meta`
            : "Sin meta definida"}
        </p>
      </div>

      <div className={`p-4 rounded-2xl flex items-center justify-between ${ventasDetalle.total_ventas >= empleado.meta_mensual && empleado.meta_mensual > 0 ? "bg-emerald-50" : "bg-neutral-50"}`}>
        <span className="text-[9px] font-black text-neutral-500 uppercase tracking-widest">Bono actual</span>
        <span className={`text-sm font-black ${empleado.meta_mensual > 0 && ventasDetalle.total_ventas >= empleado.meta_mensual ? "text-emerald-600" : "text-neutral-900"}`}>
          {formatMoney(empleado.bono)}
        </span>
      </div>

      {/* ROI individual */}
      <div className={`p-4 rounded-2xl border ${ventasDetalle.total_ventas - empleado.salario_semanal < 0 ? "bg-red-50 border-red-200" : "bg-neutral-50 border-neutral-100"}`}>
        <p className="text-[9px] font-black uppercase tracking-widest text-neutral-400">ROI individual</p>
        <p className="text-[11px] font-black uppercase tracking-widest mt-2 flex items-center gap-1.5">
          <span className={`flex items-center gap-1.5 ${ventasDetalle.total_ventas - empleado.salario_semanal < 0 ? "text-red-600" : "text-emerald-600"}`}>
            <MorphIcon
              icon={ventasDetalle.total_ventas - empleado.salario_semanal < 0 ? ICONO_ALERTA : ICONO_CHECK}
              size={13}
              strokeWidth={2.5}
              spring="snappy"
              reducedMotion="user"
            />
            {ventasDetalle.total_ventas - empleado.salario_semanal < 0 ? "Pérdida detectada" : "Renta positivo"}
          </span>
          <span className="text-neutral-900 ml-auto text-sm">{formatMoney(ventasDetalle.total_ventas - empleado.salario_semanal)}</span>
        </p>
      </div>
    </div>
  );
};
