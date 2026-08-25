// ═══════════════════════════════════════════════════════════════════════════
// SECCIÓN GASTOS — Administración de gastos recurrentes.
// Tarea única: renderizar la pestaña "Gastos": cabecera con contador y botón
// de nuevo gasto, tabla de gastos (proyectado, real, estado de pago, días
// para vencer) con acciones de pago/editar/eliminar. La lógica CRUD vive en
// el orquestador; esta sección solo notifica intenciones vía callbacks.
// ═══════════════════════════════════════════════════════════════════════════

import { invoke } from "@tauri-apps/api/core";
import { MorphIcon } from "morphicons/react";
import {
  BotonAnimado, ICONO_MAS, ICONO_MAS_CIRCULO,
  ICONO_BILLETE, ICONO_EDITAR, ICONO_BORRAR, ICONO_CALCULADORA,
} from "../../../../components/ui";
import type { GastoRecurrente } from "../../../types";
import { notificarError } from "../../../../components/notificaciones";
import { moneda } from "../nucleo/utilidades";
import { EmptyLargo } from "./ui-finanzas";

interface Props {
  gastos: GastoRecurrente[];
  onNuevo: () => void;
  onEditar: (g: GastoRecurrente) => void;
  onPago: (g: GastoRecurrente) => void;
  onRecargar: () => void;
}

const ESTADO_CLS: Record<string, string> = {
  pagado: "bg-emerald-50 text-emerald-600",
  vencido: "bg-red-50 text-red-500",
  proximo_vencer: "bg-amber-50 text-amber-600",
};

export default function SeccionGastos({ gastos, onNuevo, onEditar, onPago, onRecargar }: Props) {
  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <div className="flex items-center gap-3">
          <div className="w-1.5 h-5 bg-neutral-950 rounded-full" />
          <h3 className="text-base sm:text-xl font-black text-neutral-950 uppercase tracking-tight">Gastos Recurrentes</h3>
          <span className="px-3 py-1 bg-neutral-950 text-white text-[9px] font-black rounded-lg">{gastos.length}</span>
        </div>
        <BotonAnimado
          icono={ICONO_MAS}
          iconoHover={ICONO_MAS_CIRCULO}
          onClick={onNuevo}
          className="bg-neutral-950 text-neutral-50 hover:bg-neutral-800 shadow-xl shadow-neutral-200"
        >
          Nuevo Gasto
        </BotonAnimado>
      </div>

      {gastos.length > 0 ? (
        <div className="bg-white rounded-[2.5rem] border border-neutral-200 shadow-sm overflow-hidden">
          <div className="max-h-[560px] overflow-y-auto custom-scrollbar">
            <table className="w-full text-left border-collapse">
              <thead className="sticky top-0 z-10">
                <tr className="bg-neutral-950">
                  <th className="px-8 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Nombre</th>
                  <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Categoria</th>
                  <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Proyectado</th>
                  <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Real</th>
                  <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Estado</th>
                  <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Vence</th>
                  <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest text-right">Acciones</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-neutral-50">
                {gastos.map((g) => (
                  <tr key={g.id} className="group hover:bg-neutral-50/50 transition-all">
                    <td className="px-8 py-4">
                      <span className="text-xs font-black text-neutral-950 uppercase">{g.nombre}</span>
                      <p className="text-[9px] text-neutral-400 font-bold">{g.tipo} / {g.frecuencia}</p>
                    </td>
                    <td className="px-6 py-4">
                      <span className="px-2.5 py-1 bg-neutral-950 text-white text-[9px] font-black rounded-lg uppercase">{g.categoria}</span>
                    </td>
                    <td className="px-6 py-4 text-xs font-black text-neutral-900">{moneda(g.monto_proyectado)}</td>
                    <td className="px-6 py-4 text-xs font-black text-neutral-900">{moneda(g.monto_real)}</td>
                    <td className="px-6 py-4">
                      <span className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-[9px] font-black uppercase ${ESTADO_CLS[g.estado_pago] ?? "bg-neutral-100 text-neutral-500"}`}>
                        <span className={`w-1.5 h-1.5 rounded-full ${g.estado_pago === "pagado" ? "bg-emerald-500" :
                            g.estado_pago === "vencido" ? "bg-red-500" :
                              g.estado_pago === "proximo_vencer" ? "bg-amber-400" : "bg-neutral-300"
                          }`} />
                        {g.estado_pago}
                      </span>
                    </td>
                    <td className="px-6 py-4">
                      <span className={`text-[10px] font-black ${g.dias_para_vencer !== null && g.dias_para_vencer <= 3 ? "text-red-500" : "text-neutral-400"}`}>
                        {g.dias_para_vencer !== null ? `${g.dias_para_vencer}d` : "-"}
                      </span>
                    </td>
                    <td className="px-6 py-4">
                      <div className="flex justify-end gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                        <button onClick={() => onPago(g)} className="p-2 bg-emerald-50 text-emerald-500 rounded-xl hover:bg-emerald-100 transition-all" title="Registrar pago">
                          <MorphIcon icon={ICONO_BILLETE} size={13} strokeWidth={2.5} spring="snappy" reducedMotion="user" />
                        </button>
                        <button onClick={() => onEditar(g)} className="p-2 bg-neutral-100 text-neutral-400 rounded-xl hover:text-neutral-900 hover:bg-neutral-200 transition-all" title="Editar">
                          <MorphIcon icon={ICONO_EDITAR} size={13} strokeWidth={2.5} spring="snappy" reducedMotion="user" />
                        </button>
                        <button
                          onClick={async () => { try { await invoke("eliminar_gasto", { id: g.id }); onRecargar(); } catch (e) { console.error("Error eliminando gasto:", e); notificarError("No se pudo eliminar el gasto", e); } }}
                          className="p-2 bg-neutral-100 text-neutral-400 rounded-xl hover:text-red-500 hover:bg-red-50 transition-all"
                          title="Eliminar"
                        >
                          <MorphIcon icon={ICONO_BORRAR} size={13} strokeWidth={2.5} spring="snappy" reducedMotion="user" />
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      ) : (
        <EmptyLargo icono={ICONO_CALCULADORA} mensaje="No hay gastos recurrentes" sub="Crea tu primer gasto para empezar a rastrearlos" />
      )}
    </div>
  );
}
