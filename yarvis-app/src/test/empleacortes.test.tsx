// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Módulo EMPLEACORTES (front-empleado, botón CORTE / F3).
// Cubre: ModalCorte — pastillas X/Z con su mensaje, Z exige confirmación,
// Cancelar no guarda nada, Continuar pide el reporte y muestra la previa
// con folios TICKET-NNNN, e imprimir manda el corte a la térmica.
// ═══════════════════════════════════════════════════════════════════════════

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { mockInvoke } from "./setup";
import ModalCorte from "../front-empleado/ventanas/empleacortes/modal-corte";

const REPORTE_X = {
  corte_id: 7,
  cajero_id: 3,
  ancla: "2026-09-14 09:12:00",
  cierre: "2026-09-14 18:40:00",
  tickets: [
    { venta_id: 10099, folio: "TICKET-10099", fecha: "2026-09-14 10:00:00", total: 95, metodo_pago: "efectivo" },
    { venta_id: 10100, folio: "TICKET-10100", fecha: "2026-09-14 10:05:00", total: 32, metodo_pago: "efectivo" },
  ],
  totales: { total_ventas: 127, total_efectivo: 127, total_tarjeta: 0, total_transferencia: 0, num_tickets: 2 },
};

const REPORTE_Z = {
  ...REPORTE_X,
  corte_id: 3,
  productos: [
    { producto_nombre: "COCA 3L", cantidad: 3, monto: 120 },
  ],
};

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockImplementation((cmd: string) => {
    if (cmd === "corte_x_reporte") return Promise.resolve(REPORTE_X);
    if (cmd === "corte_z_cierre") return Promise.resolve(REPORTE_Z);
    if (cmd === "listar_impresoras") return Promise.resolve([{ nombre: "XPrinter", predeterminada: true }]);
    if (cmd === "get_tienda_info") return Promise.resolve({ nombre: "MI TIENDA", ubicacion: null, cp: null });
    if (cmd === "imprimir_corte") return Promise.resolve("CORTE-X-#7 enviado al spooler (100 bytes).");
    return Promise.resolve(undefined);
  });
});

const montar = (onClose = vi.fn()) => {
  render(<ModalCorte onClose={onClose} cajero="JUAN" />);
  return { onClose };
};

describe("empleacortes · elegir tipo", () => {
  it("muestra el mensaje del X por defecto y Cancelar no guarda nada", () => {
    const { onClose } = montar();
    expect(screen.getByText(/foto de tu turno/i)).toBeInTheDocument();
    fireEvent.click(screen.getByText(/^Cancelar$/i));
    expect(onClose).toHaveBeenCalledTimes(1);
    expect(mockInvoke).not.toHaveBeenCalledWith("corte_x_reporte", expect.anything());
    expect(mockInvoke).not.toHaveBeenCalledWith("corte_z_cierre", expect.anything());
  });

  it("al elegir Z pide confirmación explícita", () => {
    montar();
    fireEvent.click(screen.getByRole("button", { name: /^Z$/ }));
    expect(screen.getByText(/cierre DEFINITIVO/i)).toBeInTheDocument();
    expect(screen.getByText(/¿Quieres continuar\?/i)).toBeInTheDocument();
  });
});

describe("empleacortes · previa e impresión", () => {
  it("Continuar en X pide el reporte y muestra folios TICKET-NNNN", async () => {
    montar();
    fireEvent.click(screen.getByText(/^Continuar$/i));
    await vi.waitFor(() => expect(mockInvoke).toHaveBeenCalledWith("corte_x_reporte", expect.anything()));
    await vi.waitFor(() => expect(screen.getByText(/TICKET-10099/)).toBeInTheDocument());
    expect(screen.getByText(/Total ventas/)).toBeInTheDocument();
  });

  it("Imprimir manda el corte guardado a la térmica", async () => {
    montar();
    fireEvent.click(screen.getByText(/^Continuar$/i));
    await vi.waitFor(() => expect(screen.getByText(/TICKET-10099/)).toBeInTheDocument());
    fireEvent.click(screen.getByText(/Imprimir en térmica/i));
    await vi.waitFor(() => expect(mockInvoke).toHaveBeenCalledWith("imprimir_corte", expect.anything()));
    const llamada = mockInvoke.mock.calls.find(([cmd]) => cmd === "imprimir_corte")!;
    expect(llamada[1].corte_id).toBe(7);
    await vi.waitFor(() => expect(screen.getByText(/enviado al spooler/i)).toBeInTheDocument());
  });
});
