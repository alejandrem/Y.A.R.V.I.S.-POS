// Carrito de la venta que SOBREVIVE al cambio de pestaña.
//
// El problema: EmployeeDashboard desmontaba cada módulo al cambiar de tab
// y el carrito vivía en un useState de NuevaVenta: ir a Inventario a media
// venta borraba todo lo cobrado. El backend no se entera (el carrito solo
// existe en el frontend hasta cobrar), así que aquí no hay "mentira" sino
// pérdida directa de trabajo y dinero.
//
// La solución: este provider vive en EmployeeDashboard (no se desmonta al
// cambiar de tab) y es dueño del hook useCarrito. NuevaVenta lo consume.
// Al cerrar turno (logout) el dashboard se desmonta y el carrito se
// descarta solo, que es lo correcto.
//
// No hay DOM oculto ni doble montaje: es solo estado elevado.

import { createContext, useContext } from "react";
import type { ReactNode } from "react";
import { useCarrito } from "./hooks/useCarrito";

type CartApi = ReturnType<typeof useCarrito>;

const CartContext = createContext<CartApi | null>(null);

export const CartProvider = ({ children }: { children: ReactNode }) => {
  // Sin inputRef: el foco del buscador lo gestiona NuevaVenta envolviendo
  // addToCart (el provider no conoce el DOM del módulo).
  const api = useCarrito();
  return <CartContext.Provider value={api}>{children}</CartContext.Provider>;
};

export const useCart = (): CartApi => {
  const ctx = useContext(CartContext);
  if (!ctx) throw new Error("useCart fuera de CartProvider");
  return ctx;
};
