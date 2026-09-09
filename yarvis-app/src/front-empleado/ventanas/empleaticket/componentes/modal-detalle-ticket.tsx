// ═══════════════════════════════════════════════════════════════════════════
// MODAL DETALLE TICKET — Detalle de UN ticket propio: cabecera, renglones
// y totales. Cierra con botón, click fuera o Escape. Si el ticket no es
// del operador, el backend responde "Ticket no encontrado".
// ═══════════════════════════════════════════════════════════════════════════

import { useEffect, useState } from "react";
import { moneda, entero } from "../../../../front-admin/ventanas/adminventas/graficas/controles";
import {
  obtenerMiTicketDetalle, type MiTicket, type MiTicketDetalle,
} from "../../../../services/misventas";
import { reportarError } from "../../../../services/tauri";

interface ModalDetalleTicketProps {
  ticket: MiTicket;
  onCerrar: () => void;
}

const ModalDetalleTicket = ({ ticket, onCerrar }: ModalDetalleTicketProps) => {
  const [detalle, setDetalle] = useState<MiTicketDetalle | null>(null);

  useEffect(() => {
    let viva = true;
    obtenerMiTicketDetalle(ticket.id)
      .then((d) => {
        if (viva) setDetalle(d);
      })
      .catch((e) => reportarError("No se pudo abrir el detalle del ticket", e));
    return () => {
      viva = false;
    };
  }, [ticket.id]);

  useEffect(() => {
    const tecla = (e: KeyboardEvent) => {
      if (e.key === "Escape") onCerrar();
    };
    document.addEventListener("keydown", tecla);
    return () => document.removeEventListener("keydown", tecla);
  }, [onCerrar]);

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4 backdrop-blur-sm"
      onClick={onCerrar}
    >
      <div
        className="w-full max-w-lg bg-white rounded-[2rem] border border-neutral-200 shadow-2xl overflow-hidden animate-in fade-in zoom-in-95 duration-200"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between px-6 py-5 border-b border-neutral-100">
          <div>
            <h3 className="text-lg font-black text-neutral-900 uppercase tracking-tight">
              Ticket {ticket.folio_ticket || `#${ticket.id}`}
            </h3>
            <p className="text-[10px] font-bold text-neutral-400 uppercase tracking-widest mt-1">
              {ticket.fecha} · {ticket.metodo_pago}
            </p>
          </div>
          <button
            onClick={onCerrar}
            className="w-9 h-9 flex items-center justify-center rounded-xl bg-neutral-100 text-neutral-500 hover:bg-neutral-900 hover:text-white transition-all text-xl"
            aria-label="Cerrar detalle"
          >
            ×
          </button>
        </div>

        <div className="px-6 py-5 max-h-[55vh] overflow-y-auto custom-scrollbar">
          {detalle === null ? (
            <div className="py-10 flex justify-center">
              <div className="w-8 h-8 rounded-full border-2 border-neutral-900 border-t-transparent animate-spin" />
            </div>
          ) : detalle.items.length === 0 ? (
            <p className="text-sm text-neutral-400 font-bold text-center py-8">
              Este ticket no tiene renglones registrados
            </p>
          ) : (
            <div className="space-y-2">
              {detalle.items.map((it, i) => (
                <div
                  key={i}
                  className="flex items-center justify-between gap-3 p-3 bg-neutral-50 rounded-xl border border-neutral-100"
                >
                  <div className="min-w-0">
                    <p className="text-[11px] font-black text-neutral-900 uppercase truncate">
                      {it.producto_nombre}
                    </p>
                    <p className="text-[10px] font-bold text-neutral-400">
                      {entero(it.cantidad)} × {moneda(it.precio_unitario)}
                    </p>
                  </div>
                  <p className="text-xs font-black text-neutral-900 shrink-0">{moneda(it.subtotal)}</p>
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="px-6 py-5 border-t border-neutral-100 bg-neutral-50/60">
          {detalle !== null && (
            <div className="space-y-1.5 text-[11px] font-bold text-neutral-500">
              <div className="flex justify-between">
                <span className="uppercase tracking-widest">Subtotal</span>
                <span className="text-neutral-900">{moneda(detalle.subtotal)}</span>
              </div>
              {detalle.descuento > 0 && (
                <div className="flex justify-between">
                  <span className="uppercase tracking-widest">Descuento</span>
                  <span className="text-red-500">−{moneda(detalle.descuento)}</span>
                </div>
              )}
              <div className="flex justify-between pt-2 border-t border-neutral-200">
                <span className="uppercase tracking-widest text-neutral-900 font-black">Total</span>
                <span className="text-base font-black text-neutral-900">{moneda(detalle.total)}</span>
              </div>
            </div>
          )}
          <button
            onClick={onCerrar}
            className="mt-4 w-full py-3 rounded-xl bg-neutral-900 text-white text-[10px] font-black uppercase tracking-widest hover:bg-neutral-700 transition-all"
          >
            Cerrar
          </button>
        </div>
      </div>
    </div>
  );
};

export default ModalDetalleTicket;
