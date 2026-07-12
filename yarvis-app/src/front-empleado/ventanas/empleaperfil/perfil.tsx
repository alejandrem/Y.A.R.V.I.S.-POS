import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

const perfilNav = {
  id: "perfil",
  label: "PERFIL",
  icon: (
    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" />
      <circle cx="9" cy="7" r="4" />
    </svg>
  ),
};

interface EmployeeProfile {
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

interface EmployeeGoalSummary {
  goal_type: string;
  goal_name: string | null;
  bonus_amount: number;
  bonus_percentage: number;
  ventas_threshold: string;
  is_completed: boolean;
}

interface EmployeeProfileFull {
  profile: EmployeeProfile;
  goals: EmployeeGoalSummary[];
}

interface PerfilProps {
  activeTab: string;
  operatorName: string;
}

const Perfil = ({ activeTab, operatorName }: PerfilProps) => {
  const [data, setData] = useState<EmployeeProfileFull | null>(null);

  useEffect(() => {
    if (activeTab === "perfil" && operatorName) {
      loadProfile();
    }
  }, [activeTab, operatorName]);

  const loadProfile = async () => {
    try {
      const result = await invoke<EmployeeProfileFull>("get_employee_profile", { nombre: operatorName });
      setData(result);
    } catch (error) {
      console.error("Error al cargar perfil:", error);
    }
  };

  if (!data) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <p className="text-neutral-400 text-xs font-bold uppercase tracking-widest">Cargando perfil...</p>
      </div>
    );
  }

  const { profile, goals } = data;

  const calcShiftProgress = () => {
    if (!profile.ultimo_login) return 0;
    const parseH = (t: string) => {
      const parts = t.split(":").map(Number);
      return parts[0] + parts[1] / 60;
    };
    const inicio = parseH(profile.horario_inicio);
    const fin = parseH(profile.horario_fin);
    const total = fin > inicio ? fin - inicio : (24 - inicio) + fin;

    const loginDate = new Date(profile.ultimo_login);
    const loginHours = loginDate.getHours() + loginDate.getMinutes() / 60;
    const elapsed = loginHours - inicio;
    return Math.min(100, Math.max(0, (elapsed / total) * 100));
  };

  const ventasGoal = goals.find(g => g.goal_type === "ventas");
  const puntualidadGoal = goals.find(g => g.goal_type === "puntualidad");
  const customGoals = goals.filter(g => g.goal_type === "custom");

  return (
    <div className="w-full mx-auto space-y-8">
      {/* HEADER */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <div className="w-16 h-16 bg-neutral-900 rounded-2xl flex items-center justify-center text-white font-black text-2xl shadow-xl">
            {profile.nombre.charAt(0)}
          </div>
          <div>
            <h2 className="text-2xl font-black text-neutral-900 uppercase tracking-tight">{profile.nombre}</h2>
            <div className="flex items-center gap-2 mt-1">
              <span className={`px-2 py-0.5 rounded-full text-[9px] font-black uppercase ${profile.estado === 'activo' ? 'bg-green-100 text-green-600' : 'bg-red-100 text-red-600'}`}>
                {profile.estado}
              </span>
              <span className="text-[10px] font-bold text-neutral-400 uppercase">{profile.turno}</span>
            </div>
          </div>
        </div>
      </div>

      {/* TURNO - BARRA LINEAL */}
      <div className="bg-white rounded-[2rem] border border-neutral-200 p-8 shadow-sm">
        <div className="flex items-center gap-3 mb-6">
          <div className="w-10 h-10 bg-blue-50 text-blue-500 rounded-2xl flex items-center justify-center text-lg">🕐</div>
          <div>
            <h3 className="text-xs font-black text-neutral-900 uppercase tracking-widest">Mi Turno</h3>
            <p className="text-[9px] text-neutral-400 font-bold uppercase tracking-tighter">Horario de trabajo</p>
          </div>
        </div>

        <div className="flex items-center gap-4 mb-4">
          <div className="text-center">
            <p className="text-3xl font-black text-neutral-900">{profile.horario_inicio}</p>
            <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest mt-1">Entrada</p>
          </div>
          <div className="flex-1">
            <div className="h-4 bg-neutral-100 rounded-full overflow-hidden border border-neutral-200">
              <div
                className="h-full bg-neutral-900 rounded-full transition-all duration-1000 ease-out"
                style={{ width: `${calcShiftProgress()}%` }}
              />
            </div>
            <div className="flex justify-between mt-2">
              <span className="text-[8px] font-black text-neutral-300 uppercase">{profile.ultimo_login ? new Date(profile.ultimo_login).toLocaleTimeString('es-MX', { hour: '2-digit', minute: '2-digit' }) : 'Sin registro'}</span>
              <span className="text-[8px] font-black text-neutral-300 uppercase">Progreso: {calcShiftProgress().toFixed(0)}%</span>
            </div>
          </div>
          <div className="text-center">
            <p className="text-3xl font-black text-neutral-900">{profile.horario_fin}</p>
            <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest mt-1">Salida</p>
          </div>
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-3 gap-3 mt-6">
          <div className="bg-neutral-50 rounded-xl p-4 text-center border border-neutral-100">
            <p className="text-[8px] font-black text-neutral-400 uppercase tracking-widest">Horas/Día</p>
            <p className="text-xl font-black text-neutral-900 mt-1">{profile.horas_por_dia.toFixed(1)}h</p>
          </div>
          <div className="bg-neutral-50 rounded-xl p-4 text-center border border-neutral-100">
            <p className="text-[8px] font-black text-neutral-400 uppercase tracking-widest">Días/Semana</p>
            <p className="text-xl font-black text-neutral-900 mt-1">{profile.dias_semana}</p>
          </div>
          <div className="bg-neutral-50 rounded-xl p-4 text-center border border-neutral-100">
            <p className="text-[8px] font-black text-neutral-400 uppercase tracking-widest">Último Login</p>
            <p className="text-sm font-black text-neutral-900 mt-2">{profile.ultimo_login ? new Date(profile.ultimo_login).toLocaleDateString('es-MX') : 'N/A'}</p>
          </div>
        </div>
      </div>

      {/* SALARIO - CUADROS GRANDES */}
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-4">
        <div className="bg-neutral-900 rounded-[2rem] p-6 text-center text-white shadow-xl relative overflow-hidden">
          <div className="absolute top-0 right-0 w-20 h-20 bg-white/5 rounded-full -translate-y-8 translate-x-8"></div>
          <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest mb-2">Por Hora</p>
          <p className="text-3xl font-black">${profile.salario_hora.toFixed(0)}</p>
          <p className="text-[8px] font-black text-neutral-500 uppercase mt-1">/hora</p>
        </div>
        <div className="bg-white rounded-[2rem] border-2 border-neutral-200 p-6 text-center shadow-sm">
          <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest mb-2">Por Día</p>
          <p className="text-3xl font-black text-neutral-900">${profile.salario_diario.toFixed(0)}</p>
          <p className="text-[8px] font-black text-neutral-300 uppercase mt-1">/día</p>
        </div>
        <div className="bg-white rounded-[2rem] border-2 border-neutral-200 p-6 text-center shadow-sm">
          <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest mb-2">Por Semana</p>
          <p className="text-3xl font-black text-neutral-900">${profile.salario_semanal.toFixed(0)}</p>
          <p className="text-[8px] font-black text-neutral-300 uppercase mt-1">/semana</p>
        </div>
        <div className="bg-white rounded-[2rem] border-2 border-neutral-200 p-6 text-center shadow-sm">
          <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest mb-2">Por Mes</p>
          <p className="text-3xl font-black text-neutral-900">${profile.salario_mensual.toFixed(0)}</p>
          <p className="text-[8px] font-black text-neutral-300 uppercase mt-1">/mes</p>
        </div>
      </div>

      {/* METAS Y BONOS */}
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-6">
        {/* METAS DEL SISTEMA */}
        <div className="bg-white rounded-[2rem] border border-neutral-200 p-8 shadow-sm">
          <div className="flex items-center gap-3 mb-6">
            <div className="w-10 h-10 bg-amber-50 text-amber-500 rounded-2xl flex items-center justify-center text-lg">🏆</div>
            <div>
              <h3 className="text-xs font-black text-neutral-900 uppercase tracking-widest">Metas del Sistema</h3>
              <p className="text-[9px] text-neutral-400 font-bold uppercase tracking-tighter">Definidas por administrador</p>
            </div>
          </div>

          <div className="space-y-4">
            {/* Meta de Ventas */}
            <div className={`rounded-2xl p-5 border-2 transition-all ${ventasGoal?.is_completed ? 'border-green-500 bg-green-50' : 'border-neutral-100 bg-neutral-50'}`}>
              <div className="flex items-center justify-between mb-3">
                <div className="flex items-center gap-2">
                  <span className="text-lg">📈</span>
                  <p className="text-[10px] font-black text-neutral-600 uppercase tracking-widest">Meta de Ventas</p>
                </div>
                {ventasGoal?.is_completed && (
                  <span className="text-[9px] font-black text-green-600 uppercase bg-green-100 px-2 py-0.5 rounded-lg">Cumplida</span>
                )}
              </div>
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <p className="text-[8px] font-black text-neutral-400 uppercase">Umbral Semanal</p>
                  <p className="text-lg font-black text-neutral-900">${parseFloat(ventasGoal?.ventas_threshold || "0").toFixed(0)}</p>
                </div>
                <div>
                  <p className="text-[8px] font-black text-neutral-400 uppercase">Bono</p>
                  <p className="text-lg font-black text-neutral-900">{ventasGoal?.bonus_percentage || 0}%</p>
                </div>
              </div>
            </div>

            {/* Meta de Puntualidad */}
            <div className={`rounded-2xl p-5 border-2 transition-all ${puntualidadGoal?.is_completed ? 'border-green-500 bg-green-50' : 'border-neutral-100 bg-neutral-50'}`}>
              <div className="flex items-center justify-between mb-3">
                <div className="flex items-center gap-2">
                  <span className="text-lg">⏰</span>
                  <p className="text-[10px] font-black text-neutral-600 uppercase tracking-widest">Meta de Puntualidad</p>
                </div>
                {puntualidadGoal?.is_completed && (
                  <span className="text-[9px] font-black text-green-600 uppercase bg-green-100 px-2 py-0.5 rounded-lg">Cumplida</span>
                )}
              </div>
              <p className="text-[9px] font-bold text-neutral-400 mb-2">Registrar antes de 5 min del inicio del turno</p>
              <div>
                <p className="text-[8px] font-black text-neutral-400 uppercase">Bono Fijo</p>
                <p className="text-lg font-black text-neutral-900">${puntualidadGoal?.bonus_amount || 0}</p>
              </div>
            </div>
          </div>
        </div>

        {/* METAS PERSONALIZADAS Y BONO */}
        <div className="space-y-6">
          {/* Bono */}
          <div className="bg-white rounded-[2rem] border border-neutral-200 p-8 shadow-sm">
            <div className="flex items-center gap-3 mb-4">
              <div className="w-10 h-10 bg-purple-50 text-purple-500 rounded-2xl flex items-center justify-center text-lg">💎</div>
              <div>
                <h3 className="text-xs font-black text-neutral-900 uppercase tracking-widest">Bono</h3>
                <p className="text-[9px] text-neutral-400 font-bold uppercase tracking-tighter">Asignado por admin</p>
              </div>
            </div>
            <div className="bg-purple-50 rounded-2xl p-6 text-center border border-purple-100">
              <p className="text-4xl font-black text-purple-600">${profile.bono}</p>
              <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest mt-2">Bono Actual</p>
            </div>
          </div>

          {/* Metas Personalizadas */}
          <div className="bg-white rounded-[2rem] border border-neutral-200 p-8 shadow-sm">
            <div className="flex items-center gap-3 mb-4">
              <div className="w-10 h-10 bg-indigo-50 text-indigo-500 rounded-2xl flex items-center justify-center text-lg">➕</div>
              <div>
                <h3 className="text-xs font-black text-neutral-900 uppercase tracking-widest">Metas Personalizadas</h3>
                <p className="text-[9px] text-neutral-400 font-bold uppercase tracking-tighter">Objetivos adicionales</p>
              </div>
            </div>

            {customGoals.length > 0 ? (
              <div className="space-y-3">
                {customGoals.map((g, idx) => (
                  <div key={idx} className={`flex items-center justify-between p-4 rounded-2xl border-2 transition-all ${g.is_completed ? 'border-green-500 bg-green-50' : 'border-neutral-100 bg-neutral-50'}`}>
                    <div className="flex items-center gap-3">
                      <div className={`w-3 h-3 rounded-full ${g.is_completed ? 'bg-green-500' : 'bg-neutral-300'}`}></div>
                      <span className="text-xs font-black text-neutral-700 uppercase">{g.goal_name}</span>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className="text-xs font-bold text-neutral-500">${g.bonus_amount}</span>
                      {g.is_completed && <span className="text-[9px] font-black text-green-600">✅</span>}
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <div className="py-8 text-center">
                <p className="text-[10px] font-black text-neutral-300 uppercase tracking-widest italic">Sin metas personalizadas</p>
              </div>
            )}
          </div>

          {/* Meta Mensual */}
          <div className="bg-white rounded-[2rem] border border-neutral-200 p-8 shadow-sm">
            <div className="flex items-center gap-3 mb-4">
              <div className="w-10 h-10 bg-orange-50 text-orange-500 rounded-2xl flex items-center justify-center text-lg">🎯</div>
              <div>
                <h3 className="text-xs font-black text-neutral-900 uppercase tracking-widest">Meta Mensual</h3>
                <p className="text-[9px] text-neutral-400 font-bold uppercase tracking-tighter">Objetivo de ventas del mes</p>
              </div>
            </div>
            <div className="bg-orange-50 rounded-2xl p-6 text-center border border-orange-100">
              <p className="text-4xl font-black text-orange-600">${profile.meta_mensual.toFixed(0)}</p>
              <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest mt-2">Meta Mensual</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Perfil;
export { perfilNav };
