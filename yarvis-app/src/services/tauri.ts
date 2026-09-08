// ═══════════════════════════════════════════════════════════════════════════
// SERVICIO BASE TAURI — Único punto de salida hacia el backend.
// Tarea única: envolver `invoke` con manejo de errores visible (toast) y
// log centralizado. Los componentes NO deben llamar `invoke` crudo ni usar
// `alert()` / `console.error` dispersos: usan `invokeTauri` + `reportarError`.
// ═══════════════════════════════════════════════════════════════════════════

import { invoke } from "@tauri-apps/api/core";
import { notificarError } from "../components/notificaciones";

/** Invoca un comando Tauri. En éxito devuelve el valor; en fallo lanza el error tal cual. */
export async function invokeTauri<T>(comando: string, args?: Record<string, unknown>): Promise<T> {
  // Sin args se invoca con un solo parámetro para no romper firmas exactas
  // (tests y comandos sin parámetros).
  return args === undefined ? invoke<T>(comando) : invoke<T>(comando, args);
}

/**
 * Reporta un error al usuario (toast rojo) y lo deja en consola en UN solo
 * lugar. Reemplaza el patrón `console.error(...); notificarError(...)`.
 */
export function reportarError(titulo: string, error: unknown): void {
  // Log centralizado: un solo console.error en todo el frontend.
  // eslint-disable-next-line no-console
  console.error(`[${titulo}]`, error);
  notificarError(titulo, error);
}

/** Descarga un texto (CSV, TXT) como archivo en el navegador/webview. */
export function descargarTexto(nombre: string, contenido: string, mime = "text/csv;charset=utf-8"): void {
  const blob = new Blob(["\uFEFF" + contenido], { type: mime });
  descargarBlob(nombre, blob);
}

/** Descarga bytes binarios (PDF) como archivo. */
export function descargarBinario(nombre: string, bytes: number[] | Uint8Array, mime: string): void {
  const blob = new Blob([new Uint8Array(bytes as unknown as number[])], { type: mime });
  descargarBlob(nombre, blob);
}

function descargarBlob(nombre: string, blob: Blob): void {
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = nombre;
  document.body.appendChild(a);
  a.click();
  a.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 5000);
}
