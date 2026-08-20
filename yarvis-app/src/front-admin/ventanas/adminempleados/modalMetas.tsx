// Definición de salarios, metas y bonos por empleado.
// Piel reconstruida con ModalShell: selector, salario con proyección en vivo,
// metas del sistema y metas personalizadas. La lógica se conserva.
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { MorphIcon } from "morphicons/react";
import { ModalShell, Campo, inputCls, ICONO_TARGET, ICONO_CHECK, ICONO_USUARIO, ICONO_DOLAR, ICONO_MAS, ICONO_PREMIO, ICONO_BORRAR, ICONO_TRENDING, ICONO_RELOJ } from "./ui";

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
  ultimo_login: string | null;
}

interface EmployeeGoal {
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

interface SalarioInfo {
  salario_diario: number;
  horas_por_dia: number;
  salario_hora: number;
  salario_semanal: number;
  salario_mensual: number;
  dias_semana: number;
}

interface ModalMetasProps {
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

const ModalMetas = ({ empleados, onClose, onSaved }: ModalMetasProps) => {
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [salarioDiario, setSalarioDiario] = useState(0);
  const [diasSemana, setDiasSemana] = useState(6);
  const [horasPorDia, setHorasPorDia] = useState(8);
  const [goals, setGoals] = useState<EmployeeGoal[]>([]);
  const [ventasThreshold, setVentasThreshold] = useState("");
  const [ventasBonusPct, setVentasBonusPct] = useState(3);
  const [puntualidadBonus, setPuntualidadBonus] = useState(100);
  const [customName, setCustomName] = useState("");
  const [customBonus, setCustomBonus] = useState(0);

  useEffect(() => {
    if (selectedId) {
      loadSalarioInfo();
      loadGoals();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedId]);

  const loadSalarioInfo = async () => {
    if (!selectedId) return;
    try {
      const info = await invoke<SalarioInfo>("get_salario_info", { empleadoId: selectedId });
      setSalarioDiario(info.salario_diario);
      setDiasSemana(info.dias_semana);
      setHorasPorDia(info.horas_por_dia);
    } catch (e) {
      console.error(e);
    }
  };

  const loadGoals = async () => {
    if (!selectedId) return;
    try {
      const g = await invoke<EmployeeGoal[]>("check_employee_goals", { empleadoId: selectedId });
      setGoals(g);

      const ventas = g.find((x) => x.goal_type === "ventas");
      if (ventas) {
        setVentasThreshold(ventas.ventas_threshold);
        setVentasBonusPct(ventas.bonus_percentage);
      } else {
        setVentasThreshold("");
        setVentasBonusPct(3);
      }

      const punt = g.find((x) => x.goal_type === "puntualidad");
      if (punt) {
        setPuntualidadBonus(punt.bonus_amount);
      }
    } catch (e) {
      console.error(e);
    }
  };

  const handleSelect = (emp: EmpleadoProfile) => {
    setSelectedId(emp.id);
  };

  const calcSalarioInfo = (diario: number, dias: number, horasPorDia: number) => {
    const semanal = diario * dias;
    const mensual = semanal * 4.33;
    const hora = horasPorDia > 0 ? diario / horasPorDia : 0;
    return { semanal, mensual, hora };
  };

  const { semanal, mensual, hora } = calcSalarioInfo(salarioDiario, diasSemana, horasPorDia);

  const ventasGoal = goals.find((g) => g.goal_type === "ventas");
  const ventasCompletada = ventasGoal?.is_completed || false;
  const umbralVenta = parseFloat(ventasThreshold) || 0;

  const puntualidadGoal = goals.find((g) => g.goal_type === "puntualidad");
  const puntualidadCompletada = puntualidadGoal?.is_completed || false;

  const customGoals = goals.filter((g) => g.goal_type === "custom");

  const handleSaveAll = async () => {
    if (!selectedId) return;
    try {
      await invoke("save_salario", {
        empleadoId: selectedId,
        salarioDiario,
        diasSemana,
      });
      await invoke("save_employee_goal", {
        empleadoId: selectedId,
        goalType: "ventas",
        goalName: null,
        ventasThreshold,
        bonusPercentage: ventasBonusPct,
        bonusAmount: 0,
      });
      await invoke("save_employee_goal", {
        empleadoId: selectedId,
        goalType: "puntualidad",
        goalName: null,
        ventasThreshold: null,
        bonusPercentage: 0,
        bonusAmount: puntualidadBonus,
      });
      onSaved();
      onClose();
    } catch (e) {
      console.error(e);
      alert("Error al guardar");
    }
  };

  const handleAddCustom = async () => {
    if (!selectedId || !customName.trim() || customBonus <= 0) return;
    try {
      await invoke("save_custom_goal", {
        empleadoId: selectedId,
        goalName: customName.trim(),
        bonusAmount: customBonus,
      });
      await loadGoals();
      setCustomName("");
      setCustomBonus(0);
    } catch (e) {
      console.error("Error guardando meta custom:", e);
    }
  };

  const handleDeleteGoal = async (goalId: number) => {
    try {
      await invoke("delete_employee_goal", { goalId });
      setGoals((prev) => prev.filter((g) => g.id !== goalId));
    } catch (e) {
      console.error(e);
    }
  };

  const selectedEmp = empleados.find((e) => e.id === selectedId);

  const proyeccion = [
    { label: "× Hora", valor: hora },
    { label: "× Día", valor: salarioDiario },
    { label: "× Semana", valor: semanal },
    { label: "× Mes", valor: mensual },
  ];

  return (
    <ModalShell icono={ICONO_TARGET} titulo="Metas y Sueldos" subtitulo="Salarios, metas del sistema y personalizadas" onClose={onClose} ancho="max-w-2xl">
      {/* SELECTOR DE EMPLEADO */}
      <div>
        <p className="text-[10px] font-black text-neutral-500 uppercase tracking-wider ml-1 mb-2">Selecciona un empleado</p>
        <div className="max-h-36 overflow-y-auto custom-scrollbar space-y-2 pr-1">
          {empleados.map((emp) => {
            const activo = selectedId === emp.id;
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
                  {emp.horario_inicio && emp.horario_fin && emp.horario_inicio !== "00:00"
                    ? `${formatTime12(emp.horario_inicio)} - ${formatTime12(emp.horario_fin)}`
                    : "Sin horario"}
                </span>
              </button>
            );
          })}
        </div>
      </div>

      {selectedId && (
        <div className="animate-in slide-in-from-top-2 duration-200 space-y-6">
          {/* SALARIO */}
          <div className="bg-neutral-50 rounded-2xl p-5 space-y-4">
            <div className="flex items-center gap-2">
              <span className="text-neutral-950">
                <MorphIcon icon={ICONO_DOLAR} size={15} strokeWidth={2.4} spring="smooth" />
              </span>
              <p className="text-[10px] font-black text-neutral-500 uppercase tracking-widest">Salario</p>
            </div>

            <div className="grid grid-cols-2 gap-3">
              <Campo label="Pago Diario ($)">
                <input
                  type="number"
                  min={0}
                  step={10}
                  value={salarioDiario || ""}
                  onChange={(e) => setSalarioDiario(Number(e.target.value))}
                  placeholder="0"
                  className={inputCls}
                />
              </Campo>
              <Campo label="Días / Semana">
                <input
                  type="number"
                  min={1}
                  max={7}
                  value={diasSemana || ""}
                  onChange={(e) => setDiasSemana(Number(e.target.value))}
                  placeholder="6"
                  className={inputCls}
                />
              </Campo>
            </div>

            <div className="grid grid-cols-2 sm:grid-cols-4 gap-2">
              {proyeccion.map((p) => (
                <div key={p.label} className="bg-white rounded-xl p-3 text-center border border-neutral-100">
                  <p className="text-[8px] font-black text-neutral-400 uppercase tracking-widest">{p.label}</p>
                  <p className="text-sm font-black text-neutral-900 mt-1">${p.valor.toFixed(2)}</p>
                </div>
              ))}
            </div>

            <p className="text-[8px] font-bold text-neutral-400 text-center">
              Basado en horario: {horasPorDia.toFixed(1)}h/día · {diasSemana} días/semana · {selectedEmp?.horario_inicio && selectedEmp.horario_inicio !== "00:00" ? `${formatTime12(selectedEmp.horario_inicio)} - ${formatTime12(selectedEmp?.horario_fin ?? "")}` : "sin turno"}
            </p>
          </div>

          {/* METAS DEL SISTEMA */}
          <div className="bg-neutral-50 rounded-2xl p-5 space-y-4">
            <div className="flex items-center gap-2">
              <span className="text-neutral-950">
                <MorphIcon icon={ICONO_PREMIO} size={15} strokeWidth={2.4} spring="smooth" />
              </span>
              <p className="text-[10px] font-black text-neutral-500 uppercase tracking-widest">Metas del Sistema</p>
            </div>

            {/* META VENTAS */}
            <div className={`rounded-xl p-4 border-2 transition-all ${ventasCompletada ? "border-emerald-400 bg-emerald-50" : "border-neutral-200 bg-white"}`}>
              <div className="flex items-center justify-between mb-3">
                <p className="flex items-center gap-2 text-[9px] font-black text-neutral-500 uppercase tracking-widest">
                  <MorphIcon icon={ICONO_TRENDING} size={14} strokeWidth={2.4} spring="smooth" />
                  Meta de Ventas
                </p>
                {ventasCompletada && (
                  <span className="inline-flex items-center gap-1 text-[9px] font-black text-emerald-600 uppercase bg-emerald-100 px-2 py-0.5 rounded-lg">
                    <MorphIcon icon={ICONO_CHECK} size={11} strokeWidth={3} spring="snappy" reducedMotion="user" />
                    Cumplida
                  </span>
                )}
              </div>
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 mb-3">
                <Campo label="Meta de Venta Semanal ($)">
                  <input value={ventasThreshold} onChange={(e) => setVentasThreshold(e.target.value)} placeholder="0" className={inputCls} />
                </Campo>
                <Campo label="Si cumple, % de lo vendido">
                  <div className="relative">
                    <input
                      type="number"
                      min={1}
                      max={10}
                      step={1}
                      value={ventasBonusPct || ""}
                      onChange={(e) => {
                        const v = Number(e.target.value);
                        if (v >= 1 && v <= 10) setVentasBonusPct(v);
                      }}
                      placeholder="3"
                      className={`${inputCls} pr-9`}
                    />
                    <span className="absolute right-3.5 top-1/2 -translate-y-1/2 text-xs font-black text-neutral-400">%</span>
                  </div>
                </Campo>
              </div>
              <div className="bg-neutral-100 rounded-xl p-3 text-center">
                <p className="text-sm font-black text-neutral-700">
                  Si vende ${umbralVenta.toFixed(2)} → bono de ${(umbralVenta * ventasBonusPct / 100).toFixed(2)}
                </p>
              </div>
            </div>

            {/* META PUNTUALIDAD */}
            <div className={`rounded-xl p-4 border-2 transition-all ${puntualidadCompletada ? "border-emerald-400 bg-emerald-50" : "border-neutral-200 bg-white"}`}>
              <div className="flex items-center justify-between mb-3">
                <p className="flex items-center gap-2 text-[9px] font-black text-neutral-500 uppercase tracking-widest">
                  <MorphIcon icon={ICONO_RELOJ} size={14} strokeWidth={2.2} spring="smooth" />
                  Meta de Puntualidad
                </p>
                {puntualidadCompletada && (
                  <span className="inline-flex items-center gap-1 text-[9px] font-black text-emerald-600 uppercase bg-emerald-100 px-2 py-0.5 rounded-lg">
                    <MorphIcon icon={ICONO_CHECK} size={11} strokeWidth={3} spring="snappy" reducedMotion="user" />
                    Cumplida
                  </span>
                )}
              </div>
              <p className="text-[9px] font-bold text-neutral-400 mb-2">Si se registra antes de 5 min del inicio de su turno</p>
              <Campo label="Bono fijo ($)">
                <input
                  type="number"
                  min={0}
                  step={10}
                  value={puntualidadBonus || ""}
                  onChange={(e) => setPuntualidadBonus(Number(e.target.value))}
                  placeholder="0"
                  className={inputCls}
                />
              </Campo>
            </div>
          </div>

          {/* METAS PERSONALIZADAS */}
          <div className="bg-neutral-50 rounded-2xl p-5 space-y-4">
            <div className="flex items-center gap-2">
              <span className="text-neutral-950">
                <MorphIcon icon={ICONO_MAS} size={15} strokeWidth={2.4} spring="smooth" />
              </span>
              <p className="text-[10px] font-black text-neutral-500 uppercase tracking-widest">Metas Personalizadas</p>
            </div>

            <div className="grid grid-cols-[1fr_100px_44px] gap-2">
              <input
                type="text"
                placeholder="Nombre de la meta"
                value={customName}
                onChange={(e) => setCustomName(e.target.value)}
                className={inputCls}
              />
              <input
                type="number"
                min={0}
                step={10}
                placeholder="Bono $"
                value={customBonus || ""}
                onChange={(e) => setCustomBonus(Number(e.target.value))}
                className={inputCls}
              />
              <button
                onClick={handleAddCustom}
                disabled={!customName.trim() || customBonus <= 0}
                className="rounded-xl bg-neutral-950 text-neutral-50 font-black hover:bg-neutral-800 transition-all disabled:opacity-30 disabled:cursor-not-allowed flex items-center justify-center active:scale-[0.95]"
              >
                <MorphIcon icon={ICONO_MAS} size={16} strokeWidth={2.5} spring="snappy" />
              </button>
            </div>

            {customGoals.length > 0 ? (
              <div className="space-y-2">
                {customGoals.map((g) => (
                  <div key={g.id} className="flex items-center justify-between bg-white rounded-xl px-4 py-3 border border-neutral-100">
                    <div className="flex items-center gap-3">
                      <span className={`w-2 h-2 rounded-full ${g.is_completed ? "bg-emerald-500" : "bg-neutral-300"}`} />
                      <span className="text-[11px] font-black text-neutral-700 uppercase">{g.goal_name}</span>
                    </div>
                    <div className="flex items-center gap-3">
                      <span className="text-xs font-black text-neutral-900">${g.bonus_amount}</span>
                      {g.is_completed && (
                        <span className="text-emerald-600 flex items-center">
                          <MorphIcon icon={ICONO_CHECK} size={11} strokeWidth={3} spring="snappy" reducedMotion="user" />
                        </span>
                      )}
                      <button
                        onClick={() => handleDeleteGoal(g.id)}
                        aria-label={`Eliminar meta ${g.goal_name}`}
                        className="text-neutral-300 hover:text-red-500 transition-colors flex items-center"
                      >
                        <MorphIcon icon={ICONO_BORRAR} size={15} strokeWidth={2.2} spring="snappy" />
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-[9px] font-bold text-neutral-400 text-center py-2 uppercase tracking-widest">No hay metas personalizadas aún</p>
            )}
          </div>
        </div>
      )}

      <div className="pt-2 space-y-2">
        <button
          onClick={handleSaveAll}
          disabled={!selectedId}
          className="w-full inline-flex items-center justify-center gap-2.5 py-4 rounded-xl bg-neutral-950 text-neutral-50 text-xs font-black uppercase tracking-[0.2em] hover:bg-neutral-800 transition-all shadow-xl shadow-neutral-200 disabled:opacity-30 disabled:cursor-not-allowed active:scale-[0.98]"
        >
          <MorphIcon icon={ICONO_CHECK} size={16} strokeWidth={2.5} spring="snappy" />
          Guardar Todo
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

export default ModalMetas;