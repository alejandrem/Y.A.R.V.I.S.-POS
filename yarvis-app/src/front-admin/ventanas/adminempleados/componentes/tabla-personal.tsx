// Tabla "LISTA DE PERSONAL": card blanco con la tabla de empleados,
// estados en vivo y botones Editar/Ver, más empty state.
import { MorphIcon } from "morphicons/react";
import type { EmpleadoProfile } from "../../../../services/empleado";
import {
  ICONO_USUARIO,
  ICONO_FLECHA,
  ICONO_CERRAR,
  ICONO_EDITAR,
  ICONO_CHECK,
} from "../../../../components/ui";
import {
  detectTurno,
  estadoDot,
  estadoVisual,
  formatEntrada,
} from "../utilidades/helpers";

interface TablaPersonalProps {
  empleados: EmpleadoProfile[];
  loading: boolean;
  selectedId: number | null;
  recargado: boolean;
  onEditar: (emp: EmpleadoProfile) => void;
  onToggleDetalle: (id: number) => void;
  onRecargar: () => void;
}

export const TablaPersonal = ({ empleados, loading, selectedId, recargado, onEditar, onToggleDetalle, onRecargar }: TablaPersonalProps) => {
  return (
    <div className="bg-white rounded-[2.5rem] border border-neutral-200 shadow-sm overflow-hidden">
      <div className="flex items-center justify-between px-6 sm:px-10 pt-6 sm:pt-8 pb-2">
        <div>
          <h3 className="text-base sm:text-xl font-black text-neutral-900 uppercase tracking-tight">Personal</h3>
          <p className="text-[9px] text-neutral-400 uppercase font-black tracking-widest">Estado en vivo por horario</p>
        </div>
        <div className="flex items-center gap-3">
          <span className="px-3 py-1 bg-neutral-950 text-neutral-50 text-[9px] font-black rounded-lg">
            {empleados.length} REGISTROS
          </span>
          <button
            onClick={onRecargar}
            className="inline-flex items-center gap-2 px-3 py-1.5 text-[9px] font-black uppercase tracking-widest text-neutral-400 hover:text-neutral-950 transition-colors"
          >
            <MorphIcon icon={recargado ? ICONO_CHECK : ICONO_FLECHA} size={13} strokeWidth={2.5} spring="snappy" reducedMotion="user" />
            Recargar datos
          </button>
        </div>
      </div>

      <div className="overflow-x-auto custom-scrollbar">
        <table className="w-full text-left border-collapse min-w-[720px]">
          <thead className="sticky top-0 z-10">
            <tr className="bg-neutral-50/95 backdrop-blur-sm border-y border-neutral-100">
              <th className="px-6 sm:px-10 py-4 text-[10px] font-black text-neutral-400 uppercase tracking-widest">Empleado</th>
              <th className="px-6 py-4 text-[10px] font-black text-neutral-400 uppercase tracking-widest">Estado</th>
              <th className="px-6 py-4 text-[10px] font-black text-neutral-400 uppercase tracking-widest">Turno</th>
              <th className="px-6 py-4 text-[10px] font-black text-neutral-400 uppercase tracking-widest">Entrada / Último acceso</th>
              <th className="px-6 py-4 text-[10px] font-black text-neutral-400 uppercase tracking-widest text-right">Detalle y Edición</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-neutral-50">
            {empleados.map((emp) => {
              const estado = estadoDot(emp);
              const visual = estadoVisual[estado];
              const turno = detectTurno(emp.horario_inicio);
              return (
                <tr key={emp.id} className={`hover:bg-neutral-50/40 transition-colors ${emp.estado === "inactivo" ? "opacity-50" : ""}`}>
                  <td className="px-6 sm:px-10 py-4">
                    <div className="flex items-center gap-3">
                      <div className="w-9 h-9 bg-neutral-950 text-neutral-50 rounded-xl flex items-center justify-center">
                        <MorphIcon icon={ICONO_USUARIO} size={15} strokeWidth={2.2} spring="smooth" />
                      </div>
                      <span className="text-xs font-black text-neutral-900 uppercase">{emp.nombre}</span>
                    </div>
                  </td>
                  <td className="px-6 py-4">
                    <span className={`inline-flex items-center gap-2 px-3 py-1.5 rounded-full text-[9px] font-black uppercase tracking-widest ${visual.fondo} ${visual.texto}`}>
                      <span className={`w-1.5 h-1.5 rounded-full ${visual.dot} ${estado === "En turno" ? "animate-pulse" : ""}`} />
                      {estado}
                    </span>
                  </td>
                  <td className="px-6 py-4">
                    <span className="text-[10px] font-bold text-neutral-900 uppercase bg-neutral-100 px-3 py-1.5 rounded-lg">
                      {turno || "Sin turno"}
                    </span>
                  </td>
                  <td className="px-6 py-4">
                    <span className="text-[10px] font-bold text-neutral-500">{formatEntrada(emp)}</span>
                  </td>
                  <td className="px-6 py-4 text-right">
                    <div className="inline-flex items-center gap-2">
                      <button
                        onClick={() => onEditar(emp)}
                        className="inline-flex items-center gap-2 px-4 py-2.5 rounded-xl border-2 border-neutral-300 text-neutral-500 text-[9px] font-black uppercase tracking-widest hover:border-neutral-950 hover:text-neutral-950 transition-all active:scale-[0.97]"
                      >
                        <MorphIcon icon={ICONO_EDITAR} size={13} strokeWidth={2.5} spring="snappy" reducedMotion="user" />
                        Editar
                      </button>
                      <button
                        onClick={() => onToggleDetalle(emp.id)}
                        className="inline-flex items-center gap-2 px-5 py-2.5 rounded-xl border-2 border-neutral-950 text-neutral-900 text-[9px] font-black uppercase tracking-widest hover:bg-neutral-950 hover:text-neutral-50 transition-all active:scale-[0.97]"
                      >
                        <MorphIcon icon={selectedId === emp.id ? ICONO_CERRAR : ICONO_FLECHA} size={13} strokeWidth={2.5} spring="snappy" reducedMotion="user" />
                        {selectedId === emp.id ? "Cerrar" : "Ver"}
                      </button>
                    </div>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>

      {!empleados.length && (
        <p className="py-14 text-center text-[11px] font-black text-neutral-300 uppercase tracking-widest italic">
          {loading ? "Cargando personal..." : "Aún no hay empleados registrados"}
        </p>
      )}
    </div>
  );
};
