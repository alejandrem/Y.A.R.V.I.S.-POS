// ═══════════════════════════════════════════════════════════════════════════
// TEST DE REGRESIÓN — Bug #23: login después del fin del turno marcaba
// horas extra fantasma (ej: login 20:33 con salida 17:00 → +3h34 con
// 5 minutos reales). La regla: llegar después del fin es presencia fuera
// de turno, NO extra. Cubre geometriaBarra (topbar + tarjeta Mi Turno).
// ═══════════════════════════════════════════════════════════════════════════

import { describe, it, expect } from "vitest";
import { geometriaBarra, type MiTurno } from "../components/turno";

const turnoBase = (primer_login: string | null): MiTurno => ({
  dia_laborable: true,
  bloques_hoy: [{ hora_inicio: "09:00", hora_fin: "17:00" }],
  primer_login,
  horas_por_dia: 8,
  dias_semana: 6,
  ultimo_login: primer_login,
});

// 14/sep/2026 a la hora/minuto dados (solo importa HH:MM).
const ahora = (h: number, m: number) => new Date(2026, 8, 14, h, m);

describe("turno-extra · bug #23 login después del fin", () => {
  it("login 20:33 con salida 17:00 → 0 extra y fuera de turno", () => {
    const b = geometriaBarra(turnoBase("20:33"), ahora(20, 38))!;
    expect(b.enExtra).toBe(false);
    expect(b.enExtraPost).toBe(false);
    expect(b.extraMinutos).toBe(0);
    expect(b.fueraDeTurno).toBe(true);
  });

  it("sin login y reloj pasado el fin → 0 extra", () => {
    const b = geometriaBarra(turnoBase(null), ahora(20, 38))!;
    expect(b.enExtra).toBe(false);
    expect(b.extraMinutos).toBe(0);
    expect(b.fueraDeTurno).toBe(false);
  });

  it("corte Z tardío no resucita el fantasma (ultimo_login 23:00)", () => {
    const t = turnoBase("20:33");
    t.ultimo_login = "23:00";
    const b = geometriaBarra(t, ahora(23, 5))!;
    expect(b.enExtra).toBe(false);
    expect(b.extraMinutos).toBe(0);
  });
});

describe("turno-extra · extra genuino sí cuenta", () => {
  it("llegó 08:40 y sigue 18:20 → 20 pre + 80 post", () => {
    const b = geometriaBarra(turnoBase("08:40"), ahora(18, 20))!;
    expect(b.enExtra).toBe(true);
    expect(b.extraMinutos).toBe(100);
    expect(b.fueraDeTurno).toBe(false);
  });

  it("entró justo a la salida y siguió 30 min → 30 post", () => {
    const b = geometriaBarra(turnoBase("17:00"), ahora(17, 30))!;
    expect(b.enExtra).toBe(true);
    expect(b.extraMinutos).toBe(30);
  });

  it("llegó 10 min antes (< umbral) y salió a tiempo → 0 extra", () => {
    const b = geometriaBarra(turnoBase("08:50"), ahora(10, 0))!;
    expect(b.enExtra).toBe(false);
    expect(b.llegoPuntual).toBe(true);
  });

  it("llegó tarde pero dentro del turno → el negro arranca en su llegada", () => {
    const b = geometriaBarra(turnoBase("09:05"), ahora(12, 0))!;
    expect(b.enExtra).toBe(false);
    expect(b.tTrabIniPct).toBeGreaterThan(0);
    expect(b.minutosTarde).toBe(5);
  });
});
