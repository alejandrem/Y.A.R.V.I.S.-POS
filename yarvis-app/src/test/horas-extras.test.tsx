// ═══════════════════════════════════════════════════════════════════════════
// TEST FUNCIONAL — Horas extra "en curso": si el turno sigue abierto (hoy
// sin corte Z), la fila no miente una salida: muestra pastilla EN CURSO y
// el desglose dice que se define con el corte Z.
// ═══════════════════════════════════════════════════════════════════════════

import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import HorasExtras from "../front-empleado/ventanas/empleaperfil/componentes/horas-extras";
import type { DiaExtra } from "../components/turno-extra";

const dia = (en_curso: boolean): DiaExtra => ({
  fecha: "2026-09-14",
  dia_label: "Lunes",
  primer_login: "21:41",
  ultimo_login: "22:02",
  entrada_oficial: "09:00",
  salida_oficial: "17:00",
  extra_pre_min: 0,
  extra_post_min: 21,
  trabajo_min: 21,
  en_curso,
});

describe("horas-extras · turno en curso", () => {
  it("muestra pastilla EN CURSO en vez de hora de salida", () => {
    render(<HorasExtras extras={[dia(true)]} expandidas={new Set()} onToggle={() => {}} />);
    expect(screen.getByText("En curso")).toBeInTheDocument();
    expect(screen.queryByText("22:02")).not.toBeInTheDocument();
  });

  it("el desglose dice que la salida se define con el corte Z", () => {
    render(<HorasExtras extras={[dia(true)]} expandidas={new Set(["2026-09-14"])} onToggle={() => {}} />);
    expect(screen.getByText(/se define con tu corte Z/i)).toBeInTheDocument();
  });

  it("día cerrado sigue mostrando su hora de salida", () => {
    const onToggle = vi.fn();
    render(<HorasExtras extras={[dia(false)]} expandidas={new Set()} onToggle={onToggle} />);
    expect(screen.getByText("22:02")).toBeInTheDocument();
    expect(screen.queryByText("En curso")).not.toBeInTheDocument();
  });
});
