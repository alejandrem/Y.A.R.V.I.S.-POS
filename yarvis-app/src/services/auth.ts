// ═══════════════════════════════════════════════════════════════════════════
// SERVICIO DE AUTH — Primer inicio, login por rol y cierre de sesión.
// ═══════════════════════════════════════════════════════════════════════════

import { invokeTauri } from "./tauri";
import type { HorarioEnvio } from "./empleados";

export interface AdminProfile {
  nombre: string;
  tienda: string;
  ubicacion: string | null;
  cp: string | null;
  google_email?: string | null;
  google_client_id?: string | null;
}

export const verificarSetup = () =>
  invokeTauri<boolean>("check_setup_done");

export const guardarAdmin = (name: string, store: string, pass: string) =>
  invokeTauri<string>("guardar_admin", { data: { name, store, pass } });

export const loginAdmin = (pass: string) =>
  invokeTauri<boolean>("validar_login_admin", { pass });

export const loginAdminGoogle = () =>
  invokeTauri<boolean>("validar_login_google");

export const guardarGoogleConfig = (email: string, clientId: string) =>
  invokeTauri<string>("guardar_google_config", { email, clientId });

export const obtenerAdminData = () =>
  invokeTauri<AdminProfile>("get_admin_data");

export const guardarEmpleadoInicial = (name: string, pass: string, salarioSemanal = 0, horarios: HorarioEnvio[] = []) =>
  invokeTauri<string>("guardar_empleado", { name, pass, salarioSemanal, horarios });

export const loginEmpleado = (pass: string) =>
  invokeTauri<string | null>("validar_login_empleado", { pass });

export const cerrarSesion = () =>
  invokeTauri("cerrar_sesion");
