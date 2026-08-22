// ═══════════════════════════════════════════════════════════════════════════
// TEST DE ESTRÉS — Módulo FINANZAS.
// Presión: tabla de métricas con 5,000 filas, KPI grid masivo y 300 cambios
// seguidos del SelectorRango. Verifica estabilidad y tiempos de render.
// ═══════════════════════════════════════════════════════════════════════════

import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import SeccionMetricas from "../front-admin/ventanas/adminfinanzas/componentes/seccion-metricas";
import SeccionResumen from "../front-admin/ventanas/adminfinanzas/componentes/seccion-resumen";
import SelectorRango from "../front-admin/ventanas/adminfinanzas/componentes/selector-rango";

const metricas = Array.from({ length: 5000 }, (_, i) => ({
  fecha: `2026-01-${String((i % 28) + 1).padStart(2, "0")}`,
  ventas_totales: 100 + i,
  costo_ventas: 50,
  utilidad_bruta: 50 + i,
  gastos_operativos: 10,
  utilidad_operativa: 40,
  impuestos_comisiones: 0,
  utilidad_neta: i % 2 === 0 ? 40 : -40,
  margen_neto_pct: i % 2 === 0 ? 30 : -30,
}));

describe("estres finanzas · tabla de métricas masiva", () => {
  it("renderiza 5,000 filas sin crash en tiempo razonable", () => {
    const t0 = performance.now();
    const { container } = render(
      <SeccionMetricas metricas={metricas} rango={{ inicio: "2026-01-01", fin: "2026-12-31" }} onRango={() => {}} />,
    );
    const ms = performance.now() - t0;
    expect(container.querySelectorAll("tbody tr").length).toBe(5000);
    // jsdom es lento: 15s es holgado para CI; si se dispara, algo está O(n²)
    expect(ms).toBeLessThan(15000);
  });
});

describe("estres finanzas · resumen con gráficas grandes", () => {
  it("renderiza el resumen con series de 3,000 puntos sin crash", () => {
    const pl = Array.from({ length: 3000 }, (_, i) => ({
      fecha: `d${i}`,
      ingresos: 100 + i,
      gastos: 50,
      utilidad_neta: 50,
    }));
    const { container } = render(
      <SeccionResumen
        resumen={{
          periodo_inicio: "2026-01-01", periodo_fin: "2026-12-31",
          total_ventas: 1, total_costo_ventas: 0, total_utilidad_bruta: 1,
          total_gastos_operativos: 0, total_utilidad_operativa: 1,
          total_impuestos_comisiones: 0, total_utilidad_neta: 1,
          margen_promedio_pct: 1, punto_equilibrio_ventas: 0,
        }}
        puntoEq={null}
        plData={pl}
        gastosCat={[]}
        ventasGastos={[]}
        cortesZ={[]}
        predicciones={[]}
        diasPrediccion={30}
        rango={{ inicio: "2026-01-01", fin: "2026-12-31" }}
        onRango={() => {}}
        onDiasPrediccion={() => {}}
      />,
    );
    expect(container).toBeTruthy();
  });
});

describe("estres finanzas · selector de rango bajo fuego", () => {
  it("sobrevive 300 cambios consecutivos de preset manteniendo consistencia", () => {
    const onChange = vi.fn();
    let actual = { inicio: "2026-01-01", fin: "2026-06-30" };
    const { rerender } = render(<SelectorRango rango={actual} onChange={onChange} />);
    for (let i = 0; i < 300; i++) {
      fireEvent.click(screen.getByText(["7D", "30D", "3M", "6M"][i % 4]));
      actual = onChange.mock.calls[i][0];
      rerender(<SelectorRango rango={actual} onChange={onChange} />);
      const dias = Math.round((+new Date(actual.fin) - +new Date(actual.inicio)) / 86400000);
      expect([7, 30, 90, 180]).toContain(dias);
    }
    expect(onChange).toHaveBeenCalledTimes(300);
  });
});
