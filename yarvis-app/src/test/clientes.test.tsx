// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Módulo CLIENTES (adminclientes).
// Placeholder en producción: verifica su aviso de construcción.
// ═══════════════════════════════════════════════════════════════════════════

import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import AdminClientes from "../front-admin/ventanas/adminclientes/clientes";

describe("clientes · placeholder", () => {
  it("anuncia que el módulo está en producción", () => {
    render(<AdminClientes />);
    expect(screen.getByText(/Gestión de clientes/i)).toBeInTheDocument();
    expect(screen.getByText(/Módulo en producción/i)).toBeInTheDocument();
  });
});
