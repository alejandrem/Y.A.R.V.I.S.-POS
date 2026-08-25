// ═══════════════════════════════════════════════════════════════════════════
// HORARIO EMPLEADO — Tipos, constantes y cálculos puros de horarios/salario
// usados por ModalEmpleados y sus secciones visuales. Sin estado ni JSX.
// ═══════════════════════════════════════════════════════════════════════════

export interface EmpleadoEditable {
  id: number;
  nombre: string;
  estado: string;
  salario_semanal: number;
  horarios: { dias: number[]; hora_inicio: string; hora_fin: string }[];
}

export interface Bloque {
  dias: number[];
  inicio: string;
  fin: string;
}

export const DIAS = [
  { corto: "L", label: "Lunes" },
  { corto: "M", label: "Martes" },
  { corto: "X", label: "Miércoles" },
  { corto: "J", label: "Jueves" },
  { corto: "V", label: "Viernes" },
  { corto: "S", label: "Sábado" },
  { corto: "D", label: "Domingo" },
];

export const bloqueVacio = (): Bloque => ({ dias: [0, 1, 2, 3, 4], inicio: "09:00", fin: "17:00" });

export const detectTurno = (inicio: string) => {
  if (!inicio) return "";
  const h = parseInt(inicio.split(":")[0], 10);
  if (h >= 5 && h < 12) return "Matutino";
  if (h >= 12 && h < 19) return "Vespertino";
  return "Nocturno";
};

/** Suma de horas de todos los bloques; un turno que cruza medianoche cuenta como tal. */
export const calcularHorasTotales = (bloques: Bloque[]) =>
  bloques.reduce((acc, b) => {
    if (!b.inicio || !b.fin) return acc;
    const [ih, im] = b.inicio.split(":").map(Number);
    const [fh, fm] = b.fin.split(":").map(Number);
    let mins = fh * 60 + fm - (ih * 60 + im);
    if (mins <= 0) mins += 24 * 60; // turno nocturno cruza medianoche
    return acc + mins / 60;
  }, 0);
