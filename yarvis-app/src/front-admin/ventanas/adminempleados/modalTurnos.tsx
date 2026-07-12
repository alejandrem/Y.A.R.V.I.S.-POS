import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

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
    <div
      className="fixed inset-0 bg-black/40 backdrop-blur-sm z-50 flex items-center justify-center"
      onClick={onClose}
    >
      <div
        className="bg-white rounded-[2rem] shadow-2xl w-full max-w-lg p-8 space-y-5 animate-in zoom-in-95 duration-200"
        onClick={(e) => e.stopPropagation()}
      >
        <header className="text-center">
          <h2 className="text-lg font-black text-neutral-900 uppercase">
            🕒 Configurar Turnos / Horarios
          </h2>
          <div className="h-0.5 w-8 bg-neutral-900 mx-auto mt-2 rounded-full"></div>
          <p className="text-[10px] font-bold text-neutral-400 uppercase tracking-widest mt-2">
            Define el horario libre para cada empleado
          </p>
        </header>

        <div className="max-h-[180px] overflow-y-auto custom-scrollbar space-y-2">
          {empleados.map((emp) => (
            <button
              key={emp.id}
              onClick={() => handleSelect(emp)}
              className={`w-full p-4 rounded-2xl border-2 transition-all text-left ${
                selectedId === emp.id
                  ? "border-neutral-900 bg-neutral-900 text-white"
                  : "border-neutral-100 bg-neutral-50 hover:border-neutral-300"
              }`}
            >
              <div className="flex items-center justify-between">
                <span className="text-xs font-black uppercase">{emp.nombre}</span>
                <span className={`text-[9px] font-bold ${selectedId === emp.id ? 'text-neutral-400' : 'text-neutral-400'}`}>
                  {emp.horario_inicio && emp.horario_fin && emp.horario_inicio !== "00:00"
                    ? `${emp.horario_inicio} - ${emp.horario_fin}`
                    : "Sin horario"}
                </span>
              </div>
            </button>
          ))}
        </div>

        {selectedId && (
          <div className="space-y-4 animate-in slide-in-from-top-2 duration-200">
            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-1">
                <label className="text-[10px] font-bold text-neutral-500 uppercase tracking-wider ml-1">
                  Hora de Entrada
                </label>
                <input
                  type="time"
                  value={horarioInicio}
                  onChange={(e) => setHorarioInicio(e.target.value)}
                  className="w-full px-3 py-2.5 rounded-xl bg-neutral-50 border border-neutral-200 text-sm font-bold focus:outline-none focus:border-neutral-900"
                />
              </div>
              <div className="space-y-1">
                <label className="text-[10px] font-bold text-neutral-500 uppercase tracking-wider ml-1">
                  Hora de Salida
                </label>
                <input
                  type="time"
                  value={horarioFin}
                  onChange={(e) => setHorarioFin(e.target.value)}
                  className="w-full px-3 py-2.5 rounded-xl bg-neutral-50 border border-neutral-200 text-sm font-bold focus:outline-none focus:border-neutral-900"
                />
              </div>
            </div>

            {horarioInicio && horarioFin && (
              <div className="bg-neutral-50 rounded-xl p-3 text-center">
                <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest">Horario definido</p>
                <p className="text-sm font-black text-neutral-900 mt-1">
                  {horarioInicio} - {horarioFin}
                </p>
              </div>
            )}
          </div>
        )}

        <div className="pt-2 space-y-2">
          <button
            onClick={handleSave}
            disabled={!selectedId}
            className="w-full py-4 rounded-xl bg-neutral-900 text-white text-xs font-black uppercase tracking-[0.2em] hover:bg-neutral-800 transition-all shadow-xl shadow-neutral-200 disabled:opacity-30 disabled:cursor-not-allowed"
          >
            Guardar Horario
          </button>
          <button
            onClick={onClose}
            className="w-full py-3 text-[10px] font-bold text-neutral-400 uppercase tracking-widest"
          >
            Cancelar
          </button>
        </div>
      </div>
    </div>
  );
};

export default ModalTurnos;
