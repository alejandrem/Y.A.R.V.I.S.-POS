// ═══════════════════════════════════════════════════════════════════════════
// PIE COBRO — Footer oscuro del detalle de venta.
// Tarea única: renderizar la sugerencia IA, el desglose SUBTOTAL/DESCUENTO
// (solo si hay descuento) y el BotonAnimado "Cobrar $X" con el NETO.
// 100% presentacional: recibe iaSuggestion, subtotal, descuento, total,
// onCobrar y disabled.
// ═══════════════════════════════════════════════════════════════════════════

import { MorphIcon } from "morphicons/react";
import {
  BotonAnimado,
  ICONO_BILLETE, ICONO_ESCANER, ICONO_ESTRELLA,
} from "../../../../components/ui";

interface PieCobroProps {
  iaSuggestion: string;
  /** Bruto sin descuentos. */
  subtotal: number;
  /** Suma de descuentos por línea. */
  descuento: number;
  /** NETO a cobrar (subtotal − descuento). */
  total: number;
  onCobrar: () => void;
  disabled: boolean;
}

export default function PieCobro({ iaSuggestion, subtotal, descuento, total, onCobrar, disabled }: PieCobroProps) {
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
      <div className="flex flex-col items-stretch gap-2 sm:min-w-[220px]">
        {descuento > 0 && (
          <div className="px-4 text-[10px] font-black uppercase tracking-widest space-y-1">
            <div className="flex justify-between text-neutral-400">
              <span>Subtotal</span>
              <span>${subtotal.toFixed(2)}</span>
            </div>
            <div className="flex justify-between text-amber-300">
              <span>Descuento</span>
              <span>−${descuento.toFixed(2)}</span>
            </div>
          </div>
        )}
        <BotonAnimado
          icono={ICONO_BILLETE}
          iconoHover={ICONO_ESCANER}
          onClick={onCobrar}
          disabled={disabled}
          className="bg-white hover:bg-neutral-50 text-neutral-950 shadow-xl shadow-black/30 justify-center !rounded-3xl !py-5 !text-lg"
        >
          Cobrar ${total.toFixed(2)}
        </BotonAnimado>
      </div>
    </div>
  );
}
