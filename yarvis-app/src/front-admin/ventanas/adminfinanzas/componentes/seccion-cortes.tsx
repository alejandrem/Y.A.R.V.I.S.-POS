// ═══════════════════════════════════════════════════════════════════════════
// SECCIÓN CORTES — Historial de cortes de caja X/Z.
// Tarea única: renderizar la pestaña "Cortes": selector de rango + tabla de
// cortes (tipo, apertura relativa, cajero, ventas, diferencia con semáforo,
// estado) con botón de detalle por fila que notifica vía onVerDetalle.
// ═══════════════════════════════════════════════════════════════════════════

import { MorphIcon } from "morphicons/react";
import { ICONO_CAJA, ICONO_OJO } from "../../../../components/ui";
import type { CorteCaja } from "../../../types";
import { moneda, fechaRelativa, type RangoFechas } from "../nucleo/utilidades";
import SelectorRango from "./selector-rango";
import { EmptyLargo } from "./ui-finanzas";

interface Props {
  cortes: CorteCaja[];
  rango: RangoFechas;
  onRango: (r: RangoFechas) => void;
  onVerDetalle: (c: CorteCaja) => void;
}

export default function SeccionCortes({ cortes, rango, onRango, onVerDetalle }: Props) {
  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex items-center gap-3">
          <div className="w-1.5 h-5 bg-neutral-950 rounded-full" />
          <h3 className="text-base sm:text-xl font-black text-neutral-950 uppercase tracking-tight">Cortes de Caja</h3>
          <span className="px-3 py-1 bg-neutral-950 text-white text-[9px] font-black rounded-lg">{cortes.length}</span>
        </div>
        <SelectorRango rango={rango} onChange={onRango} />
      </div>

      {cortes.length > 0 ? (
        <div className="bg-white rounded-[2.5rem] border border-neutral-200 shadow-sm overflow-hidden">
          <div className="max-h-[560px] overflow-y-auto custom-scrollbar">
            <table className="w-full text-left border-collapse">
              <thead className="sticky top-0 z-10">
                <tr className="bg-neutral-950">
                  <th className="px-8 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Tipo</th>
                  <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Apertura</th>
                  <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Cajero</th>
                  <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Ventas</th>
                  <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Diferencia</th>
                  <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Estado</th>
                  <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest text-right">Detalle</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-neutral-50">
                {cortes.map((c) => (
                  <tr key={c.id} className="group hover:bg-neutral-50/50 transition-all">
                    <td className="px-8 py-4">
                      <span className={`inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-[9px] font-black uppercase ${c.tipo_corte === "Z" ? "bg-neutral-950 text-white" : "bg-neutral-200 text-neutral-700"}`}>
                        {c.tipo_corte}
                      </span>
                    </td>
                    <td className="px-6 py-4 text-xs font-bold text-neutral-900">{fechaRelativa(c.fecha_apertura)}</td>
                    <td className="px-6 py-4 text-xs font-bold text-neutral-700">{c.usuario_nombre ?? "-"}</td>
                    <td className="px-6 py-4 text-xs font-black text-neutral-950">{moneda(c.total_ventas)}</td>
                    <td className="px-6 py-4">
                      <span className={`text-xs font-black ${c.diferencia === 0 ? "text-neutral-400" : Math.abs(c.diferencia) > c.total_ventas * 0.05 ? "text-red-500" : "text-amber-500"}`}>
                        {moneda(c.diferencia)}
                      </span>
                    </td>
                    <td className="px-6 py-4">
                      <span className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-[9px] font-black uppercase ${c.estado === "cerrado" ? "bg-emerald-50 text-emerald-600" : "bg-amber-50 text-amber-600"}`}>
                        <span className={`w-1.5 h-1.5 rounded-full ${c.estado === "cerrado" ? "bg-emerald-500" : "bg-amber-400 animate-pulse"}`} />
                        {c.estado}
                      </span>
                    </td>
                    <td className="px-6 py-4 text-right">
                      <button
                        onClick={() => onVerDetalle(c)}
                        className="inline-flex items-center gap-2 px-4 py-2 rounded-xl bg-neutral-950 text-white text-[9px] font-black uppercase tracking-widest hover:bg-neutral-800 transition-all active:scale-[0.97] opacity-0 group-hover:opacity-100"
                      >
                        <MorphIcon icon={ICONO_OJO} size={11} strokeWidth={2.5} spring="snappy" reducedMotion="user" />
                        Ver
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      ) : (
        <EmptyLargo icono={ICONO_CAJA} mensaje="Sin cortes de caja" sub="Los cortes se registran desde el punto de venta" />
      )}
    </div>
  );
}
