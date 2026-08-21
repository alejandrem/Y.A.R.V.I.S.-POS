// ═══════════════════════════════════════════════════════════════════════════
// CONSTANTES DE FINANZAS — Configuración estática del módulo.
// Tarea única: centralizar la paleta de colores de las gráficas, los presets
// del selector de rango, la definición de tabs de navegación y las clases
// Tailwind compartidas de los inputs. Nada de lógica ni componentes.
// ═══════════════════════════════════════════════════════════════════════════

import {
  ICONO_DOLAR, ICONO_TRENDING, ICONO_GRAFICA, ICONO_CALCULADORA,
  ICONO_CAJA, ICONO_CAMPANA, inputCls,
} from "../../../../components/ui";

// ── Paletas de gráficas ─────────────────────────────────────────────────────

export const COLORS_PL = { ingresos: "#0a0a0a", gastos: "#525252", utilidad: "#22c55e" };
export const COLORS_PIE = ["#0a0a0a", "#3b82f6", "#22c55e", "#f59e0b", "#a855f7", "#ef4444", "#737373"];
export const COLORS_PREDICCION = { prediccion: "#3b82f6", confianza: "#3b82f620" };

// ── Presets del selector de rango ───────────────────────────────────────────

export const PRESETS_RANGO: { dias: number; label: string }[] = [
  { dias: 7, label: "7D" },
  { dias: 30, label: "30D" },
  { dias: 90, label: "3M" },
  { dias: 180, label: "6M" },
];

// ── Tabs / secciones del panel ──────────────────────────────────────────────

export type Seccion = "resumen" | "gastos" | "cortes" | "alertas" | "metricas";

export const TABS: { id: Seccion; label: string; icono: typeof ICONO_DOLAR }[] = [
  { id: "resumen", label: "Resumen", icono: ICONO_GRAFICA },
  { id: "gastos", label: "Gastos", icono: ICONO_CALCULADORA },
  { id: "cortes", label: "Cortes", icono: ICONO_CAJA },
  { id: "alertas", label: "Alertas", icono: ICONO_CAMPANA },
  { id: "metricas", label: "Metricicas", icono: ICONO_TRENDING },
];

// ── Clases de inputs ────────────────────────────────────────────────────────

/** Input de fecha para formularios en modales (ancho completo). */
export const inputFecha = `${inputCls} text-xs`;

/** Input de fecha compacto para el SelectorRango (ancho fijo). */
export const inputFechaCompacto = "px-3 py-2 rounded-xl bg-neutral-50 border border-neutral-100 text-[11px] font-bold text-neutral-900 focus:outline-none focus:border-neutral-950 focus:ring-4 focus:ring-neutral-950/5 transition-all w-[10.5rem]";
