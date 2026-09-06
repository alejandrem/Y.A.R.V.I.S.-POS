// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Módulo PARSEADOR (parseador de tickets).
// Verifica que el panel de importación renderiza sin crash y que no toca
// al backend hasta que el usuario actúa.
// ═══════════════════════════════════════════════════════════════════════════

import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, act } from "@testing-library/react";
import { mockInvoke, mockListen } from "./setup";
import Parseador from "../front-admin/ventanas/parseador/parseador";
import { BatchProgressProvider, useBatchProgress } from "../front-admin/ventanas/parseador/tickets/batchProgress";

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockResolvedValue(undefined);
  mockListen.mockClear();
});

describe("parseador · panel de importación", () => {
  it("renderiza el panel sin crash y sin invocar al backend", () => {
    const { container } = render(<BatchProgressProvider><Parseador /></BatchProgressProvider>);
    expect(container).toBeTruthy();
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("el lote en curso sobrevive al desmontar y remontar la pantalla", async () => {
    const Probe = () => {
      const ctx = useBatchProgress();
      return (
        <>
          <button onClick={() => ctx.beginBatch({ carpeta: "/x", totalArchivos: 100, deteccion: null })}>
            empezar
          </button>
          <span data-testid="fase">{ctx.phase}:{ctx.meta?.totalArchivos ?? 0}</span>
        </>
      );
    };
    const tree = (
      <BatchProgressProvider>
        <Probe />
        <Parseador />
      </BatchProgressProvider>
    );
    const { rerender } = render(tree);
    act(() => { screen.getByText("empezar").click(); });
    expect(screen.getByTestId("fase").textContent).toBe("procesando:100");

    // "Cambia de pestaña": Parseador se desmonta, el provider no.
    rerender(<BatchProgressProvider><Probe /></BatchProgressProvider>);
    // "Vuelve": fase y meta siguen ahí, y Parseador remontado no crashea.
    rerender(tree);
    expect(screen.getByTestId("fase").textContent).toBe("procesando:100");
  });

  it("el evento complete del backend cierra el lote aunque la pantalla se haya remontado", async () => {
    const Fase = () => {
      const ctx = useBatchProgress();
      return <span data-testid="fase2">{ctx.phase}</span>;
    };
    render(<BatchProgressProvider><Fase /></BatchProgressProvider>);
    // El provider registra UN solo listener global.
    expect(mockListen).toHaveBeenCalledTimes(1);
    expect(mockListen.mock.calls[0][0]).toBe("batch-progress");
    const handler = mockListen.mock.calls[0][1] as (e: { payload: unknown }) => void;
    // Inicia y simula progreso + complete del backend.
    await act(async () => {
      handler({ payload: { type: "progress", procesados: 50, total: 100, exitosos: 50, errores: 0 } });
    });
    await act(async () => {
      handler({ payload: { type: "complete", procesados: 100, total: 100, exitosos: 100, errores: 0 } });
    });
    expect(screen.getByTestId("fase2").textContent).toBe("completo");
  });
});
