// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Módulo VENTAS / POS (front-empleado, emplea_new_venta).
// Cubre: VentanaCobro (1 fase) — guard de cobro insuficiente, cobro
// principal (cobra + auto-imprime + termina), cobro sin ticket (abre
// cajón, no imprime), Enter para cobrar, anti-doble-cobro, print que
// falla (la venta manda igual), Escape y descuento por línea.
// ═══════════════════════════════════════════════════════════════════════════

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { mockInvoke } from "./setup";
import VentanaCobro from "../front-empleado/ventanas/emplea_new_venta/ventana-cobro/ventana-cobro";

const CART = [
  { id: 1, nombre: "Coca-Cola 600ml", precio_venta: 18, cantidad: 2, stock: 50, descuento: 0 },
  { id: 2, nombre: "Sabritas", precio_venta: 20, cantidad: 1, stock: 30, descuento: 0 },
];
const TOTAL = 56;

const montar = (onTerminar = vi.fn(), onCancelar = vi.fn()) => {
  render(
    <VentanaCobro
      onCancelar={onCancelar}
      onTerminar={onTerminar}
      cart={CART}
      cartTotal={TOTAL}
    />,
  );
  return { onTerminar, onCancelar };
};

const ponerMonto = (label: RegExp, valor: string) => {
  const labelEl = screen.getByText(label);
  const input = labelEl.parentElement!.querySelector("input") as HTMLInputElement;
  fireEvent.change(input, { target: { value: valor } });
};

const btnCobrar = () => screen.getByText(/Cobrar \$/i) as HTMLButtonElement;
const btnSinTicket = () => screen.getByText(/Cobrar sin ticket/i) as HTMLButtonElement;

/** Monta y espera a que cargue el destino de impresión (la ventana
 * lista impresoras al montar; sin esto el auto-print no tiene a dónde). */
const montarLista = async (onTerminar = vi.fn(), onCancelar = vi.fn()) => {
  const r = montar(onTerminar, onCancelar);
  await vi.waitFor(() => expect(screen.getByDisplayValue(/Termica/)).toBeInTheDocument());
  return r;
};

const comandos = (cmd: string) => mockInvoke.mock.calls.filter(([c]) => c === cmd);

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockImplementation((cmd: string) => {
    if (cmd === "completar_venta") return Promise.resolve({ venta_id: 99, ticket_number: 42 });
    if (cmd === "get_tienda_info") return Promise.resolve({ nombre: null, ubicacion: null, cp: null });
    if (cmd === "listar_impresoras") {
      return Promise.resolve([{ nombre: "Termica", predeterminada: true }]);
    }
    if (cmd === "imprimir_ticket_venta") return Promise.resolve("Ticket impreso");
    if (cmd === "abrir_cajon") return Promise.resolve("Cajón abierto");
    return Promise.resolve(null);
  });
});

describe("ventas · guard de cobro insuficiente", () => {
  it("deshabilita Cobrar si el monto pagado es menor al total", () => {
    montar();
    expect(btnCobrar()).toBeDisabled(); // sin montos capturados
    expect(btnSinTicket()).toBeDisabled();
    ponerMonto(/Efectivo/i, "10");
    expect(btnCobrar()).toBeDisabled();
    expect(btnSinTicket()).toBeDisabled();
    expect(comandos("completar_venta")).toHaveLength(0);
  });

  it("habilita los botones justo cuando el pago cubre el total", () => {
    montar();
    ponerMonto(/Efectivo/i, String(TOTAL));
    expect(btnCobrar()).not.toBeDisabled();
    expect(btnSinTicket()).not.toBeDisabled();
  });
});

describe("ventas · cobro principal (cobra + imprime + termina)", () => {
  it("completa, auto-imprime y termina la venta", async () => {
    const { onTerminar } = await montarLista();
    ponerMonto(/Efectivo/i, String(TOTAL));
    fireEvent.click(btnCobrar());

    await vi.waitFor(() => expect(comandos("completar_venta")).toHaveLength(1));
    const [, args] = comandos("completar_venta")[0] as [string, any];
    expect(args.venta.total).toBe(TOTAL);
    expect(args.venta.items).toHaveLength(2);
    expect(args.venta.items[0]).toMatchObject({ id: 1, nombre: "Coca-Cola 600ml", cantidad: 2 });
    await vi.waitFor(() => expect(comandos("imprimir_ticket_venta")).toHaveLength(1));
    const [, printArgs] = comandos("imprimir_ticket_venta")[0] as [string, any];
    expect(printArgs.ticket.folio).toBe("#42");
    await vi.waitFor(() => expect(onTerminar).toHaveBeenCalledTimes(1));
  });

  it("acepta pago mixto efectivo + tarjeta", async () => {
    await montarLista();
    ponerMonto(/Efectivo/i, "30");
    ponerMonto(/Tarjeta/i, "26");
    fireEvent.click(btnCobrar());
    await vi.waitFor(() => expect(comandos("completar_venta")).toHaveLength(1));
    const [, args] = comandos("completar_venta")[0] as [string, any];
    expect(args.venta.monto_efectivo).toBe(30);
    expect(args.venta.monto_tarjeta).toBe(26);
  });

  it("si el backend falla no cobra ni termina y muestra el error", async () => {
    const { onTerminar } = montar();
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "completar_venta") return Promise.reject("stock insuficiente");
      return Promise.resolve(null);
    });
    ponerMonto(/Efectivo/i, String(TOTAL));
    fireEvent.click(btnCobrar());
    await vi.waitFor(() => expect(screen.getByText(/stock insuficiente/i)).toBeInTheDocument());
    expect(comandos("imprimir_ticket_venta")).toHaveLength(0);
    expect(onTerminar).not.toHaveBeenCalled();
  });

  it("si el print falla la venta manda igual y termina", async () => {
    const { onTerminar } = await montarLista();
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "completar_venta") return Promise.resolve({ venta_id: 99, ticket_number: 42 });
      if (cmd === "imprimir_ticket_venta") return Promise.reject("sin papel");
      if (cmd === "listar_impresoras") {
        return Promise.resolve([{ nombre: "Termica", predeterminada: true }]);
      }
      return Promise.resolve(null);
    });
    ponerMonto(/Efectivo/i, String(TOTAL));
    fireEvent.click(btnCobrar());
    await vi.waitFor(() => expect(onTerminar).toHaveBeenCalledTimes(1));
    expect(comandos("completar_venta")).toHaveLength(1);
  });
});

describe("ventas · cobro sin ticket (abre cajón, no imprime)", () => {
  it("completa, abre el cajón y termina sin imprimir", async () => {
    const { onTerminar } = await montarLista();
    ponerMonto(/Efectivo/i, String(TOTAL));
    fireEvent.click(btnSinTicket());
    await vi.waitFor(() => expect(comandos("completar_venta")).toHaveLength(1));
    await vi.waitFor(() => expect(comandos("abrir_cajon")).toHaveLength(1));
    expect(comandos("imprimir_ticket_venta")).toHaveLength(0);
    await vi.waitFor(() => expect(onTerminar).toHaveBeenCalledTimes(1));
  });
});

describe("ventas · Enter y anti-doble-cobro", () => {
  it("Enter en el monto cobra con el flujo principal", async () => {
    const { onTerminar } = await montarLista();
    const labelEl = screen.getByText(/Efectivo/i);
    const input = labelEl.parentElement!.querySelector("input") as HTMLInputElement;
    fireEvent.change(input, { target: { value: String(TOTAL) } });
    fireEvent.keyDown(input, { key: "Enter" });
    await vi.waitFor(() => expect(comandos("completar_venta")).toHaveLength(1));
    await vi.waitFor(() => expect(comandos("imprimir_ticket_venta")).toHaveLength(1));
    await vi.waitFor(() => expect(onTerminar).toHaveBeenCalledTimes(1));
  });

  it("varios Enters rápidos cobran UNA sola vez", async () => {
    await montarLista();
    const labelEl = screen.getByText(/Efectivo/i);
    const input = labelEl.parentElement!.querySelector("input") as HTMLInputElement;
    fireEvent.change(input, { target: { value: String(TOTAL) } });
    // Ráfaga en el mismo tick: sin candado síncrono serían N cobros.
    fireEvent.keyDown(input, { key: "Enter" });
    fireEvent.keyDown(input, { key: "Enter" });
    fireEvent.keyDown(input, { key: "Enter" });
    await vi.waitFor(() => expect(comandos("completar_venta")).toHaveLength(1));
    await new Promise((r) => setTimeout(r, 50));
    expect(comandos("completar_venta")).toHaveLength(1);
    expect(comandos("imprimir_ticket_venta")).toHaveLength(1);
  });
});

describe("ventas · UX de la ventana", () => {
  it("cierra con la tecla Escape sin cobrar", () => {
    const { onCancelar, onTerminar } = montar();
    fireEvent.keyDown(window, { key: "Escape" });
    expect(onCancelar).toHaveBeenCalled();
    expect(onTerminar).not.toHaveBeenCalled();
    expect(comandos("completar_venta")).toHaveLength(0);
  });

  it("muestra los productos en venta antes de cobrar", () => {
    montar();
    expect(screen.getByText(/Coca-Cola 600ml/)).toBeInTheDocument();
    expect(screen.getByText(/Sabritas/)).toBeInTheDocument();
  });

  it("muestra destino de impresión y cajón manual", () => {
    montar();
    expect(screen.getByText("Local")).toBeInTheDocument();
    expect(screen.getByText("Red")).toBeInTheDocument();
    expect(screen.getByText(/Abrir cajón/i)).toBeInTheDocument();
  });
});

describe("ventas · descuento por línea", () => {
  const CART_DESC = [
    { id: 1, nombre: "Coca-Cola 600ml", precio_venta: 18, cantidad: 2, stock: 50, descuento: 6 },
    { id: 2, nombre: "Sabritas", precio_venta: 20, cantidad: 1, stock: 30, descuento: 0 },
  ];
  const BRUTO = 56;
  const NETO = 50;

  it("manda descuento por línea, subtotal bruto y total neto al backend", async () => {
    render(
      <VentanaCobro
        onCancelar={vi.fn()}
        onTerminar={vi.fn()}
        cart={CART_DESC}
        cartTotal={NETO}
      />,
    );
    await vi.waitFor(() => expect(screen.getByDisplayValue(/Termica/)).toBeInTheDocument());
    ponerMonto(/Efectivo/i, String(NETO));
    fireEvent.click(btnCobrar());

    await vi.waitFor(() => expect(comandos("completar_venta")).toHaveLength(1));
    const [, args] = comandos("completar_venta")[0] as [string, any];
    expect(args.venta.subtotal).toBe(BRUTO);
    expect(args.venta.total).toBe(NETO);
    expect(args.venta.items[0]).toMatchObject({ id: 1, descuento: 6 });
    expect(args.venta.items[1]).toMatchObject({ id: 2, descuento: 0 });
  });

  it("muestra la línea de Descuento en el resumen solo si hay rebaja", () => {
    const { rerender } = render(
      <VentanaCobro onCancelar={vi.fn()} onTerminar={vi.fn()} cart={CART_DESC} cartTotal={NETO} />,
    );
    expect(screen.getByText("Descuento")).toBeInTheDocument();
    expect(screen.getAllByText(/\$6\.00/).length).toBeGreaterThanOrEqual(1);

    rerender(
      <VentanaCobro onCancelar={vi.fn()} onTerminar={vi.fn()} cart={CART} cartTotal={TOTAL} />,
    );
    expect(screen.queryByText("Descuento")).toBeNull();
  });
});
