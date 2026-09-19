// ═══════════════════════════════════════════════════════════════════════════
// TEST DE ESTRÉS — Módulo VENTAS / POS.
// Presión: 100 cobros secuenciales completos (ventana → completar →
// auto-print → terminar), carritos grandes de 200 items y montajes
// repetidos de la ventana única de cobro.
// ═══════════════════════════════════════════════════════════════════════════

import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { mockInvoke } from "./setup";
import VentanaCobro from "../front-empleado/ventanas/emplea_new_venta/ventana-cobro/ventana-cobro";

const item = (i: number) => ({
  id: i,
  nombre: `Producto ${i}`,
  precio_venta: 10,
  cantidad: 1,
  stock: 9999,
  descuento: 0,
});

const comandos = (cmd: string) => mockInvoke.mock.calls.filter(([c]) => c === cmd);

/** Espera al destino de impresión (la ventana lista impresoras al montar). */
const esperarDestino = async () => {
  await waitFor(() => expect(screen.getByDisplayValue(/Termica/)).toBeInTheDocument());
};

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockImplementation((cmd: string) => {
    if (cmd === "completar_venta") return Promise.resolve({ venta_id: 1, ticket_number: 1 });
    if (cmd === "get_tienda_info") return Promise.resolve({ nombre: null, ubicacion: null, cp: null });
    if (cmd === "listar_impresoras") {
      return Promise.resolve([{ nombre: "Termica", predeterminada: true }]);
    }
    if (cmd === "imprimir_ticket_venta") return Promise.resolve("ok");
    return Promise.resolve(null);
  });
});

describe("estres ventas · ráfaga de cobros", () => {
  it("procesa 100 cobros secuenciales sin perder ninguno", async () => {
    let exitosas = 0;
    let terminadas = 0;
    for (let i = 0; i < 100; i++) {
      const { unmount } = render(
        <VentanaCobro
          onCancelar={() => {}}
          onTerminar={() => { terminadas++; }}
          cart={[item(1)]}
          cartTotal={10}
        />,
      );
      await esperarDestino();
      const label = screen.getByText(/Efectivo/i);
      fireEvent.change(label.parentElement!.querySelector("input")!, { target: { value: "10" } });
      fireEvent.click(screen.getByText(/Cobrar \$/i));
      // Cobro completo = completar + auto-print + terminar.
      await waitFor(() => expect(terminadas).toBe(i + 1));
      exitosas++;
      unmount();
    }
    expect(exitosas).toBe(100);
    expect(comandos("completar_venta")).toHaveLength(100);
    expect(comandos("imprimir_ticket_venta")).toHaveLength(100);
  }, 60000);

  it("carga y cobra un carrito de 200 items", async () => {
    const cart = Array.from({ length: 200 }, (_, i) => item(i));
    let terminadas = 0;
    render(
      <VentanaCobro
        onCancelar={() => {}}
        onTerminar={() => { terminadas++; }}
        cart={cart}
        cartTotal={2000}
      />,
    );
    await esperarDestino();
    const label = screen.getByText(/Efectivo/i);
    fireEvent.change(label.parentElement!.querySelector("input")!, { target: { value: "2000" } });
    fireEvent.click(screen.getByText(/Cobrar \$/i));
    await waitFor(() => expect(terminadas).toBe(1));
    const llamada = comandos("completar_venta")[0]!;
    const [, args] = llamada as [string, any];
    expect(args.venta.items).toHaveLength(200);
    expect(args.venta.total).toBe(2000);
  });
});
