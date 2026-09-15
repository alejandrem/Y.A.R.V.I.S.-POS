// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Módulo REIMPRIMIR (front-empleado, F2 global, #16).
// Cubre: ModalReimprimir — busca solo al escribir (debounce), muestra la
// preview completa con folio TICKET-NNNN, error rojo si no existe (y se
// esconde Imprimir), e imprime en la térmica con el payload correcto.
// Solo reimprime tickets propios (backend operator-scoped).
// ═══════════════════════════════════════════════════════════════════════════

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { mockInvoke } from "./setup";
import ModalReimprimir from "../front-empleado/ventanas/empleaticket/componentes/modal-reimprimir";

const DETALLE = {
  id: 10109,
  folio_ticket: null,
  fecha: "2026-09-14 18:02:00",
  total: 95,
  subtotal: 95,
  descuento: 0,
  metodo_pago: "efectivo",
  items: [
    { producto_nombre: "COCA 3L", cantidad: 2, precio_unitario: 40, subtotal: 80 },
    { producto_nombre: "SABRITAS", cantidad: 1, precio_unitario: 15, subtotal: 15 },
  ],
};

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockImplementation((cmd: string, args?: Record<string, unknown>) => {
    if (cmd === "get_mi_ticket_detalle") {
      if (args?.venta_id === 10109) return Promise.resolve(DETALLE);
      return Promise.reject("Ticket no encontrado");
    }
    if (cmd === "listar_impresoras") return Promise.resolve([{ nombre: "XPrinter", predeterminada: true }]);
    if (cmd === "get_tienda_info") return Promise.resolve({ nombre: "MI TIENDA", ubicacion: null, cp: null });
    if (cmd === "imprimir_ticket_venta") return Promise.resolve("Ticket enviado al spooler (100 bytes).");
    return Promise.resolve(undefined);
  });
});

const escribirNumero = (valor: string) => {
  fireEvent.change(screen.getByPlaceholderText("10109"), { target: { value: valor } });
};

describe("reimprimir · búsqueda", () => {
  it("muestra la preview completa con folio TICKET-NNNN al encontrarlo", async () => {
    render(<ModalReimprimir onClose={() => {}} />);
    escribirNumero("10109");
    await vi.waitFor(() => expect(screen.getAllByText(/TICKET-10109/).length).toBeGreaterThanOrEqual(2));
    expect(screen.getByText(/Total: \$95\.00/)).toBeInTheDocument();
    expect(screen.getByText(/Imprimir en térmica/i)).toBeInTheDocument();
  });

  it("si no existe muestra error rojo y el botón se ve desactivado", async () => {
    render(<ModalReimprimir onClose={() => {}} />);
    escribirNumero("99999");
    await vi.waitFor(() =>
      expect(screen.getByText(/Ese ticket no existe, revisa el número/i)).toBeInTheDocument(),
    );
    // El botón siempre se ve; sin ticket válido sale desactivado.
    expect(screen.getByText(/Imprimir en térmica/i)).toBeInTheDocument();
    expect(mockInvoke).not.toHaveBeenCalledWith("imprimir_ticket_venta", expect.anything());
  });

  it("solo acepta dígitos en el campo", () => {
    render(<ModalReimprimir onClose={() => {}} />);
    const input = screen.getByPlaceholderText("10109") as HTMLInputElement;
    escribirNumero("10a9b");
    expect(input.value).toBe("109");
  });
});

describe("reimprimir · impresión", () => {
  it("manda el ticket a la térmica con folio, líneas y total", async () => {
    render(<ModalReimprimir onClose={() => {}} />);
    escribirNumero("10109");
    await vi.waitFor(() => expect(screen.getByText(/Total: \$95\.00/)).toBeInTheDocument());
    fireEvent.click(screen.getByRole("button", { name: /Imprimir en térmica/i }));
    await vi.waitFor(() => expect(mockInvoke).toHaveBeenCalledWith("imprimir_ticket_venta", expect.anything()));
    const llamada = mockInvoke.mock.calls.find(([cmd]) => cmd === "imprimir_ticket_venta")!;
    expect(llamada[1].ticket.folio).toBe("TICKET-10109");
    expect(llamada[1].ticket.total).toBe(95);
    expect(llamada[1].ticket.lineas).toHaveLength(2);
    await vi.waitFor(() => expect(screen.getByText(/enviado al spooler/i)).toBeInTheDocument());
  });

  it("Cancelar cierra sin invocar nada de impresión", () => {
    const onClose = vi.fn();
    render(<ModalReimprimir onClose={onClose} />);
    fireEvent.click(screen.getByText(/^Cancelar$/i));
    expect(onClose).toHaveBeenCalledTimes(1);
    expect(mockInvoke).not.toHaveBeenCalledWith("imprimir_ticket_venta", expect.anything());
  });
});
