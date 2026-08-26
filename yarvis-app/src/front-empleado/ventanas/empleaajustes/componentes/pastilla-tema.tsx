// ═══════════════════════════════════════════════════════════════════════════
// PASTILLA DE TEMA — Selector segmentado claro / oscuro / sistema.
// Tarea única: renderizar la pastilla de 3 opciones y delegar el cambio al
// hook useTheme (misma fuente de verdad que adminconfig: localStorage +
// clase `dark` en <html>). Iconografía 100% MorphIcon.
// ═══════════════════════════════════════════════════════════════════════════

import { MorphIcon } from "morphicons/react";
import type { IconInput } from "morphicons/react";
import type { Theme } from "../../../../hooks/useTheme";
import { ICONO_SOL, ICONO_LUNA, ICONO_PANTALLA } from "../../../../icons";

interface PastillaTemaProps {
  tema: Theme;
  onCambiar: (tema: Theme) => void;
}

const OPCIONES: { id: Theme; label: string; icono: IconInput }[] = [
  { id: "claro", label: "Claro", icono: ICONO_SOL },
  { id: "oscuro", label: "Oscuro", icono: ICONO_LUNA },
  { id: "sistema", label: "Sistema", icono: ICONO_PANTALLA },
];

export default function PastillaTema({ tema, onCambiar }: PastillaTemaProps) {
  return (
    <div className="inline-flex p-1.5 bg-white border border-neutral-200 rounded-full shadow-sm">
      {OPCIONES.map((opcion) => (
        <button
          key={opcion.id}
          onClick={() => onCambiar(opcion.id)}
          className={`flex items-center gap-2 px-5 py-2.5 rounded-full text-[10px] font-black uppercase tracking-widest transition-all duration-200 ${
            tema === opcion.id
              ? "bg-neutral-950 text-white shadow-md"
              : "text-neutral-400 hover:text-neutral-900"
          }`}
        >
          <MorphIcon
            icon={opcion.icono}
            size={13}
            strokeWidth={2.4}
            spring="snappy"
            reducedMotion="user"
          />
          {opcion.label}
        </button>
      ))}
    </div>
  );
}
