// Historial de avisos del reloj del equipo asociado al empleado.
import { MorphIcon } from "morphicons/react";
import { ICONO_RELOJ, ICONO_ALERTA } from "../../../../components/ui";
import type { EventoRelojEmpleado } from "../../../../services/turno";

interface DetalleRelojProps {
  eventos: EventoRelojEmpleado[];
}

export const DetalleReloj = ({ eventos }: DetalleRelojProps) => {
  const abiertos = eventos.filter((evento) => !evento.resuelto_en).length;

  return (
    <div className="bg-white rounded-[2.5rem] border border-neutral-200 p-6 sm:p-8 space-y-5">
      <div className="flex items-center gap-3">
        <div className={`w-10 h-10 rounded-xl flex items-center justify-center ${abiertos ? "bg-amber-100 text-amber-700" : "bg-neutral-950 text-neutral-50"}`}>
          <MorphIcon icon={abiertos ? ICONO_ALERTA : ICONO_RELOJ} size={16} strokeWidth={2.2} spring="smooth" />
        </div>
        <div>
          <h4 className="text-xs font-black text-neutral-900 uppercase tracking-widest">Reloj del equipo</h4>
          <p className="text-[9px] text-neutral-400 font-bold uppercase tracking-wider">
            {abiertos ? `${abiertos} aviso${abiertos === 1 ? "" : "s"} pendiente${abiertos === 1 ? "" : "s"}` : "Cambios y desfases detectados"}
          </p>
        </div>
      </div>

      <div className="space-y-2.5 max-h-72 overflow-y-auto custom-scrollbar pr-1">
        {eventos.length ? eventos.map((evento) => {
          const abierto = !evento.resuelto_en;
          return (
            <div key={evento.id} className={`p-4 rounded-2xl border ${abierto ? "bg-amber-50 border-amber-200" : "bg-neutral-50 border-neutral-100"}`}>
              <div className="flex items-center justify-between gap-3">
                <p className={`text-[9px] font-black uppercase tracking-widest ${abierto ? "text-amber-700" : "text-neutral-500"}`}>
                  {evento.tipo === "cambio_brusco" ? "Salto del reloj local" : evento.tipo === "reloj_atrasado" ? "Hora local retrocedida" : "Aviso de hora"}
                </p>
                <span className={`shrink-0 text-[8px] font-black uppercase tracking-wider ${abierto ? "text-amber-700" : "text-neutral-400"}`}>
                  {abierto ? "Pendiente" : "Resuelto"}
                </span>
              </div>
              <p className="text-[10px] font-bold text-neutral-700 leading-relaxed mt-2">{evento.mensaje}</p>
              <p className="text-[9px] font-bold text-neutral-400 mt-2">
                Hora local observada: {evento.hora_sistema}
              </p>
              {evento.resuelto_en && <p className="text-[8px] font-black text-neutral-400 uppercase tracking-wider mt-1">Revisado: {evento.resuelto_en}</p>}
            </div>
          );
        }) : (
          <div className="py-8 px-4 text-center bg-neutral-50 rounded-2xl border border-dashed border-neutral-200">
            <p className="text-[10px] font-black text-neutral-400 uppercase tracking-widest">Sin avisos registrados</p>
            <p className="text-[9px] font-bold text-neutral-400 mt-2 leading-relaxed">Un aviso señala un cambio o desfase observado durante su sesión; no confirma quién lo causó. Las ventas y los cortes no se bloquean.</p>
            <p className="text-[9px] font-bold text-neutral-400 mt-2 leading-relaxed">Para corregirlo: Windows → Configuración → Hora e idioma → Fecha y hora. Revisa la hora y la zona horaria configuradas.</p>
          </div>
        )}
      </div>
    </div>
  );
};
