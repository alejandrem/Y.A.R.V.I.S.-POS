// ══════════════════════════════════════════════════════════════════
// TAREA: Tipos compartidos del módulo de perfil de empleado
// Define la forma de los datos que devuelve el backend
// (`get_employee_profile`) para perfil.tsx y sus componentes.
// ══════════════════════════════════════════════════════════════════

export interface EmployeeProfile {
  id: number;
  nombre: string;
  turno: string;
  horario_inicio: string;
  horario_fin: string;
  salario_diario: number;
  salario_semanal: number;
  salario_mensual: number;
  salario_hora: number;
  horas_por_dia: number;
  dias_semana: number;
  meta_mensual: number;
  bono: number;
  ultimo_login: string | null;
  estado: string;
}

export interface EmployeeGoalSummary {
  goal_type: string;
  goal_name: string | null;
  bonus_amount: number;
  bonus_percentage: number;
  ventas_threshold: string;
  is_completed: boolean;
}

export interface EmployeeProfileFull {
  profile: EmployeeProfile;
  goals: EmployeeGoalSummary[];
}
