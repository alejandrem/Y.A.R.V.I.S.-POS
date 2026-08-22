// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Módulo YARVIS (adminyarvis, chat con IA).
// Cubre: carga del estado del modelo local (get_model_status) y render
// del panel sin crash. La capa de streaming usa listeners mockeados.
// ═══════════════════════════════════════════════════════════════════════════

import { describe, it, expect, beforeEach } from "vitest";
import { render, waitFor } from "@testing-library/react";
import { mockInvoke } from "./setup";
import AdminYarvis from "../front-admin/ventanas/adminyarvis/yarvis";

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockImplementation((cmd: string) => {
    if (cmd === "get_model_status") {
      return Promise.resolve({
        models: {},
        ram_libre_gb: 8,
        local_model_path: "/modelos/qwen.gguf",
        local_model_name: "qwen-1.7b.gguf",
      });
    }
    return Promise.resolve({ models: [] });
  });
});

describe("yarvis · estado del modelo", () => {
  it("consulta get_model_status al montar", async () => {
    render(<AdminYarvis active={true} />);
    await waitFor(() => expect(mockInvoke).toHaveBeenCalledWith("get_model_status"));
  });

  it("renderiza el panel sin crash", async () => {
    const { container } = render(<AdminYarvis active={true} />);
    await waitFor(() => expect(mockInvoke).toHaveBeenCalledWith("get_model_status"));
    expect(container).toBeTruthy();
  });
});
