// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Módulo ATAJOS (front-empleado, shell global).
// Cubre: useAtajosEmpleados — F3 dispara onCorte desde cualquier lado,
// otras teclas se ignoran, el bloqueo (modal local abierto) y el flag
// deshabilitado (modal de corte ya abierto) lo suprimen.
// ═══════════════════════════════════════════════════════════════════════════

import { describe, it, expect, vi } from "vitest";
import { render } from "@testing-library/react";
import { useAtajosEmpleados } from "../front-empleado/atajos/useAtajos";
import { TECLA_CORTE, fijarBloqueoAtajo } from "../front-empleado/atajos/atajos";

function Harness({ onCorte, deshabilitado = false }: { onCorte: () => void; deshabilitado?: boolean }) {
  useAtajosEmpleados({ onCorte, deshabilitado });
  return null;
}

const pulsar = (key: string) =>
  window.dispatchEvent(new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true }));

describe("atajos · F3 global", () => {
  it("dispara onCorte con F3", () => {
    const onCorte = vi.fn();
    render(<Harness onCorte={onCorte} />);
    pulsar("F3");
    expect(onCorte).toHaveBeenCalledTimes(1);
  });

  it("ignora otras teclas", () => {
    const onCorte = vi.fn();
    render(<Harness onCorte={onCorte} />);
    pulsar("F5");
    pulsar("Enter");
    expect(onCorte).not.toHaveBeenCalled();
  });

  it("no dispara si el atajo está bloqueado por un modal local", () => {
    const onCorte = vi.fn();
    render(<Harness onCorte={onCorte} />);
    fijarBloqueoAtajo(TECLA_CORTE, true);
    pulsar("F3");
    expect(onCorte).not.toHaveBeenCalled();
    fijarBloqueoAtajo(TECLA_CORTE, false);
    pulsar("F3");
    expect(onCorte).toHaveBeenCalledTimes(1);
  });

  it("no dispara si ya está abierto el modal de corte", () => {
    const onCorte = vi.fn();
    render(<Harness onCorte={onCorte} deshabilitado />);
    pulsar("F3");
    expect(onCorte).not.toHaveBeenCalled();
  });
});
