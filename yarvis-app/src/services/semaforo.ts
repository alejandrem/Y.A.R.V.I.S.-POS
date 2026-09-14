// ═══════════════════════════════════════════════════════════════════════════
// SERVICIO SEMÁFORO — Wrappers de los comandos Tauri del automatizador de
// códigos (semaforo_verde / semaforo_rojo / semaforo_amarillo). Los
// componentes consumen estas funciones; los args viajan en camelCase
// (Tauri los mapea a snake_case del backend automáticamente).
// ═══════════════════════════════════════════════════════════════════════════

import { invoke } from "@tauri-apps/api/core";

export interface PendienteCodigo {
  id: number;
  ean: string | null;
  nombre_crudo: string;
  nombre_norm: string;
  estado: string;
  mejor_candidato_id: number | null;
  mejor_score: number | null;
  veces_visto: number;
}

export interface ConteosPendientes {
  rojo: number;
  amarillo: number;
  conflicto: number;
  resuelto: number;
}

export interface ConteosVerde {
  verdes_hoy: number;
  total_vinculos: number;
}

export interface CandidatoCodigo {
  producto_id: number;
  nombre: string;
  score: number;
  origen: string;
}

export interface SugerenciaTop {
  pendiente_id: number;
  estado: string;
  candidatos: CandidatoCodigo[];
}

export async function contarVerdeHoy(): Promise<ConteosVerde> {
  return invoke<ConteosVerde>("verde_contar_hoy");
}

export async function contarPendientes(): Promise<ConteosPendientes> {
  return invoke<ConteosPendientes>("rojo_contar_pendientes");
}

export async function listarPendientes(estado?: string, limite = 200): Promise<PendienteCodigo[]> {
  return invoke<PendienteCodigo[]>("rojo_listar_pendientes", { estado, limite });
}

export async function sugerirAmarillo(
  nombreCrudo: string,
  marca?: string,
  ean?: string,
): Promise<SugerenciaTop> {
  return invoke<SugerenciaTop>("amarillo_sugerir", { nombreCrudo, marca, ean });
}

export async function confirmarAmarillo(
  pendienteId: number,
  productoId: number,
  eanOverride?: string,
): Promise<void> {
  await invoke("amarillo_confirmar", { pendienteId, productoId, eanOverride });
}

export async function rechazarAmarillo(
  pendienteId: number,
  productoId: number,
  score: number,
): Promise<void> {
  await invoke("amarillo_rechazar", { pendienteId, productoId, score });
}

export async function ningunoAmarillo(pendienteId: number): Promise<void> {
  await invoke("amarillo_ninguno", { pendienteId });
}

export async function resolverRojoAsignando(
  pendienteId: number,
  productoId: number,
  eanOverride?: string,
): Promise<void> {
  await invoke("rojo_resolver_asignando", { pendienteId, productoId, eanOverride });
}

export async function resolverRojoAlta(
  pendienteId: number,
  nombre: string,
  codigoBarras?: string,
  categoria?: string,
  marca?: string,
  cantidad?: number,
  unidad?: string,
): Promise<number> {
  return invoke<number>("rojo_resolver_con_alta", {
    pendienteId,
    nombre,
    codigoBarras,
    categoria,
    marca,
    cantidad,
    unidad,
  });
}
