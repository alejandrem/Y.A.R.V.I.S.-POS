// ═══════════════════════════════════════════════════════════════════════════
// PIE COBRO — Footer oscuro del detalle de venta.
// Tarea única: renderizar la sugerencia IA y el BotonAnimado "Cobrar $X".
// 100% presentacional: recibe iaSuggestion, total, onCobrar y disabled.
// ═══════════════════════════════════════════════════════════════════════════

import { MorphIcon } from "morphicons/react";
import {
  BotonAnimado,
  ICONO_BILLETE, ICONO_ESCANER, ICONO_ESTRELLA,
} from "../../../../components/ui";

interface PieCobroProps {
  iaSuggestion: string;
  total: number;
  onCobrar: () => void;
  disabled: boolean;
}

export default function PieCobro({ iaSuggestion, total, onCobrar, disabled }: PieCobroProps) {
  return (
    <div className="p-5 sm:p-6 bg-neutral-950 flex flex-col sm:flex-row items-stretch sm:items-center gap-4">
      <div className="flex-1 flex items-center gap-4 bg-white/5 p-4 rounded-3xl border border-white/10">
        <div className="w-10 h-10 bg-white/10 rounded-2xl flex items-center justify-center shrink-0">
          <MorphIcon icon={ICONO_ESTRELLA} size={17} strokeWidth={2} spring="smooth" className="text-amber-300" />
        </div>
        <div className="min-w-0">
          <p className="text-[8px] font-black text-neutral-500 uppercase tracking-[0.2em] mb-1">Sugerencia IA</p>
          <p className="text-[11px] font-bold text-neutral-200 leading-tight truncate">
            {iaSuggestion || <span className="opacity-30 italic">Sin recomendaciones...</span>}
          </p>
        </div>
      </div>
      <BotonAnimado
        icono={ICONO_BILLETE}
        iconoHover={ICONO_ESCANER}
        onClick={onCobrar}
        disabled={disabled}
        className="bg-white hover:bg-neutral-50 text-neutral-950 shadow-xl shadow-black/30 sm:min-w-[220px] justify-center !rounded-3xl !py-5 !text-lg"
      >
        Cobrar ${total.toFixed(2)}
      </BotonAnimado>
    </div>
  );
}
