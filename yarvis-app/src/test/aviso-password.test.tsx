// TEST — Aviso de contraseña predeterminada (empleados auto-creados).
// El backend dice si el operador logueado sigue con pass débil; el banner
// se muestra cada login y NO se puede ocultar: persiste hasta que el admin
// la cambie en Empleados (apaga password_defecto).
import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
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
    expect(await screen.findByText(/Contraseña predeterminada/i)).toBeInTheDocument();
    expect(screen.getByText(/pídele a tu administrador/i)).toBeInTheDocument();
    expect(mockInvoke).toHaveBeenCalledWith("aviso_password_defecto");
  });

  it("no sale si ya la cambiaron", async () => {
    mockInvoke.mockResolvedValue(false);
    const { container } = render(<AvisoPasswordDefecto />);
    await waitFor(() => expect(mockInvoke).toHaveBeenCalled());
    expect(container.textContent).toBe("");
  });

  it("no se puede ocultar: no hay botón Entendido", async () => {
    mockInvoke.mockResolvedValue(true);
    render(<AvisoPasswordDefecto />);
    await screen.findByText(/Contraseña predeterminada/i);
    expect(screen.queryByText("Entendido")).toBeNull();
    expect(screen.getByText(/no se puede ocultar/i)).toBeInTheDocument();
  });
});
