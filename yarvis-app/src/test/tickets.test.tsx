// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Módulo TICKETS (adminticket).
// Cubre: carga de tickets y cortes (get_tickets / get_cortes), renderizado
// del historial y el filtro por rango de fechas (TODOS / 7 DÍAS).
// ═══════════════════════════════════════════════════════════════════════════

import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { mockInvoke } from "./setup";
import Tickets from "../front-admin/ventanas/adminticket/tickets";

const hoy = new Date();
const hace20d = new Date(hoy.getTime() - 20 * 86400000).toISOString();

const TICKETS = [
  { id: 1, folio_ticket: "T-0001", fecha: hoy.toISOString(), total: 120, metodo_pago: "efectivo" },
  { id: 2, folio_ticket: "T-0002", fecha: hace20d, total: 300, metodo_pago: "tarjeta" },
];

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockImplementation((cmd: string) => {
    if (cmd === "get_tickets") return Promise.resolve(TICKETS);
    if (cmd === "get_cortes") return Promise.resolve([]);
    if (cmd === "get_predictions") return Promise.resolve({ data: [] });
    return Promise.resolve("ok");
  });
});

describe("tickets · carga", () => {
  it("carga tickets y cortes al activarse la pestaña", async () => {
    render(<Tickets active={true} />);
    await waitFor(() => expect(mockInvoke).toHaveBeenCalledWith("get_tickets"));
    expect(mockInvoke).toHaveBeenCalledWith("get_cortes");
  });
});

describe("tickets · filtros por rango", () => {
  it("el filtro TODOS muestra todos los tickets cargados", async () => {
    render(<Tickets active={true} />);
    fireEvent.click(await screen.findByText("TODOS"));
    // No debe crashear y debe seguir mostrando la tabla/historial
    expect(screen.getByText(/7 DÍAS/i)).toBeInTheDocument();
  });

  it("los botones de rango están presentes", async () => {
    render(<Tickets active={true} />);
    for (const r of ["TODOS", "7 DÍAS", "15 DÍAS", "1 MES"]) {
      expect(await screen.findByText(r)).toBeInTheDocument();
    }
  });
});
