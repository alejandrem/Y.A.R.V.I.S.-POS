// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Módulo EMPLEADOS (adminempleados).
// Cubre: ModalEmpleados en modo crear (validaciones + payload correcto),
// modo editar (datos precargados, contraseña opcional) y el flujo de
// desactivación con advertencia Aceptar/Cancelar.
// ═══════════════════════════════════════════════════════════════════════════

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { mockInvoke } from "./setup";
import ModalEmpleados from "../front-admin/ventanas/adminempleados/modalEmpleados";

const EMPLEADO = {
  id: 7,
  nombre: "Peter Parker",
  estado: "activo",
  salario_semanal: 1500,
  horarios: [
    { dias: [0, 2, 3], hora_inicio: "08:00", hora_fin: "17:00" },
    { dias: [5, 6], hora_inicio: "08:00", hora_fin: "12:00" },
  ],
};

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockResolvedValue("ok");
  vi.spyOn(window, "alert").mockImplementation(() => {});
});

// Campo no asocia htmlFor, así que localizamos el input hermano del label.
const inputDe = (label: string | RegExp): HTMLInputElement => {
  const labelEl = screen.getByText(label);
  return labelEl.parentElement!.querySelector("input") as HTMLInputElement;
};

const escribir = (label: string | RegExp, valor: string) => {
  fireEvent.change(inputDe(label), { target: { value: valor } });
};

describe("empleados · alta de empleado", () => {
  it("rechaza guardar sin nombre", () => {
    render(<ModalEmpleados onClose={() => {}} onSaved={() => {}} />);
    fireEvent.click(screen.getByText(/Registrar Empleado/i));
    expect(window.alert).toHaveBeenCalledWith(expect.stringContaining("nombre"));
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("rechaza contraseña sin números", () => {
    render(<ModalEmpleados onClose={() => {}} onSaved={() => {}} />);
    escribir(/Nombre del Empleado/, "Gwen Stacy");
    escribir(/^Contraseña$/, "solo-letras");
    fireEvent.click(screen.getByText(/Registrar Empleado/i));
    expect(window.alert).toHaveBeenCalledWith(expect.stringContaining("contraseña"));
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("rechaza contraseñas que no coinciden", () => {
    render(<ModalEmpleados onClose={() => {}} onSaved={() => {}} />);
    escribir(/Nombre del Empleado/, "Gwen Stacy");
    escribir(/^Contraseña$/, "clave123");
    escribir(/Confirmar Contraseña/, "clave456");
    fireEvent.click(screen.getByText(/Registrar Empleado/i));
    expect(window.alert).toHaveBeenCalledWith(expect.stringContaining("coinciden"));
  });

  it("guarda via guardar_empleado con pago semanal y bloques de horario", async () => {
    const onSaved = vi.fn();
    render(<ModalEmpleados onClose={() => {}} onSaved={onSaved} />);
    escribir(/Nombre del Empleado/, "Gwen Stacy");
    escribir(/^Contraseña$/, "clave123");
    escribir(/Confirmar Contraseña/, "clave123");

    // Quitar viernes del bloque #1 (viene L-V activo por defecto)
    fireEvent.click(screen.getByTitle("Viernes"));

    fireEvent.click(screen.getByText(/Registrar Empleado/i));
    await vi.waitFor(() => expect(mockInvoke).toHaveBeenCalledTimes(1));

    const [cmd, args] = mockInvoke.mock.calls[0];
    expect(cmd).toBe("guardar_empleado");
    expect(args.name).toBe("Gwen Stacy");
    expect(args.salarioSemanal).toBe(0);
    expect(args.horarios[0].dias).toEqual([0, 1, 2, 3]); // L-J (sin V)
    expect(onSaved).toHaveBeenCalled();
  });
});

describe("empleados · edición de empleado", () => {
  it("precarga nombre, pago semanal y los dos bloques de horario", () => {
    render(<ModalEmpleados onClose={() => {}} onSaved={() => {}} empleado={EMPLEADO} />);
    expect(inputDe(/Nombre del Empleado/i).value).toBe("Peter Parker");
    expect(inputDe(/Pago por semana/i).value).toBe("1500");
    expect(screen.getByText("#1")).toBeInTheDocument();
    expect(screen.getByText("#2")).toBeInTheDocument();
    // Bloque 1: L,X,J activos
    expect(screen.getAllByTitle("Lunes")[0]).toHaveClass("bg-neutral-950");
    expect(screen.getAllByTitle("Martes")[0]).not.toHaveClass("bg-neutral-950");
  });

  it("guarda via editar_empleado con contraseña null si no se toca", async () => {
    render(<ModalEmpleados onClose={() => {}} onSaved={() => {}} empleado={EMPLEADO} />);
    fireEvent.click(screen.getByText(/Guardar Cambios/i));
    await vi.waitFor(() => expect(mockInvoke).toHaveBeenCalledTimes(1));
    const [cmd, args] = mockInvoke.mock.calls[0];
    expect(cmd).toBe("editar_empleado");
    expect(args.empleadoId).toBe(7);
    expect(args.nombre).toBe("Peter Parker");
    expect(args.nuevaPassword).toBeNull();
    expect(args.horarios).toHaveLength(2);
  });

  it("los días ocupados por otro bloque aparecen deshabilitados", () => {
    render(<ModalEmpleados onClose={() => {}} onSaved={() => {}} empleado={EMPLEADO} />);
    // Bloque #1 usa [0,2,3] → Martes(1) libre; Sábado/Domingo ocupados por bloque #2
    const martesBloque1 = screen.getAllByTitle("Martes")[0];
    expect(martesBloque1).not.toBeDisabled();
    expect(screen.getAllByTitle(/^Sábado/)[0]).toBeDisabled();
  });
});

describe("empleados · desactivación con advertencia", () => {
  it("muestra la advertencia con Cancelar y no llama al backend al cancelar", () => {
    render(<ModalEmpleados onClose={() => {}} onSaved={() => {}} empleado={EMPLEADO} />);
    fireEvent.click(screen.getByText(/Desactivar Empleado/i));
    expect(screen.getByText(/¿Desactivar a Peter Parker\?/)).toBeInTheDocument();
    fireEvent.click(screen.getAllByText("Cancelar")[0]); // el de la advertencia
    expect(screen.queryByText(/¿Desactivar a Peter Parker\?/)).not.toBeInTheDocument();
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("Aceptar desactiva via set_estado_empleado con estado 'inactivo'", async () => {
    render(<ModalEmpleados onClose={() => {}} onSaved={() => {}} empleado={EMPLEADO} />);
    fireEvent.click(screen.getByText(/Desactivar Empleado/i));
    fireEvent.click(screen.getByText("Aceptar"));
    await vi.waitFor(() => expect(mockInvoke).toHaveBeenCalledTimes(1));
    const [cmd, args] = mockInvoke.mock.calls[0];
    expect(cmd).toBe("set_estado_empleado");
    expect(args.empleadoId).toBe(7);
    expect(args.estado).toBe("inactivo");
  });

  it("empleado inactivo muestra botón Reactivar en vez de Desactivar", () => {
    render(
      <ModalEmpleados
        onClose={() => {}}
        onSaved={() => {}}
        empleado={{ ...EMPLEADO, estado: "inactivo" }}
      />,
    );
    expect(screen.queryByText(/Desactivar Empleado/i)).toBeDisabled();
    expect(screen.getByText(/Reactivar Empleado/i)).toBeInTheDocument();
  });
});
