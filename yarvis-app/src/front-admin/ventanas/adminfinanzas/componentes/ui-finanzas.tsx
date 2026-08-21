// ═══════════════════════════════════════════════════════════════════════════
// UI DE FINANZAS — Primitivas visuales reutilizables del módulo.
// Tarea única: renderizar los bloques de presentación sin lógica de negocio:
//   · KPI           → tarjeta grande de métrica con icono morfo y color por semántica
//   · SeccionGrafica → contenedor blanco con título/subtítulo/acción para cada gráfica
//   · EmptyGrafica  → placeholder compacto cuando una gráfica no tiene datos
//   · EmptyLargo    → placeholder grande de sección vacía con CTA implícito
// ═══════════════════════════════════════════════════════════════════════════

import { useState, type ReactNode } from "react";
import { MorphIcon } from "morphicons/react";
import { IconoMorph, ICONO_CHECK, ICONO_GRAFICA, ICONO_DOLAR } from "../../../../components/ui";

export function KPI({ icono, label, valor, color }: { icono: typeof ICONO_DOLAR; label: string; valor: string; color: "neutral" | "verde" | "rojo" }) {
  const [hover, setHover] = useState(false);
  const bg = color === "verde" ? "bg-emerald-500" : color === "rojo" ? "bg-red-500" : "bg-neutral-950";
  const texto = color === "verde" ? "text-emerald-600" : color === "rojo" ? "text-red-500" : "text-neutral-950";

  return (
    <div
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      className="rounded-[2rem] p-5 sm:p-6 border bg-white border-neutral-200 hover:shadow-lg transition-all group"
    >
      <div className={`w-12 h-12 rounded-2xl flex items-center justify-center ${bg} shadow-lg`}>
        <IconoMorph
          icono={icono}
          iconoHover={ICONO_CHECK}
          size={18}
          strokeWidth={2}
          hover={hover}
          className="text-white"
        />
      </div>
      <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest mt-4">{label}</p>
      <p className={`text-2xl font-black mt-1.5 ${texto}`}>{valor}</p>
    </div>
  );
}

export function SeccionGrafica({ titulo, subtitulo, accion, children }: { titulo: string; subtitulo: string; accion?: ReactNode; children: ReactNode }) {
  return (
    <div className="bg-white rounded-[2.5rem] border border-neutral-200 shadow-sm overflow-hidden">
      <div className="flex items-center justify-between px-8 pt-6 pb-2">
        <div>
          <h4 className="text-sm font-black text-neutral-950 uppercase tracking-tight">{titulo}</h4>
          <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest mt-0.5">{subtitulo}</p>
        </div>
        {accion}
      </div>
      <div className="px-6 pb-6 pt-2">
        {children}
      </div>
    </div>
  );
}

export function EmptyGrafica({ mensaje }: { mensaje: string }) {
  return (
    <div className="py-14 text-center">
      <div className="w-12 h-12 mx-auto bg-neutral-100 rounded-2xl flex items-center justify-center mb-3">
        <MorphIcon icon={ICONO_GRAFICA} size={20} strokeWidth={1.8} spring="smooth" className="text-neutral-300" />
      </div>
      <p className="text-[10px] font-black text-neutral-300 uppercase tracking-widest">{mensaje}</p>
    </div>
  );
}

export function EmptyLargo({ icono, mensaje, sub }: { icono: typeof ICONO_DOLAR; mensaje: string; sub: string }) {
  return (
    <div className="bg-white rounded-[2.5rem] border border-neutral-200 shadow-sm py-20 text-center">
      <div className="mx-auto w-16 h-16 bg-neutral-950 rounded-2xl flex items-center justify-center shadow-lg">
        <MorphIcon icon={icono} size={28} strokeWidth={1.8} spring="smooth" className="text-white" />
      </div>
      <p className="text-[10px] font-black text-neutral-300 uppercase tracking-[0.2em] mt-5">{mensaje}</p>
      <p className="text-[9px] font-bold text-neutral-400 mt-1.5">{sub}</p>
    </div>
  );
}
