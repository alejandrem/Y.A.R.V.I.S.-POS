// Ganancia neta por día con la venta apilada por empleado.
// La altura total la manda la utilidad neta (métrica auditada); los tramos
// muestran cuánto vendió cada quien en su turno ese día.
import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  ComposedChart, Bar, Line, XAxis, YAxis, CartesianGrid,
  Tooltip, ResponsiveContainer, Legend,
} from "recharts";
import { RangoDropdown, RANGOS_VENTANA, moneda, fechaCorta, Tarjeta, TituloSeccion, Vacio, Cargando } from "./controles";

interface VentaEmp {
  fecha: string;
  cajero: string;
  total: number;
}
interface Metrica {
  fecha: string;
  utilidad_neta: number;
}

const COLORES = ["#171717", "#525252", "#a3a3a3", "#d4d4d4"];

/** Apila venta por cajero por día (top 3 + Otros) y alinea la utilidad
 * neta por fecha. Puro y testeable: la UI solo dibuja lo que esto devuelve. */
export const apilarGanancia = (ventas: VentaEmp[], metricas: Metrica[]) => {
  const porCajero = new Map<string, number>();
  const porFecha = new Map<string, Map<string, number>>();
  for (const v of ventas) {
    porCajero.set(v.cajero, (porCajero.get(v.cajero) ?? 0) + v.total);
    if (!porFecha.has(v.fecha)) porFecha.set(v.fecha, new Map());
    const m = porFecha.get(v.fecha)!;
    m.set(v.cajero, (m.get(v.cajero) ?? 0) + v.total);
  }
  // Top 3 cajeros + "Otros" para que la pila siga legible.
  const top = [...porCajero.entries()].sort((a, b) => b[1] - a[1]).slice(0, 3).map(([c]) => c);
  const neta = new Map(metricas.map((m) => [m.fecha, m.utilidad_neta]));
  const fechas = [...new Set([...porFecha.keys(), ...neta.keys()])].sort();
  const filas = fechas.map((fecha) => {
    const m = porFecha.get(fecha) ?? new Map<string, number>();
    const fila: Record<string, string | number> = { fecha, neta: neta.get(fecha) ?? 0 };
    let otros = 0;
    for (const [c, t] of m) {
      if (top.includes(c)) fila[c] = t;
      else otros += t;
    }
    if (otros > 0) fila["Otros"] = otros;
    return fila;
  });
  const cols = [...top];
  if (filas.some((f) => typeof f["Otros"] === "number")) cols.push("Otros");
  return { datos: filas, cajeros: cols };
};

const isoHace = (dias: number) => {
  const d = new Date();
  d.setDate(d.getDate() - dias);
  return d.toISOString().slice(0, 10);
};

const GananciaNeta = () => {
  const [rango, setRango] = useState("30");
  const [customDias, setCustomDias] = useState<number | undefined>(undefined);
  const [ventas, setVentas] = useState<VentaEmp[] | null>(null);
  const [metricas, setMetricas] = useState<Metrica[] | null>(null);

  const dias = rango === "custom" ? customDias ?? 30 : Number(rango);

  useEffect(() => {
    let vivo = true;
    setVentas(null);
    setMetricas(null);
    const inicio = isoHace(dias);
    const fin = isoHace(0);
    Promise.all([
      invoke<VentaEmp[]>("get_ventas_por_empleado_dia", { days: dias }).catch(() => [] as VentaEmp[]),
      invoke<Metrica[]>("get_metricas_diarias", { fechaInicio: inicio, fechaFin: fin }).catch(() => [] as Metrica[]),
    ]).then(([v, m]) => {
      if (!vivo) return;
      setVentas(v);
      setMetricas(m);
    });
    return () => {
      vivo = false;
    };
  }, [dias]);

  const { datos, cajeros } = useMemo(() => {
    if (!ventas || !metricas) return { datos: null, cajeros: [] as string[] };
    return apilarGanancia(ventas, metricas);
  }, [ventas, metricas]);

  return (
    <Tarjeta>
      <TituloSeccion
        titulo="Ganancia neta"
        control={
          <RangoDropdown
            valor={rango}
            opciones={RANGOS_VENTANA}
            onChange={(v, c) => {
              setRango(v);
              setCustomDias(c);
            }}
          />
        }
      />
      {(!datos || !ventas || !metricas) && <Cargando />}
      {datos && datos.length === 0 && <Vacio mensaje="Sin ventas en este rango." />}
      {datos && datos.length > 0 && (
        <>
          <div className="h-72">
            <ResponsiveContainer width="100%" height="100%">
              <ComposedChart data={datos} margin={{ top: 5, right: 10, left: 0, bottom: 0 }} barCategoryGap="30%">
                <CartesianGrid stroke="#f0f0f0" vertical={false} />
                <XAxis dataKey="fecha" tickFormatter={fechaCorta} tick={{ fontSize: 10, fill: "#a3a3a3" }} minTickGap={40} />
                <YAxis
                  tickFormatter={(v: number) => `$${v >= 1000 || v <= -1000 ? `${(v / 1000).toFixed(0)}k` : v}`}
                  tick={{ fontSize: 10, fill: "#a3a3a3" }}
                  width={45}
                />
                <Tooltip formatter={(v: unknown, name: unknown) => [moneda(Number(v ?? 0)), name === "neta" ? "Ganancia neta" : `Vende ${String(name)}`]} />
                <Legend formatter={(v: string) => <span className="text-[11px] font-bold text-neutral-600">{v === "neta" ? "Ganancia neta" : v}</span>} />
                {cajeros.map((c, i) => (
                  <Bar key={c} dataKey={c} stackId="venta" fill={COLORES[i % COLORES.length]} radius={i === cajeros.length - 1 ? [6, 6, 0, 0] : 0} />
                ))}
                <Line type="monotone" dataKey="neta" stroke="#22c55e" strokeWidth={2.5} dot={false} />
              </ComposedChart>
            </ResponsiveContainer>
          </div>
          <p className="text-[11px] text-neutral-400 font-bold mt-3">
            Barras = venta del día por cajero <span className="text-neutral-500">(nombre impreso en el ticket, no tu plantilla)</span> · línea verde = ganancia neta (ventas − costo − gastos − IVA)
          </p>
        </>
      )}
    </Tarjeta>
  );
};

export default GananciaNeta;
