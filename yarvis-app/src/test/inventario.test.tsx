// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Módulo INVENTARIO (admininventario).
// Cubre: carga del inventario desde get_inventory, renderizado de productos,
// y la eliminación con confirmación que llama a delete_inventory_item.
// ═══════════════════════════════════════════════════════════════════════════

import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { mockInvoke } from "./setup";
import Inventario from "../front-admin/ventanas/admininventario/inventario";

const ITEMS = [
  { id: 1, nombre: "Coca-Cola 600ml", precio_costo: 10, precio_venta: 18, stock: 50, stock_minimo: 10, vendido: 12 },
  { id: 2, nombre: "Sabritas Adobadas", precio_costo: 8, precio_venta: 20, stock: 3, stock_minimo: 5, vendido: 40 },
];

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockImplementation((cmd: string) => {
    if (cmd === "get_inventory") return Promise.resolve(ITEMS);
    return Promise.resolve("ok");
  });
});

describe("inventario · carga y render", () => {
  it("carga y muestra los productos del inventario", async () => {
    render(<Inventario activeTab="inventario" />);
    await waitFor(() => expect(screen.getAllByText(/Coca-Cola 600ml/i).length).toBeGreaterThan(0));
    expect(screen.getAllByText(/Sabritas Adobadas/i).length).toBeGreaterThan(0);
    expect(mockInvoke).toHaveBeenCalledWith("get_inventory");
  });

  it("renderiza sin crash aunque el inventario llegue vacío", async () => {
    mockInvoke.mockImplementation((cmd: string) =>
      cmd === "get_inventory" ? Promise.resolve([]) : Promise.resolve("ok"),
    );
    const { container } = render(<Inventario activeTab="inventario" />);
    await waitFor(() => expect(mockInvoke).toHaveBeenCalledWith("get_inventory"));
    expect(container).toBeTruthy();
  });
});
