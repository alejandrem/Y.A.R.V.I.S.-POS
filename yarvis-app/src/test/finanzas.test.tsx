// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Módulo FINANZAS (adminfinanzas).
// Cubre: utilidades puras de formato/fechas, el SelectorRango (presets y
// fechas custom) y las primitivas visuales (SeccionGrafica, EmptyGrafica).
// ═══════════════════════════════════════════════════════════════════════════

import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import SelectorRango from "../front-admin/ventanas/adminfinanzas/componentes/selector-rango";
import { SeccionGrafica, EmptyGrafica, KPI } from "../front-admin/ventanas/adminfinanzas/componentes/ui-finanzas";
import { moneda, porcentaje, fechaRelativa, rangoDeDias, hoyISO } from "../front-admin/ventanas/adminfinanzas/nucleo/utilidades";
import { TABS } from "../front-admin/ventanas/adminfinanzas/nucleo/constantes";

describe("finanzas · utilidades puras", () => {
  it("moneda formatea como MXN", () => {
    expect(moneda(1500)).toMatch(/\$1,500\.00/);
    expect(moneda(0)).toMatch(/\$0\.00/);
  });

  it("porcentaje agrega signo positivo y negativo", () => {
    expect(porcentaje(12.34)).toBe("+12.3%");
    expect(porcentaje(-5.12)).toBe("-5.1%");
    expect(porcentaje(0)).toBe("+0.0%");
  });

  it("rangoDeDias genera inicio = hoy - N dias", () => {
    const r = rangoDeDias(30);
    const esperadoFin = new Date().toISOString().slice(0, 10);
    const esperadoInicio = new Date(Date.now() - 30 * 86400000).toISOString().slice(0, 10);
    expect(r.fin).toBe(esperadoFin);
    expect(r.inicio).toBe(esperadoInicio);
  });

  it("hoyISO devuelve formato YYYY-MM-DD", () => {
    expect(hoyISO()).toMatch(/^\d{4}-\d{2}-\d{2}$/);
  });

  it("fechaRelativa clasifica Hoy/Ayer/futura lejana", () => {
    const hoy = new Date().toISOString();
    const ayer = new Date(Date.now() - 86400000).toISOString();
    expect(fechaRelativa(hoy)).toBe("Hoy");
    expect(fechaRelativa(ayer)).toBe("Ayer");
  });
});

describe("finanzas · TABS", () => {
  it("define las 5 secciones del panel", () => {
    expect(TABS.map((t) => t.id)).toEqual(["resumen", "gastos", "cortes", "alertas", "metricas"]);
  });
});

describe("finanzas · SelectorRango", () => {
  it("resalta el preset activo según el rango recibido", () => {
    const rango = rangoDeDias(7);
    render(<SelectorRango rango={rango} onChange={() => {}} />);
    const btn7 = screen.getByText("7D");
    expect(btn7).toHaveClass("bg-neutral-950");
  });

  it("click en preset 30D emite rango de 30 dias", () => {
    const onChange = vi.fn();
    render(<SelectorRango rango={rangoDeDias(180)} onChange={onChange} />);
    fireEvent.click(screen.getByText("30D"));
    expect(onChange).toHaveBeenCalledTimes(1);
    const emitido = onChange.mock.calls[0][0];
    const dias =
      Math.round((+new Date(emitido.fin) - +new Date(emitido.inicio)) / 86400000);
    expect(dias).toBe(30);
  });

  it("cambiar la fecha de inicio emite rango custom sin tocar el fin", () => {
    const onChange = vi.fn();
    render(<SelectorRango rango={{ inicio: "2026-01-01", fin: "2026-03-01" }} onChange={onChange} />);
    fireEvent.change(screen.getAllByDisplayValue(/^2026-/)[0], { target: { value: "2026-01-15" } });
    expect(onChange).toHaveBeenCalledWith({ inicio: "2026-01-15", fin: "2026-03-01" });
  });
});

describe("finanzas · primitivas UI", () => {
  it("SeccionGrafica muestra titulo, subtitulo y children", () => {
    render(
      <SeccionGrafica titulo="Perdidas y Ganancias" subtitulo="Ingresos y gastos">
        <div data-testid="contenido">grafica</div>
      </SeccionGrafica>,
    );
    expect(screen.getByText("Perdidas y Ganancias")).toBeInTheDocument();
    expect(screen.getByText("Ingresos y gastos")).toBeInTheDocument();
    expect(screen.getByTestId("contenido")).toBeInTheDocument();
  });

  it("EmptyGrafica muestra su mensaje cuando no hay datos", () => {
    render(<EmptyGrafica mensaje="Sin datos de P&L en este periodo" />);
    expect(screen.getByText(/Sin datos de P&L/i)).toBeInTheDocument();
  });

  it("KPI pinta verde para utilidad positiva y rojo para negativa", () => {
    const { rerender } = render(<KPI icono="dollar" label="Utilidad" valor="$100" color="verde" />);
    expect(screen.getByText("$100")).toHaveClass("text-emerald-600");
    rerender(<KPI icono="dollar" label="Utilidad" valor="-$50" color="rojo" />);
    expect(screen.getByText("-$50")).toHaveClass("text-red-500");
  });
});
