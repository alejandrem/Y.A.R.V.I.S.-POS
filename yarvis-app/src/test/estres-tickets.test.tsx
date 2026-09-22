// ═══════════════════════════════════════════════════════════════════════════
// TEST DE ESTRÉS — Módulo TICKETS.
// Presión: historial de 3,000 tickets y 20 ciclos de cambio de rango.
// ═══════════════════════════════════════════════════════════════════════════

import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { mockInvoke } from "./setup";
import Tickets from "../front-admin/ventanas/adminticket/tickets";

const TICKETS = Array.from({ length: 3000 }, (_, i) => ({
  id: i,
  folio_ticket: `T-${i}`,
  fecha: new Date(Date.now() - (i % 60) * 86400000).toISOString(),
  total: 100,
  metodo_pago: "efectivo",
}));

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockImplementation((cmd: string) => {
    if (cmd === "get_tickets") return Promise.resolve(TICKETS);
    if (cmd === "get_cortes") return Promise.resolve([]);
    if (cmd === "get_predictions") return Promise.resolve({ data: [] });
    return Promise.resolve("ok");
  });
});

describe("estres tickets · historial masivo", () => {
  it("muestra 100 tickets con botón Ver más y sobrevive cambios de rango", async () => {
    render(<Tickets active={true} />);
    await waitFor(() => expect(mockInvoke).toHaveBeenCalledWith("get_tickets", expect.anything()));

    // Ventana visible acotada: 100 nodos + botón progresivo, no 3000.
    expect(await screen.findByText(/restantes/i)).toBeInTheDocument();

    for (let i = 0; i < 20; i++) {
      fireEvent.click(await screen.findByText(/7 DÍAS/i));
      fireEvent.click(await screen.findByText("TODOS"));
    }
    expect(screen.getByText(/15 DÍAS/i)).toBeInTheDocument();
  }, 30000);
});
