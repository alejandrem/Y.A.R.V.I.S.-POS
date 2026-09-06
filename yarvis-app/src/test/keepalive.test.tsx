// TEST — Keep-alive de pestañas en los dashboards.
// Contrato: cambiar de tab NO desmonta el módulo (solo lo oculta), así el
// estado local sobrevive: importaciones en curso, chat a medias, carrito,
// filtros y formularios. Las pestañas se montan en su primera visita.
import { describe, it, expect, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { mockInvoke } from "./setup";
import AdminDashboard from "../front-admin/AdminDashboard";

const baseProps = {
  setActiveTab: () => {},
  onLogout: () => {},
  adminName: "Admin",
  storeName: "Tienda",
  adminPass: "x",
};

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockResolvedValue(undefined);
});

describe("dashboard · keep-alive", () => {
  it("cambiar de pestaña oculta pero no desmonta el módulo", () => {
    const { rerender, container } = render(<AdminDashboard activeTab="ventas" {...baseProps} />);
    expect(screen.getByText("Ventas y predicciones")).toBeVisible();

    // Va al parseador: ventas sigue en el DOM pero oculta.
    rerender(<AdminDashboard activeTab="parseador" {...baseProps} />);
    expect(screen.getByText("Parseador de Tickets")).toBeVisible();
    const ventas = screen.getByText("Ventas y predicciones");
    expect(ventas).toBeInTheDocument();
    expect(ventas.closest("div.hidden")).not.toBeNull();

    // Vuelve: el parseador sigue montado (oculto) con su estado intacto.
    rerender(<AdminDashboard activeTab="ventas" {...baseProps} />);
    expect(screen.getByText("Ventas y predicciones")).toBeVisible();
    expect(screen.getByText("Parseador de Tickets")).toBeInTheDocument();
    expect(container.querySelectorAll("div.hidden").length).toBeGreaterThan(0);
  });

  it("las pestañas no visitadas no se montan (login rápido)", () => {
    const { container } = render(<AdminDashboard activeTab="ventas" {...baseProps} />);
    // Solo ventas está en el DOM; finanzas (30+ comandos) ni se monta.
    expect(container.textContent).not.toMatch(/Métricas|Punto de equilibrio/i);
  });
});
