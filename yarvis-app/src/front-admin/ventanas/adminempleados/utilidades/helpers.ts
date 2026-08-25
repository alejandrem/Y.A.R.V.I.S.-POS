// ═══════════════════════════════════════════════════════════════
// Helpers puros y tipos compartidos del panel de empleados.
// Sin estado ni JSX: solo funciones de formato/detección y tipos.
// ═══════════════════════════════════════════════════════════════
import type { EmpleadoProfile } from "../../../../services/empleado";

export interface EmpleadoVentas {
  empleado_id: number;
  nombre: string;
  total_ventas: number;
  ventas_canceladas: number;
  total_canceladas_count: number;
  ventas_con_descuento: number;
  ticket_count: number;
}

export interface EmpleadoResumen {
  empleados_activos: number;
  ventas_totales: number;
  tickets_totales: number;
  costo_nomina: number;
  roi_neto: number;
}

export interface CorteEmpleado {
  id: number;
  fecha_apertura: string | null;
  fecha_cierre: string | null;
  monto_inicial: number;
  total_ventas: number;
  estado: string;
}

export const detectTurno = (horarioInicio: string) => {
  if (!horarioInicio || horarioInicio === "00:00") return "";
  const h = parseInt(horarioInicio.split(":")[0], 10);
  if (h >= 5 && h < 12) return "Matutino";
  if (h >= 12) return "Vespertino";
  return "Nocturno";
};

// Índice de chip del día actual: Lunes=0 .. Domingo=6.
export const hoyChipIdx = () => (new Date().getDay() + 6) % 7;

// Minutos desde medianoche de un horario "HH:MM".
export const minsDe = (t: string) => {
  const [h, m] = t.split(":").map(Number);
  return h * 60 + m;
};

// ¿La hora actual cae dentro del rango? Soporta turnos que cruzan medianoche.
export const enRango = (inicio: string, fin: string, ahoraMins: number) => {
  const start = minsDe(inicio);
  const end = minsDe(fin);
  if (start <= end) return ahoraMins >= start && ahoraMins <= end;
  return ahoraMins >= start || ahoraMins <= end;
};

export const isInShift = (emp: EmpleadoProfile) => {
  const ahora = new Date();
  const ahoraMins = ahora.getHours() * 60 + ahora.getMinutes();
  const hoy = hoyChipIdx();

  // Bloques completos (jornadas partidas).
  if (emp.horarios?.length) {
    return emp.horarios.some((b) => b.dias.includes(hoy) && enRango(b.hora_inicio, b.hora_fin, ahoraMins));
  }
  // Fallback legacy: rango único en columnas planas.
  if (!emp.horario_inicio || !emp.horario_fin || emp.horario_inicio === "00:00") return false;
  return enRango(emp.horario_inicio, emp.horario_fin, ahoraMins);
};

export const estadoDot = (emp: EmpleadoProfile) => {
  if (emp.estado === "inactivo") return "Inactivo";
  if (isInShift(emp)) return "En turno";
  if (emp.estado === "descanso") return "Descanso";
  return "Fuera de turno";
};

export const estadoVisual: Record<string, { dot: string; texto: string; fondo: string }> = {
  "En turno": { dot: "bg-emerald-500", texto: "text-emerald-600", fondo: "bg-emerald-50" },
  Descanso: { dot: "bg-amber-400", texto: "text-amber-600", fondo: "bg-amber-50" },
  "Fuera de turno": { dot: "bg-neutral-300", texto: "text-neutral-400", fondo: "bg-neutral-50" },
  Inactivo: { dot: "bg-red-400", texto: "text-red-500", fondo: "bg-red-50" },
};

export const formatMoney = (v: number) =>
  `$${v.toLocaleString("es-MX", { minimumFractionDigits: 2 })}`;

export const formatTime12 = (t: string) => {
  if (!t || t === "00:00") return "";
  const [h, m] = t.split(":").map(Number);
  const ampm = h >= 12 ? "PM" : "AM";
  const h12 = h % 12 || 12;
  return `${String(h12).padStart(2, "0")}:${String(m).padStart(2, "0")}${ampm}`;
};

export const formatShortDate = (d: string | null) => {
  if (!d) return "";
  const date = new Date(d);
  const hours = date.getHours();
  const ampm = hours >= 12 ? "PM" : "AM";
  const h12 = hours % 12 || 12;
  const mins = String(date.getMinutes()).padStart(2, "0");
  return `${String(h12).padStart(2, "0")}:${mins} ${ampm}`;
};

export const DIAS_CORTOS = ["L", "M", "X", "J", "V", "S", "D"];

export const formatBloques = (emp: EmpleadoProfile) => {
  if (emp.horarios?.length) {
    return emp.horarios
      .map((b) => `${b.dias.map((d) => DIAS_CORTOS[d]).join("")} ${formatTime12(b.hora_inicio)}-${formatTime12(b.hora_fin)}`)
      .join(" · ");
  }
  const hasHorario = emp.horario_inicio && emp.horario_fin && emp.horario_inicio !== "00:00";
  return hasHorario ? `${formatTime12(emp.horario_inicio)}-${formatTime12(emp.horario_fin)}` : "";
};

export const formatEntrada = (emp: EmpleadoProfile) => {
  const horario = formatBloques(emp);
  const login = emp.ultimo_login ? formatShortDate(emp.ultimo_login) : "";
  if (horario && login) return `${horario} / ${login}`;
  if (horario) return horario;
  return "Sin horario";
};

export const formatDate = (d: string | null) => {
  if (!d) return "—";
  const date = new Date(d);
  const day = String(date.getDate()).padStart(2, "0");
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const hours = date.getHours();
  const ampm = hours >= 12 ? "PM" : "AM";
  const h12 = hours % 12 || 12;
  const mins = String(date.getMinutes()).padStart(2, "0");
  return `${day}/${month} - ${String(h12).padStart(2, "0")}:${mins} ${ampm}`;
};
