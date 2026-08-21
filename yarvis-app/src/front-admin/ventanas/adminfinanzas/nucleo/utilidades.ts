// ═══════════════════════════════════════════════════════════════════════════
// UTILIDADES DE FINANZAS — Funciones puras de formato y fechas.
// Tarea única: convertir datos crudos en texto presentable (moneda MXN,
// porcentajes con signo, fechas relativas) y construir rangos de fechas
// para el selector de periodo. Sin React, sin estado, sin side-effects.
// ═══════════════════════════════════════════════════════════════════════════

export type RangoFechas = { inicio: string; fin: string };

/** Formatea un número como moneda MXN (es-MX). */
export const moneda = (v: number) =>
  new Intl.NumberFormat("es-MX", { style: "currency", currency: "MXN" }).format(v);

/** Formatea un número como porcentaje con signo (+/-). */
export const porcentaje = (v: number) => `${v >= 0 ? "+" : ""}${v.toFixed(1)}%`;

/** Convierte una fecha ISO a texto relativo ("Hoy", "Ayer", "Hace N dias", "12 ene"). */
export const fechaRelativa = (f: string) => {
  const d = new Date(f);
  const hoy = new Date();
  const diff = Math.floor((hoy.getTime() - d.getTime()) / 86400000);
  if (diff === 0) return "Hoy";
  if (diff === 1) return "Ayer";
  if (diff < 7) return `Hace ${diff} dias`;
  return d.toLocaleDateString("es-MX", { day: "2-digit", month: "short" });
};

/** Rango de los últimos N dias hasta hoy (formato YYYY-MM-DD). */
export const rangoDeDias = (dias: number): RangoFechas => {
  const fin = new Date();
  const ini = new Date();
  ini.setDate(ini.getDate() - dias);
  return { inicio: ini.toISOString().slice(0, 10), fin: fin.toISOString().slice(0, 10) };
};

/** Fecha de hoy en formato YYYY-MM-DD. */
export const hoyISO = () => new Date().toISOString().slice(0, 10);
