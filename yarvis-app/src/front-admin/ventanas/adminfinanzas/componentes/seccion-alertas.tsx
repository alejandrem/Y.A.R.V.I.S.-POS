// ═══════════════════════════════════════════════════════════════════════════
// SECCIÓN ALERTAS — Bandeja de alertas financieras.
// Tarea única: renderizar la pestaña "Alertas": cabecera con contador de no
// leídas y lista de tarjetas coloreadas por severidad (rojo/amarillo/verde),
// cada una con botón para marcar como leída vía onMarcarLeida.
// ═══════════════════════════════════════════════════════════════════════════

import { MorphIcon } from "morphicons/react";
import {
  ICONO_CALCULADORA, ICONO_CAJA, ICONO_ALERTA,
  ICONO_TRENDING, ICONO_CHECK, ICONO_CAMPANA,
} from "../../../../components/ui";
import type { AlertaFinanciera } from "../../../types";
import { fechaRelativa } from "../nucleo/utilidades";
import { EmptyLargo } from "./ui-finanzas";

interface Props {
  alertas: AlertaFinanciera[];
  alertasNoLeidas: number;
  onMarcarLeida: (id: number) => void;
}

const ICONO_POR_TIPO = (tipo: string) =>
  tipo === "gasto_vencimiento" ? ICONO_CALCULADORA :
    tipo === "corte_pendiente" ? ICONO_CAJA :
      tipo === "diferencia_caja" ? ICONO_ALERTA : ICONO_TRENDING;

export default function SeccionAlertas({ alertas, alertasNoLeidas, onMarcarLeida }: Props) {
  return (
    <div className="space-y-6">
      <div className="flex items-center gap-3">
        <div className="w-1.5 h-5 bg-neutral-950 rounded-full" />
        <h3 className="text-base sm:text-xl font-black text-neutral-950 uppercase tracking-tight">Alertas Financieras</h3>
        {alertasNoLeidas > 0 && (
          <span className="px-3 py-1 bg-red-500 text-white text-[9px] font-black rounded-lg shadow-lg shadow-red-500/20">{alertasNoLeidas} SIN LEER</span>
        )}
      </div>

      {alertas.length > 0 ? (
        <div className="space-y-3">
          {alertas.map((a) => (
            <div
              key={a.id}
              className={`flex items-start gap-4 p-5 rounded-[2rem] border transition-all ${a.leida ? "bg-white border-neutral-100 opacity-50" :
                  a.severidad === "rojo" ? "bg-red-50/60 border-red-200 shadow-sm shadow-red-100" :
                    a.severidad === "amarillo" ? "bg-amber-50/60 border-amber-200 shadow-sm shadow-amber-100" :
                      "bg-emerald-50/60 border-emerald-200 shadow-sm shadow-emerald-100"
                }`}
            >
              <div className={`w-11 h-11 rounded-2xl flex items-center justify-center shrink-0 ${a.severidad === "rojo" ? "bg-red-500 text-white shadow-lg shadow-red-500/20" :
                  a.severidad === "amarillo" ? "bg-amber-400 text-white shadow-lg shadow-amber-400/20" :
                    "bg-emerald-500 text-white shadow-lg shadow-emerald-500/20"
                }`}>
                <MorphIcon icon={ICONO_POR_TIPO(a.tipo)} size={18} strokeWidth={2.2} spring="smooth" />
              </div>
              <div className="flex-1 min-w-0">
                <p className="text-xs font-black text-neutral-950 uppercase">{a.titulo}</p>
                <p className="text-[11px] text-neutral-500 font-bold mt-1">{a.mensaje}</p>
                <p className="text-[9px] text-neutral-400 font-bold mt-1.5">{fechaRelativa(a.creada_en)}</p>
              </div>
              {!a.leida && (
                <button
                  onClick={() => onMarcarLeida(a.id)}
                  className="p-2.5 bg-neutral-100 text-neutral-400 rounded-xl hover:text-emerald-500 hover:bg-emerald-50 transition-all shrink-0"
                  title="Marcar como leida"
                >
                  <MorphIcon icon={ICONO_CHECK} size={14} strokeWidth={2.5} spring="snappy" reducedMotion="user" />
                </button>
              )}
            </div>
          ))}
        </div>
      ) : (
        <EmptyLargo icono={ICONO_CAMPANA} mensaje="No hay alertas financieras" sub="Todo esta en orden" />
      )}
    </div>
  );
}
