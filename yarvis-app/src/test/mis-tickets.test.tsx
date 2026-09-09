// ═══════════════════════════════════════════════════════════════════════════
// TEST — Mis tickets del empleado (helpers puros + smoke de la vista).
// Los helpers se prueban con datos fijos; la vista con invoke mockeado
// (nunca toca el backend real).
// ═══════════════════════════════════════════════════════════════════════════

import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { mockInvoke } from "./setup";
import {
  rangoMisADias, fmtExtra, sumarExtrasPorRango, DIAS_TODOS,
} from "../front-empleado/ventanas/empleaticket/componentes/kpis-mis-tickets";
import { rellenarDiasVentas } from "../front-empleado/ventanas/empleaticket/componentes/grafica-dia-sueldo";
import Tickets from "../front-empleado/ventanas/empleaticket/ticket";
import type { DiaExtra } from "../components/turno";

const extra = (fecha: string, pre: number, post: number): DiaExtra => ({
  fecha,
  dia_label: "LUN",
  primer_login: "08:00",
  ultimo_login: "16:00",
  entrada_oficial: "09:00",
  salida_oficial: "15:00",
  extra_pre_min: pre,
  extra_post_min: post,
  trabajo_min: 480,
});

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockImplementation((cmd: string) => {
    if (cmd === "get_mis_kpis") return Promise.resolve({ total: 1250, tickets: 5, ticket_promedio: 250 });
    if (cmd === "get_mis_horas_extra")
      return Promise.resolve([extra("2026-09-08", 60, 40), extra("2026-09-07", 0, 30)]);
    if (cmd === "get_mis_ventas_por_dia")
      return Promise.resolve([{ fecha: "2026-09-08", total: 1250 }]);
    if (cmd === "get_mis_tickets")
      return Promise.resolve([
        { id: 1042, folio_ticket: null, fecha: "2026-09-08 10:32:00", total: 320, metodo_pago: "efectivo" },
      ]);
    if (cmd === "get_employee_profile")
      return Promise.resolve({ profile: { salario_diario: 350 } });
    if (cmd === "get_mi_ticket_detalle")
      return Promise.resolve({
        id: 1042, folio_ticket: null, fecha: "2026-09-08 10:32:00",
        total: 320, subtotal: 320, descuento: 0, metodo_pago: "efectivo",
        items: [{ producto_nombre: "Coca", cantidad: 2, precio_unitario: 160, subtotal: 320 }],
      });
    return Promise.resolve(null);
  });
});

describe("mis tickets · helpers puros", () => {
  it("rangoMisADias mapea hoy/7/15/30/todos y custom con clamp", () => {
    expect(rangoMisADias("hoy")).toBe(1);
    expect(rangoMisADias("7")).toBe(7);
    expect(rangoMisADias("15")).toBe(15);
    expect(rangoMisADias("30")).toBe(30);
    expect(rangoMisADias("todos")).toBe(DIAS_TODOS);
    expect(rangoMisADias("custom", 20)).toBe(20);
    expect(rangoMisADias("custom", 0)).toBe(1);
    expect(rangoMisADias("custom", 999)).toBe(365);
    expect(rangoMisADias("raro")).toBe(1);
  });

  it("fmtExtra formatea horas y minutos", () => {
    expect(fmtExtra(400)).toBe("6h 40m");
    expect(fmtExtra(30)).toBe("0h 30m");
    expect(fmtExtra(0)).toBe("0h 0m");
  });

  it("sumarExtrasPorRango respeta la ventana y solo días con extra", () => {
    const dias = [
      extra("2026-09-08", 60, 40), // 100, dentro
      extra("2026-09-07", 0, 30), // 30, dentro de 7 pero fuera de 1
      extra("2026-08-01", 120, 0), // fuera de todo
      extra("2026-09-08", 0, 0), // sin extra: no cuenta día
    ];
    expect(sumarExtrasPorRango(dias, 1, "2026-09-08")).toEqual({ minutos: 100, diasConExtra: 1 });
    expect(sumarExtrasPorRango(dias, 7, "2026-09-08")).toEqual({ minutos: 130, diasConExtra: 2 });
  });

  it("rellenarDiasVentas ordena viejo→nuevo y pone ceros", () => {
    const pts = rellenarDiasVentas([{ fecha: "2026-09-08", total: 1250 }], 3, "2026-09-08");
    expect(pts.map((p) => p.fecha)).toEqual(["2026-09-06", "2026-09-07", "2026-09-08"]);
    expect(pts.map((p) => p.total)).toEqual([0, 0, 1250]);
    expect(pts[2].etiqueta).toMatch(/08/);
  });
});

describe("mis tickets · vista", () => {
  it("muestra KPIs, gráfica, tickets y cortes próximamente", async () => {
    render(<Tickets activeTab="tickets" operatorName="Pedro" />);
    expect(screen.getByText("Tickets y cortes")).toBeInTheDocument();
    await waitFor(() => {
      expect(screen.getByText("$1,250.00")).toBeInTheDocument();
    });
    expect(screen.getByText(/Vendido por día vs mi sueldo/i)).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: /mis tickets/i })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: /mis cortes/i })).toBeInTheDocument();
    await waitFor(() => {
      expect(screen.getByText(/Ticket #1042/)).toBeInTheDocument();
    });
  });

  it("no renderiza nada fuera de su pestaña", () => {
    const { container } = render(<Tickets activeTab="perfil" operatorName="Pedro" />);
    expect(container.innerHTML).toBe("");
  });

  it("cada bloque trae su propio rango ([hoy >] en KPI, [7d >] en extras y gráfica, [Todos >] en lista)", async () => {
    render(<Tickets activeTab="tickets" operatorName="Pedro" />);
    await waitFor(() => {
      expect(screen.getByText("$1,250.00")).toBeInTheDocument();
    });
    expect(screen.getAllByText("[hoy >]")).toHaveLength(1); // KPI vendi
    expect(screen.getAllByText("[7d >]")).toHaveLength(2); // extras + gráfica
    expect(screen.getAllByText("[Todos >]")).toHaveLength(2); // lista + cortes
    // La lista pide el historial completo por defecto.
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith(
        "get_mis_tickets", { limit: 100, offset: 0, dias: DIAS_TODOS },
      );
    });
  });

  it("cambiar el rango de la lista no afecta a KPI ni gráfica", async () => {
    render(<Tickets activeTab="tickets" operatorName="Pedro" />);
    await waitFor(() => {
      expect(screen.getByText("$1,250.00")).toBeInTheDocument();
    });
    // El primer [Todos >] es el de la lista (el segundo es el de cortes).
    fireEvent.click(screen.getAllByText("[Todos >]")[0]);
    fireEvent.click(screen.getByText(/30 días/));
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("get_mis_tickets", { limit: 100, offset: 0, dias: 30 });
    });
    // KPI y gráfica siguen en su periodo: nadie los movió a 30.
    const kpis = mockInvoke.mock.calls.filter(([c]) => c === "get_mis_kpis");
    expect(kpis.length).toBeGreaterThan(0);
    for (const [, args] of kpis) expect(args).toEqual({ dias: 1 });
    const graf = mockInvoke.mock.calls.filter(([c]) => c === "get_mis_ventas_por_dia");
    expect(graf.length).toBeGreaterThan(0);
    for (const [, args] of graf) expect(args).toEqual({ dias: 7 });
  });

  it("abrir un ticket pide el detalle con venta_id (snake_case del comando)", async () => {
    render(<Tickets activeTab="tickets" operatorName="Pedro" />);
    const fila = await screen.findByText(/Ticket #1042/);
    fireEvent.click(fila.closest("button")!);
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("get_mi_ticket_detalle", { venta_id: 1042 });
    });
    expect(await screen.findByText("Coca")).toBeInTheDocument();
  });
});
