// Las 4 tarjetas KPI: vendido, tickets, ticket promedio y ganancia neta.
// Cada una trae su propio selector y solo se afecta a sí misma.
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { RangoDropdown, RANGOS_KPI, moneda, entero, variacion, Cargando } from "./controles";

interface KpiDia {
  total: number;
  tickets: number;
  ticket_promedio: number;
  utilidad_neta: number;
  margen_pct: number;
}

const rangoADias = (valor: string, custom?: number): { dias: number; desfase: number } => {
  if (valor === "custom" && custom) return { dias: custom, desfase: 0 };
  const op = RANGOS_KPI.find((o) => o.valor === valor);
  return { dias: op?.dias ?? 1, desfase: op?.desfase ?? 0 };
};

const TarjetaKpi = ({
  titulo,
  rango,
  setRango,
  grande,
  chico,
  delta,
}: {
  titulo: string;
  rango: string;
  setRango: (v: string, c?: number) => void;
  grande: string;
  chico: string;
  delta?: { texto: string; sube: boolean } | null;
}) => (
  <div className="bg-white rounded-[2rem] border border-neutral-100 shadow-xl p-5 sm:p-6">
    <div className="flex items-center justify-between gap-2 mb-3">
      <p className="text-[10px] font-black uppercase tracking-widest text-neutral-400">{titulo}</p>
      <RangoDropdown valor={rango} opciones={RANGOS_KPI} onChange={setRango} />
    </div>
    <p className="text-2xl sm:text-3xl font-black text-neutral-900">{grande}</p>
    <p className="text-[11px] text-neutral-400 font-bold mt-1">{chico}</p>
    {delta && (
      <p className={`text-[11px] font-black mt-1 ${delta.sube ? "text-emerald-600" : "text-red-500"}`}>
        {delta.texto} vs periodo anterior
      </p>
    )}
  </div>
);

const Kpis = () => {
  const [rangos, setRangos] = useState<Record<string, { v: string; c?: number }>>({
    vendido: { v: "hoy" },
    tickets: { v: "hoy" },
    promedio: { v: "hoy" },
    ganancia: { v: "hoy" },
  });
  const [datos, setDatos] = useState<Record<string, { actual: KpiDia; anterior: KpiDia } | null>>({});
  const [cargando, setCargando] = useState(true);

  useEffect(() => {
    let viva = true;
    const cargar = async () => {
      setCargando(true);
      const entradas = Object.entries(rangos);
      const res = await Promise.all(
        entradas.map(async ([k, r]) => {
          const { dias, desfase } = rangoADias(r.v, r.c);
          try {
            const d = await invoke<{ actual: KpiDia; anterior: KpiDia }>("get_kpis_ventas", { dias, desfase });
            return [k, d] as const;
          } catch {
            return [k, null] as const;
          }
        }),
      );
      if (viva) {
        setDatos(Object.fromEntries(res));
        setCargando(false);
      }
    };
    cargar();
    return () => {
      viva = false;
    };
  }, [rangos]);

  const setRango = (k: string) => (v: string, c?: number) =>
    setRangos((p) => ({ ...p, [k]: { v, c } }));

  if (cargando && !Object.keys(datos).length) return <Cargando />;

  const card = (k: string, titulo: string, grande: (d: KpiDia) => string, chico: (d: KpiDia) => string, base: (d: KpiDia) => number) => {
    const d = datos[k];
    if (!d) return <TarjetaKpi titulo={titulo} rango={rangos[k].v} setRango={setRango(k)} grande="—" chico="Sin datos" delta={null} />;
    const v = variacion(base(d.actual), base(d.anterior));
    return (
      <TarjetaKpi
        titulo={titulo}
        rango={rangos[k].v}
        setRango={setRango(k)}
        grande={grande(d.actual)}
        chico={chico(d.actual)}
        delta={{ texto: v.texto, sube: v.sube }}
      />
    );
  };

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-4 gap-4">
      {card("vendido", "Vendido", (d) => moneda(d.total), (d) => `${entero(d.tickets)} tickets`, (d) => d.total)}
      {card("tickets", "Tickets", (d) => entero(d.tickets), (d) => `${moneda(d.total)} acumulado`, (d) => d.tickets)}
      {card("promedio", "Ticket promedio", (d) => moneda(d.ticket_promedio), () => "por ticket cobrado", (d) => d.ticket_promedio)}
      {card("ganancia", "Ganancia neta", (d) => moneda(d.utilidad_neta), (d) => `margen: ${d.margen_pct.toFixed(1)}%`, (d) => d.utilidad_neta)}
    </div>
  );
};

export default Kpis;
