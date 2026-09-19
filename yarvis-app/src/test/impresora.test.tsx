// ═══════════════════════════════════════════════════════════════════════════
// TEST — Impresoras virtuales (PDF/XPS): el POS manda ESC/POS de térmica,
// así que mandar a una virtual solo genera archivos vacíos de 0 bytes.
// Cubre: detección por nombre y bloqueo en cada servicio de impresión.
// ═══════════════════════════════════════════════════════════════════════════

import { describe, it, expect, beforeEach } from "vitest";
import { mockInvoke } from "./setup";
import {
  esImpresoraVirtual,
  imprimirTicketVenta,
  imprimirListaConciliacion,
  imprimirListaStockBajo,
  imprimirBytesRaw,
  abrirCajon,
} from "../services/impresora";

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockResolvedValue("ok");
});

describe("impresora · esImpresoraVirtual", () => {
  it("detecta las virtuales típicas de Windows", () => {
    expect(esImpresoraVirtual("Microsoft Print to PDF")).toBe(true);
    expect(esImpresoraVirtual("Microsoft Print to PDF (default)")).toBe(true);
    expect(esImpresoraVirtual("Microsoft XPS Document Writer")).toBe(true);
    expect(esImpresoraVirtual("OneNote for Windows 10")).toBe(true);
    expect(esImpresoraVirtual("Fax")).toBe(true);
  });

  it("deja pasar las térmicas reales", () => {
    expect(esImpresoraVirtual("Mi Termica 80mm")).toBe(false);
    expect(esImpresoraVirtual("XPrinter 58")).toBe(false);
    expect(esImpresoraVirtual("POS-80")).toBe(false);
    expect(esImpresoraVirtual("")).toBe(false);
  });
});

describe("impresora · bloqueo de virtuales", () => {
  const ticket = {
    tienda: "T",
    folio: "#1",
    lineas: [],
    total: 10,
    pagos: [],
  };

  it("imprimirTicketVenta frena sin tocar el backend", async () => {
    await expect(
      imprimirTicketVenta({ Spooler: { nombre: "Microsoft Print to PDF" } }, ticket),
    ).rejects.toThrow(/virtual/);
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("imprimirTicketVenta deja pasar la térmica y la red", async () => {
    await imprimirTicketVenta({ Spooler: { nombre: "Mi Termica 80mm" } }, ticket);
    await imprimirTicketVenta({ Red: { ip: "192.168.1.50", puerto: 9100 } }, ticket);
    expect(mockInvoke).toHaveBeenCalledTimes(2);
  });

  it("listas y bytes raw también frenan", async () => {
    await expect(imprimirListaConciliacion("Microsoft XPS Document Writer", [], "T", 80)).rejects.toThrow(/virtual/);
    await expect(imprimirListaStockBajo("OneNote for Windows 10", [], "T", 80)).rejects.toThrow(/virtual/);
    await expect(imprimirBytesRaw("Fax", [27, 64])).rejects.toThrow(/virtual/);
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("abrirCajon frena en spooler virtual pero no en red", async () => {
    await expect(abrirCajon({ Spooler: { nombre: "Microsoft Print to PDF" } })).rejects.toThrow(/virtual/);
    await abrirCajon({ Red: { ip: "192.168.1.50", puerto: 9100 } });
    expect(mockInvoke).toHaveBeenCalledTimes(1);
  });
});
