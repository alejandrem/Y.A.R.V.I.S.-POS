// Configuración de turnos/horarios por empleado.
// Piel reconstruida con ModalShell + selector de empleado + campos de hora.
// La lógica de selección y guardado se conserva.
import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { MorphIcon } from "morphicons/react";
import { ModalShell, Campo, inputCls, ICONO_RELOJ, ICONO_CHECK, ICONO_USUARIO } from "../../../components/ui";

interface EmpleadoProfile {
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
}

interface ModalTurnosProps {
  empleados: EmpleadoProfile[];
  onClose: () => void;
  onSaved: () => void;
}

const formatTime12 = (t: string) => {
  if (!t || t === "00:00") return "Sin horario";
  const [h, m] = t.split(":").map(Number);
  const ampm = h >= 12 ? "PM" : "AM";
  const h12 = h % 12 || 12;
  return `${String(h12).padStart(2, "0")}:${String(m).padStart(2, "0")}${ampm}`;
};

const ModalTurnos = ({ empleados, onClose, onSaved }: ModalTurnosProps) => {
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [horarioInicio, setHorarioInicio] = useState("");
  const [horarioFin, setHorarioFin] = useState("");

  const selectedEmp = empleados.find((e) => e.id === selectedId);

  const handleSelect = (emp: EmpleadoProfile) => {
    setSelectedId(emp.id);
    setHorarioInicio(emp.horario_inicio || "");
    setHorarioFin(emp.horario_fin || "");
  };

  const handleSave = async () => {
    if (!selectedId || !selectedEmp) return;
    try {
      await invoke("update_empleado", {
        empleadoId: selectedId,
        nombre: selectedEmp.nombre,
        estado: selectedEmp.estado,
        turno: selectedEmp.turno,
        horarioInicio: horarioInicio || "00:00",
        horarioFin: horarioFin || "00:00",
        salarioSemanal: selectedEmp.salario_semanal,
        salarioDiario: selectedEmp.salario_diario,
        diasSemana: selectedEmp.dias_semana,
        metaMensual: selectedEmp.meta_mensual,
        bono: selectedEmp.bono,
      });
      onSaved();
      onClose();
    } catch (error) {
      console.error("Error al guardar turno:", error);
      alert("Error al guardar configuración de turno");
    }
  };

  return (
    <ModalShell icono={ICONO_RELOJ} titulo="Configurar Turnos" subtitulo="Define el horario libre de cada empleado" onClose={onClose} ancho="max-w-lg">
      <div className="space-y-4">
        <div>
          <p className="text-[10px] font-black text-neutral-500 uppercase tracking-wider ml-1 mb-2">Selecciona un empleado</p>
          <div className="max-h-44 overflow-y-auto custom-scrollbar space-y-2 pr-1">
            {empleados.map((emp) => {
              const activo = selectedId === emp.id;
              const tieneHorario = emp.horario_inicio && emp.horario_fin && emp.horario_inicio !== "00:00";
              return (
                <button
                  key={emp.id}
                  onClick={() => handleSelect(emp)}
                  className={`w-full flex items-center justify-between p-3.5 rounded-2xl border-2 transition-all ${
                    activo
                      ? "border-neutral-950 bg-neutral-950 text-neutral-50"
                      : "border-neutral-100 bg-neutral-50 hover:border-neutral-300"
                  }`}
                >
                  <span className="flex items-center gap-2.5">
                    <MorphIcon icon={ICONO_USUARIO} size={15} strokeWidth={2.2} spring="smooth" className={activo ? "text-neutral-50" : "text-neutral-400"} />
                    <span className="text-[11px] font-black uppercase">{emp.nombre}</span>
                  </span>
                  <span className={`text-[9px] font-bold uppercase tracking-widest ${activo ? "text-neutral-400" : "text-neutral-400"}`}>
                    {tieneHorario ? `${formatTime12(emp.horario_inicio!)} - ${formatTime12(emp.horario_fin!)}` : "Sin horario"}
                  </span>
                </button>
              );
            })}
          </div>
        </div>

        {selectedId && (
          <div className="animate-in slide-in-from-top-2 duration-200 space-y-4 bg-neutral-50 rounded-2xl p-4">
            <div className="grid grid-cols-2 gap-3">
              <Campo label="Hora de Entrada">
                <input type="time" value={horarioInicio} onChange={(e) => setHorarioInicio(e.target.value)} className={inputCls} />
              </Campo>
              <Campo label="Hora de Salida">
                <input type="time" value={horarioFin} onChange={(e) => setHorarioFin(e.target.value)} className={inputCls} />
              </Campo>
            </div>
            {horarioInicio && horarioFin && (
              <div className="bg-white rounded-xl p-3 text-center border border-neutral-100">
                <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest">Horario definido</p>
                <p className="text-sm font-black text-neutral-900 mt-1 uppercase">
                  {formatTime12(horarioInicio)} - {formatTime12(horarioFin)}
                </p>
              </div>
            )}
          </div>
        )}
      </div>

      <div className="pt-2 space-y-2">
        <button
          onClick={handleSave}
          disabled={!selectedId}
          className="w-full inline-flex items-center justify-center gap-2.5 py-4 rounded-xl bg-neutral-950 text-neutral-50 text-xs font-black uppercase tracking-[0.2em] hover:bg-neutral-800 transition-all shadow-xl shadow-neutral-200 disabled:opacity-30 disabled:cursor-not-allowed active:scale-[0.98]"
        >
          <MorphIcon icon={ICONO_CHECK} size={16} strokeWidth={2.5} spring="snappy" />
          Guardar Horario
        </button>
        <button
          onClick={onClose}
          className="w-full py-3 text-[10px] font-black text-neutral-400 uppercase tracking-widest hover:text-neutral-900 transition-colors"
        >
          Cancelar
        </button>
      </div>
    </ModalShell>
  );
};

export default ModalTurnos;