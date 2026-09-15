// ═══════════════════════════════════════════════════════════════════════════
// TURNO-EXTRA · GEOMETRÍA — Cálculo de las barras (puro, testeable).
// Regla anti-fantasma (#23): el extra post solo existe si la entrada fue
// antes o durante el turno; sin login o con login tardío no hay verde.
// Tests: src/test/turno-extra.test.tsx (10 casos).
// ═══════════════════════════════════════════════════════════════════════════

import type { BarraTurno, MiTurno } from "./tipos";

const UMBRAL_TEMPRANO = 15;

export const minsDe = (t: string): number => {
  const p = t.split(":").map(Number);
  return (p[0] || 0) * 60 + (p[1] || 0);
};

export const fmtHM = (mins: number): string =>
  `${String(Math.floor(mins / 60) % 24).padStart(2, "0")}:${String(Math.round(mins % 60)).padStart(2, "0")}`;

/** Minutos del corte Z si es DE HOY ("YYYY-MM-DD HH:MM:SS"); si es de
    otro día o viene roto, null (= aún no cierra hoy). */
function minutosDeCorteZ(ultimoZ: string | null, ahora: Date): number | null {
  if (!ultimoZ || ultimoZ.length < 16) return null;
  const pad = (n: number) => String(n).padStart(2, "0");
  const hoyISO = `${ahora.getFullYear()}-${pad(ahora.getMonth() + 1)}-${pad(ahora.getDate())}`;
  if (ultimoZ.slice(0, 10) !== hoyISO) return null;
  const h = Number(ultimoZ.slice(11, 13));
  const m = Number(ultimoZ.slice(14, 16));
  if (!Number.isFinite(h) || !Number.isFinite(m)) return null;
  return h * 60 + m;
}

/** Calcula toda la geometría de las barras para un momento dado. */
export function geometriaBarra(turno: MiTurno | null, ahora: Date): BarraTurno | null {
  if (!turno?.dia_laborable || turno.bloques_hoy.length === 0) return null;
  const inicio = minsDe(turno.bloques_hoy[0].hora_inicio);
  let fin = minsDe(turno.bloques_hoy[turno.bloques_hoy.length - 1].hora_fin);
  if (fin <= inicio) fin += 24 * 60; // nocturno cruza medianoche
  const ahoraMins = ahora.getHours() * 60 + ahora.getMinutes();

  // LLEGADA TEMPRANA: <15 min antes = solo felicitación; >=15 min cuenta
  // como tiempo extra (el patrón lo pidió expresamente).
  const loginRaw = turno.primer_login ? minsDe(turno.primer_login) : null;
  const llegoTemprano = loginRaw !== null && loginRaw < inicio;
  const minutosTemprano = llegoTemprano ? inicio - (loginRaw as number) : 0;
  const extraTemprana = llegoTemprano && minutosTemprano > UMBRAL_TEMPRANO;

  const enTurnoActivo = loginRaw !== null && ahoraMins >= loginRaw;

  // Fin real de actividad: si ya hizo corte Z hoy, TODO congela ahí
  // (números y barras). Sin Z, el fin es el reloj.
  const zMin = minutosDeCorteZ(turno.ultimo_corte_z, ahora);
  const freezeMin = zMin === null ? ahoraMins : Math.min(ahoraMins, zMin);

  // #23 REGLA B: el extra post existe si estaba trabajando cuando
  // terminó su horario (entrada <= fin); y si entró DE NOCHE tras el
  // fin, toda su presencia hasta el Z cuenta (login → freeze).
  // Sin login no hay nada.
  const fueraDeTurno = loginRaw !== null && loginRaw > fin;
  const enExtraPost = loginRaw !== null && !fueraDeTurno && ahoraMins > fin;
  const extraPostMin = enExtraPost ? Math.max(0, freezeMin - fin) : 0;
  const extraNocturnoMin =
    fueraDeTurno && loginRaw !== null ? Math.max(0, freezeMin - loginRaw) : 0;
  // Extra pre-turno: llegó ≥15 min antes y sigue aquí. Solo cuenta lo
  // que pasa la cortesía (8:44 con entrada 9:00 = 1 min, no 16).
  const extraPreMin = extraTemprana && enTurnoActivo
    ? Math.max(0, Math.min(ahoraMins, inicio) - (loginRaw as number) - UMBRAL_TEMPRANO)
    : 0;
  const extraTotalMin = extraPreMin + extraPostMin + extraNocturnoMin;
  const enExtra = extraTotalMin > 0;

  // Ventana visible (compacta, topbar e historial): desde llegada
  // tempranera (si aplica) hasta max(fin, freeze). Tras el Z se congela.
  const ventanaIni = extraTemprana && loginRaw !== null ? loginRaw : inicio;
  const ventanaFin = Math.max(fin, freezeMin);
  const span = Math.max(1, ventanaFin - ventanaIni);
  const pct = (m: number) => Math.min(100, Math.max(0, ((m - ventanaIni) / span) * 100));

  // Trabajo real: arranca cuando llega (no pinta negro lo no trabajado),
  // en fueraDeTurno no pinta nada, y SIN LOGIN NO HAY PROGRESO (#23:
  // el reloj solo no es trabajo).
  const hayLogin = loginRaw !== null;
  const baseTrabajo = hayLogin ? Math.max(inicio, loginRaw as number) : inicio;
  const topeTrabajo = !hayLogin || fueraDeTurno ? inicio : Math.min(Math.max(ahoraMins, baseTrabajo), fin);

  // Barra 1 (turno): si llegó extra-temprano la ventana se extiende
  // a su llegada para que la bolita se recorra hacia atrás; si no,
  // ventana oficial [inicio, fin]. Los extremos visibles son siempre
  // el horario del admin.
  const tIni = extraTemprana && loginRaw !== null ? loginRaw : inicio;
  const spanT = Math.max(1, fin - tIni);
  const pctT = (m: number) => Math.min(100, Math.max(0, ((m - tIni) / spanT) * 100));
  // Barra 2 (extra): la ventana arranca en la llegada siempre que esta
  // quede fuera del horario (temprano o nocturno: la bolita empieza a
  // la izquierda); si no, en el inicio oficial.
  // - Termina en la entrada oficial si solo hay pre (al llegar la hora,
  //   la extra termina ahí y la normal toma el relevo).
  // - De noche termina en login+jornada (escala fija: el avance se ve
  //   pequeño y crece hasta el corte Z, nunca 100% de golpe).
  // - Con post termina en max(fin, freeze).
  const jornadaMin = Math.max(60, Math.round((turno.horas_por_dia || 0) * 60));
  const loginFuera = loginRaw !== null && (loginRaw < inicio || loginRaw > fin);
  const xIni = loginFuera ? (loginRaw as number) : inicio;
  const xFin = fueraDeTurno
    ? (loginRaw as number) + jornadaMin
    : enExtraPost
      ? Math.max(fin, freezeMin)
      : inicio;
  const spanX = Math.max(1, xFin - xIni);
  const pctX = (m: number) => Math.min(100, Math.max(0, ((m - xIni) / spanX) * 100));

  // Negro de referencia: [max(ventana, inicio), min(freeze, fin-ventana)].
  // De noche queda bajo el verde (invisible); con pre congelado da 0.
  const negroIni = Math.max(xIni, inicio);
  const negroFin = Math.min(freezeMin, xFin);
  // Verde genuino: desde el fin oficial (post) o desde la llegada (noche).
  const verdeIni = fueraDeTurno ? xIni : fin;
  const verdeAncho = Math.max(0, pctX(Math.min(freezeMin, xFin)) - pctX(verdeIni));

  return {
    inicio,
    fin,
    inicioPct: pct(inicio),
    finPct: pct(fin),
    trabajoPct: pct(topeTrabajo) - (extraTemprana ? pct(inicio) : 0),
    preExtraActivo: extraTemprana && enTurnoActivo,
    preExtraPct: extraTemprana && enTurnoActivo ? pct(inicio) - pct(loginRaw as number) : 0,
    enExtraPost,
    postExtraPct: enExtraPost ? pct(freezeMin) - pct(fin) : 0,
    enExtra,
    extraMinutos: extraTotalMin,
    extraPreMinutos: extraPreMin,
    minutosTemprano,
    llegoPuntual: llegoTemprano && !extraTemprana,
    loginPct: loginRaw !== null ? pct(loginRaw) : null,
    minutosTarde: loginRaw !== null ? Math.max(0, loginRaw - inicio) : 0,
    fueraDeTurno,
    tLoginPct: loginRaw !== null ? pctT(loginRaw) : null,
    tIniPct: pctT(inicio),
    tTrabIniPct: pctT(!hayLogin || fueraDeTurno ? tIni : baseTrabajo),
    tTrabFinPct: pctT(!hayLogin || fueraDeTurno ? tIni : Math.min(Math.max(ahoraMins, baseTrabajo), fin)),
    xLoginPct: pctX(loginRaw ?? inicio),
    xIniPct: pctX(inicio),
    xFinPct: pctX(fin),
    xNegroIzq: pctX(negroIni),
    xNegroAncho: Math.max(0, pctX(negroFin) - pctX(negroIni)),
    xVerdeIzq: pctX(verdeIni),
    xVerdeAncho: verdeAncho,
  };
}

/** Etiqueta izquierda de la barra de turno: la hora real de llegada
    SOLO si llegó extra-temprano (≥15 min) y ya está trabajando; en
    cualquier otro caso, la entrada oficial. La llegada real ya tiene
    su bolita sobre la barra, no debe suplantar el horario. */
export function etiquetaEntrada(barra: BarraTurno, primerLogin: string | null): string {
  if (barra.preExtraActivo && primerLogin) return primerLogin;
  return fmtHM(barra.inicio);
}
