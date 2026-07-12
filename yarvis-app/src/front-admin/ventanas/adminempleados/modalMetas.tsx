import { useState, useEffect } from "react";
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

  return (
    <div
      className="fixed inset-0 bg-black/40 backdrop-blur-sm z-50 flex items-center justify-center"
      onClick={onClose}
    >
      <div
        className="bg-white rounded-[2rem] shadow-2xl w-full max-w-2xl max-h-[85vh] overflow-y-auto custom-scrollbar p-8 space-y-5 animate-in zoom-in-95 duration-200"
        onClick={(e) => e.stopPropagation()}
      >
        <header className="text-center">
          <h2 className="text-lg font-black text-neutral-900 uppercase">
            🎯 Definir Metas y Salarios
          </h2>
          <div className="h-0.5 w-8 bg-neutral-900 mx-auto mt-2 rounded-full"></div>
          <p className="text-[10px] font-bold text-neutral-400 uppercase tracking-widest mt-2">
            Los cambios se guardan automáticamente
          </p>
        </header>

        {/* SELECTOR DE EMPLEADO */}
        <div className="max-h-[120px] overflow-y-auto custom-scrollbar space-y-2">
          {empleados.map((emp) => (
            <button
              key={emp.id}
              onClick={() => handleSelect(emp)}
              className={`w-full p-3 rounded-2xl border-2 transition-all text-left ${
                selectedId === emp.id
                  ? "border-neutral-900 bg-neutral-900 text-white"
                  : "border-neutral-100 bg-neutral-50 hover:border-neutral-300"
              }`}
            >
              <div className="flex items-center justify-between">
                <span className="text-xs font-black uppercase">{emp.nombre}</span>
                <span className="text-[9px] font-bold text-neutral-400">
                  {emp.horario_inicio && emp.horario_fin && emp.horario_inicio !== "00:00"
                    ? `${emp.horario_inicio} - ${emp.horario_fin}`
                    : "Sin horario"}
                </span>
              </div>
            </button>
          ))}
        </div>

        {selectedId && (
          <div className="space-y-5 animate-in slide-in-from-top-2 duration-200">

            {/* SECCION: SALARIO */}
            <div className="bg-neutral-50 rounded-2xl p-5 space-y-4">
              <div className="flex items-center gap-2">
                <span className="text-base">💰</span>
                <p className="text-[10px] font-black text-neutral-500 uppercase tracking-widest">
                  Salario
                </p>
              </div>

              <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 sm:gap-3">
                <div className="space-y-1">
                  <label className="text-[8px] sm:text-[9px] font-bold text-neutral-400 uppercase tracking-wider ml-1">
                    Pago Diario ($)
                  </label>
                  <input
                    type="number"
                    min={0}
                    step={10}
                    value={salarioDiario || ""}
                    onChange={(e) => setSalarioDiario(Number(e.target.value))}
                    placeholder="0"
                    className="w-full px-3 py-2.5 rounded-xl bg-white border border-neutral-200 text-sm font-bold focus:outline-none focus:border-neutral-900"
                  />
                </div>
                <div className="space-y-1">
                  <label className="text-[9px] font-bold text-neutral-400 uppercase tracking-wider ml-1">
                    Días / Semana
                  </label>
                  <input
                    type="number"
                    min={1}
                    max={7}
                    value={diasSemana || ""}
                    onChange={(e) => setDiasSemana(Number(e.target.value))}
                    placeholder="6"
                    className="w-full px-3 py-2.5 rounded-xl bg-white border border-neutral-200 text-sm font-bold focus:outline-none focus:border-neutral-900"
                  />
                </div>
              </div>

              <div className="grid grid-cols-2 sm:grid-cols-4 gap-1.5 sm:gap-2">
                <div className="bg-white rounded-lg sm:rounded-xl p-2 sm:p-3 text-center border border-neutral-100">
                  <p className="text-[8px] font-black text-neutral-400 uppercase tracking-widest">x Hora</p>
                  <p className="text-sm font-black text-neutral-900 mt-1">${hora.toFixed(2)}</p>
                </div>
                <div className="bg-white rounded-lg sm:rounded-xl p-2 sm:p-3 text-center border border-neutral-100">
                  <p className="text-[8px] font-black text-neutral-400 uppercase tracking-widest">x Día</p>
                  <p className="text-sm font-black text-neutral-900 mt-1">${salarioDiario.toFixed(2)}</p>
                </div>
                <div className="bg-white rounded-lg sm:rounded-xl p-2 sm:p-3 text-center border border-neutral-100">
                  <p className="text-[8px] font-black text-neutral-400 uppercase tracking-widest">x Semana</p>
                  <p className="text-sm font-black text-neutral-900 mt-1">${semanal.toFixed(2)}</p>
                </div>
                <div className="bg-white rounded-lg sm:rounded-xl p-2 sm:p-3 text-center border border-neutral-100">
                  <p className="text-[8px] font-black text-neutral-400 uppercase tracking-widest">x Mes</p>
                  <p className="text-sm font-black text-neutral-900 mt-1">${mensual.toFixed(2)}</p>
                </div>
              </div>

              <p className="text-[8px] font-bold text-neutral-300 text-center">
                Basado en horario: {horasPorDia.toFixed(1)}h/día · {diasSemana} días/semana
              </p>
            </div>

            {/* SECCION: METAS DEL SISTEMA */}
            <div className="bg-neutral-50 rounded-2xl p-5 space-y-4">
              <div className="flex items-center gap-2">
                <span className="text-base">🏆</span>
                <p className="text-[10px] font-black text-neutral-500 uppercase tracking-widest">
                  Metas del Sistema
                </p>
              </div>

              {/* META VENTAS */}
              <div className={`rounded-xl p-4 border-2 transition-all ${
                ventasCompletada ? "border-green-500 bg-green-50" : "border-neutral-200 bg-white"
              }`}>
                <div className="flex items-center justify-between mb-3">
                  <div className="flex items-center gap-2">
                    <span className="text-sm">📈</span>
                    <p className="text-[10px] font-black text-neutral-600 uppercase tracking-widest">
                      Meta de Ventas
                    </p>
                  </div>
                  {ventasCompletada && (
                    <span className="text-[9px] font-black text-green-600 uppercase bg-green-100 px-2 py-0.5 rounded-lg">
                      ✅ Cumplida
                    </span>
                  )}
                </div>

                <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 mb-3">
                  <div className="space-y-1">
                    <label className="text-[9px] font-bold text-neutral-400 uppercase tracking-wider ml-1">
                      Meta de Venta Semanal
                    </label>
                    <div className="flex items-center gap-1">
                      <span className="text-sm font-black text-neutral-500">$</span>
                      <input
                        type="number"
                        min={0}
                        step={50}
                        value={ventasThreshold || ""}
                        onChange={(e) => {
                          setVentasThreshold(e.target.value);
                        }}
                        placeholder="0"
                        className="w-full px-3 py-2 rounded-xl bg-neutral-50 border border-neutral-200 text-sm font-bold focus:outline-none focus:border-neutral-900"
                      />
                    </div>
                  </div>
                  <div className="space-y-1">
                    <label className="text-[9px] font-bold text-neutral-400 uppercase tracking-wider ml-1">
                      Si cumple, darle % de lo que vendió
                    </label>
                    <div className="flex items-center gap-1">
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
                        className="w-full px-3 py-2 rounded-xl bg-neutral-50 border border-neutral-200 text-sm font-bold focus:outline-none focus:border-neutral-900"
                      />
                      <span className="text-xs font-black text-neutral-400">%</span>
                    </div>
                  </div>
                </div>

                <div className="bg-neutral-100 rounded-xl p-3 text-center">
                  <p className="text-sm font-black text-neutral-700">
                    Si vende ${umbralVenta.toFixed(2)} → bono de ${(umbralVenta * ventasBonusPct / 100).toFixed(2)}
                  </p>
                </div>
              </div>

              {/* META PUNTUALIDAD */}
              <div className={`rounded-xl p-4 border-2 transition-all ${
                puntualidadCompletada ? "border-green-500 bg-green-50" : "border-neutral-200 bg-white"
              }`}>
                <div className="flex items-center justify-between mb-3">
                  <div className="flex items-center gap-2">
                    <span className="text-sm">⏰</span>
                    <p className="text-[10px] font-black text-neutral-600 uppercase tracking-widest">
                      Meta de Puntualidad
                    </p>
                  </div>
                  {puntualidadCompletada && (
                    <span className="text-[9px] font-black text-green-600 uppercase bg-green-100 px-2 py-0.5 rounded-lg">
                      ✅ Cumplida
                    </span>
                  )}
                </div>

                <p className="text-[9px] font-bold text-neutral-400 mb-2">
                  Si se registra antes de 5 min del inicio de su turno
                </p>

                <div className="space-y-1">
                  <label className="text-[9px] font-bold text-neutral-400 uppercase tracking-wider ml-1">
                    Bono fijo ($)
                  </label>
                  <input
                    type="number"
                    min={0}
                    step={10}
                    value={puntualidadBonus || ""}
                    onChange={(e) => {
                      setPuntualidadBonus(Number(e.target.value));
                    }}
                    placeholder="0"
                    className="w-full px-3 py-2 rounded-xl bg-neutral-50 border border-neutral-200 text-sm font-bold focus:outline-none focus:border-neutral-900"
                  />
                </div>
              </div>
            </div>

            {/* SECCION: METAS PERSONALIZADAS */}
            <div className="bg-neutral-50 rounded-2xl p-5 space-y-4">
              <div className="flex items-center gap-2">
                <span className="text-base">➕</span>
                <p className="text-[10px] font-black text-neutral-500 uppercase tracking-widest">
                  Metas Personalizadas
                </p>
              </div>

              <div className="grid grid-cols-[1fr_100px_40px] gap-2">
                <input
                  type="text"
                  placeholder="Nombre de la meta"
                  value={customName}
                  onChange={(e) => setCustomName(e.target.value)}
                  className="px-3 py-2 rounded-xl bg-white border border-neutral-200 text-sm font-bold focus:outline-none focus:border-neutral-900"
                />
                <input
                  type="number"
                  min={0}
                  step={10}
                  placeholder="Bono $"
                  value={customBonus || ""}
                  onChange={(e) => setCustomBonus(Number(e.target.value))}
                  className="px-3 py-2 rounded-xl bg-white border border-neutral-200 text-sm font-bold focus:outline-none focus:border-neutral-900"
                />
                <button
                  onClick={handleAddCustom}
                  disabled={!customName.trim() || customBonus <= 0}
                  className="rounded-xl bg-neutral-900 text-white font-black text-lg hover:bg-neutral-800 transition-all disabled:opacity-30 disabled:cursor-not-allowed"
                >
                  +
                </button>
              </div>

              {customGoals.length > 0 ? (
                <div className="space-y-2">
                  {customGoals.map((g) => (
                    <div
                      key={g.id}
                      className="flex items-center justify-between bg-white rounded-xl px-4 py-3 border border-neutral-100"
                    >
                      <div className="flex items-center gap-3">
                        <span className={`w-2.5 h-2.5 rounded-full ${g.is_completed ? "bg-green-500" : "bg-neutral-300"}`}></span>
                        <span className="text-xs font-black text-neutral-700 uppercase">{g.goal_name}</span>
                      </div>
                      <div className="flex items-center gap-3">
                        <span className="text-xs font-bold text-neutral-500">${g.bonus_amount}</span>
                        {g.is_completed && (
                          <span className="text-[8px] font-black text-green-600 uppercase">✅</span>
                        )}
                        <button
                          onClick={() => handleDeleteGoal(g.id)}
                          className="text-neutral-300 hover:text-red-500 transition-colors text-sm font-bold"
                        >
                          ×
                        </button>
                      </div>
                    </div>
                  ))}
                </div>
              ) : (
                <p className="text-[9px] font-bold text-neutral-300 text-center py-2">
                  No hay metas personalizadas aún
                </p>
              )}
            </div>
          </div>
        )}

        <div className="pt-2">
          <button
            onClick={handleSaveAll}
            disabled={!selectedId}
            className="w-full py-4 rounded-xl bg-neutral-900 text-white text-xs font-black uppercase tracking-[0.2em] hover:bg-neutral-800 transition-all shadow-xl shadow-neutral-200 disabled:opacity-30 disabled:cursor-not-allowed"
          >
            Guardar Todo
          </button>
        </div>
      </div>
    </div>
  );
};

export default ModalMetas;
