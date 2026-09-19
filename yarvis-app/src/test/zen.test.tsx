// ═══════════════════════════════════════════════════════════════════════════
// TEST — Cuenta OpenCode vinculada (zen_estado/login/salir).
// Solo verifica el contrato invoke (comando + respuesta tipada); el
// OAuth real vive en el backend (zen_auth.rs, testeado en Rust).
// ═══════════════════════════════════════════════════════════════════════════

import { describe, it, expect, beforeEach } from "vitest";
import { mockInvoke } from "./setup";
import {
  obtenerEstadoZen,
  vincularCuentaZen,
  desvincularCuentaZen,
} from "../services/yarvis";

beforeEach(() => {
  mockInvoke.mockReset();
});

describe("cuenta zen · contrato invoke", () => {
  it("zen_estado pide estado sin mandar secretos", async () => {
    mockInvoke.mockResolvedValue({ vinculado: true, email: "a@b.c" });
    const estado = await obtenerEstadoZen();
    expect(mockInvoke).toHaveBeenCalledWith("zen_estado");
    expect(estado).toEqual({ vinculado: true, email: "a@b.c" });
  });

  it("zen_login devuelve el email vinculado", async () => {
    mockInvoke.mockResolvedValue("a@b.c");
    await expect(vincularCuentaZen()).resolves.toBe("a@b.c");
    expect(mockInvoke).toHaveBeenCalledWith("zen_login");
  });

  it("zen_salir desvincula", async () => {
    mockInvoke.mockResolvedValue(undefined);
    await desvincularCuentaZen();
    expect(mockInvoke).toHaveBeenCalledWith("zen_salir");
  });
});
