// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Modal F8 (rejilla de cuadritos, issue #21).
// Cubre: salen los 6 cuadritos de TABLA_ATAJOS (F6 listo), click en un
// cuadrito ejecuta su acción (incluido CAJÓN), Escape cierra.
// ═══════════════════════════════════════════════════════════════════════════

import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import ModalAtajos from "../front-empleado/atajos/modal-atajos";
import { TABLA_ATAJOS } from "../front-empleado/atajos/tabla-atajos";

describe("modal-atajos · rejilla", () => {
  it("renderiza los 6 cuadritos de la tabla", () => {
    render(<ModalAtajos onClose={() => {}} onAccion={() => {}} />);
    for (const a of TABLA_ATAJOS.filter((x) => x.enMenu)) {
      expect(screen.getByText(a.etiqueta)).toBeInTheDocument();
      expect(screen.getByText(a.tecla)).toBeInTheDocument();
    }
  });

  it("click en CORTE ejecuta su acción", () => {
    const onAccion = vi.fn();
    render(<ModalAtajos onClose={() => {}} onAccion={onAccion} />);
    fireEvent.click(screen.getByText("CORTE"));
    expect(onAccion).toHaveBeenCalledWith("corte");
  });

  it("F6 (listo) está prendido y ejecuta cajon", () => {
    const onAccion = vi.fn();
    render(<ModalAtajos onClose={() => {}} onAccion={onAccion} />);
    const cajon = screen.getByText("CAJÓN").closest("button") as HTMLButtonElement;
    expect(cajon.disabled).toBe(false);
    fireEvent.click(cajon);
    expect(onAccion).toHaveBeenCalledWith("cajon");
  });

  it("Escape cierra", () => {
    const onClose = vi.fn();
    render(<ModalAtajos onClose={onClose} onAccion={() => {}} />);
    fireEvent.keyDown(window, { key: "Escape" });
    expect(onClose).toHaveBeenCalledTimes(1);
  });
});
