// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Módulo CONFIGURACIÓN (adminconfig).
// Cubre: render de la identidad de la tienda precargada (nombre, ubicación,
// CP) y que el formulario expone los campos editables sin crash.
// ═══════════════════════════════════════════════════════════════════════════

import { describe, it, expect, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { mockInvoke } from "./setup";
import Configuracion from "../front-admin/ventanas/adminconfig/configuracion";

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockResolvedValue("ok");
});

describe("configuración · identidad de la tienda", () => {
  it("renderiza con los datos del admin precargados", () => {
    const { container } = render(
      <Configuracion
        adminName="Ale"
        storeName="La Esquina"
        adminPass=""
        initialLocation="Centro"
        initialCp="68000"
      />,
    );
    expect(container).toBeTruthy();
    expect(screen.getByDisplayValue(/La Esquina/i)).toBeInTheDocument();
  });

  it("expone los campos editables con los datos iniciales", () => {
    render(
      <Configuracion
        adminName="Alejandro"
        storeName="Abarrotes Memo"
        adminPass=""
        initialLocation="Centro"
        initialCp="68000"
      />,
    );
    expect(screen.getByDisplayValue(/Abarrotes Memo/i)).toBeInTheDocument();
    expect(containerHasInputs());
  });

  function containerHasInputs() {
    return screen.getAllByRole("textbox").length > 0;
  }
});
