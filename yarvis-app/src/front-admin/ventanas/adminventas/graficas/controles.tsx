// Controles compartidos de la vista Ventas: selector de rango con
// mini-menú desplegable ([hoy >]) + helpers de formato. Cada tarjeta y
// gráfica trae el suyo y solo se afecta a sí misma.
import { useEffect, useRef, useState } from "react";

export interface OpcionRango {
  valor: string;
  etiqueta: string;
  dias?: number;
  desfase?: number;
}

export const RANGOS_KPI: OpcionRango[] = [
  { valor: "hoy", etiqueta: "Hoy", dias: 1, desfase: 0 },
  { valor: "ayer", etiqueta: "Ayer", dias: 1, desfase: 1 },
  { valor: "7", etiqueta: "Últimos 7 días", dias: 7, desfase: 0 },
  { valor: "30", etiqueta: "Últimos 30 días", dias: 30, desfase: 0 },
  { valor: "custom", etiqueta: "Personalizado…" },
];

export const RANGOS_VENTANA: OpcionRango[] = [
  { valor: "7", etiqueta: "7 días", dias: 7 },
  { valor: "15", etiqueta: "15 días", dias: 15 },
  { valor: "30", etiqueta: "30 días", dias: 30 },
  { valor: "90", etiqueta: "3 meses", dias: 90 },
  { valor: "custom", etiqueta: "Personalizado…" },
];

export const RANGOS_NOMINA = [
  { valor: "empleado", etiqueta: "Por empleado" },
  { valor: "dia", etiqueta: "Por día" },
  { valor: "semanal", etiqueta: "Semanal" },
];

/** Etiqueta corta para el botón ([hoy >], [30d >], [3m >]). */
export const etiquetaCorta = (etiqueta: string) => {
  if (etiqueta === "Hoy") return "hoy";
  if (etiqueta === "Ayer") return "ayer";
  const ultimos = etiqueta.match(/Últimos (\d+) días/);
  if (ultimos) return `${ultimos[1]}d`;
  const dias = etiqueta.match(/^(\d+) días$/);
  if (dias) return `${dias[1]}d`;
  if (etiqueta === "3 meses") return "3m";
  if (etiqueta === "Personalizado…") return "…";
  return etiqueta;
};

interface RangoDropdownProps {
  valor: string;
  opciones: OpcionRango[] | { valor: string; etiqueta: string }[];
  onChange: (valor: string, diasCustom?: number) => void;
  conCustom?: boolean;
}

export const RangoDropdown = ({ valor, opciones, onChange, conCustom = true }: RangoDropdownProps) => {
  const [abierto, setAbierto] = useState(false);
  const [custom, setCustom] = useState("");
  const ref = useRef<HTMLDivElement>(null);
  const actual = opciones.find((o) => o.valor === valor);

  useEffect(() => {
    const cerrar = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setAbierto(false);
    };
    document.addEventListener("mousedown", cerrar);
    return () => document.removeEventListener("mousedown", cerrar);
  }, []);

  return (
    <div ref={ref} className="relative">
      <button
        onClick={() => setAbierto((v) => !v)}
        className="rounded-xl border border-neutral-200 bg-white px-3 py-1.5 text-[10px] font-black uppercase tracking-widest text-neutral-500 hover:text-neutral-900 hover:border-neutral-900 transition-all"
      >
        {`[${etiquetaCorta(actual?.etiqueta ?? valor)} >]`}
      </button>
      {abierto && (
        <div className="absolute right-0 top-full z-50 mt-2 w-44 rounded-2xl border border-neutral-200 bg-white p-1.5 shadow-xl">
          {opciones.map((o) => (
            <button
              key={o.valor}
              onClick={() => {
                if (o.valor === "custom" && conCustom) return;
                onChange(o.valor);
                setAbierto(false);
              }}
              className={`w-full rounded-xl px-3 py-2 text-left text-[11px] font-bold transition-colors ${
                o.valor === valor ? "bg-neutral-900 text-white" : "text-neutral-600 hover:bg-neutral-100"
              }`}
            >
              {o.valor === valor ? "● " : "○ "}{o.etiqueta}
            </button>
          ))}
          {conCustom && (
            <div className="mt-1 flex gap-1.5 border-t border-neutral-100 px-1.5 pt-1.5 pb-0.5">
              <input
                value={custom}
                onChange={(e) => setCustom(e.target.value.replace(/\D/g, "").slice(0, 3))}
                onKeyDown={(e) => {
                  if (e.key === "Enter" && Number(custom) >= 1) {
                    onChange("custom", Math.min(365, Number(custom)));
                    setAbierto(false);
                  }
                }}
                placeholder="N días"
                className="w-full rounded-lg border border-neutral-200 px-2 py-1.5 text-[11px] font-bold outline-none focus:border-neutral-900"
              />
              <button
                onClick={() => {
                  if (Number(custom) >= 1) {
                    onChange("custom", Math.min(365, Number(custom)));
                    setAbierto(false);
                  }
                }}
                className="rounded-lg bg-neutral-900 px-2.5 text-[11px] font-black text-white"
              >
                →
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  );
};

export const moneda = (n: number) =>
  `$${Number(n || 0).toLocaleString("es-MX", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;

export const entero = (n: number) => Number(n || 0).toLocaleString("es-MX");

/** "2026-09-06" → "06 sep". */
export const fechaCorta = (iso: string) => {
  const meses = ["ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sep", "oct", "nov", "dic"];
  const [y, m, d] = iso.split("-");
  if (!y || !m || !d) return iso;
  return `${d} ${meses[Number(m) - 1] ?? ""}`;
};

/** Variación % actual vs anterior, con signo y flecha. */
export const variacion = (actual: number, anterior: number) => {
  if (!anterior) return actual > 0 ? { texto: "nuevo", pct: 0, sube: true } : { texto: "—", pct: 0, sube: true };
  const pct = ((actual - anterior) / Math.abs(anterior)) * 100;
  return {
    texto: `${pct >= 0 ? "▲" : "▼"} ${Math.abs(pct).toFixed(1)}%`,
    pct,
    sube: pct >= 0,
  };
};

export const Tarjeta = ({ children }: { children: React.ReactNode }) => (
  <section className="bg-white rounded-[2.5rem] border border-neutral-100 shadow-xl p-6 sm:p-8">
    {children}
  </section>
);

export const TituloSeccion = ({ titulo, control }: { titulo: string; control?: React.ReactNode }) => (
  <div className="flex items-center justify-between gap-3 mb-5">
    <h3 className="text-sm font-black text-neutral-900 uppercase tracking-widest">{titulo}</h3>
    {control}
  </div>
);

export const Vacio = ({ mensaje }: { mensaje: string }) => (
  <p className="text-sm text-neutral-400 text-center py-8 border-2 border-dashed border-neutral-100 rounded-2xl">
    {mensaje}
  </p>
);

export const Cargando = () => (
  <div className="py-8 flex justify-center">
    <div className="w-8 h-8 rounded-full border-2 border-neutral-900 border-t-transparent animate-spin" />
  </div>
);
