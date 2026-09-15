// ═══════════════════════════════════════════════════════════════════════════
// TEST DE REGRESIÓN — Bug #23: login después del fin del turno marcaba
// horas extra fantasma (ej: login 20:33 con salida 17:00 → +3h34 con
// 5 minutos reales). La regla: llegar después del fin es presencia fuera
// de turno, NO extra. Cubre geometriaBarra (topbar + tarjeta Mi Turno).
// ═══════════════════════════════════════════════════════════════════════════

import { describe, it, expect } from "vitest";
import { geometriaBarra, etiquetaEntrada, geometriaMiniBarra, type MiTurno } from "../components/turno-extra";
import type { DiaExtra } from "../components/turno-extra";

const turnoBase = (primer_login: string | null): MiTurno => ({
  dia_laborable: true,
  bloques_hoy: [{ hora_inicio: "09:00", hora_fin: "17:00" }],
  primer_login,
  horas_por_dia: 8,
  dias_semana: 6,
  ultimo_login: primer_login,
  ultimo_corte_z: null,
});

// 14/sep/2026 a la hora/minuto dados (solo importa HH:MM).
const ahora = (h: number, m: number) => new Date(2026, 8, 14, h, m);

describe("turno-extra · regla B: login nocturno tras el fin", () => {
  it("login 20:33, 5 min después → +5 (presencia real, no +3:34)", () => {
    const b = geometriaBarra(turnoBase("20:33"), ahora(20, 38))!;
    expect(b.fueraDeTurno).toBe(true);
    expect(b.enExtra).toBe(true);
    expect(b.enExtraPost).toBe(false);
    expect(b.extraMinutos).toBe(5);
  });

  it("login 21:41 y corte Z 23:00 → +79 nocturnos y barra 2 visible", () => {
    const t = turnoBase("21:41");
    t.ultimo_login = "23:00";
    const b = geometriaBarra(t, ahora(23, 5))!;
    expect(b.fueraDeTurno).toBe(true);
    expect(b.enExtra).toBe(true);
    expect(b.extraMinutos).toBe(23 * 60 + 5 - (21 * 60 + 41));
    // Ventana fija [login, login+jornada]: bolita a la izquierda y el
    // verde mide 84/480 ≈ 17.5%, no 100%.
    expect(b.xLoginPct).toBe(0);
    expect(b.xVerdeIzq).toBe(0);
    expect(b.xVerdeAncho).toBeCloseTo((84 / 480) * 100, 5);
  });

  it("tras el corte Z la barra se congela (ya no avanza con el reloj)", () => {
    const t = turnoBase("21:41");
    t.ultimo_login = "23:00";
    t.ultimo_corte_z = "2026-09-14 23:00:00";
    const b = geometriaBarra(t, ahora(23, 30))!;
    expect(b.extraMinutos).toBe(79);
    expect(b.xVerdeAncho).toBeCloseTo((79 / 480) * 100, 5);
  });

  it("pre-extra en curso termina en la entrada (ahí toma el relevo la normal)", () => {
    const b = geometriaBarra(turnoBase("08:40"), ahora(8, 50))!;
    expect(b.enExtra).toBe(false); // cortesía aún no consumida
    expect(b.xFinPct).toBe(100); // la ventana de la barra 2 llega a la entrada
  });

  it("sin login y reloj pasado el fin → 0 extra", () => {
    const b = geometriaBarra(turnoBase(null), ahora(20, 38))!;
    expect(b.enExtra).toBe(false);
    expect(b.extraMinutos).toBe(0);
    expect(b.fueraDeTurno).toBe(false);
  });

  it("sin login a media mañana → progreso en 0 aunque el reloj avance", () => {
    const b = geometriaBarra(turnoBase(null), ahora(14, 6))!;
    expect(b.trabajoPct).toBe(0);
    expect(b.tTrabIniPct).toBe(0);
    expect(b.tTrabFinPct).toBe(0);
    expect(b.enExtra).toBe(false);
  });
});

describe("turno-extra · extra genuino sí cuenta", () => {
  it("llegó 08:40 y sigue 18:20 → 5 pre (20−15) + 80 post", () => {
    const b = geometriaBarra(turnoBase("08:40"), ahora(18, 20))!;
    expect(b.enExtra).toBe(true);
    expect(b.extraPreMinutos).toBe(5);
    expect(b.extraMinutos).toBe(85);
    expect(b.fueraDeTurno).toBe(false);
  });

  it("08:44 con entrada 09:00 → 1 min extra, no 16 (y sí abre la barra 2)", () => {
    const b = geometriaBarra(turnoBase("08:44"), ahora(9, 30))!;
    expect(b.enExtra).toBe(true);
    expect(b.extraPreMinutos).toBe(1);
    expect(b.extraMinutos).toBe(1);
  });

  it("jornada previa 06:00 quedándose al turno → 165 pre (180−15)", () => {
    const b = geometriaBarra(turnoBase("06:00"), ahora(10, 0))!;
    expect(b.enExtra).toBe(true);
    expect(b.extraPreMinutos).toBe(165);
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
    expect(b.tIniPct).toBe(0);
    expect(b.minutosTarde).toBe(5);
  });

  it("vino extra-temprano (08:40) → pista extendida, bolita atrás y palito en la entrada", () => {
    const b = geometriaBarra(turnoBase("08:40"), ahora(9, 30))!;
    expect(b.preExtraActivo).toBe(true);
    expect(b.tLoginPct).toBe(0);
    expect(b.tIniPct).toBeGreaterThan(0);
    // El negro arranca justo en el palito de entrada.
    expect(b.tTrabIniPct).toBeCloseTo(b.tIniPct, 5);
  });

  it("llegó 8:45 en punto (15 min) → felicitación, sin palito ni extra", () => {
    const b = geometriaBarra(turnoBase("08:45"), ahora(9, 30))!;
    expect(b.llegoPuntual).toBe(true);
    expect(b.enExtra).toBe(false);
    expect(b.preExtraActivo).toBe(false);
    expect(b.tIniPct).toBe(0);
  });
});

describe("turno-extra · etiqueta de la barra normal", () => {
  it("muestra la entrada oficial aunque haya login tardío", () => {
    const b = geometriaBarra(turnoBase("21:00"), ahora(21, 5))!;
    expect(etiquetaEntrada(b, "21:00")).toBe("09:00");
  });

  it("muestra la hora real solo si llegó extra-temprano y trabaja", () => {
    const b = geometriaBarra(turnoBase("08:40"), ahora(9, 30))!;
    expect(etiquetaEntrada(b, "08:40")).toBe("08:40");
  });

  it("sin login muestra la oficial", () => {
    const b = geometriaBarra(turnoBase(null), ahora(10, 0))!;
    expect(etiquetaEntrada(b, null)).toBe("09:00");
  });
});

describe("turno-extra · mini del historial", () => {
  const diaBase = (primer: string, ultimo: string, pre: number, post: number): DiaExtra => ({
    fecha: "2026-09-14",
    dia_label: "Lunes",
    primer_login: primer,
    ultimo_login: ultimo,
    entrada_oficial: "09:00",
    salida_oficial: "17:00",
    extra_pre_min: pre,
    extra_post_min: post,
    trabajo_min: 0,
    en_curso: false,
  });

  it("fila nocturna: ventana fija de 8h, no 100% de golpe", () => {
    const g = geometriaMiniBarra(diaBase("21:41", "22:02", 0, 21));
    expect(g.nocturno).toBe(true);
    expect(g.ventanaIni).toBe(21 * 60 + 41);
    expect(g.ventanaFin - g.ventanaIni).toBe(480);
  });

  it("fila normal: ventana hasta la salida real", () => {
    const g = geometriaMiniBarra(diaBase("09:00", "18:00", 0, 60));
    expect(g.nocturno).toBe(false);
    expect(g.ventanaIni).toBe(9 * 60);
    expect(g.ventanaFin).toBe(18 * 60);
  });
});
