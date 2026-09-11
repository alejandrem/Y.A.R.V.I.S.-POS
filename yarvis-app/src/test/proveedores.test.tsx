// ═══════════════════════════════════════════════════════════════════════════
// TEST — Módulo PROVEEDORES del empleado (pantalla + modales).
// Render con invokes mockeados; alta con validación; flujo completo
// de compra (buscar → renglón → confirmar → factura).
// ═══════════════════════════════════════════════════════════════════════════

import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { mockInvoke } from "./setup";
import Proveedores from "../front-empleado/ventanas/empleaproveedores/proveedores";

const ITEM = {
  id: 7, nombre: "Coca-Cola 600", descripcion: null, precio_costo: 10,
  precio_venta: 18, stock: 50, stock_minimo: 5, vendido: 0,
  codigo_barras: "750123", categoria: "Refrescos",
};

const FILA = {
  id: 12, proveedor: "Don Chuy", fecha: "2026-09-10 10:32:00",
  pagado: 216, sugerido: 216, metodo_pago: "efectivo",
  items: 1, movimiento_pendiente: false, rectifica_a: null, rectificada: false,
};

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockImplementation((cmd: string) => {
    if (cmd === "listar_proveedores")
      return Promise.resolve([{ id: 3, nombre: "Don Chuy", telefono: null, correo: null, total_compras: 1, total_pagado: 216 }]);
    if (cmd === "historial_compras") return Promise.resolve([FILA]);
    if (cmd === "guardar_proveedor") return Promise.resolve(5);
    if (cmd === "crear_proveedor_generico")
      return Promise.resolve({ id: 9, nombre: "MOSTRADOR 00001", telefono: null, correo: null, total_compras: 0, total_pagado: 0 });
    if (cmd === "get_inventory") return Promise.resolve([ITEM]);
    if (cmd === "sugerir_pago") return Promise.resolve({ sugerido: 120, precio_costo: 10 });
    if (cmd === "registrar_compra")
      return Promise.resolve({ compra_id: 12, sugerido: 120, pagado: 120, movimiento_id: 4, movimiento_pendiente: false });
    if (cmd === "get_compra_detalle")
      return Promise.resolve({
        id: 12, proveedor: "Don Chuy", fecha: "2026-09-10 10:32:00",
        pagado: 120, sugerido: 120, metodo_pago: "efectivo", comentario: null, movimiento_id: 4,
        rectifica_a: null, rectificada_por: [],
        items: [{ nombre: "Coca-Cola 600", presentacion: "unidad", cantidad: 12, precio_sugerido: 10, producto_id: 7 }],
      });
    if (cmd === "rectificar_compra")
      return Promise.resolve({ compra_id: 13, sugerido: 60, pagado: 60, movimiento_id: 5, movimiento_pendiente: false });
    if (cmd === "listar_impresoras") return Promise.resolve([]);
    if (cmd === "get_tienda_info") return Promise.resolve({ nombre: "Mi Tienda", ubicacion: null, cp: null });
    return Promise.resolve(null);
  });
});

describe("proveedores · pantalla", () => {
  it("muestra botones gordos e historial", async () => {
    render(<Proveedores activeTab="proveedores" />);
    expect(screen.getByText("Proveedores")).toBeInTheDocument();
    expect(screen.getByText("Agregar proveedor")).toBeInTheDocument();
    expect(screen.getByText("Pagar al proveedor")).toBeInTheDocument();
    // Don Chuy sale dos veces: en la lista de proveedores y en el historial.
    expect((await screen.findAllByText(/Don Chuy/)).length).toBeGreaterThanOrEqual(2);
    expect(screen.getByText("Mis proveedores")).toBeInTheDocument();
  });

  it("no renderiza fuera de su pestaña", () => {
    const { container } = render(<Proveedores activeTab="perfil" />);
    expect(container.innerHTML).toBe("");
  });
});

describe("proveedores · alta", () => {
  it("pide el nombre y guarda", async () => {
    render(<Proveedores activeTab="proveedores" />);
    fireEvent.click(screen.getByText("Agregar proveedor"));
    // Sin nombre no invoca al backend.
    fireEvent.click(screen.getAllByText("Guardar")[0]);
    expect(mockInvoke).not.toHaveBeenCalledWith("guardar_proveedor", expect.anything());
    fireEvent.change(screen.getByPlaceholderText("Don Chuy"), { target: { value: "Nuevo" } });
    fireEvent.click(screen.getAllByText("Guardar")[0]);
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("guardar_proveedor", { nombre: "Nuevo", telefono: null, correo: null });
    });
  });
});

describe("proveedores · editar renglón", () => {
  it("el lápiz carga al editor y guardar reemplaza la cantidad", async () => {
    render(<Proveedores activeTab="proveedores" />);
    fireEvent.click(screen.getByText("Pagar al proveedor"));

    const input = screen.getByPlaceholderText(/Escanea o busca/);
    fireEvent.change(input, { target: { value: "coca" } });
    expect(await screen.findByText("Coca-Cola 600")).toBeInTheDocument();
    fireEvent.click(screen.getByText("Coca-Cola 600"));
    fireEvent.click(screen.getByText("+ Agregar"));
    expect(await screen.findByText(/1 unidad/)).toBeInTheDocument();

    // Lápiz → el editor se llena y el botón se vuelve Guardar cambios.
    fireEvent.click(screen.getByLabelText("Editar renglón"));
    expect(screen.getByText("✓ Guardar cambios")).toBeInTheDocument();
    // Cant. es el único number visible del editor (monto va después).
    const cant = screen.getByDisplayValue("1");
    fireEvent.change(cant, { target: { value: "5" } });
    fireEvent.click(screen.getByText("✓ Guardar cambios"));
    expect(await screen.findByText(/5 unidad/)).toBeInTheDocument();

    fireEvent.click(screen.getByText("Confirmar"));
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith(
        "registrar_compra",
        expect.objectContaining({
          items: [expect.objectContaining({ cantidad: 5 })],
        }),
      );
    });
  });

  it("el bote quita con contorno negro", async () => {
    render(<Proveedores activeTab="proveedores" />);
    fireEvent.click(screen.getByText("Pagar al proveedor"));
    const input = screen.getByPlaceholderText(/Escanea o busca/);
    fireEvent.change(input, { target: { value: "coca" } });
    expect(await screen.findByText("Coca-Cola 600")).toBeInTheDocument();
    fireEvent.click(screen.getByText("Coca-Cola 600"));
    fireEvent.click(screen.getByText("+ Agregar"));
    expect(await screen.findByText(/1 unidad/)).toBeInTheDocument();
    fireEvent.click(screen.getByLabelText("Quitar renglón"));
    await waitFor(() => {
      expect(screen.queryByText(/1 unidad/)).not.toBeInTheDocument();
    });
  });
});

describe("proveedores · rectificar", () => {
  it("editar precarga bloqueada y confirma rectificativa (sin borrar)", async () => {
    render(<Proveedores activeTab="proveedores" />);
    // Abrir factura #12 desde el historial.
    const fila = await screen.findByText(/Don Chuy · \$216/);
    fireEvent.click(fila.closest("button")!);
    expect(await screen.findByText("Factura de compra")).toBeInTheDocument();
    // No existe botón de borrar en ningún lado.
    expect(screen.queryByText(/Eliminar/i)).not.toBeInTheDocument();
    expect(screen.queryByLabelText(/borrar/i)).not.toBeInTheDocument();

    // Editar → modal precargado con proveedor bloqueado.
    fireEvent.click(screen.getByText(/Editar/));
    expect(await screen.findByText(/Rectificando factura #12/)).toBeInTheDocument();
    expect(screen.getByText("Guardar rectificación")).toBeInTheDocument();
    // Cantidades originales intactas: renglón y monto precargados.
    expect(screen.getByText(/12 unidad/)).toBeInTheDocument();
    expect((screen.getByPlaceholderText("0.00") as HTMLInputElement).value).toBe("120.00");

    fireEvent.click(screen.getByText("Guardar rectificación"));
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith(
        "rectificar_compra",
        expect.objectContaining({ compraOriginalId: 12 }),
      );
    });
    // Nueva factura visible.
    expect(await screen.findByText("Factura de compra")).toBeInTheDocument();
  });

  it("el lápiz del historial abre rectificativa directo", async () => {
    render(<Proveedores activeTab="proveedores" />);
    const lapiz = await screen.findByLabelText("Editar factura #12");
    expect(lapiz.className).toMatch(/bg-neutral-950/);
    fireEvent.click(lapiz);
    // Sin pasar por la factura: modal precargado con proveedor bloqueado.
    expect(await screen.findByText(/Rectificando factura #12/)).toBeInTheDocument();
    expect(mockInvoke).toHaveBeenCalledWith("get_compra_detalle", { compraId: 12 });
  });

  it("paquete legacy sin desglose se edita con total intacto", async () => {
    // Factura vieja: paquete guardado sin piezas/paquetes (NULL).
    const detalleLegacy = {
      id: 7, proveedor: "Don Chuy", fecha: "2026-09-01 10:00:00",
      pagado: 100, sugerido: 0, metodo_pago: "efectivo", comentario: null, movimiento_id: null,
      rectifica_a: null, rectificada_por: [],
      items: [{ nombre: "Galleta", presentacion: "paquete", cantidad: 24, precio_sugerido: 0, producto_id: null, piezas_por_paquete: null, paquetes: null }],
    };
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "listar_proveedores") return Promise.resolve([]);
      if (cmd === "historial_compras")
        return Promise.resolve([
          { id: 7, proveedor: "Don Chuy", fecha: "2026-09-01", pagado: 100, sugerido: 0, metodo_pago: "efectivo", items: 1, movimiento_pendiente: true, rectifica_a: null, rectificada: false },
        ]);
      if (cmd === "get_compra_detalle") return Promise.resolve(detalleLegacy);
      if (cmd === "rectificar_compra")
        return Promise.resolve({ compra_id: 8, sugerido: 0, pagado: 100, movimiento_id: null, movimiento_pendiente: true });
      return Promise.resolve(null);
    });
    render(<Proveedores activeTab="proveedores" />);
    const fila = await screen.findByText(/Don Chuy · \$100/);
    fireEvent.click(fila.closest("button")!);
    expect(await screen.findByText("Factura de compra")).toBeInTheDocument();
    fireEvent.click(screen.getByText(/Editar/));
    // El renglón legacy muestra el total intacto (24 × 1), no vacío.
    expect(await screen.findByText(/1 paq × 24 pzas = 24 unidades/)).toBeInTheDocument();
    // Lápiz: los inputs vienen rellenos, no en blanco.
    fireEvent.click(screen.getByLabelText("Editar renglón"));
    expect((screen.getByPlaceholderText("12") as HTMLInputElement).value).toBe("24");
    expect((screen.getByPlaceholderText("3") as HTMLInputElement).value).toBe("1");
    // Guardar cambios no se bloquea y rectifica con el total intacto.
    fireEvent.click(screen.getByText("✓ Guardar cambios"));
    fireEvent.click(screen.getByText("Guardar rectificación"));
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith(
        "rectificar_compra",
        expect.objectContaining({
          items: [expect.objectContaining({ cantidad: 24, piezasPorPaquete: 24, paquetes: 1 })],
        }),
      );
    });
  });
});

describe("proveedores · paquete", () => {
  it("pide piezas y paquetes, muestra el total vivo y lo manda", async () => {
    render(<Proveedores activeTab="proveedores" />);
    fireEvent.click(screen.getByText("Pagar al proveedor"));

    const input = screen.getByPlaceholderText(/Escanea o busca/);
    fireEvent.change(input, { target: { value: "coca" } });
    expect(await screen.findByText("Coca-Cola 600")).toBeInTheDocument();
    fireEvent.click(screen.getByText("Coca-Cola 600"));

    // Modo paquete: aparecen los dos inputs + total vivo.
    fireEvent.click(screen.getByText("paquete"));
    expect(screen.getByPlaceholderText("12")).toBeInTheDocument();
    expect(screen.getByPlaceholderText("3")).toBeInTheDocument();
    expect(screen.getByText("escribe piezas y paquetes")).toBeInTheDocument();
    fireEvent.change(screen.getByPlaceholderText("12"), { target: { value: "12" } });
    fireEvent.change(screen.getByPlaceholderText("3"), { target: { value: "3" } });
    expect(await screen.findByText("= 36 unidades en total")).toBeInTheDocument();

    fireEvent.click(screen.getByText("+ Agregar"));
    expect(await screen.findByText(/3 paq × 12 pzas = 36 unidades/)).toBeInTheDocument();
    fireEvent.click(screen.getByText("Confirmar"));
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith(
        "registrar_compra",
        expect.objectContaining({
          items: [
            expect.objectContaining({
              presentacion: "paquete",
              cantidad: 36,
              piezasPorPaquete: 12,
              paquetes: 3,
            }),
          ],
        }),
      );
    });
  });
});

describe("proveedores · compra", () => {
  it("flujo buscar → renglón → confirmar → factura", async () => {
    render(<Proveedores activeTab="proveedores" />);
    fireEvent.click(screen.getByText("Pagar al proveedor"));

    // Buscador embebido con la lupa de nueva_venta.
    const input = screen.getByPlaceholderText(/Escanea o busca/);
    fireEvent.change(input, { target: { value: "coca" } });
    expect(await screen.findByText("Coca-Cola 600")).toBeInTheDocument();
    fireEvent.click(screen.getByText("Coca-Cola 600"));

    // Mini menú unidad/paquete + cantidad + sugerencia.
    expect(screen.getByText("unidad")).toBeInTheDocument();
    expect(screen.getByText("paquete")).toBeInTheDocument();
    expect(await screen.findByText(/Sugerido:/)).toBeInTheDocument();
    fireEvent.click(screen.getByText("+ Agregar"));

    // Monto prellenado con la suma sugerida; sin proveedor elegido
    // se genera MOSTRADOR 00001 al confirmar y se registra con su id.
    fireEvent.click(screen.getByText("Confirmar"));
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("crear_proveedor_generico");
    });
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith(
        "registrar_compra",
        expect.objectContaining({ proveedorId: 9, montoPagado: 120 }),
      );
    });
    // Factura visible con su bloque de totales.
    expect(await screen.findByText("Factura de compra")).toBeInTheDocument();
    expect(screen.getByText("Pagado")).toBeInTheDocument();
  });
});
