// Card "Ventas" del grid del detalle: desglose de vendido,
// canceladas y con descuento del empleado seleccionado.
import { MorphIcon } from "morphicons/react";
import { ICONO_TRENDING } from "../../../../components/ui";
import { formatMoney, type EmpleadoVentas } from "../utilidades/helpers";

interface DetalleVentasProps {
  ventasDetalle: EmpleadoVentas;
}

export const DetalleVentas = ({ ventasDetalle }: DetalleVentasProps) => {
  return (
    <div className="bg-white rounded-[2.5rem] border border-neutral-200 p-6 sm:p-8 space-y-5">
      <div className="flex items-center gap-3">
        <div className="w-10 h-10 bg-neutral-950 text-neutral-50 rounded-xl flex items-center justify-center">
          <MorphIcon icon={ICONO_TRENDING} size={16} strokeWidth={2.2} spring="smooth" />
        </div>
        <div>
          <h4 className="text-xs font-black text-neutral-900 uppercase tracking-widest">Ventas</h4>
          <p className="text-[9px] text-neutral-400 font-bold uppercase tracking-wider">Desglose del cajero</p>
        </div>
      </div>
      <div className="space-y-3">
        <div className="p-4 bg-neutral-50 rounded-2xl">
          <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest">Vendido</p>
          <p className="text-xl font-black text-neutral-900 mt-1">{formatMoney(ventasDetalle.total_ventas)}</p>
          <p className="text-[9px] font-bold text-neutral-400 mt-0.5">{ventasDetalle.ticket_count} tickets</p>
        </div>
        <div className="p-4 bg-neutral-50 rounded-2xl">
          <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest">Canceladas</p>
          <p className="text-xl font-black text-neutral-900 mt-1">{formatMoney(ventasDetalle.ventas_canceladas)}</p>
          <p className="text-[9px] font-bold text-neutral-400 mt-0.5">{ventasDetalle.total_canceladas_count} canceladas</p>
        </div>
        <div className="p-4 bg-neutral-50 rounded-2xl">
          <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest">Con descuento</p>
          <p className="text-xl font-black text-neutral-900 mt-1">{formatMoney(ventasDetalle.ventas_con_descuento)}</p>
        </div>
      </div>
    </div>
  );
};
