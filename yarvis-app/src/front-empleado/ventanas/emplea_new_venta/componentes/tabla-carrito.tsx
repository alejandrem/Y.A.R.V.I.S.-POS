// ═══════════════════════════════════════════════════════════════════════════
// TABLA CARRITO — Card "Detalle de Venta" del punto de venta.
// Tarea única: renderizar el header con contador de artículos y botón LIMPIAR,
// la tabla de productos (controles +/-/eliminar) y el empty state
// "Esperando productos...". 100% presentacional: recibe cart y callbacks.
// ═══════════════════════════════════════════════════════════════════════════

import { MorphIcon } from "morphicons/react";
import type { CartItem } from "../hooks/useCarrito";
import {
  ICONO_CARRITO, ICONO_MAS, ICONO_RESTA, ICONO_EQUIS,
} from "../../../../components/ui";

interface TablaCarritoProps {
  cart: CartItem[];
  onUpdateCantidad: (id: number | undefined, delta: number) => void;
  onEliminar: (id: number | undefined) => void;
  onLimpiar: () => void;
  children?: React.ReactNode;
}

export default function TablaCarrito({
  cart,
  onUpdateCantidad,
  onEliminar,
  onLimpiar,
  children,
}: TablaCarritoProps) {
  return (
    <div className="flex-1 bg-white rounded-[2.5rem] border border-neutral-200 shadow-sm overflow-hidden flex flex-col">
      <div className="px-8 py-5 flex justify-between items-center">
        <h3 className="text-sm font-black text-neutral-950 uppercase tracking-tight flex items-center gap-3">
          <div className="w-10 h-10 bg-neutral-950 rounded-2xl flex items-center justify-center shadow-md">
            <MorphIcon icon={ICONO_CARRITO} size={16} strokeWidth={2.2} spring="smooth" className="text-white" />
          </div>
          Detalle de Venta
        </h3>
        <div className="flex items-center gap-4">
          <span className="px-3 py-1.5 bg-neutral-950 text-white text-[9px] font-black rounded-lg uppercase tracking-widest">
            {cart.reduce((acc, item) => acc + item.cantidad, 0)} ARTÍCULOS
          </span>
          {cart.length > 0 && (
            <button
              onClick={onLimpiar}
              className="text-[9px] font-black text-red-400 hover:text-red-600 uppercase tracking-widest transition-colors"
            >
              LIMPIAR
            </button>
          )}
        </div>
      </div>

      <div className="flex-1 overflow-y-auto px-8 pb-4 custom-scrollbar">
        {cart.length === 0 ? (
          <div className="py-20 text-center">
            <div className="w-16 h-16 mx-auto bg-neutral-100 rounded-3xl flex items-center justify-center mb-5">
              <MorphIcon icon={ICONO_CARRITO} size={26} strokeWidth={1.8} spring="smooth" className="text-neutral-300" />
            </div>
            <p className="text-[11px] font-black uppercase tracking-[0.25em] text-neutral-300">Esperando productos...</p>
            <p className="text-[10px] font-bold text-neutral-200 mt-2">Escanea un código o busca arriba para empezar</p>
          </div>
        ) : (
          <table className="w-full text-left border-collapse">
            <thead>
              <tr className="text-[9px] font-black text-neutral-400 uppercase tracking-widest border-b-2 border-neutral-100">
                <th className="pb-3 px-2">Cantidad</th>
                <th className="pb-3 px-2">Producto</th>
                <th className="pb-3 px-2">P. Unitario</th>
                <th className="pb-3 px-2 text-right">Subtotal</th>
                <th className="pb-3 px-2 w-12"></th>
              </tr>
            </thead>
            <tbody className="divide-y divide-neutral-100">
              {cart.map((item) => (
                <tr key={item.id} className="group hover:bg-neutral-50/60 transition-colors">
                  <td className="py-3.5 px-2">
                    <div className="flex items-center gap-2 bg-neutral-50 rounded-2xl p-1 w-fit border border-neutral-100">
                      <button
                        onClick={() => onUpdateCantidad(item.id, -1)}
                        className="w-8 h-8 rounded-xl bg-white hover:bg-neutral-950 hover:text-white flex items-center justify-center shadow-sm transition-all active:scale-90"
                        title="Quitar uno"
                      >
                        <MorphIcon icon={ICONO_RESTA} size={13} strokeWidth={3} spring="snappy" reducedMotion="user" />
                      </button>
                      <span className="w-8 text-center text-sm font-black text-neutral-900">{item.cantidad}</span>
                      <button
                        onClick={() => onUpdateCantidad(item.id, 1)}
                        disabled={item.cantidad >= item.stock}
                        className="w-8 h-8 rounded-xl bg-white hover:bg-neutral-950 hover:text-white flex items-center justify-center shadow-sm transition-all active:scale-90 disabled:opacity-30 disabled:cursor-not-allowed disabled:hover:bg-white disabled:hover:text-black"
                        title="Agregar uno"
                      >
                        <MorphIcon icon={ICONO_MAS} size={13} strokeWidth={3} spring="snappy" reducedMotion="user" />
                      </button>
                    </div>
                  </td>
                  <td className="py-3.5 px-2 font-black text-neutral-800 text-xs uppercase">{item.nombre}</td>
                  <td className="py-3.5 px-2 font-bold text-neutral-400 text-xs">${item.precio_venta.toFixed(2)}</td>
                  <td className="py-3.5 px-2 text-right font-black text-neutral-950 text-base">
                    ${(item.precio_venta * item.cantidad).toFixed(2)}
                  </td>
                  <td className="py-3.5 px-2 text-right">
                    <button
                      onClick={() => onEliminar(item.id)}
                      className="p-2 rounded-xl bg-neutral-100 text-neutral-400 hover:bg-red-50 hover:text-red-500 transition-all opacity-0 group-hover:opacity-100 active:scale-90"
                      title="Eliminar del carrito"
                    >
                      <MorphIcon icon={ICONO_EQUIS} size={13} strokeWidth={2.5} spring="snappy" reducedMotion="user" />
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      {/* ── FOOTER OSCURO: IA + TOTAL (inyectado por el orquestador) ── */}
      {children}
    </div>
  );
}
