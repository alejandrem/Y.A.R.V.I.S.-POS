// ═══════════════════════════════════════════════════════════════════════════
// SECCIÓN METRICAS — Métricas diarias de utilidad.
// Tarea única: renderizar la pestaña "Metricas": selector de rango, gráfica
// de área de utilidad neta diaria con línea de ventas totales y tabla
// detallada (ventas, COGS, utilidad bruta, gastos, utilidad neta, margen).
// ═══════════════════════════════════════════════════════════════════════════

import { AreaChart, Area, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from "recharts";
import { ICONO_TRENDING } from "../../../../components/ui";
import type { MetricasUtilidad } from "../../../types";
import { moneda, porcentaje, type RangoFechas } from "../nucleo/utilidades";
import SelectorRango from "./selector-rango";
import { SeccionGrafica, EmptyLargo } from "./ui-finanzas";

interface Props {
  metricas: MetricasUtilidad[];
  rango: RangoFechas;
  onRango: (r: RangoFechas) => void;
}

const tooltipCls = { backgroundColor: "#0a0a0a", border: "1px solid #262626", borderRadius: "16px", color: "#fff", fontSize: 11 };

export default function SeccionMetricas({ metricas, rango, onRango }: Props) {
  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div className="flex items-center gap-3">
          <div className="w-1.5 h-5 bg-neutral-950 rounded-full" />
          <h3 className="text-base sm:text-xl font-black text-neutral-950 uppercase tracking-tight">Metricicas Diarias</h3>
          <span className="px-3 py-1 bg-neutral-950 text-white text-[9px] font-black rounded-lg">{metricas.length} dias</span>
        </div>
        <SelectorRango rango={rango} onChange={onRango} />
      </div>

      {metricas.length > 0 ? (
        <>
          <SeccionGrafica titulo="Utilidad Neta Diaria" subtitulo="Tendencia del periodo">
            <ResponsiveContainer width="100%" height={280}>
              <AreaChart data={metricas}>
                <CartesianGrid strokeDasharray="3 3" stroke="#f5f5f5" />
                <XAxis dataKey="fecha" tick={{ fontSize: 10, fontWeight: 700 }} tickFormatter={(v) => v.slice(5)} />
                <YAxis tick={{ fontSize: 10, fontWeight: 700 }} tickFormatter={(v) => `$${(v / 1000).toFixed(0)}k`} />
                <Tooltip contentStyle={tooltipCls} formatter={(v) => moneda(Number(v))} />
                <Area type="monotone" dataKey="utilidad_neta" stroke="#22c55e" fill="#22c55e" fillOpacity={0.1} strokeWidth={2.5} />
                <Line type="monotone" dataKey="ventas_totales" stroke="#0a0a0a" strokeWidth={1.5} strokeDasharray="5 5" dot={false} />
              </AreaChart>
            </ResponsiveContainer>
          </SeccionGrafica>

          <div className="bg-white rounded-[2.5rem] border border-neutral-200 shadow-sm overflow-hidden">
            <div className="max-h-[560px] overflow-y-auto custom-scrollbar">
              <table className="w-full text-left border-collapse">
                <thead className="sticky top-0 z-10">
                  <tr className="bg-neutral-950">
                    <th className="px-8 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Fecha</th>
                    <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Ventas</th>
                    <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">COGS</th>
                    <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Util. Bruta</th>
                    <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Gastos</th>
                    <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Util. Neta</th>
                    <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Margen</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-neutral-50">
                  {metricas.map((m) => (
                    <tr key={m.fecha} className="group hover:bg-neutral-50/50 transition-all">
                      <td className="px-8 py-4 text-xs font-bold text-neutral-900">{m.fecha}</td>
                      <td className="px-6 py-4 text-xs font-black text-neutral-950">{moneda(m.ventas_totales)}</td>
                      <td className="px-6 py-4 text-xs font-bold text-neutral-500">{moneda(m.costo_ventas)}</td>
                      <td className="px-6 py-4 text-xs font-black text-neutral-900">{moneda(m.utilidad_bruta)}</td>
                      <td className="px-6 py-4 text-xs font-black text-red-500">{moneda(m.gastos_operativos)}</td>
                      <td className="px-6 py-4">
                        <span className={`text-xs font-black ${m.utilidad_neta >= 0 ? "text-emerald-600" : "text-red-500"}`}>
                          {moneda(m.utilidad_neta)}
                        </span>
                      </td>
                      <td className="px-6 py-4">
                        <span className={`text-[10px] font-black px-2.5 py-1 rounded-lg ${m.margen_neto_pct >= 0 ? "bg-emerald-50 text-emerald-600" : "bg-red-50 text-red-500"}`}>
                          {porcentaje(m.margen_neto_pct)}
                        </span>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </>
      ) : (
        <EmptyLargo icono={ICONO_TRENDING} mensaje="Sin metricicas" sub="Selecciona un rango con actividad" />
      )}
    </div>
  );
}
