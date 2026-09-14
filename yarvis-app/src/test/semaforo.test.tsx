// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Pestaña CÓDIGOS en Inventario (issue #13).
// Cubre: contadores por color, tarjetas amarillas (SI/NO) y rojas
// (asignar/alta), y resolución en lote. Backend mockeado vía invoke.
// ═══════════════════════════════════════════════════════════════════════════

import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { mockInvoke } from "./setup";
import Inventario from "../front-admin/ventanas/admininventario/inventario";

const ITEMS = [
  { id: 7, nombre: "Coca-Cola 600ml", precio_costo: 10, precio_venta: 18, stock: 50, stock_minimo: 10, vendido: 12, codigo_barras: null, categoria: "refrescos" },
];

const PENDIENTES = [
  { id: 1, ean: "7501026000123", nombre_crudo: "cocacola 600", nombre_norm: "cocacola 600", estado: "amarillo", mejor_candidato_id: 7, mejor_score: 0.92, veces_visto: 3 },
  { id: 2, ean: "7509990000111", nombre_crudo: "PAN DULCE SURTIDO", nombre_norm: "pan dulce surtido", estado: "rojo", mejor_candidato_id: null, mejor_score: null, veces_visto: 1 },
];

function mockBase() {
  mockInvoke.mockImplementation((cmd: string) => {
    if (cmd === "get_inventory") return Promise.resolve(ITEMS);
    if (cmd === "rojo_contar_pendientes") return Promise.resolve({ rojo: 1, amarillo: 1, conflicto: 0, resuelto: 4 });
    if (cmd === "verde_contar_hoy") return Promise.resolve({ verdes_hoy: 34, total_vinculos: 40 });
    if (cmd === "rojo_listar_pendientes") return Promise.resolve(PENDIENTES);
    return Promise.resolve("ok");
  });
}

async function irACodigos() {
  render(<Inventario activeTab="inventario" />);
  fireEvent.click(screen.getByRole("button", { name: /códigos/i }));
  await waitFor(() => expect(mockInvoke).toHaveBeenCalledWith("rojo_listar_pendientes", expect.anything()));
}

beforeEach(() => {
  mockInvoke.mockReset();
  mockBase();
});

describe("códigos · cola semáforo", () => {
  it("muestra contadores y tarjetas por color", async () => {
    await irACodigos();
    expect(screen.getByText("34")).toBeTruthy();
    expect(screen.getByText(/amarillas por confirmar \(1\)/i)).toBeTruthy();
    expect(screen.getByText(/rojas sin match \(1\)/i)).toBeTruthy();
    expect(screen.getByText("7501026000123")).toBeTruthy();
    expect(screen.getByText("7509990000111")).toBeTruthy();
    expect(screen.getByText("Coca-Cola 600ml")).toBeTruthy();
  });

  it("SI ES ESTE confirma el candidato", async () => {
    await irACodigos();
    fireEvent.click(screen.getByRole("button", { name: /sí es este/i }));
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("amarillo_confirmar", {
        pendienteId: 1,
        productoId: 7,
        eanOverride: "7501026000123",
      }),
    );
  });

  it("NO ES abre el top-5 con NINGUNO a rojo", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "amarillo_sugerir") {
        return Promise.resolve({
          pendiente_id: 1,
          estado: "amarillo",
          candidatos: [
            { producto_id: 7, nombre: "Coca-Cola 600ml", score: 0.92, origen: "scoring" },
            { producto_id: 9, nombre: "Coca-Cola 1L", score: 0.81, origen: "scoring" },
          ],
        });
      }
      if (cmd === "get_inventory") return Promise.resolve(ITEMS);
      if (cmd === "rojo_contar_pendientes") return Promise.resolve({ rojo: 1, amarillo: 1, conflicto: 0, resuelto: 4 });
      if (cmd === "verde_contar_hoy") return Promise.resolve({ verdes_hoy: 34, total_vinculos: 40 });
      if (cmd === "rojo_listar_pendientes") return Promise.resolve(PENDIENTES);
      return Promise.resolve("ok");
    });
    await irACodigos();
    fireEvent.click(screen.getByRole("button", { name: /^no es/i }));
    await waitFor(() => expect(screen.getByText("Coca-Cola 1L")).toBeTruthy());
    expect(screen.getByRole("button", { name: /ninguno → a rojo/i })).toBeTruthy();
  });

  it("resuelve en lote las seleccionadas", async () => {
    await irACodigos();
    fireEvent.click(screen.getByLabelText("Seleccionar para lote"));
    fireEvent.click(screen.getByRole("button", { name: /confirmar lote/i }));
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("amarillo_confirmar", {
        pendienteId: 1,
        productoId: 7,
        eanOverride: "7501026000123",
      }),
    );
  });

  it("roja asigna a existente desde el buscador", async () => {
    await irACodigos();
    fireEvent.click(screen.getByRole("button", { name: /asignar a existente/i }));
    fireEvent.change(screen.getByPlaceholderText(/buscar producto/i), { target: { value: "coca" } });
    fireEvent.click(screen.getByRole("button", { name: /pegar código/i }));
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("rojo_resolver_asignando", {
        pendienteId: 2,
        productoId: 7,
        eanOverride: "7509990000111",
      }),
    );
  });
});
