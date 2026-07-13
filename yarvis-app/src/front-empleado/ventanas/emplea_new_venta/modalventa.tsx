import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

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
  cajero: string;
}

export default function ModalVenta({ onClose, onVentaCompletada, cart, cartTotal, cajero }: ModalVentaProps) {
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
        cajero,
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

  return (
    <div className="fixed inset-0 bg-black/40 backdrop-blur-sm z-50 flex items-center justify-center" onClick={onClose}>
      <div
        className="bg-white rounded-[2rem] shadow-2xl w-full max-w-md p-8 space-y-5 animate-in zoom-in-95 duration-200"
        onClick={(e) => e.stopPropagation()}
      >
        <header className="text-center">
          <h2 className="text-lg font-black text-neutral-900 uppercase">Cobro</h2>
          <div className="h-0.5 w-8 bg-neutral-900 mx-auto mt-2 rounded-full"></div>
          <p className="text-[10px] font-bold text-neutral-400 uppercase tracking-widest mt-2">
            Total a cobrar: ${cartTotal.toFixed(2)}
          </p>
        </header>

        <div className="space-y-3">
          <div className="space-y-1">
            <label className="text-[10px] font-bold text-neutral-500 uppercase tracking-wider ml-1">
              Efectivo
            </label>
            <input
              type="number"
              min="0"
              step="0.01"
              value={efectivo}
              onChange={(e) => { setEfectivo(e.target.value); setError(""); }}
              placeholder="$0.00"
              className="w-full px-3 py-2 rounded-xl bg-neutral-50 border border-neutral-200 text-sm focus:outline-none focus:border-neutral-900"
            />
          </div>

          <div className="space-y-1">
            <label className="text-[10px] font-bold text-neutral-500 uppercase tracking-wider ml-1">
              Tarjeta
            </label>
            <input
              type="number"
              min="0"
              step="0.01"
              value={tarjeta}
              onChange={(e) => { setTarjeta(e.target.value); setError(""); }}
              placeholder="$0.00"
              className="w-full px-3 py-2 rounded-xl bg-neutral-50 border border-neutral-200 text-sm focus:outline-none focus:border-neutral-900"
            />
          </div>

          <div className="space-y-1">
            <label className="text-[10px] font-bold text-neutral-500 uppercase tracking-wider ml-1">
              Transferencia
            </label>
            <input
              type="number"
              min="0"
              step="0.01"
              value={transferencia}
              onChange={(e) => { setTransferencia(e.target.value); setError(""); }}
              placeholder="$0.00"
              className="w-full px-3 py-2 rounded-xl bg-neutral-50 border border-neutral-200 text-sm focus:outline-none focus:border-neutral-900"
            />
          </div>
        </div>

        <div className="bg-neutral-50 rounded-xl p-4 space-y-2">
          <div className="flex justify-between text-xs font-bold text-neutral-500">
            <span>Total a pagar</span>
            <span className="text-neutral-900">${cartTotal.toFixed(2)}</span>
          </div>
          <div className="flex justify-between text-xs font-bold text-neutral-500">
            <span>Recibido</span>
            <span className="text-neutral-900">${totalPagado.toFixed(2)}</span>
          </div>
          <div className="h-px bg-neutral-200"></div>
          <div className="flex justify-between text-sm font-black">
            <span className="text-neutral-900">Cambio</span>
            <span className={cambio >= 0 ? "text-emerald-600" : "text-red-500"}>
              ${cambio >= 0 ? cambio.toFixed(2) : "0.00"}
            </span>
          </div>
        </div>

        {error && (
          <p className="text-[10px] font-bold text-red-500 text-center uppercase tracking-wider">{error}</p>
        )}

        <div className="pt-2 space-y-2">
          <button
            onClick={handleConfirmar}
            disabled={!esValido || procesando}
            className="w-full py-4 rounded-xl bg-neutral-900 text-white text-xs font-black uppercase tracking-[0.2em] hover:bg-neutral-800 transition-all shadow-xl shadow-neutral-200 disabled:opacity-30 disabled:cursor-not-allowed"
          >
            {procesando ? "Procesando..." : "Confirmar Venta"}
          </button>
          <button
            onClick={onClose}
            className="w-full py-3 text-[10px] font-bold text-neutral-400 uppercase tracking-widest"
          >
            Cancelar
          </button>
        </div>
      </div>
    </div>
  );
}
