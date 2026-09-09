// ═══════════════════════════════════════════════════════════════════════════
// GRÁFICA DÍA vs SUELDO — Barras de lo vendido por día (solo YO) contra
// una ReferenceLine de mi sueldo diario. Mismo Tarjeta/Titulo/Cargando/
// Vacio y estilo recharts que adminventas/graficas/pronostico.tsx.
// El sueldo sale de get_employee_profile (para rol empleado el backend
// usa session.user_id aunque se mande nombre).
// ═══════════════════════════════════════════════════════════════════════════

import { useEffect, useMemo, useState } from "react";
import {
  ComposedChart, Bar, XAxis, YAxis, CartesianGrid,
  Tooltip, ResponsiveContainer, ReferenceLine,
} from "recharts";
import {
  moneda, fechaCorta, Tarjeta, TituloSeccion, Vacio, Cargando, RangoDropdown,
} from "../../../../front-admin/ventanas/adminventas/graficas/controles";
import { RANGOS_MIS, rangoMisADias, type RangoSel } from "./kpis-mis-tickets";
import { invokeTauri, reportarError } from "../../../../services/tauri";
import { obtenerMisVentasPorDia, type VentaDia } from "../../../../services/misventas";

export interface PuntoDia {
  fecha: string; // YYYY-MM-DD
  total: number;
  etiqueta: string; // "06 sep"
}

/** Rellena los días sin venta con ceros, ordenados de viejo a nuevo (puro, testeable). */
export const rellenarDiasVentas = (
  ventas: VentaDia[],
  dias: number,
  hoyISO?: string,
): PuntoDia[] => {
  const hoy = hoyISO ?? new Date().toISOString().slice(0, 10);
  const mapa = new Map(ventas.map((v) => [v.fecha.slice(0, 10), v.total]));
  const puntos: PuntoDia[] = [];
  for (let i = dias - 1; i >= 0; i--) {
    const d = new Date(`${hoy}T00:00:00`);
    d.setDate(d.getDate() - i);
    const iso = d.toISOString().slice(0, 10);
    puntos.push({ fecha: iso, total: mapa.get(iso) ?? 0, etiqueta: fechaCorta(iso) });
  }
  return puntos;
};

interface PerfilMin {
  profile: { salario_diario: number };
}

// Rango propio: solo afecta a esta gráfica (igual que en adminventas).
const GraficaDiaSueldo = ({ operatorName }: { operatorName: string }) => {
  const [rango, setRangoRaw] = useState<RangoSel>({ v: "7" });
  const setRango = (v: string, c?: number) => setRangoRaw({ v, c });
  const dias = rangoMisADias(rango.v, rango.c);
  const [ventas, setVentas] = useState<VentaDia[] | null>(null);
  const [sueldo, setSueldo] = useState<number | null>(null);

  useEffect(() => {
    let viva = true;
    setVentas(null);
    obtenerMisVentasPorDia(dias)
      .then((v) => {
        if (viva) setVentas(v);
      })
      .catch((e) => reportarError("No se pudo cargar tu gráfica de ventas", e));
    return () => {
      viva = false;
    };
  }, [dias]);

  useEffect(() => {
    let viva = true;
    if (!operatorName) {
      setSueldo(0);
      return;
    }
    invokeTauri<PerfilMin>("get_employee_profile", { nombre: operatorName })
      .then((p) => {
        if (viva) setSueldo(p?.profile?.salario_diario ?? 0);
      })
      .catch(() => {
        if (viva) setSueldo(0);
      });
    return () => {
      viva = false;
    };
  }, [operatorName]);

  const puntos = useMemo(
    () => (ventas ? rellenarDiasVentas(ventas, dias) : null),
    [ventas, dias],
  );

  return (
    <Tarjeta>
      <TituloSeccion
        titulo="Vendido por día vs mi sueldo"
        control={<RangoDropdown valor={rango.v} opciones={RANGOS_MIS} onChange={setRango} />}
      />
      {puntos === null || sueldo === null ? (
        <Cargando />
      ) : puntos.every((p) => p.total === 0) ? (
        <Vacio mensaje="Aún no tienes ventas en este periodo" />
      ) : (
        <div className="h-64">
          <ResponsiveContainer width="100%" height="100%">
            <ComposedChart data={puntos} margin={{ top: 8, right: 8, bottom: 0, left: 0 }}>
              <CartesianGrid strokeDasharray="3 3" stroke="#e5e5e5" vertical={false} />
              <XAxis
                dataKey="etiqueta"
                tick={{ fontSize: 10, fontWeight: 700, fill: "#a3a3a3" }}
                axisLine={false}
                tickLine={false}
                interval="preserveStartEnd"
              />
              <YAxis
                tick={{ fontSize: 10, fontWeight: 700, fill: "#a3a3a3" }}
                axisLine={false}
                tickLine={false}
                width={70}
                tickFormatter={(v: number) => `$${Number(v || 0).toLocaleString("es-MX", { maximumFractionDigits: 0 })}`}
              />
              <Tooltip
                formatter={(value) => [moneda(Number(value)), "Vendido"]}
                labelFormatter={(_, payload) => {
                  const p = payload?.[0]?.payload as PuntoDia | undefined;
                  return p ? `${p.etiqueta} · sueldo ${moneda(sueldo)}` : "";
                }}
              />
              <ReferenceLine
                y={sueldo}
                stroke="#10b981"
                strokeDasharray="6 4"
                label={{ value: `sueldo ${moneda(sueldo)}`, fontSize: 10, fontWeight: 800, fill: "#10b981", position: "insideTopRight" }}
              />
              <Bar dataKey="total" fill="#171717" radius={[8, 8, 2, 2]} maxBarSize={38} />
            </ComposedChart>
          </ResponsiveContainer>
        </div>
      )}
      <p className="text-[10px] text-neutral-400 font-bold mt-4">
        Barras = lo que vendiste tú · línea verde = tu sueldo diario
      </p>
    </Tarjeta>
  );
};

export default GraficaDiaSueldo;
