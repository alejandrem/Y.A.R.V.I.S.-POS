// ═══════════════════════════════════════════════════════════════════════════
// MODAL VENTA — Cobro de la venta (empleado).
// Tarea única: capturar montos por método de pago y confirmar el cobro via
// completar_venta (que es transaccional en el backend). Escape cierra.
// El descuento de inventario ocurre SOLO aquí, al confirmar.
// ═══════════════════════════════════════════════════════════════════════════

import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { MorphIcon } from "morphicons/react";
import {
  ICONO_BILLETE, ICONO_TARJETA, ICONO_CELULAR,
  ICONO_CHECK, ICONO_EQUIS, ICONO_CAJA,
} from "../../../components/ui";

interface CartItem {
  id?: number;
  nombre: string;
  precio_venta: number;
  cantidad: number;
  stock: number;
}

interface ModalVentaProps {
  onClose: () => void;
  onVentaCompletada: (ventaId: number, ticketNumber: number, efectivo: number, tarjeta: number, transferencia: number) => void;
  cart: CartItem[];
  cartTotal: number;
}

export default function ModalVenta({ onClose, onVentaCompletada, cart, cartTotal }: ModalVentaProps) {
  const [efectivo, setEfectivo] = useState("");
  const [tarjeta, setTarjeta] = useState("");
  const [transferencia, setTransferencia] = useState("");
  const [procesando, setProcesando] = useState(false);
  const [error, setError] = useState("");

  const montoEfectivo = parseFloat(efectivo) || 0;
  const montoTarjeta = parseFloat(tarjeta) || 0;
  const montoTransferencia = parseFloat(transferencia) || 0;
  const totalPagado = montoEfectivo + montoTarjeta + montoTransferencia;
  const cambio = totalPagado - cartTotal;
  const esValido = totalPagado >= cartTotal && cartTotal > 0;

  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [onClose]);

  const handleConfirmar = async () => {
    if (!esValido) {
      setError("El monto total no coincide con el cobro");
      return;
    }

    setProcesando(true);
    setError("");

    try {
      const venta = {
        items: cart.map((item) => ({
          id: item.id ?? null,
          nombre: item.nombre,
          precio_venta: item.precio_venta,
          cantidad: item.cantidad,
        })),
        total: cartTotal,
        subtotal: cartTotal,
        descuento: 0,
        monto_efectivo: montoEfectivo,
        monto_tarjeta: montoTarjeta,
        monto_transferencia: montoTransferencia,
        cliente_id: null,
      };

      const result = await invoke<{ venta_id: number; ticket_number: number }>("completar_venta", { venta });
      onVentaCompletada(result.venta_id, result.ticket_number, montoEfectivo, montoTarjeta, montoTransferencia);
    } catch (err) {
      setError(String(err));
    } finally {
      setProcesando(false);
    }
  };

  // ── RENDER ──────────────────────────────────────────────────────────────

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4" onClick={onClose}>
      <div
        className="bg-white rounded-[2.5rem] shadow-2xl w-full max-w-md overflow-hidden animate-in zoom-in-95 fade-in duration-200"
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
            <MorphIcon icon={ICONO_CAJA} size={20} strokeWidth={2.2} spring="smooth" className="text-white" />
          </div>
          <h2 className="text-lg font-black text-white uppercase tracking-tight">Cobrar Venta</h2>
          <p className="text-[9px] font-black text-neutral-500 uppercase tracking-[0.25em] mt-1">
            Total a cobrar: ${cartTotal.toFixed(2)}
          </p>
        </div>

        <div className="p-7 space-y-4">
          {/* MONTOS POR MÉTODO */}
          <div className="space-y-3">
            {[
              { label: "Efectivo", icono: ICONO_BILLETE, valor: efectivo, set: setEfectivo },
              { label: "Tarjeta", icono: ICONO_TARJETA, valor: tarjeta, set: setTarjeta },
              { label: "Transferencia", icono: ICONO_CELULAR, valor: transferencia, set: setTransferencia },
            ].map((m) => (
              <div key={m.label} className="flex items-center gap-3 bg-neutral-50 border-2 border-transparent focus-within:border-neutral-900 focus-within:bg-white rounded-2xl px-4 py-3 transition-all duration-200">
                <MorphIcon icon={m.icono} size={17} strokeWidth={2.2} spring="smooth" className="text-neutral-400 shrink-0" />
                <span className="text-[10px] font-black uppercase tracking-widest text-neutral-400 w-28 shrink-0">{m.label}</span>
                <input
                  type="number"
                  min="0"
                  step="0.01"
                  value={m.valor}
                  onChange={(e) => { m.set(e.target.value); setError(""); }}
                  placeholder="$0.00"
                  className="w-full text-right text-sm font-black text-neutral-900 bg-transparent focus:outline-none placeholder:text-neutral-300 placeholder:font-bold"
                />
              </div>
            ))}
          </div>

          {/* RESUMEN */}
          <div className="bg-neutral-50 rounded-3xl p-5 space-y-2.5 border border-neutral-100">
            <div className="flex justify-between text-xs font-black text-neutral-400 uppercase tracking-wider">
              <span>Total a pagar</span>
              <span className="text-neutral-900">${cartTotal.toFixed(2)}</span>
            </div>
            <div className="flex justify-between text-xs font-black text-neutral-400 uppercase tracking-wider">
              <span>Recibido</span>
              <span className={totalPagado > 0 ? "text-neutral-900" : ""}>${totalPagado.toFixed(2)}</span>
            </div>
            <div className="h-px bg-neutral-200" />
            <div className="flex justify-between items-center">
              <span className="text-sm font-black text-neutral-900">Cambio</span>
              <span className={`text-xl font-black ${cambio >= 0 ? "text-emerald-600" : "text-red-500"}`}>
                ${cambio >= 0 ? cambio.toFixed(2) : "0.00"}
              </span>
            </div>
          </div>

          {error && (
            <p className="text-[11px] font-black text-red-500 text-center uppercase tracking-widest bg-red-50 rounded-2xl py-3 px-4">
              {error}
            </p>
          )}

          {/* BOTONES GORDITOS */}
          <div className="pt-1 space-y-2.5">
            <button
              onClick={handleConfirmar}
              disabled={!esValido || procesando}
              className="w-full py-5 rounded-3xl bg-neutral-950 text-white text-sm font-black uppercase tracking-[0.2em] hover:bg-neutral-800 transition-all shadow-xl shadow-neutral-300 active:scale-[0.98] disabled:opacity-30 disabled:cursor-not-allowed flex items-center justify-center gap-3"
            >
              <MorphIcon icon={ICONO_CHECK} size={18} strokeWidth={3} spring="snappy" reducedMotion="user" />
              {procesando ? "Procesando..." : `Cobrar $${cartTotal.toFixed(2)}`}
            </button>
            <button
              onClick={onClose}
              className="w-full py-3.5 text-[10px] font-black text-neutral-400 uppercase tracking-widest hover:text-neutral-950 transition-colors"
            >
              Cancelar
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
