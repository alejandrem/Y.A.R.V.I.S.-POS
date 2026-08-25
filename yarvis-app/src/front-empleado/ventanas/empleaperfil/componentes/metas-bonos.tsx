// ══════════════════════════════════════════════════════════════════
// TAREA: Sección "Metas y Bonos" — metas del sistema (ventas y
// puntualidad), bono asignado, metas personalizadas y meta mensual.
// Presentacional: recibe las metas y el perfil.
// ══════════════════════════════════════════════════════════════════
import type { EmployeeGoalSummary, EmployeeProfile } from "../utilidades/tipos";

interface MetasBonosProps {
  goals: EmployeeGoalSummary[];
  profile: EmployeeProfile;
}

const MetasBonos = ({ goals, profile }: MetasBonosProps) => {
  const ventasGoal = goals.find(g => g.goal_type === "ventas");
  const puntualidadGoal = goals.find(g => g.goal_type === "puntualidad");
  const customGoals = goals.filter(g => g.goal_type === "custom");

  return (
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
  );
};

export default MetasBonos;
