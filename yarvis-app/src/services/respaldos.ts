// ═══════════════════════════════════════════════════════════════════════════
// SERVICIO DE RESPALDOS — Única salida del frontend hacia los comandos
// de respaldo cifrado. Los nombres y formas coinciden EXACTAMENTE con
// `respaldos/comandos.rs` (snake_case, como el resto del backend).
// ═══════════════════════════════════════════════════════════════════════════

import { invokeTauri } from "./tauri";

export interface ConfigRespaldo {
  activo: boolean;
  destino: string | null;
  cada_horas: number;
  retener: number;
  ultimo_exito: string | null;
  tiene_passphrase: boolean;
}

export interface EntradaRespaldo {
  activo: boolean;
  destino: string | null;
  cada_horas: number;
  retener: number;
  passphrase?: string;
}

export interface InfoRespaldo {
  archivo: string;
  nombre: string;
  bytes: number;
  fecha: string;
}

export interface PruebaRespaldo {
  ok: boolean;
  mensaje: string;
  tablas: number;
  usuarios: number;
  ventas: number;
  productos: number;
  detalle_ventas: number;
  version_esquema: number;
}

export function leerConfigRespaldo(): Promise<ConfigRespaldo> {
  return invokeTauri<ConfigRespaldo>("respaldo_config_leer");
}

export function guardarConfigRespaldo(entrada: EntradaRespaldo): Promise<ConfigRespaldo> {
  return invokeTauri<ConfigRespaldo>("respaldo_config_guardar", { entrada });
}

export function crearRespaldoAhora(passphrase?: string): Promise<InfoRespaldo> {
  return invokeTauri<InfoRespaldo>(
    "respaldo_crear_ahora",
    passphrase ? { passphrase } : undefined,
  );
}

export function listarRespaldos(): Promise<InfoRespaldo[]> {
  return invokeTauri<InfoRespaldo[]>("respaldo_listar");
}

export function probarRespaldo(archivo: string, passphrase?: string): Promise<PruebaRespaldo> {
  return invokeTauri<PruebaRespaldo>("respaldo_probar", {
    archivo,
    passphrase: passphrase || null,
  });
}

/** Restaura y reinicia la app; normalmente esta promesa no resuelve. */
export function restaurarRespaldo(archivo: string, passphrase?: string): Promise<void> {
  return invokeTauri<void>("respaldo_restaurar", {
    archivo,
    passphrase: passphrase || null,
  });
}
