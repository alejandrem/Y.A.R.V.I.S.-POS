import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface CartItem {
  id?: number;
  nombre: string;
  precio_venta: number;
  cantidad: number;
  stock: number;
}

interface TiendaInfo {
  nombre: string | null;
  ubicacion: string | null;
  cp: string | null;
}

interface ModalTicketProps {
  onClose: () => void;
  cart: CartItem[];
  cartTotal: number;
  ticketNumber: number;
  ventaId: number;
  montoEfectivo: number;
  montoTarjeta: number;
  montoTransferencia: number;
}

export default function ModalTicket({
  onClose,
  cart,
  cartTotal,
  ticketNumber,
  montoEfectivo,
  montoTarjeta,
  montoTransferencia,
}: ModalTicketProps) {
  const [tienda, setTienda] = useState<TiendaInfo | null>(null);

  useEffect(() => {
    invoke<TiendaInfo>("get_tienda_info")
      .then(setTienda)
      .catch(() => setTienda({ nombre: null, ubicacion: null, cp: null }));
  }, []);

  const cambio = (montoEfectivo + montoTarjeta + montoTransferencia) - cartTotal;

  return (
    <div className="fixed inset-0 bg-black/40 backdrop-blur-sm z-50 flex items-center justify-center" onClick={onClose}>
      <div
        className="bg-white rounded-[2rem] shadow-2xl w-full max-w-sm p-8 space-y-5 animate-in zoom-in-95 duration-200"
        onClick={(e) => e.stopPropagation()}
      >
        <header className="text-center">
          <h2 className="text-lg font-black text-neutral-900 uppercase">Ticket</h2>
          <div className="h-0.5 w-8 bg-neutral-900 mx-auto mt-2 rounded-full"></div>
          <p className="text-[10px] font-bold text-neutral-400 uppercase tracking-widest mt-2">
            #{ticketNumber}
          </p>
        </header>

        <div className="bg-neutral-50 rounded-xl p-4 space-y-3">
          {tienda?.nombre && (
            <p className="text-xs font-black text-neutral-900 text-center uppercase">{tienda.nombre}</p>
          )}
          {tienda?.ubicacion && (
            <p className="text-[10px] font-bold text-neutral-500 text-center">{tienda.ubicacion}</p>
          )}
          {tienda?.cp && (
            <p className="text-[10px] font-bold text-neutral-500 text-center">CP: {tienda.cp}</p>
          )}

          <div className="h-px bg-neutral-200"></div>

          <div className="space-y-1">
            <p className="text-[8px] font-black text-neutral-400 uppercase tracking-widest">Productos</p>
            {cart.map((item, idx) => (
              <div key={idx} className="flex justify-between text-[10px] font-bold text-neutral-700">
                <span>{item.cantidad}x {item.nombre}</span>
                <span>${(item.precio_venta * item.cantidad).toFixed(2)}</span>
              </div>
            ))}
          </div>

          <div className="h-px bg-neutral-200"></div>

          <div className="flex justify-between text-xs font-black text-neutral-900">
            <span>TOTAL</span>
            <span>${cartTotal.toFixed(2)}</span>
          </div>

          <div className="space-y-1">
            <p className="text-[8px] font-black text-neutral-400 uppercase tracking-widest">Método de pago</p>
            {montoEfectivo > 0 && (
              <div className="flex justify-between text-[10px] font-bold text-neutral-600">
                <span>Efectivo</span>
                <span>${montoEfectivo.toFixed(2)}</span>
              </div>
            )}
            {montoTarjeta > 0 && (
              <div className="flex justify-between text-[10px] font-bold text-neutral-600">
                <span>Tarjeta</span>
                <span>${montoTarjeta.toFixed(2)}</span>
              </div>
            )}
            {montoTransferencia > 0 && (
              <div className="flex justify-between text-[10px] font-bold text-neutral-600">
                <span>Transferencia</span>
                <span>${montoTransferencia.toFixed(2)}</span>
              </div>
            )}
          </div>

          {cambio > 0 && (
            <>
              <div className="h-px bg-neutral-200"></div>
              <div className="flex justify-between text-xs font-black text-emerald-600">
                <span>Cambio</span>
                <span>${cambio.toFixed(2)}</span>
              </div>
            </>
          )}
        </div>

        <div className="pt-2 space-y-2">
          <button
            onClick={onClose}
            className="w-full py-4 rounded-xl bg-neutral-900 text-white text-xs font-black uppercase tracking-[0.2em] hover:bg-neutral-800 transition-all shadow-xl shadow-neutral-200"
          >
            Cerrar
          </button>
        </div>
      </div>
    </div>
  );
}
