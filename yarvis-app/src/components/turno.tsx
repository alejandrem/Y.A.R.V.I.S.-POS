// ═══════════════════════════════════════════════════════════════════════════
// TURNO — Tipos y geometría de la barra de asistencia.
// Fuente única usada por la tarjeta "Mi Turno" del perfil Y la topbar del
// dashboard, para que ambas muestren exactamente la misma información
// (versión completa vs resumida) sin divergencias.
// ═══════════════════════════════════════════════════════════════════════════

export interface BloqueHoy {
  hora_inicio: string;
  hora_fin: string;
}

/** Shape que devuelve el comando Tauri get_mi_turno. */
export interface MiTurno {
  dia_laborable: boolean;
  bloques_hoy: BloqueHoy[];
  primer_login: string | null;
  horas_por_dia: number;
  dias_semana: number;
  ultimo_login: string | null;
}

export interface BarraTurno {
  inicio: number; // minutos desde medianoche (entrada oficial)
  fin: number;
  inicioPct: number;
  finPct: number;
  /** % de trabajo dentro del horario (negro) */
  trabajoPct: number;
  /** Extra tempranero activo: llegó ≥15 min antes y ya está trabajando */
  preExtraActivo: boolean;
  preExtraPct: number;
  /** Extra post-turno: sigue después de la salida */
  enExtraPost: boolean;
  postExtraPct: number;
  enExtra: boolean;
  extraMinutos: number;
  minutosTemprano: number;
  /** Llegó antes pero ≤15 min → solo felicitación, no cuenta extra */
  llegoPuntual: boolean;
  loginPct: number | null;
  minutosTarde: number;
}

const UMBRAL_TEMPRANO = 15;

export const minsDe = (t: string): number => {
  const p = t.split(":").map(Number);
  return (p[0] || 0) * 60 + (p[1] || 0);
};

export const fmtHM = (mins: number): string =>
  `${String(Math.floor(mins / 60) % 24).padStart(2, "0")}:${String(Math.round(mins % 60)).padStart(2, "0")}`;

/** Calcula toda la geometría de la barra para un momento dado. */
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

  // Extra post-turno (seguir después de la salida)
  const enExtraPost = ahoraMins > fin;
  const extraPostMin = enExtraPost ? ahoraMins - fin : 0;
  // Extra pre-turno (llegó ≥15 min antes y ya está trabajando)
  const extraPreMin = extraTemprana && enTurnoActivo ? inicio - (loginRaw as number) : 0;
  const extraTotalMin = extraPreMin + extraPostMin;
  const enExtra = extraTotalMin > 0;

  // Ventana visible: desde llegada tempranera (si aplica) hasta max(fin, ahora)
  const ventanaIni = extraTemprana && loginRaw !== null ? loginRaw : inicio;
  const ventanaFin = Math.max(fin, ahoraMins);
  const span = Math.max(1, ventanaFin - ventanaIni);
  const pct = (m: number) => Math.min(100, Math.max(0, ((m - ventanaIni) / span) * 100));

  return {
    inicio,
    fin,
    inicioPct: pct(inicio),
    finPct: pct(fin),
    trabajoPct: pct(Math.min(Math.max(ahoraMins, inicio), fin)) - (extraTemprana ? pct(inicio) : 0),
    preExtraActivo: extraTemprana && enTurnoActivo,
    preExtraPct: extraTemprana && enTurnoActivo ? pct(inicio) - pct(loginRaw as number) : 0,
    enExtraPost,
    postExtraPct: enExtraPost ? pct(ahoraMins) - pct(fin) : 0,
    enExtra,
    extraMinutos: extraTotalMin,
    minutosTemprano,
    llegoPuntual: llegoTemprano && !extraTemprana,
    loginPct: loginRaw !== null ? pct(loginRaw) : null,
    minutosTarde: loginRaw !== null ? Math.max(0, loginRaw - inicio) : 0,
  };
}

// ═══════════════════════════════════════════════════════════════════════════
// HISTORIAL DE HORAS EXTRA — tipos + mini barra reutilizable.
// La usan el perfil del empleado y el detalle de personal del admin.
// ═══════════════════════════════════════════════════════════════════════════

export interface DiaExtra {
  fecha: string;
  dia_label: string;
  primer_login: string;
  ultimo_login: string;
  entrada_oficial: string;
  salida_oficial: string;
  extra_pre_min: number;
  extra_post_min: number;
  trabajo_min: number;
}

/** Mini barra histórica: verde pre + negro trabajo + verde post. */
export function MiniBarraDia({ d }: { d: DiaExtra }) {
  const mDe = (t: string) => {
    const p = t.split(":").map(Number);
    return p[0] * 60 + p[1];
  };
  let ini = mDe(d.entrada_oficial);
  let fin = mDe(d.salida_oficial);
  if (fin <= ini) fin += 24 * 60;
  const login = mDe(d.primer_login);
  let salida = Math.max(mDe(d.ultimo_login), login);
  if (salida <= login) salida = login + 1;
  const ventanaIni = d.extra_pre_min > 0 ? login : ini;
  const ventanaFin = Math.max(fin, salida);
  const span = Math.max(1, ventanaFin - ventanaIni);
  const pct = (m: number) => Math.min(100, Math.max(0, ((m - ventanaIni) / span) * 100));

  return (
    <div className="relative flex-1 h-2.5 bg-neutral-100 rounded-full overflow-visible min-w-[80px]">
      {d.extra_pre_min > 0 && (
        <div className="absolute inset-y-0 bg-emerald-400" style={{ left: `${pct(login)}%`, width: `${pct(ini) - pct(login)}%`, borderRadius: "999px 0 0 999px" }} />
      )}
      <div className="absolute inset-y-0 bg-neutral-950 rounded-full transition-all duration-700 ease-out" style={{ left: `${pct(ini)}%`, width: `${Math.max(0.5, pct(Math.min(salida, fin)) - pct(ini))}%` }} />
      {d.extra_post_min > 0 && (
        <div className="absolute inset-y-0 bg-emerald-500" style={{ left: `${pct(fin)}%`, width: `${pct(salida) - pct(fin)}%`, borderRadius: "0 999px 999px 0" }} />
      )}
      <div className="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-2 h-2 bg-white border-2 border-neutral-900 rounded-full shadow-sm z-10" style={{ left: `${pct(login)}%` }} title={`Entró ${d.primer_login}`} />
      {d.extra_pre_min > 0 && (
        <div className="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-2 h-2 bg-white border-2 border-neutral-900 rounded-full shadow-sm z-10" style={{ left: `${pct(ini)}%` }} title={`Entrada oficial ${d.entrada_oficial}`} />
      )}
      {d.extra_post_min > 0 && (
        <div className="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-2 h-2 bg-white border-2 border-emerald-500 rounded-full shadow-sm z-10" style={{ left: `${pct(fin)}%` }} title={`Salida oficial ${d.salida_oficial}`} />
      )}
    </div>
  );
}
