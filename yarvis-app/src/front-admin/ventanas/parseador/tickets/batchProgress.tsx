// Estado del lote de tickets que SOBREVIVE al cambio de pestaña.
//
// El problema: AdminDashboard desmonta cada módulo al cambiar de tab
// (ahorro de RAM, intencional). El progreso del parseo vivía en useState
// + un `listen("batch-progress")` dentro de Parseador: al irte a
// Inventario y volver, el listener ya no existía, el estado nacía de cero
// y la UI decía "no se está parseando nada" aunque Rust seguía trabajando.
//
// La solución: este provider vive en AdminDashboard (que NO se desmonta
// al cambiar de tab), registra UN solo listener global y guarda el último
// progreso + la fase. Parseador solo lo lee y lo alimenta al iniciar.
// Al volver a la pestaña, ves el progreso real o el resultado.
//
// Reglas:
// - Un solo listener en toda la app (se registra al montar el provider).
// - `beginBatch` al iniciar, `completeBatch`/`failBatch` al terminar.
// - Si ya hay un lote en "procesando", `beginBatch` lo rechaza: dos
//   importaciones concurrentes corromperían folios y stock.
import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState } from "react";
import type { ReactNode } from "react";
import { listen } from "@tauri-apps/api/event";
import type { BatchProgress, DeteccionMapeo } from "./compartido";

export type BatchPhase = "idle" | "procesando" | "completo";

export interface BatchMeta {
  carpeta: string;
  totalArchivos: number;
  deteccion: DeteccionMapeo | null;
}

interface BatchProgressContextValue {
  phase: BatchPhase;
  batch: BatchProgress | null;
  meta: BatchMeta | null;
  error: string;
  /** Inicia un lote. Devuelve false si ya hay otro en curso. */
  beginBatch: (meta: BatchMeta) => boolean;
  /** Marca el lote como terminado (idempotente). */
  completeBatch: () => void;
  /** Marca el lote como fallido con mensaje para la UI. */
  failBatch: (msg: string) => void;
  /** Descarta el error heredado (al navegar o reintentar). */
  clearBatchError: () => void;
  /** Vuelve a idle (botón "Procesar otra carpeta"). */
  resetBatch: () => void;
}

const BatchProgressContext = createContext<BatchProgressContextValue | null>(null);

export const BatchProgressProvider = ({ children }: { children: ReactNode }) => {
  const [phase, setPhase] = useState<BatchPhase>("idle");
  const [batch, setBatch] = useState<BatchProgress | null>(null);
  const [meta, setMeta] = useState<BatchMeta | null>(null);
  const [error, setError] = useState("");
  // Espejo para decidir dentro de callbacks/async sin clausuras viejas.
  const phaseRef = useRef<BatchPhase>("idle");
  const setPhaseSync = useCallback((p: BatchPhase) => {
    phaseRef.current = p;
    setPhase(p);
  }, []);

  // Un solo listener global: vive lo que vive el dashboard.
  useEffect(() => {
    let alive = true;
    let unlisten: (() => void) | null = null;
    listen<BatchProgress>("batch-progress", (event) => {
      if (!alive) return;
      setBatch(event.payload);
      if (event.payload.type === "complete") setPhaseSync("completo");
    }).then((fn) => {
      unlisten = fn;
      // StrictMode en dev monta/desmonta dos veces: si ya nos fuimos,
      // soltamos el listener de inmediato para no duplicar.
      if (!alive) fn();
    });
    return () => {
      alive = false;
      unlisten?.();
    };
  }, [setPhaseSync]);

  const beginBatch = useCallback((m: BatchMeta) => {
    if (phaseRef.current === "procesando") return false;
    setMeta(m);
    setBatch(null);
    setError("");
    setPhaseSync("procesando");
    return true;
  }, [setPhaseSync]);

  const completeBatch = useCallback(() => {
    if (phaseRef.current === "procesando") setPhaseSync("completo");
  }, [setPhaseSync]);

  const failBatch = useCallback((msg: string) => {
    setError(msg);
    setPhaseSync("idle");
  }, [setPhaseSync]);

  const clearBatchError = useCallback(() => setError(""), []);

  const resetBatch = useCallback(() => {
    setMeta(null);
    setBatch(null);
    setError("");
    setPhaseSync("idle");
  }, [setPhaseSync]);

  const value = useMemo(
    () => ({ phase, batch, meta, error, beginBatch, completeBatch, failBatch, clearBatchError, resetBatch }),
    [phase, batch, meta, error, beginBatch, completeBatch, failBatch, clearBatchError, resetBatch],
  );
  return <BatchProgressContext.Provider value={value}>{children}</BatchProgressContext.Provider>;
};

export const useBatchProgress = (): BatchProgressContextValue => {
  const ctx = useContext(BatchProgressContext);
  if (!ctx) throw new Error("useBatchProgress fuera de BatchProgressProvider");
  return ctx;
};
