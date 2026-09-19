// TEST — Pitazo sin match en NUEVA VENTA alimenta la cola del semáforo.
// Cubre: escáner HID escribe código + Enter sin resultados -> se invoca
// `rojo_registrar_pendiente` (ean). Basura corta (<3) no registra nada.

import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { mockInvoke } from "./setup";
import NuevaVenta from "../front-empleado/ventanas/emplea_new_venta/nueva_venta";
import { CartProvider } from "../front-empleado/ventanas/emplea_new_venta/CartProvider";

const montar = () => {
  render(
    <CartProvider>
      <NuevaVenta activeTab="nueva_venta" onAbrirCorte={() => {}} />
    </CartProvider>,
  );
};

const inputBuscador = () =>
  screen.getByPlaceholderText(/escanea/i) as HTMLInputElement;

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockImplementation((cmd: unknown) => {
    if (cmd === "get_inventory") return Promise.resolve([]);
    if (cmd === "buscar_producto_similar") return Promise.reject("sin match");
    if (cmd === "rojo_registrar_pendiente") return Promise.resolve(7);
    return Promise.resolve(undefined);
  });
});

describe("nueva venta · pitazo desconocido va a la cola", () => {
  it("Enter con EAN sin resultados registra el pendiente con ean", async () => {
    montar();
    fireEvent.change(inputBuscador(), { target: { value: "7501234567890" } });

    // Debounce (200ms) + IA fallida -> dropdown visible pero vacío.
    await waitFor(() => expect(mockInvoke).toHaveBeenCalledWith(
      "buscar_producto_similar",
      expect.anything(),
    ));
    fireEvent.keyDown(inputBuscador(), { key: "Enter", code: "Enter" });

    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("rojo_registrar_pendiente", {
        ean: "7501234567890",
        nombreCrudo: "7501234567890",
      }),
    );
  });

  it("basura corta no registra nada", async () => {
    montar();
    fireEvent.change(inputBuscador(), { target: { value: "ab" } });
    // Sin debounce necesario: Enter directo tras escribir.
    fireEvent.keyDown(inputBuscador(), { key: "Enter", code: "Enter" });

    await new Promise((r) => setTimeout(r, 350));
    expect(mockInvoke).not.toHaveBeenCalledWith(
      "rojo_registrar_pendiente",
      expect.anything(),
    );
  });
});
