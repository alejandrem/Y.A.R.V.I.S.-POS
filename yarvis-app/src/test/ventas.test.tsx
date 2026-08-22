// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Módulo VENTAS / POS (front-empleado, emplea_new_venta).
// Cubre: ModalVenta — guard de cobro insuficiente (botón deshabilitado),
// pago mixto, invocación de completar_venta con payload correcto,
// callback al completar y cierre con tecla Escape.
// ═══════════════════════════════════════════════════════════════════════════

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { mockInvoke } from "./setup";
import ModalVenta from "../front-empleado/ventanas/emplea_new_venta/modalventa";

const CART = [
  { id: 1, nombre: "Coca-Cola 600ml", precio_venta: 18, cantidad: 2, stock: 50 },
  { id: 2, nombre: "Sabritas", precio_venta: 20, cantidad: 1, stock: 30 },
];
const TOTAL = 56;

const montar = (onVentaCompletada = vi.fn(), onClose = vi.fn()) => {
  render(
    <ModalVenta
      onClose={onClose}
      onVentaCompletada={onVentaCompletada}
      cart={CART}
      cartTotal={TOTAL}
    />,
  );
  return { onVentaCompletada, onClose };
};

const ponerMonto = (label: RegExp, valor: string) => {
  const labelEl = screen.getByText(label);
  const input = labelEl.parentElement!.querySelector("input") as HTMLInputElement;
  fireEvent.change(input, { target: { value: valor } });
};

const btnConfirmar = () => screen.getByText(/Confirmar Venta/i) as HTMLButtonElement;

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockResolvedValue({ venta_id: 99, ticket_number: 42 });
});

describe("ventas · guard de cobro insuficiente", () => {
  it("deshabilita Confirmar Venta si el monto pagado es menor al total", () => {
    montar();
    expect(btnConfirmar()).toBeDisabled(); // sin montos capturados
    ponerMonto(/Efectivo/i, "10");
    expect(btnConfirmar()).toBeDisabled();
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("habilita el botón justo cuando el pago cubre el total", () => {
    montar();
    ponerMonto(/Efectivo/i, String(TOTAL));
    expect(btnConfirmar()).not.toBeDisabled();
  });
});

describe("ventas · cobro exitoso", () => {
  it("invoca completar_venta con items y totales correctos y notifica el resultado", async () => {
    const { onVentaCompletada } = montar();
    ponerMonto(/Efectivo/i, String(TOTAL));
    fireEvent.click(btnConfirmar());

    await vi.waitFor(() => expect(mockInvoke).toHaveBeenCalledTimes(1));
    const [cmd, args] = mockInvoke.mock.calls[0];
    expect(cmd).toBe("completar_venta");
    expect(args.venta.total).toBe(TOTAL);
    expect(args.venta.items).toHaveLength(2);
    expect(args.venta.items[0]).toMatchObject({ id: 1, nombre: "Coca-Cola 600ml", cantidad: 2 });
    await vi.waitFor(() =>
      expect(onVentaCompletada).toHaveBeenCalledWith(99, 42, TOTAL, 0, 0),
    );
  });

  it("acepta pago mixto efectivo + tarjeta", async () => {
    montar();
    ponerMonto(/Efectivo/i, "30");
    ponerMonto(/Tarjeta/i, "26");
    fireEvent.click(btnConfirmar());
    await vi.waitFor(() => expect(mockInvoke).toHaveBeenCalledTimes(1));
    const [, args] = mockInvoke.mock.calls[0];
    expect(args.venta.monto_efectivo).toBe(30);
    expect(args.venta.monto_tarjeta).toBe(26);
  });

  it("si el backend falla no se notifica éxito y muestra el error", async () => {
    const { onVentaCompletada } = montar();
    mockInvoke.mockRejectedValueOnce("stock insuficiente");
    ponerMonto(/Efectivo/i, String(TOTAL));
    fireEvent.click(btnConfirmar());
    await vi.waitFor(() => expect(screen.getByText(/stock insuficiente/i)).toBeInTheDocument());
    expect(onVentaCompletada).not.toHaveBeenCalled();
  });
});

describe("ventas · UX del modal", () => {
  it("cierra con la tecla Escape", () => {
    const { onClose } = montar();
    fireEvent.keyDown(window, { key: "Escape" });
    expect(onClose).toHaveBeenCalled();
  });
});
