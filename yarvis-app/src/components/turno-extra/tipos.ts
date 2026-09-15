// ═══════════════════════════════════════════════════════════════════════════
// TURNO-EXTRA · TIPOS — Formas de la barra de asistencia y horas extra.
// Hogar único de esta lógica (antes regada en components/turno.tsx).
// Reglas (NO cambiar sin avisar al dueño):
//   · Llegar ≤15 min antes = solo felicitación, no cuenta extra.
//   · Llegar >15 min antes = ese tiempo SÍ cuenta como extra pre.
//   · Llegar DESPUÉS del fin = presencia fuera de turno, NO extra (#23).
// Backend espejo: asistencia.rs::calcular_extras (misma regla).
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
  /** Fecha_cierre del último corte Z propio ("YYYY-MM-DD HH:MM:SS").
      La barra de extra congela ahí: el turno termina con el Z. */
  ultimo_corte_z: string | null;
}

export interface BarraTurno {
  inicio: number; // minutos desde medianoche (entrada oficial)
  fin: number;
  /** Ventana compacta (topbar e historial admin) */
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
  /** Extra pre ya con la cortesía de 15 min descontada (8:44 → 1). */
  extraPreMinutos: number;
  minutosTemprano: number;
  /** Llegó antes pero ≤15 min → solo felicitación, no cuenta extra */
  llegoPuntual: boolean;
  loginPct: number | null;
  minutosTarde: number;
  /** #23 regla B: entró de noche tras el fin. Su presencia sí cuenta
      como extra (login → salida/corte Z), pero no es continuación del
      turno: la barra 2 lo pinta separado, sin negro de jornada. */
  fueraDeTurno: boolean;
  /** Barra 1 (turno normal): % sobre [llegada, fin] si vino extra-
      temprano (la bolita se recorre atrás); si no, [inicio, fin].
      Extremos visibles: siempre el horario del admin. */
  tLoginPct: number | null;
  /** Palito verde: hora de entrada oficial. */
  tIniPct: number;
  tTrabIniPct: number;
  tTrabFinPct: number;
  /** Barra 2 (extra): % sobre su ventana (ver geometria.ts).
      Solo se pinta cuando hay extra real (enExtra). */
  xLoginPct: number;
  xIniPct: number;
  xFinPct: number;
  /** Negro de referencia y verde genuino, ya calculados sobre la ventana. */
  xNegroIzq: number;
  xNegroAncho: number;
  xVerdeIzq: number;
  xVerdeAncho: number;
}

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
  /** Turno abierto: es hoy y aún no hay corte Z (ultimo_login es solo
      última actividad; la salida real la define el Z). */
  en_curso: boolean;
}
