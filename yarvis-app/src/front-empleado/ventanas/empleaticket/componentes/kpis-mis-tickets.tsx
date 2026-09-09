// ═══════════════════════════════════════════════════════════════════════════
// KPIs MIS TICKETS — 4 tarjetas estilo adminventas/graficas/kpis.tsx:
// mismo grid, misma TarjetaKpi (rounded-[2rem] + RangoDropdown [hoy >]).
// 2 reales (VENDI + HORAS EXTRAS, cada una con su propio rango) + 2 mudas
// PROXIMAMENTE para no dejar hueco. Solo datos DEL empleado actual.
// ═══════════════════════════════════════════════════════════════════════════

import { useEffect, useState } from "react";
import {
  RangoDropdown, moneda, entero,
  type OpcionRango,
} from "../../../../front-admin/ventanas/adminventas/graficas/controles";
import { obtenerMisKpis } from "../../../../services/misventas";
import { obtenerMisHorasExtra } from "../../../../services/turno";
import { reportarError } from "../../../../services/tauri";
import type { DiaExtra } from "../../../../components/turno";

/** Rangos del empleado: Hoy, 7, 15, 30, Todos y personalizado (N días). */
export const RANGOS_MIS: OpcionRango[] = [
  { valor: "hoy", etiqueta: "Hoy", dias: 1 },
  { valor: "7", etiqueta: "7 días", dias: 7 },
  { valor: "15", etiqueta: "15 días", dias: 15 },
  { valor: "30", etiqueta: "30 días", dias: 30 },
  { valor: "todos", etiqueta: "Todos" },
  { valor: "custom", etiqueta: "Personalizado…" },
];

/** Ventana "Todos": ~100 años, igual que DIAS_TODOS del backend. */
export const DIAS_TODOS = 36500;

/** "hoy"/"7"/"15"/"30"/"todos"/"custom" → días (puro, testeable). */
export const rangoMisADias = (valor: string, custom?: number): number => {
  if (valor === "todos") return DIAS_TODOS;
  if (valor === "custom") return Math.min(365, Math.max(1, custom ?? 7));
  return RANGOS_MIS.find((o) => o.valor === valor)?.dias ?? 1;
};

/** "6h 40m" a partir de minutos (puro, testeable). */
export const fmtExtra = (mins: number): string =>
  `${Math.floor(mins / 60)}h ${mins % 60}m`;

/** Suma extras cuyo día cae en la ventana [hoy-(dias-1), hoy] (puro, testeable). */
export const sumarExtrasPorRango = (
  extras: DiaExtra[],
  dias: number,
  hoyISO?: string,
): { minutos: number; diasConExtra: number } => {
  const hoy = hoyISO ?? new Date().toISOString().slice(0, 10);
  const corte = new Date(`${hoy}T00:00:00`);
  corte.setDate(corte.getDate() - (dias - 1));
  const corteISO = corte.toISOString().slice(0, 10);
  let minutos = 0;
  let diasConExtra = 0;
  for (const d of extras) {
    if (d.fecha < corteISO || d.fecha > hoy) continue;
    const m = (d.extra_pre_min || 0) + (d.extra_post_min || 0);
    if (m > 0) {
      minutos += m;
      diasConExtra += 1;
    }
  }
  return { minutos, diasConExtra };
};

export interface RangoSel {
  v: string;
  c?: number;
}

const TarjetaKpi = ({
  titulo,
  rango,
  setRango,
  grande,
  chico,
}: {
  titulo: string;
  rango: string;
  setRango: (v: string, c?: number) => void;
  grande: string;
  chico: string;
}) => (
  <div className="bg-white rounded-[2rem] border border-neutral-100 shadow-xl p-5 sm:p-6">
    <div className="flex items-center justify-between gap-2 mb-3">
      <p className="text-[10px] font-black uppercase tracking-widest text-neutral-400">{titulo}</p>
      <RangoDropdown valor={rango} opciones={RANGOS_MIS} onChange={setRango} />
    </div>
    <p className="text-2xl sm:text-3xl font-black text-neutral-900">{grande}</p>
    <p className="text-[11px] text-neutral-400 font-bold mt-1">{chico}</p>
  </div>
);

const TarjetaProx = ({ titulo }: { titulo: string }) => (
  <div className="bg-white rounded-[2rem] border border-dashed border-neutral-200 shadow-xl p-5 sm:p-6 flex flex-col">
    <div className="flex items-center justify-between gap-2 mb-3">
      <p className="text-[10px] font-black uppercase tracking-widest text-neutral-300">{titulo}</p>
      <span className="text-[8px] font-black uppercase tracking-widest text-neutral-300 border border-neutral-200 rounded-xl px-3 py-1.5">
        [próx &gt;]
      </span>
    </div>
    <p className="text-2xl sm:text-3xl font-black text-neutral-200">—</p>
    <p className="text-[11px] text-neutral-300 font-bold mt-1">Próximamente</p>
  </div>
);

// Cada tarjeta trae su propio rango y solo se afecta a sí misma,
// igual que adminventas/graficas/kpis.tsx. Sin props.
const KpisMisTickets = () => {
  const [rangoVentas, setRangoVentasRaw] = useState<RangoSel>({ v: "hoy" });
  const [rangoExtras, setRangoExtrasRaw] = useState<RangoSel>({ v: "7" });
  const setRangoVentas = (v: string, c?: number) => setRangoVentasRaw({ v, c });
  const setRangoExtras = (v: string, c?: number) => setRangoExtrasRaw({ v, c });
  const diasVentas = rangoMisADias(rangoVentas.v, rangoVentas.c);
  const diasExtras = rangoMisADias(rangoExtras.v, rangoExtras.c);
  const [total, setTotal] = useState<number | null>(null);
  const [tickets, setTickets] = useState<number | null>(null);
  const [extras, setExtras] = useState<DiaExtra[] | null>(null);

  useEffect(() => {
    let viva = true;
    obtenerMisKpis(diasVentas)
      .then((k) => {
        if (!viva) return;
        setTotal(k.total);
        setTickets(k.tickets);
      })
      .catch((e) => reportarError("No se pudieron cargar tus ventas", e));
    return () => {
      viva = false;
    };
  }, [diasVentas]);

  useEffect(() => {
    let viva = true;
    obtenerMisHorasExtra()
      .then((d) => {
        if (viva) setExtras(d);
      })
      .catch((e) => reportarError("No se pudieron cargar tus horas extra", e));
    return () => {
      viva = false;
    };
  }, []);

  const suma = extras ? sumarExtrasPorRango(extras, diasExtras) : null;

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-4 gap-4">
      <TarjetaKpi
        titulo="Vendi"
        rango={rangoVentas.v}
        setRango={setRangoVentas}
        grande={total === null ? "…" : moneda(total)}
        chico={tickets === null ? "cargando…" : `${entero(tickets)} tickets en este periodo`}
      />
      <TarjetaKpi
        titulo="Horas extras"
        rango={rangoExtras.v}
        setRango={setRangoExtras}
        grande={suma === null ? "…" : fmtExtra(suma.minutos)}
        chico={suma === null ? "cargando…" : `${suma.diasConExtra} ${suma.diasConExtra === 1 ? "día" : "días"} con extra en este periodo`}
      />
      <TarjetaProx titulo="Próximamente" />
      <TarjetaProx titulo="Próximamente" />
    </div>
  );
};

export default KpisMisTickets;
