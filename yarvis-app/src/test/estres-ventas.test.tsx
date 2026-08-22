// ═══════════════════════════════════════════════════════════════════════════
// TEST DE ESTRÉS — Módulo VENTAS / POS.
// Presión: 100 cobros secuenciales completos (modal → invoke → callback),
// carritos grandes de 200 items y montajes repetidos del modal.
// ═══════════════════════════════════════════════════════════════════════════

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { mockInvoke } from "./setup";
import ModalVenta from "../front-empleado/ventanas/emplea_new_venta/modalventa";

const item = (i: number) => ({
  id: i,
  nombre: `Producto ${i}`,
  precio_venta: 10,
  cantidad: 1,
  stock: 9999,
});

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockResolvedValue({ venta_id: 1, ticket_number: 1 });
});

describe("estres ventas · ráfaga de cobros", () => {
  it("procesa 100 cobros secuenciales sin perder ninguno", async () => {
    let exitosas = 0;
    for (let i = 0; i < 100; i++) {
      const onDone = vi.fn();
      const { unmount } = render(
        <ModalVenta onClose={() => {}} onVentaCompletada={onDone} cart={[item(1)]} cartTotal={10} />,
      );
      const label = screen.getByText(/Efectivo/i);
      fireEvent.change(label.parentElement!.querySelector("input")!, { target: { value: "10" } });
      fireEvent.click(screen.getByText(/Confirmar Venta/i));
      await waitFor(() => expect(onDone).toHaveBeenCalled());
      expect(mockInvoke).toHaveBeenCalledTimes(i + 1);
      exitosas++;
      unmount();
    }
    expect(exitosas).toBe(100);
  }, 60000);

  it("carga y cobra un carrito de 200 items", async () => {
    const cart = Array.from({ length: 200 }, (_, i) => item(i));
    const onDone = vi.fn();
    render(<ModalVenta onClose={() => {}} onVentaCompletada={onDone} cart={cart} cartTotal={2000} />);
    const label = screen.getByText(/Efectivo/i);
    fireEvent.change(label.parentElement!.querySelector("input")!, { target: { value: "2000" } });
    fireEvent.click(screen.getByText(/Confirmar Venta/i));
    await waitFor(() => expect(onDone).toHaveBeenCalled());
    const [, args] = mockInvoke.mock.calls[0];
    expect(args.venta.items).toHaveLength(200);
    expect(args.venta.total).toBe(2000);
  });
});
