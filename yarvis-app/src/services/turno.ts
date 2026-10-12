// ═══════════════════════════════════════════════════════════════════════════
// SERVICIO DE TURNO — Mi turno y horas extra del operador actual.
// Compartido por el dashboard del empleado y su perfil (misma fuente).
// ═══════════════════════════════════════════════════════════════════════════

import { invokeTauri } from "./tauri";
import type { MiTurno, DiaExtra } from "../components/turno";

export interface TiendaInfo {
  nombre: string | null;
  ubicacion: string | null;
  cp: string | null;
}

export const obtenerMiTurno = () =>
  invokeTauri<MiTurno>("get_mi_turno");

export const obtenerMisHorasExtra = () =>
  invokeTauri<DiaExtra[]>("get_mis_horas_extra");

export interface EstadoHora {
  hora_sistema: string;
  zona: string;
  red_disponible: boolean;
  hora_red: string | null;
  desfase_seg: number | null;
  /** "ok" | "desfasado" | "reloj_atrasado" | "sin_red" */
  estado: string;
  mensaje: string;
  ultimo_evento: string | null;
}

export const obtenerHoraSistema = () =>
  invokeTauri<EstadoHora>("verificar_hora_sistema");

export const obtenerTiendaInfo = () =>
  invokeTauri<TiendaInfo>("get_tienda_info");
