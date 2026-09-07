// TEST — Aviso de contraseña predeterminada (empleados auto-creados).
// El backend dice si el operador logueado sigue con pass débil; el banner
// se muestra cada login hasta que el admin la cambie, y "Entendido" solo
// lo oculta en la sesión actual.
import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { mockInvoke } from "./setup";
import { AvisoPasswordDefecto } from "../front-empleado/EmployeeDashboard";

beforeEach(() => {
  mockInvoke.mockReset();
});

describe("aviso password por defecto", () => {
  it("se muestra cuando el backend dice que sigue débil", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "aviso_password_defecto") return Promise.resolve(true);
      return Promise.resolve(null);
    });
    render(<AvisoPasswordDefecto />);
    expect(await screen.findByText("Contraseña predeterminada")).toBeInTheDocument();
    expect(screen.getByText(/pídele a tu administrador/i)).toBeInTheDocument();
    expect(mockInvoke).toHaveBeenCalledWith("aviso_password_defecto");
  });

  it("no sale si ya la cambiaron", async () => {
    mockInvoke.mockResolvedValue(false);
    const { container } = render(<AvisoPasswordDefecto />);
    await waitFor(() => expect(mockInvoke).toHaveBeenCalled());
    expect(container.textContent).toBe("");
  });

  it("Entendido lo oculta solo en esta sesión", async () => {
    mockInvoke.mockResolvedValue(true);
    render(<AvisoPasswordDefecto />);
    await screen.findByText("Contraseña predeterminada");
    fireEvent.click(screen.getByText("Entendido"));
    expect(screen.queryByText("Contraseña predeterminada")).toBeNull();
  });
});
