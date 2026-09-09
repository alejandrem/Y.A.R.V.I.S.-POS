// Piezas comunes del módulo de cortes: fases, pasos (01/02/03) y
// re-exports de lo compartido con tickets (misma interfaz exacta,
// mismo catálogo maestro).
import type { ArchivoTicket, CatalogItem } from "../tickets/compartido";
import { formatSize, errorMessage, normalizeCatalogItem, ProgressCard } from "../tickets/compartido";

export type { ArchivoTicket, CatalogItem };
export { formatSize, errorMessage, normalizeCatalogItem, ProgressCard };

export type PhaseCortes = "catalogo" | "carpeta" | "procesando" | "completo" | "historial";

export const PasosCortes = ({ phase, onPhaseChange }: { phase: PhaseCortes; onPhaseChange?: (phase: PhaseCortes) => void }) => {
  const steps: Array<{ number: string; label: string; phaseKey: PhaseCortes }> = [
    { number: "01", label: "Catálogo maestro", phaseKey: "catalogo" },
    { number: "02", label: "Carpeta de cortes", phaseKey: "carpeta" },
    { number: "03", label: "Historial", phaseKey: "historial" },
  ];
  const activeIndex = phase === "catalogo" ? 0 : phase === "historial" ? 2 : 1;
  return (
    <div className="mb-8 flex justify-center">
      <nav className="relative flex w-full max-w-[640px] rounded-full border border-neutral-200 bg-neutral-100 p-1.5" aria-label="Pasos del parseador de cortes">
        <span
          aria-hidden="true"
          className="pointer-events-none absolute inset-y-1.5 left-1.5 w-[calc(33.333%-4px)] rounded-full bg-neutral-950 shadow-lg transition-transform duration-300 ease-out"
          style={{ transform: `translateX(${activeIndex * 100}%)` }}
        />
        {steps.map((step, index) => {
          const isActive = index === activeIndex;
          return (
            <button
              key={step.number}
              onClick={() => onPhaseChange?.(step.phaseKey)}
              className={`relative z-10 flex flex-1 items-center justify-center gap-1.5 sm:gap-2 px-3 sm:px-4 py-3 rounded-full text-[9px] sm:text-[10px] font-black uppercase tracking-widest whitespace-nowrap transition-colors duration-300 ${isActive ? "text-white" : "text-neutral-900 hover:text-neutral-600"}`}
            >
              <span className="text-[10px] sm:text-xs opacity-60">{step.number}</span>
              {step.label}
            </button>
          );
        })}
      </nav>
    </div>
  );
};
