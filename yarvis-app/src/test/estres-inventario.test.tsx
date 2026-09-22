// ═══════════════════════════════════════════════════════════════════════════
// TEST DE ESTRÉS — Módulo INVENTARIO.
// Presión: catálogo de 2,000 productos cargado desde el backend simulado.
// Verifica estabilidad del render y tiempos razonables.
// ═══════════════════════════════════════════════════════════════════════════

import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { mockInvoke } from "./setup";
import Inventario from "../front-admin/ventanas/admininventario/inventario";

const CATALOGO = Array.from({ length: 2000 }, (_, i) => ({
  id: i,
  nombre: `Producto ${String(i).padStart(4, "0")}`,
  precio_costo: 5,
  precio_venta: 10,
  stock: 10,
  stock_minimo: 1,
  vendido: 0,
}));

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockImplementation((cmd: string) =>
    cmd === "get_inventory" ? Promise.resolve(CATALOGO) : Promise.resolve("ok"),
  );
});

describe("estres inventario · catálogo masivo", () => {
  it("pagina 2,000 productos de 100 en 100 sin crash", async () => {
    const t0 = performance.now();
    const { container } = render(<Inventario activeTab="inventario" />);
    await waitFor(() => expect(mockInvoke).toHaveBeenCalledWith("get_inventory"));
    // Primera página: solo 100 filas en el DOM, no 2000.
    expect(await screen.findByText("Producto 0000")).toBeInTheDocument();
    expect(container.querySelectorAll("tbody tr").length).toBeLessThanOrEqual(200);
    // El paginador anuncia el total real (2000 productos = 20 páginas).
    expect(screen.getByText(/20 páginas|2000 productos/i)).toBeInTheDocument();
    const ms = performance.now() - t0;
    expect(container).toBeTruthy();
    // jsdom es lento; 15s holgado para detectar regresiones O(n²)
    expect(ms).toBeLessThan(15000);
  });
});
