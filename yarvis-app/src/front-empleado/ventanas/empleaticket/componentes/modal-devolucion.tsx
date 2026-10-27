// F9: búsqueda, selección y devolución atómica de un ticket completo.
import { useEffect, useState } from "react";
import { MorphIcon } from "morphicons/react";
import { buscarTicketsDevolucion, devolverTicketCompleto, obtenerTicketDevolucion, type DetalleTicketDevolucion, type TicketDevolucion } from "../../../../services/devoluciones";
import { abrirCajonPredeterminado } from "../../../../services/impresora";
import { notificarAviso, notificarExito } from "../../../../components/notificaciones";
import { reportarError } from "../../../../services/tauri";
import { ICONO_DOCUMENTO, ICONO_EQUIS } from "../../../../components/ui";

interface ModalDevolucionProps {
  onClose: () => void;
}

const dinero = (monto: number) => new Intl.NumberFormat("es-MX", {
  style: "currency",
  currency: "MXN",
}).format(monto);

const folio = (ticket: Pick<TicketDevolucion, "id" | "folio_ticket">) =>
  ticket.folio_ticket || `TICKET-${String(ticket.id).padStart(4, "0")}`;

export default function ModalDevolucion({ onClose }: ModalDevolucionProps) {
  const [termino, setTermino] = useState("");
  const [tickets, setTickets] = useState<TicketDevolucion[]>([]);
  const [seleccionado, setSeleccionado] = useState<TicketDevolucion | null>(null);
  const [detalle, setDetalle] = useState<DetalleTicketDevolucion | null>(null);
  const [buscando, setBuscando] = useState(false);
  const [cargandoDetalle, setCargandoDetalle] = useState(false);
  const [procesando, setProcesando] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    const manejarEscape = (evento: KeyboardEvent) => {
      if (evento.key === "Escape" && !procesando) onClose();
    };
    window.addEventListener("keydown", manejarEscape);
    return () => window.removeEventListener("keydown", manejarEscape);
  }, [onClose, procesando]);

  useEffect(() => {
    const q = termino.trim();
    let vigente = true;
    setBuscando(true);
    const timer = window.setTimeout(() => {
      buscarTicketsDevolucion(q)
        .then((resultado) => { if (vigente) setTickets(resultado); })
        .catch((e) => { if (vigente) { setTickets([]); setError(String(e)); } })
        .finally(() => { if (vigente) setBuscando(false); });
    }, q ? 250 : 0);
    return () => {
      vigente = false;
      window.clearTimeout(timer);
    };
  }, [termino]);

  const seleccionarTicket = async (ticket: TicketDevolucion) => {
    setSeleccionado(ticket);
    setDetalle(null);
    setError("");
    setCargandoDetalle(true);
    try {
      setDetalle(await obtenerTicketDevolucion(ticket.id));
    } catch (e) {
      setError(String(e));
    } finally {
      setCargandoDetalle(false);
    }
  };

  const confirmarDevolucion = async () => {
    if (!seleccionado || procesando) return;
    setProcesando(true);
    setError("");
    try {
      const resultado = await devolverTicketCompleto(seleccionado.id);
      onClose();
      notificarExito(`Devolución completada · ${dinero(resultado.monto)}`);
      if (!resultado.egreso_registrado) {
        notificarAviso("Egreso pendiente de corte", "La devolución quedó guardada. No había un corte abierto para asociar el egreso.");
      }
      try {
        await abrirCajonPredeterminado();
      } catch (e) {
        notificarAviso("Devolución completada; el cajón no abrió", "El dinero ya quedó devuelto. Abre el cajón manualmente.");
      }
    } catch (e) {
      const mensaje = "La devolución no se pudo realizar. Intenta de nuevo.";
      setError(mensaje);
      reportarError(mensaje, e);
    } finally {
      setProcesando(false);
    }
  };

  return (
    <div
      className="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4"
      onClick={() => { if (!procesando) onClose(); }}
    >
      <div
        className="bg-white rounded-[2.5rem] shadow-2xl w-full max-w-2xl overflow-hidden animate-in zoom-in-95 fade-in duration-200 max-h-[90vh] flex flex-col"
        onClick={(evento) => evento.stopPropagation()}
      >
        <div className="bg-neutral-950 px-8 pt-7 pb-6 text-center relative overflow-hidden shrink-0">
          <div className="absolute -top-10 -right-10 w-40 h-40 bg-white/[0.04] rounded-full blur-2xl" />
          <button
            onClick={onClose}
            disabled={procesando}
            className="absolute top-5 right-5 p-2 rounded-xl hover:bg-white/10 text-neutral-500 hover:text-white transition-all disabled:opacity-30"
            title="Cerrar (Esc)"
          >
            <MorphIcon icon={ICONO_EQUIS} size={16} strokeWidth={2.5} spring="snappy" reducedMotion="user" />
          </button>
          <div className="w-12 h-12 mx-auto bg-white/10 rounded-2xl flex items-center justify-center mb-3">
            <MorphIcon icon={ICONO_DOCUMENTO} size={20} strokeWidth={2.2} spring="smooth" className="text-white" />
          </div>
          <h2 className="text-lg font-black text-white uppercase tracking-tight">Devoluciones</h2>
          <p className="text-[9px] font-black text-neutral-500 uppercase tracking-[0.25em] mt-1">Devolución del ticket completo · F9</p>
        </div>

        <div className="p-6 sm:p-7 space-y-4 overflow-y-auto custom-scrollbar">
          <div>
            <label htmlFor="buscar-ticket-devolucion" className="text-[10px] font-black uppercase tracking-widest text-neutral-400">
              Buscar por código o fecha
            </label>
            <input
              id="buscar-ticket-devolucion"
              autoFocus
              value={termino}
              onChange={(evento) => {
                setTermino(evento.target.value);
                setSeleccionado(null);
                setDetalle(null);
                setError("");
              }}
              placeholder="Código del ticket o fecha, ej. 05/25/26"
              className="mt-2 w-full py-4 px-5 rounded-2xl bg-neutral-50 border-2 border-transparent focus:border-neutral-900 focus:bg-white text-sm font-bold text-neutral-900 focus:outline-none transition-all placeholder:text-neutral-300"
            />
          </div>

          <section className="rounded-3xl border border-neutral-200 overflow-hidden">
            <div className="px-5 py-3 bg-neutral-50 border-b border-neutral-100 flex items-center justify-between">
              <h3 className="text-[10px] font-black text-neutral-600 uppercase tracking-widest">Historial de tickets</h3>
              <span className="text-[9px] font-bold text-neutral-400">{tickets.length} encontrados</span>
            </div>
            <div className="max-h-64 overflow-y-auto custom-scrollbar p-2 space-y-1">
              {buscando ? (
                <p className="py-8 text-center text-[10px] font-black text-neutral-400 uppercase tracking-widest">Buscando tickets</p>
              ) : tickets.length ? tickets.map((ticket) => {
                const activo = seleccionado?.id === ticket.id;
                return (
                  <button
                    key={ticket.id}
                    onClick={() => void seleccionarTicket(ticket)}
                    className={`w-full text-left px-4 py-3 rounded-2xl border transition-colors ${activo ? "bg-neutral-950 border-neutral-950 text-white" : "bg-white border-transparent hover:bg-neutral-50 text-neutral-900"}`}
                  >
                    <div className="flex items-center justify-between gap-3">
                      <span className="text-[10px] font-black uppercase tracking-wider">{folio(ticket)}</span>
                      <span className={`text-[10px] font-black ${activo ? "text-white" : "text-neutral-900"}`}>{dinero(ticket.total)}</span>
                    </div>
                    <p className={`text-[9px] font-bold mt-1 ${activo ? "text-neutral-300" : "text-neutral-400"}`}>
                      {ticket.fecha} · {ticket.metodo_pago}
                    </p>
                  </button>
                );
              }) : (
                <p className="py-8 px-4 text-center text-[10px] font-bold text-neutral-400">
                  {termino.trim() ? "No se encontraron tickets disponibles para devolución" : "No hay tickets recientes disponibles para devolución"}
                </p>
              )}
            </div>
          </section>

          {seleccionado && (
            <section className="rounded-3xl bg-neutral-50 border border-neutral-100 p-5 space-y-3">
              <div className="flex items-start justify-between gap-4">
                <div>
                  <p className="text-[10px] font-black text-neutral-900 uppercase tracking-widest">{folio(seleccionado)}</p>
                  <p className="text-[9px] font-bold text-neutral-400 mt-1">{seleccionado.fecha} · {seleccionado.metodo_pago}</p>
                </div>
                <div className="text-right">
                  <p className="text-[8px] font-black text-neutral-400 uppercase tracking-widest">Total a devolver</p>
                  <p className="text-base font-black text-neutral-950 mt-1">{dinero(seleccionado.total)}</p>
                </div>
              </div>
              <div className="border-t border-neutral-200 pt-3">
                <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest mb-2">Se devolverán todos los productos del ticket</p>
                {cargandoDetalle ? (
                  <p className="text-[10px] font-bold text-neutral-400">Cargando detalle del ticket</p>
                ) : detalle?.items.length ? (
                  <div className="space-y-2 max-h-32 overflow-y-auto custom-scrollbar">
                    {detalle.items.map((item, indice) => (
                      <div key={`${item.producto_nombre}-${indice}`} className="flex items-center justify-between gap-3 text-[10px]">
                        <span className="font-bold text-neutral-700">{item.cantidad} × {item.producto_nombre}</span>
                        <span className="font-black text-neutral-900">{dinero(item.subtotal)}</span>
                      </div>
                    ))}
                  </div>
                ) : (
                  <p className="text-[10px] font-bold text-neutral-400">Este ticket no tiene detalle de productos.</p>
                )}
              </div>
            </section>
          )}

          {error && <p role="alert" className="text-[10px] font-black text-red-600 text-center bg-red-50 border border-red-100 rounded-2xl py-3 px-4">{error}</p>}

          <div className="grid grid-cols-2 gap-3 pt-1">
            <button
              onClick={onClose}
              disabled={procesando}
              className="py-4 rounded-2xl bg-neutral-100 text-neutral-500 text-[10px] font-black uppercase tracking-[0.15em] hover:bg-neutral-200 transition-all disabled:opacity-40"
            >
              Cancelar · Esc
            </button>
            <button
              onClick={() => void confirmarDevolucion()}
              disabled={!seleccionado || procesando || cargandoDetalle}
              className={`py-4 rounded-2xl text-[10px] font-black uppercase tracking-[0.15em] transition-all active:scale-[0.98] disabled:cursor-not-allowed ${seleccionado ? "bg-neutral-950 text-white hover:bg-neutral-800" : "bg-neutral-100 text-neutral-400"}`}
            >
              {procesando ? "Procesando" : "Devolver ticket"}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
