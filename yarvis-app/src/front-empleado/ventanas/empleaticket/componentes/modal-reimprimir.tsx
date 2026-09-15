// ═══════════════════════════════════════════════════════════════════════════
// REIMPRIMIR · MODAL — F2 global (issue #16), vive en el shell: funciona
// desde cualquier pestaña. Izquierda: preview completa del ticket.
// Derecha: campo de número (busca solo al escribir), mini-resumen y error
// en rojo si no existe (ahí se esconde Imprimir). Abajo: Cancelar y luego
// Imprimir en térmica (spooler, predeterminada por defecto). Escape cierra.
// Solo reimprime tickets PROPIOS (get_mi_ticket_detalle es operator-scoped).
// ═══════════════════════════════════════════════════════════════════════════

import { useState, useEffect, useCallback } from "react";
import { MorphIcon } from "morphicons/react";
import { obtenerMiTicketDetalle, type MiTicketDetalle } from "../../../../services/misventas";
import { obtenerTiendaInfo } from "../../../../services/turno";
import {
  listarImpresoras, imprimirTicketVenta, type ImpresoraInfo,
} from "../../../../services/impresora";
import { ICONO_DOCUMENTO, ICONO_EQUIS, ICONO_IMPRESORA } from "../../../../components/ui";
import { textoTicketReimpresion, folioTicket } from "./reimprimir-texto";

interface ModalReimprimirProps {
  onClose: () => void;
}

export default function ModalReimprimir({ onClose }: ModalReimprimirProps) {
  const [numero, setNumero] = useState("");
  const [detalle, setDetalle] = useState<MiTicketDetalle | null>(null);
  const [buscando, setBuscando] = useState(false);
  const [noExiste, setNoExiste] = useState(false);
  const [tienda, setTienda] = useState("MI TIENDA");
  const [impresoras, setImpresoras] = useState<ImpresoraInfo[]>([]);
  const [impresoraSel, setImpresoraSel] = useState("");
  const [imprimiendo, setImprimiendo] = useState(false);
  const [msgImpresion, setMsgImpresion] = useState("");
  const [error, setError] = useState("");

  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [onClose]);

  useEffect(() => {
    obtenerTiendaInfo()
      .then((t) => {
        if (t.nombre) setTienda(t.nombre);
      })
      .catch(() => {});
    listarImpresoras()
      .then((lista) => {
        setImpresoras(lista);
        setImpresoraSel(lista.find((i) => i.predeterminada)?.nombre ?? lista[0]?.nombre ?? "");
      })
      .catch(() => setImpresoras([]));
  }, []);

  const buscar = useCallback(async (id: number) => {
    setBuscando(true);
    setNoExiste(false);
    setError("");
    try {
      setDetalle(await obtenerMiTicketDetalle(id));
    } catch {
      setDetalle(null);
      setNoExiste(true);
    } finally {
      setBuscando(false);
    }
  }, []);

  useEffect(() => {
    const soloDigitos = numero.replace(/\D/g, "");
    if (soloDigitos !== numero) {
      setNumero(soloDigitos);
      return;
    }
    if (!soloDigitos) {
      setDetalle(null);
      setNoExiste(false);
      return;
    }
    const t = window.setTimeout(() => void buscar(Number(soloDigitos)), 300);
    return () => window.clearTimeout(t);
  }, [numero, buscar]);

  const handleImprimir = async () => {
    if (!detalle) return;
    if (!impresoraSel) {
      setError("Elige una impresora instalada.");
      return;
    }
    setImprimiendo(true);
    setError("");
    setMsgImpresion("");
    try {
      const msg = await imprimirTicketVenta(
        { Spooler: { nombre: impresoraSel } },
        {
          tienda,
          ubicacion: null,
          folio: folioTicket(detalle),
          fecha: detalle.fecha,
          lineas: detalle.items.map((it) => ({
            nombre: it.producto_nombre,
            cantidad: it.cantidad,
            precio_unitario: it.precio_unitario,
          })),
          total: detalle.total,
          pagos: [{ metodo: detalle.metodo_pago, monto: detalle.total }],
          qr: `YARVIS-${detalle.id}`,
        },
      );
      setMsgImpresion(msg);
    } catch (err) {
      setError(String(err));
    } finally {
      setImprimiendo(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4" onClick={onClose}>
      <div
        className="bg-white rounded-[2.5rem] shadow-2xl w-full max-w-3xl overflow-hidden animate-in zoom-in-95 fade-in duration-200 max-h-[90vh] overflow-y-auto custom-scrollbar"
        onClick={(e) => e.stopPropagation()}
      >
        {/* HEADER OSCURO */}
        <div className="bg-neutral-950 px-8 pt-7 pb-6 text-center relative overflow-hidden">
          <div className="absolute -top-10 -right-10 w-40 h-40 bg-white/[0.04] rounded-full blur-2xl" />
          <button
            onClick={onClose}
            className="absolute top-5 right-5 p-2 rounded-xl hover:bg-white/10 text-neutral-500 hover:text-white transition-all"
            title="Cerrar (Esc)"
          >
            <MorphIcon icon={ICONO_EQUIS} size={16} strokeWidth={2.5} spring="snappy" reducedMotion="user" />
          </button>
          <div className="w-12 h-12 mx-auto bg-white/10 rounded-2xl flex items-center justify-center mb-3">
            <MorphIcon icon={ICONO_DOCUMENTO} size={20} strokeWidth={2.2} spring="smooth" className="text-white" />
          </div>
          <h2 className="text-lg font-black text-white uppercase tracking-tight">Reimprimir ticket</h2>
          <p className="text-[9px] font-black text-neutral-500 uppercase tracking-[0.25em] mt-1">
            Escribe el número y sale en la térmica
          </p>
        </div>

        <div className="p-7 space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {/* IZQUIERDA: preview completa */}
            <pre className="bg-neutral-950 text-emerald-300 rounded-3xl p-5 text-[10px] leading-relaxed font-mono whitespace-pre-wrap break-words overflow-x-auto custom-scrollbar min-h-[280px]">
              {detalle ? textoTicketReimpresion(detalle, tienda) : "Escribe el número del ticket…"}
            </pre>

            {/* DERECHA: búsqueda + resumen */}
            <div className="space-y-3">
              <div>
                <label className="text-[10px] font-black uppercase tracking-widest text-neutral-400">
                  Número de ticket
                </label>
                <input
                  autoFocus
                  inputMode="numeric"
                  value={numero}
                  onChange={(e) => setNumero(e.target.value)}
                  placeholder="10109"
                  className="mt-2 w-full py-4 px-5 rounded-2xl bg-neutral-50 border-2 border-transparent focus:border-neutral-900 focus:bg-white text-base font-black text-neutral-900 focus:outline-none transition-all placeholder:text-neutral-300"
                />
              </div>

              {buscando && (
                <p className="text-[10px] font-black text-neutral-400 uppercase tracking-widest text-center">
                  Buscando…
                </p>
              )}

              {detalle && (
                <div className="bg-neutral-50 rounded-2xl p-4 border border-neutral-100 space-y-1.5">
                  <p className="text-xs font-black text-neutral-900">
                    {folioTicket(detalle)} · {detalle.fecha.slice(0, 16).replace("T", " ")}
                  </p>
                  <p className="text-[10px] font-bold text-neutral-500">
                    {detalle.items.length} {detalle.items.length === 1 ? "artículo" : "artículos"} · {detalle.metodo_pago}
                  </p>
                  <p className="text-sm font-black text-neutral-950">
                    Total: ${detalle.total.toFixed(2)}
                  </p>
                </div>
              )}

              {noExiste && (
                <p className="text-[11px] font-black text-red-500 text-center uppercase tracking-widest bg-red-50 rounded-2xl py-3 px-4">
                  Ese ticket no existe, revisa el número
                </p>
              )}

              <div className="space-y-2">
                <label className="text-[10px] font-black uppercase tracking-widest text-neutral-400">
                  Impresora térmica
                </label>
                <select
                  value={impresoraSel}
                  onChange={(e) => setImpresoraSel(e.target.value)}
                  className="w-full py-4 px-5 rounded-2xl bg-neutral-50 border-2 border-transparent focus:border-neutral-900 focus:bg-white text-sm font-black text-neutral-900 focus:outline-none transition-all"
                >
                  {impresoras.length === 0 && <option value="">Sin impresoras...</option>}
                  {impresoras.map((i) => (
                    <option key={i.nombre} value={i.nombre}>
                      {i.nombre}{i.predeterminada ? " (predeterminada)" : ""}
                    </option>
                  ))}
                </select>
              </div>
            </div>
          </div>

          {msgImpresion && (
            <p className="text-[11px] font-black text-emerald-600 text-center uppercase tracking-widest bg-emerald-50 rounded-2xl py-3 px-4">
              {msgImpresion}
            </p>
          )}

          {error && (
            <p className="text-[11px] font-black text-red-500 text-center uppercase tracking-widest bg-red-50 rounded-2xl py-3 px-4">
              {error}
            </p>
          )}

          <div className="pt-1 space-y-2.5">
            <button
              onClick={onClose}
              className="w-full py-3.5 rounded-3xl bg-neutral-100 text-neutral-500 text-sm font-black uppercase tracking-[0.2em] hover:bg-neutral-200 transition-all active:scale-[0.98]"
            >
              Cancelar
            </button>
            {detalle && (
              <button
                onClick={handleImprimir}
                disabled={imprimiendo || !impresoraSel}
                className="w-full py-5 rounded-3xl bg-neutral-950 text-white text-sm font-black uppercase tracking-[0.2em] hover:bg-neutral-800 transition-all shadow-xl shadow-neutral-300 active:scale-[0.98] disabled:opacity-30 disabled:cursor-not-allowed flex items-center justify-center gap-3"
              >
                <MorphIcon icon={ICONO_IMPRESORA} size={18} strokeWidth={2.5} spring="snappy" reducedMotion="user" />
                {imprimiendo ? "Imprimiendo..." : "Imprimir en térmica"}
              </button>
            )}
            {!detalle && (
              <div
                title="Escribe un número de ticket válido para activar"
                className="w-full py-5 rounded-3xl bg-neutral-100 text-neutral-300 text-sm font-black uppercase tracking-[0.2em] flex items-center justify-center gap-3 cursor-not-allowed"
              >
                <MorphIcon icon={ICONO_IMPRESORA} size={18} strokeWidth={2.5} spring="snappy" reducedMotion="user" />
                Imprimir en térmica
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
