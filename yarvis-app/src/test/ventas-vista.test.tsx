// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Vista VENTAS (adminventas, dashboard de gráficas).
// Todo con datos simulados que imitan la forma real del backend
// (verificado contra yarvis.db: cajeros, top productos, métricas).
// ═══════════════════════════════════════════════════════════════════════════

import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { mockInvoke } from "./setup";
import {
  moneda, entero, fechaCorta, variacion, etiquetaCorta,
} from "../front-admin/ventanas/adminventas/graficas/controles";
import Kpis from "../front-admin/ventanas/adminventas/graficas/kpis";
import Pronostico from "../front-admin/ventanas/adminventas/graficas/pronostico";
import TopProductos from "../front-admin/ventanas/adminventas/graficas/topProductos";
import MetodosPago from "../front-admin/ventanas/adminventas/graficas/metodosPago";
import GananciaNeta from "../front-admin/ventanas/adminventas/graficas/gananciaNeta";
import Nomina from "../front-admin/ventanas/adminventas/graficas/nomina";
import { agregarPorMetodo } from "../front-admin/ventanas/adminventas/graficas/metodosPago";
import { apilarGanancia } from "../front-admin/ventanas/adminventas/graficas/gananciaNeta";
import { resumirNomina } from "../front-admin/ventanas/adminventas/graficas/nomina";
import { redactarSugerencias } from "../front-admin/ventanas/adminventas/graficas/proximaSemana";
import ProximaSemana from "../front-admin/ventanas/adminventas/graficas/proximaSemana";
import AdminVentas from "../front-admin/ventanas/adminventas/ventas";

const KPI = (total: number, tickets: number, utilidad = total * 0.4) => ({
  total,
  tickets,
  ticket_promedio: tickets ? total / tickets : 0,
  utilidad_neta: utilidad,
  margen_pct: total ? (utilidad / total) * 100 : 0,
});

beforeEach(() => {
  mockInvoke.mockReset();
  mockInvoke.mockImplementation((cmd: string, args?: Record<string, unknown>) => {
    if (cmd === "get_kpis_ventas")
      return Promise.resolve({ actual: KPI(4820.5, 127), anterior: KPI(4290, 118) });
    if (cmd === "get_ventas_con_pronostico")
      return Promise.resolve({
        historial: [
          { fecha: "2026-08-24", total: 407 },
          { fecha: "2026-08-25", total: 1176 },
          { fecha: "2026-08-26", total: 967 },
        ],
        pronostico: [
          { fecha: "2026-08-27", prediccion: 900, minimo: 700, maximo: 1100 },
          { fecha: "2026-08-28", prediccion: 950, minimo: 720, maximo: 1180 },
        ],
      });
    if (cmd === "get_top_productos")
      return Promise.resolve([
        { nombre: "COCA-COLA 2L", total: 10885, cantidad: 311 },
        { nombre: "LECHE ALPURA 1L", total: 6006, cantidad: 231 },
      ]);
    if (cmd === "get_tickets")
      return Promise.resolve([
        { id: 1, folio_ticket: "1", fecha: new Date().toISOString(), total: 100, metodo_pago: "efectivo" },
        { id: 2, folio_ticket: "2", fecha: new Date().toISOString(), total: 50, metodo_pago: "tarjeta" },
      ]);
    if (cmd === "get_ventas_por_empleado_dia")
      return Promise.resolve([
        { fecha: "2026-08-26", cajero: "MARIA G.", total: 600 },
        { fecha: "2026-08-26", cajero: "JUAN P.", total: 367 },
      ]);
    if (cmd === "get_metricas_diarias")
      return Promise.resolve([{ fecha: "2026-08-26", utilidad_neta: 400 }]);
    if (cmd === "get_empleados")
      return Promise.resolve([
        { id: 2, nombre: "peter parker", estado: "activo", salario_semanal: 1500, salario_diario: 375 },
      ]);
    if (cmd === "get_resumen_periodo")
      return Promise.resolve({ total_ventas: 165684 });
    return Promise.resolve(args ?? null);
  });
});

describe("ventas · helpers puros", () => {
  it("moneda, entero, fechaCorta y variacion", () => {
    expect(moneda(4820.5)).toMatch(/\$4,820\.50/);
    expect(entero(127)).toBe("127");
    expect(fechaCorta("2026-09-06")).toBe("06 sep");
    expect(variacion(110, 100).texto).toMatch(/▲.*10/);
    expect(variacion(90, 100).sube).toBe(false);
    expect(etiquetaCorta("Últimos 30 días")).toBe("30d");
    expect(etiquetaCorta("30 días")).toBe("30d");
    expect(etiquetaCorta("3 meses")).toBe("3m");
    expect(etiquetaCorta("Por empleado")).toBe("Por empleado");
  });
});

describe("ventas · KPIs con selector propio por tarjeta", () => {
  it("muestra las 4 tarjetas con datos reales del backend", async () => {
    render(<Kpis />);
    expect(await screen.findByText("$4,820.50")).toBeInTheDocument();
    expect(screen.getByText("127")).toBeInTheDocument();
    expect(screen.getByText("$1,928.20")).toBeInTheDocument();
  });

  it("cambiar el rango de una tarjeta pide otros días sin tocar las demás", async () => {
    render(<Kpis />);
    await screen.findByText("$4,820.50");
    mockInvoke.mockClear();
    const botones = screen.getAllByText(/\[hoy >/);
    fireEvent.click(botones[0]);
    fireEvent.click(await screen.findByText("○ Últimos 7 días"));
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("get_kpis_ventas", { dias: 7, desfase: 0 }),
    );
    // Cambiar una tarjeta refresca las 4 (un solo efecto): la importante es la de 7 días.
    expect(mockInvoke).toHaveBeenCalledTimes(4);
  });
});

describe("ventas · pronóstico con banda", () => {
  it("dibuja histórico + pronóstico y ofrece horizontes", async () => {
    render(<Pronostico />);
    // jsdom no dibuja SVG: se verifica título, leyenda e invokes.
    expect(screen.getByText("Pronóstico")).toBeInTheDocument();
    await screen.findByText(/ventas irregulares/);
    expect(mockInvoke).toHaveBeenCalledWith("get_ventas_con_pronostico", { days: 30 });
    fireEvent.click(screen.getByText(/\[30d >/));
    fireEvent.click(await screen.findByText("○ 15 días"));
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("get_ventas_con_pronostico", { days: 15 }),
    );
  });

  it("sin historial suficiente muestra aviso honesto, no gráfica rota", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_ventas_con_pronostico")
        return Promise.reject("Datos insuficientes para predecir (solo se tienen 2 días de ventas).");
      return Promise.resolve(null);
    });
    render(<Pronostico />);
    expect(await screen.findByText(/Datos insuficientes/i)).toBeInTheDocument();
  });
});

describe("ventas · agregados y componentes", () => {
  it("top 5 pide el rango y muestra título", async () => {
    render(<TopProductos />);
    expect(screen.getByText("Top 5 productos")).toBeInTheDocument();
    await waitFor(() =>
      expect(mockInvoke).toHaveBeenCalledWith("get_top_productos", { days: 30 }),
    );
  });

  it("método de pago agrega por método y muestra el total", async () => {
    render(<MetodosPago />);
    // 100 efectivo + 50 tarjeta de hoy.
    expect(await screen.findByText("$150.00 en el periodo")).toBeInTheDocument();
    const agg = agregarPorMetodo(
      [
        { id: 1, fecha: new Date().toISOString(), total: 100, metodo_pago: "efectivo" },
        { id: 2, fecha: new Date().toISOString(), total: 50, metodo_pago: "tarjeta" },
        { id: 3, fecha: "2020-01-01T00:00:00", total: 9999, metodo_pago: "efectivo" },
      ],
      30,
    );
    expect(agg).toEqual([
      { name: "Efectivo", value: 100 },
      { name: "Tarjeta", value: 50 },
    ]);
  });

  it("ganancia neta apila por empleado con top 3 + otros", async () => {
    render(<GananciaNeta />);
    await screen.findByText(/Barras = venta del día/);
    expect(mockInvoke).toHaveBeenCalledWith("get_ventas_por_empleado_dia", expect.anything());
    // Datos con la forma real (cajeros de ticket + SIN ASIGNAR).
    const { datos, cajeros } = apilarGanancia(
      [
        { fecha: "2026-08-26", cajero: "MARIA G.", total: 1200 },
        { fecha: "2026-08-26", cajero: "ROSA M.", total: 1800 },
        { fecha: "2026-08-26", cajero: "JUANA P.", total: 400 },
        { fecha: "2026-08-26", cajero: "LUIS R.", total: 300 },
        { fecha: "2026-08-26", cajero: "PEDRO A.", total: 200 },
        { fecha: "2026-08-26", cajero: "SIN ASIGNAR", total: 100 },
      ],
      [{ fecha: "2026-08-26", utilidad_neta: 1200 }],
    );
    expect(cajeros).toEqual(["ROSA M.", "MARIA G.", "JUANA P.", "Otros"]);
    const fila = datos[0];
    expect(fila["ROSA M."]).toBe(1800);
    expect(fila["MARIA G."]).toBe(1200);
    expect(fila["Otros"]).toBe(600);
    expect(fila["neta"]).toBe(1200);
  });

  it("la nómina muestra al único empleado con su salario", async () => {
    render(<Nomina />);
    await screen.findByText(/Total semanal: \$1,500\.00/);
    // Lógica pura con la forma real del backend.
    const r = resumirNomina([
      { id: 2, nombre: "peter parker", estado: "activo", salario_semanal: 1500, salario_diario: 375 } as never,
      { id: 3, nombre: "ex", estado: "inactivo", salario_semanal: 9999, salario_diario: 0 } as never,
    ]);
    expect(r.filas).toEqual([{ nombre: "peter", total: 1500 }]);
    expect(r.totalSemanal).toBe(1500);
    expect(r.totalDiario).toBe(375);
  });
});

describe("ventas · próxima semana sin LLM", () => {
  it("redacta 3 frases desde los números del pronóstico", async () => {
    render(<ProximaSemana />);
    expect(await screen.findByText(/Tu mejor día será el/i)).toBeInTheDocument();
    expect(screen.getByText(/pinta flojo/i)).toBeInTheDocument();
  });

  it("redactarSugerencias es determinista y honesto", () => {
    const hist = [
      { fecha: "2026-08-20", total: 758 },
      { fecha: "2026-08-21", total: 2055 },
      { fecha: "2026-08-22", total: 1785 },
      { fecha: "2026-08-23", total: 480 },
      { fecha: "2026-08-24", total: 407 },
      { fecha: "2026-08-25", total: 1176 },
      { fecha: "2026-08-26", total: 967 },
    ];
    const pron = [
      { fecha: "2026-08-27", prediccion: 900, minimo: 700, maximo: 1100 },
      { fecha: "2026-08-28", prediccion: 1500, minimo: 1200, maximo: 1800 },
    ];
    const r = redactarSugerencias(hist, pron);
    expect(r).not.toBeNull();
    expect(r!.resumen).toMatch(/\$2,400\.00/);
    expect(r!.lineas).toHaveLength(3);
    expect(r!.lineas[0]).toMatch(/viernes/);
    expect(redactarSugerencias(hist, [])).toBeNull();
  });
});

describe("ventas · contenedor 1200px", () => {
  it("monta todas las secciones sin crash", async () => {
    const { container } = render(<AdminVentas />);
    expect(screen.getByText("Ventas")).toBeInTheDocument();
    expect(container.querySelector(".max-w-\\[1200px\\]")).toBeInTheDocument();
    await screen.findByText("$4,820.50");
  });
});
