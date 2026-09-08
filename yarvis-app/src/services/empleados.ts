// ═══════════════════════════════════════════════════════════════════════════
// SERVICIO DE EMPLEADOS (comandos) — Alta, edición, estado, metas y bonos.
// Los tipos de perfil viven en `./empleado`; aquí solo los comandos Tauri.
// ═══════════════════════════════════════════════════════════════════════════

import { invokeTauri } from "./tauri";

export interface HorarioEnvio {
  dias: number[];
  horaInicio: string;
  horaFin: string;
}

export interface MetaEmpleado {
  id: number;
  employee_id: number;
  goal_type: string;
  goal_name: string | null;
  ventas_threshold: string;
  bonus_percentage: number;
  bonus_amount: number;
  is_completed: boolean;
  completed_at: string | null;
  created_at: string | null;
}

export const guardarEmpleado = (args: { name: string; pass: string; salarioSemanal: number; horarios: HorarioEnvio[] }) =>
  invokeTauri<string>("guardar_empleado", args);

export const editarEmpleado = (args: {
  empleadoId: number;
  nombre: string;
  salarioSemanal: number;
  horarios: HorarioEnvio[];
  nuevaPassword: string | null;
}) => invokeTauri<string>("editar_empleado", args);

export const cambiarEstadoEmpleado = (empleadoId: number, estado: string) =>
  invokeTauri<string>("set_estado_empleado", { empleadoId, estado });

export const obtenerMetasEmpleado = (empleadoId: number) =>
  invokeTauri<MetaEmpleado[]>("check_employee_goals", { empleadoId });

export const guardarMetaSistema = (args: {
  empleadoId: number;
  goalType: string;
  goalName: string | null;
  ventasThreshold: string | number | null;
  bonusPercentage: number;
  bonusAmount: number;
}) => invokeTauri<string>("save_employee_goal", args);

export const guardarMetaCustom = (empleadoId: number, goalName: string, bonusAmount: number) =>
  invokeTauri<string>("save_custom_goal", { empleadoId, goalName, bonusAmount });

export const eliminarMeta = (goalId: number) =>
  invokeTauri<string>("delete_employee_goal", { goalId });
