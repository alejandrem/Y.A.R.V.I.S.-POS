// ═══════════════════════════════════════════════════════════════════════════
// SELECTOR DE RANGO — Control compacto de periodo de análisis.
// Tarea única: renderizar los presets rápidos (7D/30D/3M/6M) y las fechas
// custom (inicio–fin) que filtran las secciones Resumen, Cortes y Métricas.
// Componente controlado: no guarda estado, recibe el rango y emite cambios.
// ═══════════════════════════════════════════════════════════════════════════

import { useMemo } from "react";
import { PRESETS_RANGO, inputFechaCompacto } from "../nucleo/constantes";
import { rangoDeDias, hoyISO, type RangoFechas } from "../nucleo/utilidades";

export default function SelectorRango({ rango, onChange }: { rango: RangoFechas; onChange: (r: RangoFechas) => void }) {
  const presetActivo = useMemo(() => {
    const dias = Math.round((+new Date(rango.fin) - +new Date(rango.inicio)) / 86400000);
    return PRESETS_RANGO.find((p) => p.dias === dias)?.dias ?? null;
  }, [rango]);

  return (
    <div className="flex flex-wrap items-center gap-2" title="Periodo de analisis">
      <div className="flex bg-neutral-100 p-0.5 rounded-xl">
        {PRESETS_RANGO.map((p) => (
          <button
            key={p.dias}
            onClick={() => onChange(rangoDeDias(p.dias))}
            className={`px-2.5 py-1.5 text-[9px] font-black rounded-lg transition-all ${presetActivo === p.dias ? "bg-neutral-950 text-white shadow-md" : "text-neutral-400 hover:text-neutral-700"}`}
          >
            {p.label}
          </button>
        ))}
      </div>
      <input
        type="date"
        className={inputFechaCompacto}
        value={rango.inicio}
        max={rango.fin}
        onChange={(e) => e.target.value && onChange({ inicio: e.target.value, fin: rango.fin })}
      />
      <span className="text-[10px] font-black text-neutral-300">–</span>
      <input
        type="date"
        className={inputFechaCompacto}
        value={rango.fin}
        min={rango.inicio}
        max={hoyISO()}
        onChange={(e) => e.target.value && onChange({ inicio: rango.inicio, fin: e.target.value })}
      />
    </div>
  );
}
