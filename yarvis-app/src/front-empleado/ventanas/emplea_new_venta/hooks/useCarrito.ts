// ═══════════════════════════════════════════════════════════════════════════
// USE CARRITO — Hook con la lógica del carrito de la venta.
// Tarea única: poseer el estado `cart` y exponer las operaciones puras sobre
// él (agregar respetando stock, sumar/restar cantidad, eliminar, limpiar) y
// el total calculado. No toca UI, búsqueda ni cobro.
// ═══════════════════════════════════════════════════════════════════════════

import { useState } from "react";
import type { InventoryItem } from "../../../../services/inventario";

export interface CartItem {
  id?: number;
  nombre: string;
  precio_venta: number;
  cantidad: number;
  stock: number;
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
        if (existing.cantidad >= product.stock) return prev;
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
            if (newQty > item.stock) return item;
            return { ...item, cantidad: newQty };
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

  const limpiarCarrito = () => setCart([]);

  const cartTotal = cart.reduce((acc, item) => acc + item.precio_venta * item.cantidad, 0);

  return { cart, addToCart, updateQuantity, removeFromCart, limpiarCarrito, cartTotal };
}
