// ═══════════════════════════════════════════════════════════════════════════
// SERVICIO DE EMPLEADOS — Única fuente de verdad para EmpleadoProfile.
// Los componentes deben importar el tipo desde aquí, no redefinirlo.
// ═══════════════════════════════════════════════════════════════════════════

export interface HorarioBloque {
  dias: number[]; // Convención L=0 .. D=6
  hora_inicio: string;
  hora_fin: string;
}

export interface EmpleadoProfile {
  id: number;
  nombre: string;
  estado: string;
  turno: string;
  horario_inicio: string;
  horario_fin: string;
  salario_semanal: number;
  salario_diario: number;
  dias_semana: number;
  meta_mensual: number;
  bono: number;
  registrado_en: string | null;
  ultimo_login: string | null;
  horarios: HorarioBloque[];
}
