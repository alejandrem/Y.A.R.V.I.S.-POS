// ═══════════════════════════════════════════════════════════════════════════
// SECCIÓN RESUMEN — Panel ejecutivo financiero.
// Tarea única: renderizar la pestaña "Resumen": selector de rango, KPIs
// (ventas, utilidad, margen, punto de equilibrio), gráficas P&L, gastos por
// categoría, ventas vs gastos, tendencia de cortes Z, predicciones
// Holt-Winters y el panel oscuro de punto de equilibrio.
// ═══════════════════════════════════════════════════════════════════════════

import { AreaChart, Area, BarChart, Bar, PieChart, Pie, Cell, Line, ComposedChart, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend } from "recharts";
import { MorphIcon } from "morphicons/react";
import { ICONO_DOLAR, ICONO_TRENDING, ICONO_GRAFICA, ICONO_TARGET } from "../../../../components/ui";
import type {
  ResumenPeriodo, PuntoEquilibrio, DatoGraficaPL,
  DatoGraficaGastosCategoria, DatoGraficaCortesZ,
} from "../../../types";
import { COLORS_PL, COLORS_PIE, COLORS_PREDICCION } from "../nucleo/constantes";
import { moneda, porcentaje, type RangoFechas } from "../nucleo/utilidades";
import SelectorRango from "./selector-rango";
import { KPI, SeccionGrafica, EmptyGrafica } from "./ui-finanzas";

const tooltipCls = { backgroundColor: "#0a0a0a", border: "1px solid #262626", borderRadius: "16px", color: "#fff", fontSize: 11 };
const fmtEjeK = (v: number) => `$${(v / 1000).toFixed(0)}k`;
const fmtTickFecha = (v: string) => v.slice(5);

interface Props {
  resumen: ResumenPeriodo | null;
  puntoEq: PuntoEquilibrio | null;
  plData: DatoGraficaPL[];
  gastosCat: DatoGraficaGastosCategoria[];
  ventasGastos: DatoGraficaPL[];
  cortesZ: DatoGraficaCortesZ[];
  predicciones: any[];
  diasPrediccion: number;
  rango: RangoFechas;
  onRango: (r: RangoFechas) => void;
  onDiasPrediccion: (d: number) => void;
}

export default function SeccionResumen({ resumen, puntoEq, plData, gastosCat, ventasGastos, cortesZ, predicciones, diasPrediccion, rango, onRango, onDiasPrediccion }: Props) {
  return (
    <div className="space-y-10">
      <div className="flex justify-end">
        <SelectorRango rango={rango} onChange={onRango} />
      </div>

      {/* KPIs - TARJETAS GORDITAS CON NEGRO INTENSO */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4 sm:gap-5">
        <KPI icono={ICONO_DOLAR} label="Ventas Totales" valor={moneda(resumen?.total_ventas ?? 0)} color="neutral" />
        <KPI icono={ICONO_TRENDING} label="Utilidad Neta" valor={moneda(resumen?.total_utilidad_neta ?? 0)} color={(resumen?.total_utilidad_neta ?? 0) >= 0 ? "verde" : "rojo"} />
        <KPI icono={ICONO_GRAFICA} label="Margen Neto" valor={porcentaje(resumen?.margen_promedio_pct ?? 0)} color={(resumen?.margen_promedio_pct ?? 0) >= 0 ? "verde" : "rojo"} />
        <KPI icono={ICONO_TARGET} label="Punto Equilibrio" valor={moneda(puntoEq?.ventas_necesarias ?? 0)} color="neutral" />
      </div>

      {/* GRAFICA P&L */}
      <SeccionGrafica titulo="Perdidas y Ganancias" subtitulo="Ingresos, gastos y utilidad neta">
        {plData.length > 0 ? (
          <ResponsiveContainer width="100%" height={320}>
            <AreaChart data={plData}>
              <CartesianGrid strokeDasharray="3 3" stroke="#f5f5f5" />
              <XAxis dataKey="fecha" tick={{ fontSize: 10, fontWeight: 700 }} tickFormatter={fmtTickFecha} />
              <YAxis tick={{ fontSize: 10, fontWeight: 700 }} tickFormatter={fmtEjeK} />
              <Tooltip contentStyle={tooltipCls} formatter={(v) => moneda(Number(v))} />
              <Area type="monotone" dataKey="ingresos" stroke={COLORS_PL.ingresos} fill={COLORS_PL.ingresos} fillOpacity={0.08} strokeWidth={2.5} />
              <Area type="monotone" dataKey="gastos" stroke={COLORS_PL.gastos} fill={COLORS_PL.gastos} fillOpacity={0.05} strokeWidth={2} strokeDasharray="5 5" />
              <Area type="monotone" dataKey="utilidad_neta" stroke={COLORS_PL.utilidad} fill={COLORS_PL.utilidad} fillOpacity={0.1} strokeWidth={2} />
            </AreaChart>
          </ResponsiveContainer>
        ) : (
          <EmptyGrafica mensaje="Sin datos de P&L en este periodo" />
        )}
      </SeccionGrafica>

      {/* FILA: GASTOS CATEGORIA + VENTAS VS GASTOS */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-5">
        <SeccionGrafica titulo="Gastos por Categoria" subtitulo="Distribucion del periodo">
          {gastosCat.length > 0 ? (
            <ResponsiveContainer width="100%" height={280}>
              <PieChart>
                <Pie data={gastosCat} dataKey="monto" nameKey="categoria" cx="50%" cy="50%" outerRadius={90} innerRadius={50} paddingAngle={3}>
                  {gastosCat.map((_, i) => <Cell key={i} fill={COLORS_PIE[i % COLORS_PIE.length]} />)}
                </Pie>
                <Tooltip contentStyle={tooltipCls} formatter={(v) => moneda(Number(v))} />
                <Legend wrapperStyle={{ fontSize: 10, fontWeight: 700 }} />
              </PieChart>
            </ResponsiveContainer>
          ) : (
            <EmptyGrafica mensaje="Sin gastos registrados" />
          )}
        </SeccionGrafica>

        <SeccionGrafica titulo="Ventas vs Gastos" subtitulo="Ultimos 6 meses">
          {ventasGastos.length > 0 ? (
            <ResponsiveContainer width="100%" height={280}>
              <BarChart data={ventasGastos}>
                <CartesianGrid strokeDasharray="3 3" stroke="#f5f5f5" />
                <XAxis dataKey="fecha" tick={{ fontSize: 10, fontWeight: 700 }} />
                <YAxis tick={{ fontSize: 10, fontWeight: 700 }} tickFormatter={fmtEjeK} />
                <Tooltip contentStyle={tooltipCls} formatter={(v) => moneda(Number(v))} />
                <Bar dataKey="ingresos" fill="#0a0a0a" radius={[8, 8, 0, 0]} />
                <Bar dataKey="gastos" fill="#525252" radius={[8, 8, 0, 0]} />
              </BarChart>
            </ResponsiveContainer>
          ) : (
            <EmptyGrafica mensaje="Sin datos mensuales" />
          )}
        </SeccionGrafica>
      </div>

      {/* FILA: TENDENCIA CORTES Z + PREDICCIONES */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-5">
        <SeccionGrafica titulo="Tendencia Cortes Z" subtitulo="Ventas por turno de cierre">
          {cortesZ.length > 0 ? (
            <ResponsiveContainer width="100%" height={280}>
              <ComposedChart data={cortesZ}>
                <CartesianGrid strokeDasharray="3 3" stroke="#f5f5f5" />
                <XAxis dataKey="fecha" tick={{ fontSize: 10, fontWeight: 700 }} tickFormatter={fmtTickFecha} />
                <YAxis tick={{ fontSize: 10, fontWeight: 700 }} tickFormatter={fmtEjeK} />
                <Tooltip contentStyle={tooltipCls} formatter={(v) => moneda(Number(v))} />
                <Bar dataKey="total_ventas" fill="#0a0a0a" radius={[8, 8, 0, 0]} />
                <Line type="monotone" dataKey="diferencia" stroke="#ef4444" strokeWidth={2} dot={{ r: 3 }} />
              </ComposedChart>
            </ResponsiveContainer>
          ) : (
            <EmptyGrafica mensaje="Sin cortes Z en este periodo" />
          )}
        </SeccionGrafica>

        <SeccionGrafica
          titulo="Predicciones de Ventas"
          subtitulo="Modelo Holt-Winters"
          accion={
            <div className="flex bg-neutral-100 p-0.5 rounded-xl">
              {[15, 30, 90].map((d) => (
                <button
                  key={d}
                  onClick={() => onDiasPrediccion(d)}
                  className={`px-3 py-1.5 text-[9px] font-black rounded-lg transition-all ${diasPrediccion === d ? "bg-neutral-950 text-white shadow-md" : "text-neutral-400 hover:text-neutral-700"}`}
                >
                  {d}D
                </button>
              ))}
            </div>
          }
        >
          {predicciones.length > 0 ? (
            <ResponsiveContainer width="100%" height={280}>
              <AreaChart data={predicciones}>
                <CartesianGrid strokeDasharray="3 3" stroke="#f5f5f5" />
                <XAxis dataKey="fecha" tick={{ fontSize: 10, fontWeight: 700 }} tickFormatter={fmtTickFecha} />
                <YAxis tick={{ fontSize: 10, fontWeight: 700 }} tickFormatter={fmtEjeK} />
                <Tooltip contentStyle={tooltipCls} formatter={(v) => moneda(Number(v))} />
                <Area type="monotone" dataKey="prediccion" stroke={COLORS_PREDICCION.prediccion} fill={COLORS_PREDICCION.confianza} strokeWidth={2.5} strokeDasharray="6 3" />
                <Area type="monotone" dataKey="maximo" stroke="transparent" fill={COLORS_PREDICCION.prediccion} fillOpacity={0.06} />
                <Area type="monotone" dataKey="minimo" stroke="transparent" fill="#fff" fillOpacity={0} />
              </AreaChart>
            </ResponsiveContainer>
          ) : (
            <EmptyGrafica mensaje="Generando predicciones..." />
          )}
        </SeccionGrafica>
      </div>

      {/* PUNTO DE EQUILIBRIO - PANEL OSCURO INTENSO */}
      {puntoEq && (
        <div className="bg-neutral-950 rounded-[2.5rem] p-6 sm:p-10 shadow-2xl relative overflow-hidden">
          <div className="absolute top-0 right-0 w-80 h-80 bg-white/[0.03] rounded-full blur-3xl -translate-y-1/3 translate-x-1/3" />
          <div className="absolute bottom-0 left-0 w-48 h-48 bg-emerald-500/[0.04] rounded-full blur-3xl translate-y-1/2 -translate-x-1/2" />
          <div className="relative">
            <div className="flex items-center gap-3 mb-8">
              <div className="w-12 h-12 rounded-2xl bg-white/10 flex items-center justify-center">
                <MorphIcon icon={ICONO_TARGET} size={22} strokeWidth={2} spring="smooth" className="text-white" />
              </div>
              <div>
                <h3 className="text-base font-black text-white uppercase tracking-tight">Punto de Equilibrio</h3>
                <p className="text-[9px] font-black text-white/30 uppercase tracking-widest">Analisis break-even ultimos 30 dias</p>
              </div>
            </div>
            <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
              <div className="bg-white/5 rounded-2xl p-5 border border-white/5">
                <p className="text-[9px] font-black text-white/30 uppercase tracking-widest">Gastos Fijos Mensuales</p>
                <p className="text-2xl font-black text-white mt-2">{moneda(puntoEq.gastos_fijos_mensuales)}</p>
              </div>
              <div className="bg-white/5 rounded-2xl p-5 border border-white/5">
                <p className="text-[9px] font-black text-white/30 uppercase tracking-widest">Margen de Contribucion</p>
                <p className="text-2xl font-black text-emerald-400 mt-2">{porcentaje(puntoEq.margen_contribucion_pct)}</p>
              </div>
              <div className="bg-white/5 rounded-2xl p-5 border border-white/5">
                <p className="text-[9px] font-black text-white/30 uppercase tracking-widest">Ventas Necesarias</p>
                <p className="text-2xl font-black text-white mt-2">{moneda(puntoEq.ventas_necesarias)}</p>
              </div>
              <div className="bg-white/5 rounded-2xl p-5 border border-white/5">
                <p className="text-[9px] font-black text-white/30 uppercase tracking-widest">Tickets Necesarios</p>
                <p className="text-2xl font-black text-white mt-2">{puntoEq.tickets_necesarios}</p>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
