// ═══════════════════════════════════════════════════════════════════════════
// SERVICIO DE TURNO — Mi turno y horas extra del operador actual.
// Compartido por el dashboard del empleado y su perfil (misma fuente).
// ═══════════════════════════════════════════════════════════════════════════

import { invokeTauri } from "./tauri";
import type { MiTurno, DiaExtra } from "../components/turno";
import { notificarAviso } from "../components/notificaciones";

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
  /** "ok" | "reloj_atrasado" | "cambio_brusco" */
  estado: string;
  mensaje: string;
  ultimo_evento: string | null;
  evento_reloj_id: number | null;
  evento_reloj_nuevo: boolean;
}

export interface EventoRelojEmpleado {
  id: number;
  tipo: string;
  hora_sistema: string;
  hora_referencia: string | null;
  desfase_seg: number | null;
  mensaje: string;
  detectado_en: string;
  ultima_observacion: string;
  resuelto_en: string | null;
}

let ultimoEventoRelojNotificado: number | null = null;

/** Evita repetir el toast en cada sondeo mientras el mismo aviso siga activo. */
export function notificarEventoRelojSiNuevo(hora: EstadoHora) {
  if (!hora.evento_reloj_id) {
    ultimoEventoRelojNotificado = null;
    return;
  }
  if (ultimoEventoRelojNotificado === hora.evento_reloj_id) return;
  ultimoEventoRelojNotificado = hora.evento_reloj_id;
  notificarAviso("Revisa el reloj de Windows", hora.mensaje);
}

export const obtenerHoraSistema = () =>
  invokeTauri<EstadoHora>("verificar_hora_sistema");

export const obtenerEventosRelojEmpleado = (empleadoId: number) =>
  invokeTauri<EventoRelojEmpleado[]>("get_eventos_reloj_empleado", { empleadoId });

export const obtenerTiendaInfo = () =>
  invokeTauri<TiendaInfo>("get_tienda_info");
