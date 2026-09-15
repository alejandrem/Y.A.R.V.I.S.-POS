// ═══════════════════════════════════════════════════════════════════════════
// EMPLEACORTES · BOTÓN CORTE — Gemelo de la barra de búsqueda:
// 200px de ancho, misma altura y radio (rounded-[1.75rem]), oscuro.
// Vive al lado del buscador en NUEVA VENTA. El atajo F3 hace lo mismo.
// ═══════════════════════════════════════════════════════════════════════════

import { MorphIcon } from "morphicons/react";
import { ICONO_CAJA } from "../../../components/ui";

interface BotonCorteProps {
  onAbrir: () => void;
}

export default function BotonCorte({ onAbrir }: BotonCorteProps) {
  return (
    <button
      onClick={onAbrir}
      title="Corte de caja (F3)"
      className="w-full sm:w-[200px] shrink-0 py-5 px-6 bg-neutral-950 border-2 border-neutral-950 rounded-[1.75rem] shadow-xl shadow-neutral-200/60 text-white text-sm font-black uppercase tracking-[0.2em] hover:bg-neutral-800 transition-all duration-300 active:scale-[0.98] flex items-center justify-center gap-3"
    >
      <MorphIcon icon={ICONO_CAJA} size={20} strokeWidth={2.5} spring="snappy" reducedMotion="user" />
      Corte
    </button>
  );
}
