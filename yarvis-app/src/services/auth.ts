// ═══════════════════════════════════════════════════════════════════════════
// SERVICIO DE AUTH — Primer inicio, login por rol y cierre de sesión.
// ═══════════════════════════════════════════════════════════════════════════

import { invokeTauri } from "./tauri";

export interface AdminProfile {
  nombre: string;
  tienda: string;
  ubicacion: string | null;
  cp: string | null;
}

export const verificarSetup = () =>
  invokeTauri<boolean>("check_setup_done");

export const guardarAdmin = (name: string, store: string, pass: string) =>
  invokeTauri<string>("guardar_admin", { data: { name, store, pass } });

export const loginAdmin = (pass: string) =>
  invokeTauri<boolean>("validar_login_admin", { pass });

export const obtenerAdminData = () =>
  invokeTauri<AdminProfile>("get_admin_data");

export const guardarEmpleadoInicial = (name: string, pass: string) =>
  invokeTauri<string>("guardar_empleado", { name, pass });

export const loginEmpleado = (pass: string) =>
  invokeTauri<string | null>("validar_login_empleado", { pass });

export const cerrarSesion = () =>
  invokeTauri("cerrar_sesion");
