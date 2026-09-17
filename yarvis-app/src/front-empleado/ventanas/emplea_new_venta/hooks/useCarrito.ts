// ═══════════════════════════════════════════════════════════════════════════
// USE CARRITO — Hook con la lógica del carrito de la venta.
// Tarea única: poseer el estado `cart` y exponer las operaciones puras sobre
// él (agregar SIN tope de stock, sumar/restar cantidad, eliminar, limpiar) y
// el total calculado. No toca UI, búsqueda ni cobro.
//
// Regla de negocio: se permite SOBREVENTA. Si el inventario físico tiene más
// de lo capturado (reabasto aún no registrado) o el stock quedó en 0, igual
// se vende y el backend deja el stock en negativo para conciliar después.
// ═══════════════════════════════════════════════════════════════════════════

import { useState } from "react";
import type { InventoryItem } from "../../../../services/inventario";

export interface CartItem {
  id?: number;
  nombre: string;
  precio_venta: number;
  cantidad: number;
  stock: number;
  /** Descuento en PESOS de esta línea (monto, no %). 0 = sin descuento. */
  descuento: number;
}

interface UseCarritoArgs {
  inputRef?: { current: HTMLInputElement | null };
}

export function useCarrito({ inputRef }: UseCarritoArgs = {}) {
  const [cart, setCart] = useState<CartItem[]>([]);

  const addToCart = (product: InventoryItem) => {
    setCart((prev) => {
      const existing = prev.find((item) => item.id === product.id);
      if (existing) {
        // SIN tope de stock: se permite sobreventa (ver header).
        return prev.map((item) =>
          item.id === product.id ? { ...item, cantidad: item.cantidad + 1 } : item
        );
      }
      return [
        ...prev,
        {
          id: product.id,
          nombre: product.nombre,
          precio_venta: product.precio_venta,
          cantidad: 1,
          stock: product.stock,
          descuento: 0,
        },
      ];
    });
    inputRef?.current?.focus();
  };

  const updateQuantity = (id: number | undefined, delta: number) => {
    if (id === undefined) return;
    setCart((prev) =>
      prev
        .map((item) => {
          if (item.id === id) {
            const newQty = item.cantidad + delta;
            if (newQty <= 0) return null;
            // SIN tope de stock: se permite sobreventa (ver header).
            // Al bajar cantidad, el descuento no puede pasar del nuevo bruto.
            const bruto = item.precio_venta * newQty;
            return { ...item, cantidad: newQty, descuento: Math.min(item.descuento ?? 0, bruto) };
          }
          return item;
        })
        .filter(Boolean) as CartItem[]
    );
  };

  const removeFromCart = (id: number | undefined) => {
    if (id === undefined) return;
    setCart((prev) => prev.filter((item) => item.id !== id));
  };

  /** Descuento de la línea clampado a [0, bruto]: nunca deja neto negativo. */
  const updateDescuento = (id: number | undefined, monto: number) => {
    if (id === undefined) return;
    setCart((prev) =>
      prev.map((item) => {
        if (item.id !== id) return item;
        const bruto = item.precio_venta * item.cantidad;
        const d = Number.isFinite(monto) ? Math.min(Math.max(0, monto), bruto) : 0;
        return { ...item, descuento: Math.round(d * 100) / 100 };
      })
    );
  };

  const limpiarCarrito = () => setCart([]);

  /** Bruto sin descuentos (lo etiquetado). */
  const cartSubtotal = cart.reduce((acc, item) => acc + item.precio_venta * item.cantidad, 0);
  /** Lo que el cliente se ahorra en total. */
  const cartDescuento = cart.reduce((acc, item) => acc + (item.descuento ?? 0), 0);
  /** NETO a cobrar (bruto − descuentos). Lo que ya era `cartTotal`. */
  const cartTotal = cartSubtotal - cartDescuento;

  return { cart, addToCart, updateQuantity, updateDescuento, removeFromCart, limpiarCarrito, cartSubtotal, cartDescuento, cartTotal };
}
