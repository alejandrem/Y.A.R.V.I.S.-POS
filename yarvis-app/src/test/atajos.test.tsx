// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Módulo ATAJOS (front-empleado, shell global).
// Cubre: useAtajosEmpleados — F3/F5/F4 disparan su acción desde cualquier
// lado, otras teclas se ignoran, el bloqueo (modal local abierto) y el flag
// deshabilitado (modal de corte ya abierto) los suprimen. Y el registro:
// solicitar/consumir con filtro (no roba acciones ajenas).
// ═══════════════════════════════════════════════════════════════════════════

import { describe, it, expect, vi } from "vitest";
import { render } from "@testing-library/react";
import { useAtajosEmpleados } from "../front-empleado/atajos/useAtajos";
import {
  TECLA_CORTE, TECLA_COBRAR, TECLA_PAGAR, TECLA_AYUDA,
  fijarBloqueoAtajo, solicitarAccion, consumirAccion,
} from "../front-empleado/atajos/atajos";

function Harness({ onCorte, onCobrar = () => {}, onPagar = () => {}, onReimprimir = () => {}, onAyuda = () => {}, onBuscar = () => {}, deshabilitado = false }: {
  onCorte: () => void; onCobrar?: () => void; onPagar?: () => void; onReimprimir?: () => void; onAyuda?: () => void; onBuscar?: () => void; deshabilitado?: boolean;
}) {
  useAtajosEmpleados({ onCorte, onCobrar, onPagar, onReimprimir, onAyuda, onBuscar, deshabilitado });
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
    pulsar("Enter");
    pulsar("F8");
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

describe("atajos · F5 cobrar y F4 pagar", () => {
  it("F5 dispara onCobrar y F4 onPagar", () => {
    const onCorte = vi.fn();
    const onCobrar = vi.fn();
    const onPagar = vi.fn();
    render(<Harness onCorte={onCorte} onCobrar={onCobrar} onPagar={onPagar} />);
    pulsar("F5");
    pulsar("F4");
    expect(onCobrar).toHaveBeenCalledTimes(1);
    expect(onPagar).toHaveBeenCalledTimes(1);
    expect(onCorte).not.toHaveBeenCalled();
  });

  it("el bloqueo es por tecla: F5 bloqueado no frena F4", () => {
    const onCobrar = vi.fn();
    const onPagar = vi.fn();
    render(<Harness onCorte={() => {}} onCobrar={onCobrar} onPagar={onPagar} />);
    fijarBloqueoAtajo(TECLA_COBRAR, true);
    pulsar("F5");
    pulsar("F4");
    expect(onCobrar).not.toHaveBeenCalled();
    expect(onPagar).toHaveBeenCalledTimes(1);
    fijarBloqueoAtajo(TECLA_COBRAR, false);
  });

  it("deshabilitado suprime F5 y F4", () => {
    const onCobrar = vi.fn();
    const onPagar = vi.fn();
    render(<Harness onCorte={() => {}} onCobrar={onCobrar} onPagar={onPagar} deshabilitado />);
    pulsar("F5");
    pulsar("F4");
    expect(onCobrar).not.toHaveBeenCalled();
    expect(onPagar).not.toHaveBeenCalled();
  });
});

describe("atajos · F2 reimprimir", () => {
  it("F2 dispara onReimprimir", () => {
    const onReimprimir = vi.fn();
    render(<Harness onCorte={() => {}} onReimprimir={onReimprimir} />);
    pulsar("F2");
    expect(onReimprimir).toHaveBeenCalledTimes(1);
  });

  it("deshabilitado lo suprime", () => {
    const onReimprimir = vi.fn();
    render(<Harness onCorte={() => {}} onReimprimir={onReimprimir} deshabilitado />);
    pulsar("F2");
    expect(onReimprimir).not.toHaveBeenCalled();
  });
});

describe("atajos · F8 ayuda", () => {
  it("F8 dispara onAyuda", () => {
    const onAyuda = vi.fn();
    render(<Harness onCorte={() => {}} onAyuda={onAyuda} />);
    pulsar("F8");
    expect(onAyuda).toHaveBeenCalledTimes(1);
  });

  it("bloqueado o deshabilitado lo suprime", () => {
    const onAyuda = vi.fn();
    render(<Harness onCorte={() => {}} onAyuda={onAyuda} />);
    fijarBloqueoAtajo(TECLA_AYUDA, true);
    pulsar("F8");
    expect(onAyuda).not.toHaveBeenCalled();
    fijarBloqueoAtajo(TECLA_AYUDA, false);
    pulsar("F8");
    expect(onAyuda).toHaveBeenCalledTimes(1);
  });
});

describe("atajos · F7 buscar", () => {
  it("F7 dispara onBuscar", () => {
    const onBuscar = vi.fn();
    render(<Harness onCorte={() => {}} onBuscar={onBuscar} />);
    pulsar("F7");
    expect(onBuscar).toHaveBeenCalledTimes(1);
  });

  it("bloqueado o deshabilitado lo suprime", () => {
    const onBuscar = vi.fn();
    render(<Harness onCorte={() => {}} onBuscar={onBuscar} deshabilitado />);
    pulsar("F7");
    expect(onBuscar).not.toHaveBeenCalled();
  });
});

describe("atajos · acciones pendientes", () => {
  it("consumir con filtro no roba acciones ajenas", () => {
    solicitarAccion("pagar-proveedor");
    expect(consumirAccion("cobrar")).toBeNull();
    expect(consumirAccion("pagar-proveedor")).toBe("pagar-proveedor");
    expect(consumirAccion()).toBeNull();
  });

  it("F4 bloqueado por cobro abierto no pierde nada (el cobro sigue)", () => {
    const onPagar = vi.fn();
    render(<Harness onCorte={() => {}} onPagar={onPagar} />);
    fijarBloqueoAtajo(TECLA_PAGAR, true);
    pulsar("F4");
    expect(onPagar).not.toHaveBeenCalled();
    fijarBloqueoAtajo(TECLA_PAGAR, false);
  });
});
