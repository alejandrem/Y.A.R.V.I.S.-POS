// ═══════════════════════════════════════════════════════════════════════════
// TEST — Módulo CORTES del parseador (espejo del de tickets).
// Render sin backend hasta actuar; historial + detalle con invokes
// mockeados; resumen final con datos fijos.
// ═══════════════════════════════════════════════════════════════════════════

import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { mockInvoke } from "./setup";
import Cortes from "../front-admin/ventanas/parseador/cortes/cortes";
import CompletoCortes from "../front-admin/ventanas/parseador/cortes/completo";
import type { ResumenCortes } from "../services/cortes";

const FILA = {
  id: 1, tipo: "Z", folio: "54", estacion: "ESTACION01", cajero: "SISTEMA",
  fecha: "2025-01-01 11:25:57", total_caja: 2462.8, total_ventas: 2462.8,
  clientes_atendidos: 9, verificado: true,
};

const DETALLE = {
  id: 1, tipo: "Z", folio: "54", estacion: "ESTACION01", cajero: "SISTEMA",
  empresa: "EMPRESA, S.A. DE C.V.", moneda: "MXN", fecha: "2025-01-01 11:25:57",
  total_ingresos: 2462.8, total_egresos: 0, total_caja: 2462.8, total_ventas: 2462.8,
  ventas_gravadas: 0, impuesto: 0, ventas_no_gravadas: 2462.8, redondeos: 0,
  ventas_credito: 0, total_unidades: 20, clientes_atendidos: 9,
  caja_ok: true, ventas_ok: true,
  items: [
    { kind: "ARTICULO", nombre: "JACK DNL", cantidad: 1, precio_unitario: 535, subtotal: 535 },
    { kind: "INGRESO", nombre: "EFE Pago de clientes", cantidad: null, precio_unitario: 2023.8, subtotal: 2023.8 },
  ],
};

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockImplementation((cmd: string) => {
    if (cmd === "get_cortes_importados") return Promise.resolve([FILA]);
    if (cmd === "get_corte_importado_detalle") return Promise.resolve(DETALLE);
    return Promise.resolve(null);
  });
});

describe("cortes · orquestador", () => {
  it("renderiza catálogo primero sin invocar al backend", () => {
    render(<Cortes />);
    expect(screen.getByText("Catálogo maestro")).toBeInTheDocument();
    expect(screen.getByText("Carpeta de cortes")).toBeInTheDocument();
    expect(screen.getByText("Historial")).toBeInTheDocument();
    expect(screen.getByText(/Carga tu catálogo maestro/)).toBeInTheDocument();
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("saltar catálogo lleva a la carpeta de cortes", () => {
    render(<Cortes />);
    fireEvent.click(screen.getByText(/Saltar catálogo/));
    expect(screen.getByText(/Seleccionar carpeta de cortes TXT/)).toBeInTheDocument();
  });

  it("navega al historial y abre el detalle con corte_id", async () => {
    render(<Cortes />);
    fireEvent.click(screen.getByText("Historial"));
    expect(await screen.findByText(/Corte Z #54/)).toBeInTheDocument();
    fireEvent.click(screen.getByText(/Corte Z #54/).closest("button")!);
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("get_corte_importado_detalle", { corte_id: 1 });
    });
    expect(await screen.findByText("JACK DNL")).toBeInTheDocument();
    expect(screen.getByText(/caja cuadra/)).toBeInTheDocument();
  });
});

describe("cortes · resumen final", () => {
  it("muestra X/Z/omitidos/errores y botón de reinicio", () => {
    const resumen: ResumenCortes = {
      archivos: 4, cortes_x: 1, cortes_z: 1,
      omitidos_no_corte: 1, omitidos_duplicados: 1,
      productos_vinculados: 5, productos_nuevos: 2,
      errores: ["roto.txt: el archivo no es un corte de caja X/Z"],
    };
    let reseteado = false;
    render(<CompletoCortes resumen={resumen} totalArchivos={4} onReset={() => { reseteado = true; }} />);
    expect(screen.getByText("Carpeta parseada correctamente")).toBeInTheDocument();
    expect(screen.getByText(/roto.txt/)).toBeInTheDocument();
    fireEvent.click(screen.getByText("Procesar otra carpeta"));
    expect(reseteado).toBe(true);
  });
});
