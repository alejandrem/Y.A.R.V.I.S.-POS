// ═══════════════════════════════════════════════════════════════════════════
// SECCIÓN PRODUCTOS — Lista de lo que se está vendiendo dentro de la
// ventana de cobro. Solo lectura: muestra cantidad, nombre, descuento
// de línea (si hay) y subtotal. Con scroll para carritos grandes.
// ═══════════════════════════════════════════════════════════════════════════

import type { CartItem } from "../hooks/useCarrito";

interface SeccionProductosProps {
  cart: CartItem[];
  cartTotal: number;
}

const SeccionProductos = ({ cart, cartTotal }: SeccionProductosProps) => (
  <div className="bg-neutral-50 rounded-3xl p-5 border border-neutral-100 flex flex-col min-h-0">
    <p className="text-[10px] font-black text-neutral-400 uppercase tracking-widest mb-3">
      Productos · {cart.length}
    </p>
    <div className="flex-1 min-h-0 max-h-72 overflow-y-auto custom-scrollbar space-y-2 pr-1">
      {cart.map((item, idx) => {
        const bruto = item.precio_venta * item.cantidad;
        const desc = Math.min(Math.max(0, item.descuento ?? 0), bruto);
        return (
          <div key={idx} className="flex items-center justify-between gap-3 bg-white rounded-2xl px-4 py-3 border border-neutral-100 shadow-sm">
            <span className="min-w-0 flex items-center gap-2.5">
              <span className="shrink-0 min-w-9 px-2 py-1.5 bg-neutral-950 text-white text-sm font-black rounded-xl text-center">
                {item.cantidad}x
              </span>
              <span className="min-w-0">
                <span className="block truncate text-[15px] font-black text-neutral-900 leading-tight">
                  {item.nombre}
                </span>
                {desc > 0 && (
                  <span className="block text-[10px] font-black text-amber-600 uppercase tracking-wide">
                    Desc −${desc.toFixed(2)}
                  </span>
                )}
              </span>
            </span>
            <span className="shrink-0 text-lg font-black text-neutral-900 tabular-nums">
              ${(bruto - desc).toFixed(2)}
            </span>
          </div>
        );
      })}
    </div>
    <div className="flex justify-between items-center mt-3 pt-3 border-t-2 border-neutral-900/10">
      <span className="text-sm font-black text-neutral-900 uppercase tracking-widest">Total</span>
      <span className="text-2xl font-black text-neutral-900 tabular-nums">${cartTotal.toFixed(2)}</span>
    </div>
  </div>
);

export default SeccionProductos;
