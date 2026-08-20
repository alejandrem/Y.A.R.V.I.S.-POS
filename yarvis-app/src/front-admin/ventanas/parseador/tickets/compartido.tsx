// Piezas comunes del módulo de tickets: tipos, helpers de normalización,
// la tarjeta de progreso y la barra de pasos (01/02/03).
import type { ReactNode } from "react";

export interface CatalogItem {
  id: number | null;
  nombre: string;
  descripcion: string | null;
  precio_costo: number;
  precio_venta: number;
  stock: number;
  vendido: number;
  stock_minimo: number;
  codigo_barras: string | null;
  categoria: string | null;
}

export interface ArchivoTicket {
  nombre: string;
  ruta: string;
  tamano: number;
  preview: string;
}

export interface TrainingProgress {
  indice: number;
  total: number;
  archivo: string;
  estado: "ok" | "error";
  mensaje: string;
}

export interface BatchProgress {
  type: "progress" | "complete";
  procesados: number;
  total?: number;
  total_archivos?: number;
  exitosos: number;
  errores: number;
  ventas_creadas?: number;
  items_insertados?: number;
  productos_nuevos?: number;
  productos_existentes?: number;
  duplicados_detectados?: number;
}

export interface CalibrationResult {
  mapeo: Record<string, unknown>;
  analizados: number;
  total_muestras: number;
  votos_ganadores: number;
}

export type Phase = "catalogo" | "carpeta" | "calibrando" | "procesando" | "completo";

export const toNumber = (value: unknown, fallback = 0) => {
  const number = Number(value);
  return Number.isFinite(number) ? number : fallback;
};

export const normalizeCatalogItem = (item: any): CatalogItem => ({
  id: item.id ?? null,
  nombre: String(item.nombre ?? item.producto ?? "").trim(),
  descripcion: item.descripcion ?? null,
  precio_costo: toNumber(item.precio_costo),
  precio_venta: toNumber(item.precio_venta),
  stock: toNumber(item.stock),
  vendido: toNumber(item.vendido),
  stock_minimo: toNumber(item.stock_minimo, 5),
  codigo_barras: item.codigo_barras ?? null,
  categoria: item.categoria ?? null,
});

export const formatSize = (bytes: number) => {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
};

export const errorMessage = (error: unknown) => error instanceof Error ? error.message : String(error);

export const ProgressCard = ({ title, subtitle, current, total, percent, children }: { title: string; subtitle: string; current: number; total: number; percent: number; children: ReactNode }) => (
  <section className="bg-neutral-950 text-neutral-50 rounded-[2.5rem] shadow-xl p-6 sm:p-10">
    <div className="flex items-center gap-4"><div className="w-12 h-12 rounded-2xl bg-neutral-100 flex items-center justify-center"><div className="w-5 h-5 rounded-full border-2 border-neutral-950 border-t-transparent animate-spin" /></div><div><p className="text-[10px] font-black uppercase tracking-[0.35em] text-neutral-400">Procesamiento automático</p><h3 className="text-2xl font-black mt-1">{title}</h3></div></div>
    <p className="text-sm text-neutral-500 mt-6">{subtitle}</p>
    <div className="mt-8"><div className="flex justify-between text-[10px] font-black uppercase tracking-widest text-neutral-400 mb-2"><span>{current} de {total}</span><span>{percent}%</span></div><div className="h-4 rounded-full bg-neutral-100 overflow-hidden"><div className="h-full rounded-full bg-neutral-950 transition-all duration-500" style={{ width: `${percent}%` }} /></div></div>
    <div className="mt-8">{children}</div>
  </section>
);

export const PasosGrid = ({ phase }: { phase: Phase }) => (
  <div className="grid grid-cols-1 md:grid-cols-3 gap-3 mb-8">
    {[
      ["01", "Catálogo maestro", phase !== "catalogo"],
      ["02", "Carpeta de tickets", ["calibrando", "procesando", "completo"].includes(phase)],
      ["03", "Análisis y parseo", phase === "completo" || phase === "procesando" || phase === "calibrando"],
    ].map(([number, label, done]) => (
      <div key={String(number)} className={`rounded-2xl border p-4 flex items-center gap-3 ${done ? "bg-neutral-950 text-neutral-50 border-neutral-950" : "bg-white border-neutral-100 text-neutral-400"}`}>
        <span className="text-xs font-black opacity-60">{number}</span>
        <span className="text-[10px] font-black uppercase tracking-widest">{label}</span>
      </div>
    ))}
  </div>
);