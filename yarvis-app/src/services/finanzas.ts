// ═══════════════════════════════════════════════════════════════════════════
// SERVICIO DE FINANZAS — Única fuente de verdad para comandos financieros.
// Los componentes importan estas funciones en vez de `invoke` crudo.
// Los errores se propagan (throw) y el componente los muestra con
// `reportarError` (toast + log centralizado).
// ═══════════════════════════════════════════════════════════════════════════

import { invokeTauri, descargarTexto, descargarBinario } from "./tauri";
import type {
  ResumenPeriodo, DatoGraficaPL, DatoGraficaGastosCategoria, DatoGraficaCortesZ,
  PuntoEquilibrio, AlertaFinanciera, GastoRecurrente, CorteCaja, MetricasUtilidad,
  MovimientoCaja, CierreCorte,
} from "../front-admin/types";

export interface Rango {
  inicio: string;
  fin: string;
}

// ── Resumen / gráficas / métricas ────────────────────────────────────────────

export const obtenerResumenPeriodo = (r: Rango) =>
  invokeTauri<ResumenPeriodo>("get_resumen_periodo", { fechaInicio: r.inicio, fechaFin: r.fin });

export const obtenerPuntoEquilibrio = () =>
  invokeTauri<PuntoEquilibrio>("get_punto_equilibrio");

export const obtenerDatosGraficaPL = (r: Rango) =>
  invokeTauri<DatoGraficaPL[]>("get_datos_grafica_pl", { fechaInicio: r.inicio, fechaFin: r.fin, granularidad: "dia" });

export const obtenerGastosPorCategoria = (r: Rango) =>
  invokeTauri<DatoGraficaGastosCategoria[]>("get_gastos_por_categoria", { fechaInicio: r.inicio, fechaFin: r.fin });

export const obtenerVentasVsGastos = (meses = 6) =>
  invokeTauri<DatoGraficaPL[]>("get_ventas_vs_gastos_mensual", { meses });

export const obtenerTendenciaCortesZ = (r: Rango) =>
  invokeTauri<DatoGraficaCortesZ[]>("get_tendencia_cortes_z", { fechaInicio: r.inicio, fechaFin: r.fin });

export const obtenerPrediccionesFinancieras = async (dias: number): Promise<any[]> => {
  const res = await invokeTauri<{ data: any[] }>("get_predicciones_financieras", { days: dias });
  return res.data ?? [];
};

export const obtenerMetricasDiarias = (r: Rango) =>
  invokeTauri<MetricasUtilidad[]>("get_metricas_diarias", { fechaInicio: r.inicio, fechaFin: r.fin });

// ── Gastos ───────────────────────────────────────────────────────────────────

export const obtenerGastos = () =>
  invokeTauri<GastoRecurrente[]>("get_gastos_recurrentes");

export const eliminarGasto = (id: number) =>
  invokeTauri("eliminar_gasto", { id });

// ── Cortes ───────────────────────────────────────────────────────────────────

export const obtenerCortes = (r: Rango) =>
  invokeTauri<CorteCaja[]>("get_cortes_caja", {
    filtros: { cajero_id: null, fecha_inicio: r.inicio, fecha_fin: r.fin, turno: null, tipo_corte: null, estado: null },
  });

export const obtenerMovimientosCorte = (corteId: number) =>
  invokeTauri<MovimientoCaja[]>("get_movimientos_corte", { corteId });

export const cerrarCorte = (args: {
  corteId: number;
  totalVentas: number;
  totalEfectivo: number;
  totalTarjeta: number;
  totalTransferencia: number;
  entradasManuales: number;
  retirosManuales: number;
}) =>
  invokeTauri<CierreCorte>("cerrar_corte", {
    corteId: args.corteId,
    totalVentas: args.totalVentas,
    totalEfectivo: args.totalEfectivo,
    totalTarjeta: args.totalTarjeta,
    totalTransferencia: args.totalTransferencia,
    entradasManuales: args.entradasManuales,
    retirosManuales: args.retirosManuales,
  });

// ── Alertas ──────────────────────────────────────────────────────────────────

export const obtenerAlertas = async (): Promise<AlertaFinanciera[]> => {
  await invokeTauri("generar_alertas_automaticas");
  return invokeTauri<AlertaFinanciera[]>("get_alertas", { soloNoLeidas: false });
};

export const marcarAlertaLeida = (id: number) =>
  invokeTauri("marcar_alerta_leida", { id });

// ── Exports (descargan el archivo directo) ───────────────────────────────────

export async function exportarGastosCsv(r: Rango): Promise<void> {
  const csv = await invokeTauri<string>("exportar_gastos_csv", { fechaInicio: r.inicio, fechaFin: r.fin });
  descargarTexto(`gastos-${r.inicio}_${r.fin}.csv`, csv);
}

export async function exportarBalancePdf(r: Rango): Promise<void> {
  const bytes = await invokeTauri<number[]>("exportar_balance_pdf", { fechaInicio: r.inicio, fechaFin: r.fin });
  descargarBinario(`balance-${r.inicio}_${r.fin}.pdf`, bytes, "application/pdf");
}
