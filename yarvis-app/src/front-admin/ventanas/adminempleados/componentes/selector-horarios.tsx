// ═══════════════════════════════════════════════════════════════════════════
// SELECTOR HORARIOS — Sección de bloques de horario (días chips L-D + rango
// entrada/salida) del modal de empleados. 100% presentacional: recibe los
// bloques y callbacks; la lógica vive en ModalEmpleados.
// ═══════════════════════════════════════════════════════════════════════════

import { MorphIcon } from "morphicons/react";
import { Campo, inputCls, ICONO_RELOJ, ICONO_MAS, ICONO_BORRAR } from "../../../../components/ui";
import { DIAS, detectTurno, type Bloque } from "../utilidades/horario-empleado";

interface SelectorHorariosProps {
  bloques: Bloque[];
  /** Días distintos ya asignados en todos los bloques (0-7). */
  diasSemana: number;
  /** Días ocupados por bloques DISTINTOS al índice dado. */
  diasOcupadosEn: (idxBloque: number) => Set<number>;
  onToggleDia: (idxBloque: number, dia: number) => void;
  onSetBloque: (idxBloque: number, patch: Partial<Bloque>) => void;
  onEliminar: (idxBloque: number) => void;
  onAgregar: () => void;
}

const SelectorHorarios = ({
  bloques,
  diasSemana,
  diasOcupadosEn,
  onToggleDia,
  onSetBloque,
  onEliminar,
  onAgregar,
}: SelectorHorariosProps) => (
  <div className="bg-neutral-50 rounded-2xl p-4 space-y-4">
    <p className="flex items-center gap-2 text-[10px] font-black text-neutral-500 uppercase tracking-widest">
      <MorphIcon icon={ICONO_RELOJ} size={14} strokeWidth={2.4} spring="smooth" />
      Horarios de trabajo
    </p>

    {bloques.map((b, idx) => (
      <div key={idx} className="bg-white rounded-xl p-3.5 border border-neutral-100 space-y-3 relative">
        <div className="flex items-center justify-between">
          <span className="flex items-center gap-2">
            <span className="px-2 py-0.5 bg-neutral-950 text-white text-[9px] font-black rounded-lg">#{idx + 1}</span>
            {detectTurno(b.inicio) && (
              <span className="text-[9px] font-black text-neutral-400 uppercase tracking-widest">Turno {detectTurno(b.inicio)}</span>
            )}
          </span>
          {bloques.length > 1 && (
            <button
              type="button"
              onClick={() => onEliminar(idx)}
              aria-label={`Eliminar horario ${idx + 1}`}
              className="text-neutral-300 hover:text-red-500 transition-colors"
            >
              <MorphIcon icon={ICONO_BORRAR} size={14} strokeWidth={2.2} spring="snappy" />
            </button>
          )}
        </div>

        <div className="grid grid-cols-2 gap-3">
          <Campo label="Entrada">
            <input type="time" value={b.inicio} onChange={(e) => onSetBloque(idx, { inicio: e.target.value })} className={inputCls} />
          </Campo>
          <Campo label="Salida">
            <input type="time" value={b.fin} onChange={(e) => onSetBloque(idx, { fin: e.target.value })} className={inputCls} />
          </Campo>
        </div>

        <div>
          <p className="text-[10px] font-black text-neutral-500 uppercase tracking-widest mb-2">Días</p>
          <div className="grid grid-cols-7 gap-1.5">
            {DIAS.map((d, dia) => {
              const activo = b.dias.includes(dia);
              const ocupado = !activo && diasOcupadosEn(idx).has(dia);
              return (
                <button
                  key={dia}
                  type="button"
                  title={ocupado ? `${d.label}: ya está en otro horario` : d.label}
                  onClick={() => onToggleDia(idx, dia)}
                  disabled={ocupado}
                  className={`py-2 rounded-xl text-[11px] font-black transition-all active:scale-[0.92] ${
                    activo
                      ? "bg-neutral-950 text-neutral-50 shadow-md"
                      : ocupado
                        ? "bg-neutral-100 text-neutral-200 cursor-not-allowed line-through"
                        : "bg-neutral-50 text-neutral-400 border border-neutral-200 hover:border-neutral-400 hover:text-neutral-700"
                  }`}
                >
                  {d.corto}
                </button>
              );
            })}
          </div>
        </div>
      </div>
    ))}

    <button
      type="button"
      onClick={onAgregar}
      disabled={diasSemana >= 7}
      className="w-full inline-flex items-center justify-center gap-2 py-2.5 rounded-xl border-2 border-dashed border-neutral-300 text-neutral-400 text-[10px] font-black uppercase tracking-widest hover:border-neutral-950 hover:text-neutral-950 transition-all disabled:opacity-30 disabled:hover:border-neutral-300 disabled:hover:text-neutral-400"
    >
      <MorphIcon icon={ICONO_MAS} size={14} strokeWidth={2.5} spring="snappy" />
      Agregar otro horario
      {diasSemana >= 7 && " (todos los días asignados)"}
    </button>
  </div>
);

export default SelectorHorarios;
