// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Cajón de dinero (F6 + cobro sin ticket).
// Cubre: abrirCajonPredeterminado — usa la predeterminada del spooler,
// cae a la primera si no hay default y lanza error honesto sin
// impresoras. La tecla F6 vive en atajos.test.tsx.
// ═══════════════════════════════════════════════════════════════════════════

import { describe, it, expect, beforeEach } from "vitest";
import { mockInvoke } from "./setup";
import { abrirCajonPredeterminado } from "../services/impresora";

beforeEach(() => {
  mockInvoke.mockReset();
});

describe("cajón · abrirCajonPredeterminado", () => {
  it("usa la impresora predeterminada del spooler", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "listar_impresoras") {
        return Promise.resolve([
          { nombre: "Otra", predeterminada: false },
          { nombre: "Termica", predeterminada: true },
        ]);
      }
      if (cmd === "abrir_cajon") return Promise.resolve("Cajón abierto");
      return Promise.resolve(null);
    });
    await expect(abrirCajonPredeterminado()).resolves.toBe("Cajón abierto");
    const llamada = mockInvoke.mock.calls.find(([cmd]) => cmd === "abrir_cajon")!;
    const [, args] = llamada as [string, any];
    expect(args.destino).toEqual({ Spooler: { nombre: "Termica" } });
  });

  it("sin default usa la primera instalada", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "listar_impresoras") {
        return Promise.resolve([{ nombre: "Unica", predeterminada: false }]);
      }
      if (cmd === "abrir_cajon") return Promise.resolve("ok");
      return Promise.resolve(null);
    });
    await abrirCajonPredeterminado();
    const llamada = mockInvoke.mock.calls.find(([cmd]) => cmd === "abrir_cajon")!;
    const [, args] = llamada as [string, any];
    expect(args.destino).toEqual({ Spooler: { nombre: "Unica" } });
  });

  it("sin impresoras lanza error en lenguaje de tendero", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "listar_impresoras") return Promise.resolve([]);
      return Promise.resolve(null);
    });
    await expect(abrirCajonPredeterminado()).rejects.toThrow(/Solo hay impresoras virtuales|Sin impresoras/);
    expect(mockInvoke.mock.calls.some(([cmd]) => cmd === "abrir_cajon")).toBe(false);
  });

  it("salta la virtual aunque sea la predeterminada", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "listar_impresoras") {
        return Promise.resolve([
          { nombre: "Microsoft Print to PDF", predeterminada: true },
          { nombre: "Mi Termica 80mm", predeterminada: false },
        ]);
      }
      if (cmd === "abrir_cajon") return Promise.resolve("ok");
      return Promise.resolve(null);
    });
    await abrirCajonPredeterminado();
    const llamada = mockInvoke.mock.calls.find(([cmd]) => cmd === "abrir_cajon")!;
    const [, args] = llamada as [string, any];
    expect(args.destino).toEqual({ Spooler: { nombre: "Mi Termica 80mm" } });
  });

  it("solo virtuales lanza error aunque haya default", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "listar_impresoras") {
        return Promise.resolve([{ nombre: "Microsoft Print to PDF", predeterminada: true }]);
      }
      return Promise.resolve(null);
    });
    await expect(abrirCajonPredeterminado()).rejects.toThrow(/virtuales/);
    expect(mockInvoke.mock.calls.some(([cmd]) => cmd === "abrir_cajon")).toBe(false);
  });
});
