// Ventas por método de pago en el rango (agregado client-side de tickets).
import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { PieChart, Pie, Cell, Tooltip, ResponsiveContainer, Legend } from "recharts";
import { RangoDropdown, RANGOS_VENTANA, moneda, Tarjeta, TituloSeccion, Vacio, Cargando } from "./controles";

interface TicketDb {
  id: number;
  fecha: string;
  total: number;
  metodo_pago: string;
}

const COLORES = ["#171717", "#737373", "#22c55e", "#3b82f6", "#f59e0b"];

/** Agrega tickets por método de pago en los últimos `dias` (puro, testeable). */
export const agregarPorMetodo = (tickets: TicketDb[], dias: number) => {
  const corte = Date.now() - dias * 86400000;
  const agg = new Map<string, number>();
  for (const t of tickets) {
    if (new Date(t.fecha).getTime() < corte) continue;
    const k = (t.metodo_pago || "efectivo").toLowerCase();
    agg.set(k, (agg.get(k) ?? 0) + t.total);
  }
  return [...agg.entries()]
    .map(([name, value]) => ({ name: name[0].toUpperCase() + name.slice(1), value }))
    .sort((a, b) => b.value - a.value);
};

const MetodosPago = () => {
  const [rango, setRango] = useState("30");
  const [customDias, setCustomDias] = useState<number | undefined>(undefined);
  const [tickets, setTickets] = useState<TicketDb[] | null>(null);

  useEffect(() => {
    let vivo = true;
    setTickets(null);
    invoke<TicketDb[]>("get_tickets")
      .then((r) => vivo && setTickets(r || []))
      .catch(() => vivo && setTickets([]));
    return () => {
      vivo = false;
    };
  }, []);

  const dias = rango === "custom" ? customDias ?? 30 : Number(rango);
  const datos = useMemo(() => {
    if (!tickets) return null;
    return agregarPorMetodo(tickets, dias);
  }, [tickets, dias]);

  const total = (datos ?? []).reduce((a, d) => a + d.value, 0);

  return (
    <Tarjeta>
      <TituloSeccion
        titulo="Método de pago"
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
      {datos === null && <Cargando />}
      {datos !== null && datos.length === 0 && <Vacio mensaje="Sin ventas en este rango." />}
      {datos !== null && datos.length > 0 && (
        <>
          <div className="h-56">
            <ResponsiveContainer width="100%" height="100%">
              <PieChart>
                <Pie data={datos} dataKey="value" nameKey="name" innerRadius={55} outerRadius={85} paddingAngle={3}>
                  {datos.map((_, i) => (
                    <Cell key={i} fill={COLORES[i % COLORES.length]} />
                  ))}
                </Pie>
                <Tooltip formatter={(v: unknown) => moneda(Number(v ?? 0))} />
                <Legend formatter={(v: string) => <span className="text-[11px] font-bold text-neutral-600">{v}</span>} />
              </PieChart>
            </ResponsiveContainer>
          </div>
          <p className="text-center text-sm font-black text-neutral-900 mt-2">{moneda(total)} en el periodo</p>
        </>
      )}
    </Tarjeta>
  );
};

export default MetodosPago;
