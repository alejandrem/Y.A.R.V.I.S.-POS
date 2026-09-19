// ═══════════════════════════════════════════════════════════════════════════
// SECCIÓN PAGO — Captura de montos por método + resumen.
// Tarea única: mostrar lo recibido, el cambio y el error. Los botones de
// cobro viven en el orquestador (principal + sin ticket); Enter aquí
// dispara el cobro principal. Sin lógica de ticket ni impresión.
// ═══════════════════════════════════════════════════════════════════════════

import { MorphIcon } from "morphicons/react";
import {
  ICONO_BILLETE, ICONO_TARJETA, ICONO_CELULAR,
} from "../../../../components/ui";
import type { CartItem } from "../hooks/useCarrito";

interface SeccionPagoProps {
  cart: CartItem[];
  cartTotal: number;
  efectivo: string;
  tarjeta: string;
  transferencia: string;
  onEfectivo: (v: string) => void;
  onTarjeta: (v: string) => void;
  onTransferencia: (v: string) => void;
  procesando: boolean;
  error: string;
  onConfirmar: () => void;
}

const SeccionPago = ({
  cart, cartTotal,
  efectivo, tarjeta, transferencia,
  onEfectivo, onTarjeta, onTransferencia,
  procesando, error, onConfirmar,
}: SeccionPagoProps) => {
  const montoEfectivo = parseFloat(efectivo) || 0;
  const montoTarjeta = parseFloat(tarjeta) || 0;
  const montoTransferencia = parseFloat(transferencia) || 0;
  const totalPagado = montoEfectivo + montoTarjeta + montoTransferencia;
  const cambio = totalPagado - cartTotal;
  const subtotalBruto = cart.reduce((acc, it) => acc + it.precio_venta * it.cantidad, 0);
  const descuentoTotal = cart.reduce((acc, it) => {
    const bruto = it.precio_venta * it.cantidad;
    const d = it.descuento ?? 0;
    return acc + (Number.isFinite(d) ? Math.min(Math.max(0, d), bruto) : 0);
  }, 0);

  return (
    <div className="space-y-4">
      <div className="space-y-3">
        {[
          { label: "Efectivo", icono: ICONO_BILLETE, valor: efectivo, set: onEfectivo, auto: true },
          { label: "Tarjeta", icono: ICONO_TARJETA, valor: tarjeta, set: onTarjeta, auto: false },
          { label: "Transferencia", icono: ICONO_CELULAR, valor: transferencia, set: onTransferencia, auto: false },
        ].map((m) => (
          <div key={m.label} className="flex items-center gap-3 bg-neutral-50 border-2 border-transparent focus-within:border-neutral-900 focus-within:bg-white rounded-2xl px-4 py-3.5 transition-all duration-200">
            <MorphIcon icon={m.icono} size={19} strokeWidth={2.2} spring="smooth" className="text-neutral-400 shrink-0" />
            <span className="text-[11px] font-black uppercase tracking-widest text-neutral-500 w-28 shrink-0">{m.label}</span>
            <input
              type="number"
              min="0"
              step="0.01"
              value={m.valor}
              autoFocus={m.auto}
              disabled={procesando}
              onChange={(e) => m.set(e.target.value)}
              onKeyDown={(e) => { if (e.key === "Enter") { e.preventDefault(); onConfirmar(); } }}
              placeholder="$0.00"
              className="w-full text-right text-xl font-black text-neutral-900 tabular-nums bg-transparent focus:outline-none placeholder:text-neutral-300 placeholder:font-bold disabled:opacity-50"
            />
          </div>
        ))}
      </div>

      <div className="bg-neutral-950 rounded-3xl p-5 space-y-2.5">
        {descuentoTotal > 0 && (
          <>
            <div className="flex justify-between text-sm font-black text-neutral-400 uppercase tracking-wider">
              <span>Subtotal</span>
              <span className="text-white tabular-nums">${subtotalBruto.toFixed(2)}</span>
            </div>
            <div className="flex justify-between text-sm font-black uppercase tracking-wider">
              <span className="text-amber-400">Descuento</span>
              <span className="text-amber-400 tabular-nums">−${descuentoTotal.toFixed(2)}</span>
            </div>
          </>
        )}
        <div className="flex justify-between text-sm font-black text-neutral-400 uppercase tracking-wider">
          <span>Total a pagar</span>
          <span className="text-white tabular-nums">${cartTotal.toFixed(2)}</span>
        </div>
        <div className="flex justify-between text-sm font-black text-neutral-400 uppercase tracking-wider">
          <span>Recibido</span>
          <span className={totalPagado > 0 ? "text-white tabular-nums" : "tabular-nums"}>${totalPagado.toFixed(2)}</span>
        </div>
        <div className="h-px bg-white/15" />
        <div className="flex justify-between items-center bg-white/5 rounded-2xl px-4 py-3">
          <span className="text-sm font-black text-white uppercase tracking-widest">Cambio</span>
          <span className={`text-3xl font-black tabular-nums ${cambio >= 0 ? "text-emerald-400" : "text-red-400"}`}>
            ${cambio >= 0 ? cambio.toFixed(2) : "0.00"}
          </span>
        </div>
      </div>

      {error && (
        <p className="text-[11px] font-black text-red-500 text-center uppercase tracking-widest bg-red-50 rounded-2xl py-3 px-4">
          {error}
        </p>
      )}
    </div>
  );
};

export default SeccionPago;
