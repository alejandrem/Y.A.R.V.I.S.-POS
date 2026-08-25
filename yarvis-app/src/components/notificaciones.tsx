// Sistema de notificaciones tipo toast, sin dependencias externas.
// Las funciones notificarError / notificarExito despachan CustomEvents en window
// y el componente <Toaster /> (montado una vez en App.tsx) los renderiza.

import { useEffect, useState } from "react";
import { MorphIcon } from "morphicons/react";

type TipoToast = "error" | "exito";

interface Toast {
  id: number;
  tipo: TipoToast;
  titulo: string;
  detalle?: string;
}

const EVENTO_TOAST = "yarvis:toast";
const DURACION_EXITO_MS = 5000;
const DURACION_ERROR_MS = 7000;

let contador = 0;

/** Extrae un texto legible de cualquier detalle de error. Nunca devuelve "[object Object]". */
function formatearDetalle(detalle: unknown): string | undefined {
  if (detalle === null || detalle === undefined) return undefined;
  if (typeof detalle === "string") {
    const limpio = detalle.trim();
    return limpio === "" ? undefined : limpio;
  }
  if (typeof detalle === "number" || typeof detalle === "boolean") return String(detalle);
  if (detalle instanceof Error) return detalle.message || detalle.name;
  if (typeof detalle === "object") {
    const obj = detalle as Record<string, unknown>;
    const candidata = obj.razon ?? obj.mensaje ?? obj.message ?? obj.error ?? obj.reason;
    if (typeof candidata === "string" && candidata.trim() !== "") return candidata.trim();
    try {
      const json = JSON.stringify(detalle);
      if (json && json !== "{}" && json !== "[]") return json;
    } catch {
      // objeto circular u otro error de serialización
    }
    const pares = Object.entries(obj).map(([k, v]) => `${k}: ${String(v)}`);
    return pares.length > 0 ? pares.join("; ") : String(detalle);
  }
  const texto = String(detalle);
  return texto.includes("[object Object]") ? undefined : texto;
}

function despachar(tipo: TipoToast, titulo: string, detalle?: unknown) {
  const toast: Toast = { id: ++contador, tipo, titulo, detalle: formatearDetalle(detalle) };
  window.dispatchEvent(new CustomEvent<Toast>(EVENTO_TOAST, { detail: toast }));
}

/** Muestra un toast rojo de error con mensaje contextual y detalle opcional del backend. */
export function notificarError(titulo: string, detalle?: unknown) {
  despachar("error", titulo, detalle);
}

/** Muestra un toast esmeralda de confirmación. */
export function notificarExito(titulo: string) {
  despachar("exito", titulo);
}

export function Toaster() {
  const [toasts, setToasts] = useState<Toast[]>([]);

  useEffect(() => {
    const manejar = (evento: Event) => {
      const toast = (evento as CustomEvent<Toast>).detail;
      if (!toast || typeof toast.id !== "number") return;
      setToasts((previos) => [...previos.slice(-4), toast]);
      window.setTimeout(() => {
        setToasts((previos) => previos.filter((t) => t.id !== toast.id));
      }, toast.tipo === "error" ? DURACION_ERROR_MS : DURACION_EXITO_MS);
    };
    window.addEventListener(EVENTO_TOAST, manejar);
    return () => window.removeEventListener(EVENTO_TOAST, manejar);
  }, []);

  if (toasts.length === 0) return null;

  return (
    <div className="fixed top-4 right-4 z-50 flex flex-col items-end gap-2 pointer-events-none">
      {toasts.map((toast) => {
        const esError = toast.tipo === "error";
        return (
          <div
            key={toast.id}
            className={`pointer-events-auto w-80 rounded-2xl border bg-white shadow-xl shadow-neutral-200/60 overflow-hidden animate-in fade-in slide-in-from-right-4 duration-300 ${
              esError ? "border-red-200" : "border-emerald-200"
            }`}
          >
            <div className="flex items-start gap-3 p-4">
              <div
                className={`shrink-0 w-7 h-7 rounded-full flex items-center justify-center text-white ${
                  esError ? "bg-red-500" : "bg-emerald-500"
                }`}
              >
                <MorphIcon icon={esError ? "M12 9v4m0 4h.01M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" : "M20 6 9 17l-5-5"} size={14} strokeWidth={2.5} />
              </div>
              <div className="flex-1 min-w-0">
                <p
                  className={`text-[10px] font-black uppercase tracking-widest leading-snug break-words ${
                    esError ? "text-red-600" : "text-emerald-600"
                  }`}
                >
                  {toast.titulo}
                </p>
                {toast.detalle && (
                  <p className="mt-1 text-[11px] font-medium text-neutral-500 leading-relaxed break-words">
                    {toast.detalle}
                  </p>
                )}
              </div>
              <button
                onClick={() => setToasts((previos) => previos.filter((t) => t.id !== toast.id))}
                aria-label="Cerrar notificación"
                className="shrink-0 text-neutral-300 hover:text-neutral-600 transition-colors"
              >
                <MorphIcon icon="M18 6 6 18M6 6l12 12" size={14} strokeWidth={2.5} />
              </button>
            </div>
            <div className={`h-1 w-full ${esError ? "bg-red-500" : "bg-emerald-500"}`} />
          </div>
        );
      })}
    </div>
  );
}
