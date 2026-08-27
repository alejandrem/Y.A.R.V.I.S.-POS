// TAREA: Sección "Metas y Bonos" — metas del sistema (ventas y
// puntualidad), bono asignado, metas personalizadas y meta mensual.
// Presentacional: recibe las metas y el perfil. Sin emojis, solo blanco y negro con morphicons.
import { useState } from "react";
import type { EmployeeGoalSummary, EmployeeProfile } from "../utilidades/tipos";
import { IconoMorph } from "../../../../components/ui";
import {
  ICONO_TROFEO,
  ICONO_ESTRELLA,
  ICONO_TRENDING,
  ICONO_GRAFICA,
  ICONO_RELOJ,
  ICONO_CALENDARIO,
  ICONO_REGALO,
  ICONO_TROFEO as ICONO_BONO,
  ICONO_MAS,
  ICONO_MAS_CIRCULO,
  ICONO_TARGET,
  ICONO_CHECK,
  ICONO_CHECK_CIRCULO,
} from "../../../../icons";

interface MetasBonosProps {
  goals: EmployeeGoalSummary[];
  profile: EmployeeProfile;
}

const MetasBonos = ({ goals, profile }: MetasBonosProps) => {
  const ventasGoal = goals.find((g) => g.goal_type === "ventas");
  const puntualidadGoal = goals.find((g) => g.goal_type === "puntualidad");
  const customGoals = goals.filter((g) => g.goal_type === "custom");

  const [hoverSistema, setHoverSistema] = useState(false);
  const [hoverVentas, setHoverVentas] = useState(false);
  const [hoverPuntual, setHoverPuntual] = useState(false);
  const [hoverBono, setHoverBono] = useState(false);
  const [hoverCustom, setHoverCustom] = useState(false);
  const [hoverMensual, setHoverMensual] = useState(false);

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 gap-6">
      {/* METAS DEL SISTEMA */}
      <div
        className="bg-white rounded-[2rem] border border-neutral-200 p-8 shadow-sm"
        onMouseEnter={() => setHoverSistema(true)}
        onMouseLeave={() => setHoverSistema(false)}
      >
        <div className="flex items-center gap-3 mb-6">
          <div className="w-10 h-10 bg-white border-2 border-neutral-900 rounded-2xl flex items-center justify-center text-neutral-900">
            <IconoMorph icono={ICONO_TROFEO} iconoHover={ICONO_ESTRELLA} hover={hoverSistema} size={18} strokeWidth={2.2} />
          </div>
          <div>
            <h3 className="text-xs font-black text-neutral-900 uppercase tracking-widest">Metas del Sistema</h3>
            <p className="text-[9px] text-neutral-400 font-bold uppercase tracking-tighter">Definidas por administrador</p>
          </div>
        </div>

        <div className="space-y-4">
          {/* Meta de Ventas */}
          <div
            className={`rounded-2xl p-5 border-2 transition-all ${ventasGoal?.is_completed ? "border-neutral-900 bg-neutral-900" : "border-neutral-200 bg-neutral-50"}`}
            onMouseEnter={() => setHoverVentas(true)}
            onMouseLeave={() => setHoverVentas(false)}
          >
            <div className="flex items-center justify-between mb-3">
              <div className="flex items-center gap-2">
                <span className="w-8 h-8 bg-white border border-neutral-200 rounded-xl flex items-center justify-center">
                  <IconoMorph icono={ICONO_TRENDING} iconoHover={ICONO_GRAFICA} hover={hoverVentas} size={16} strokeWidth={2.2} />
                </span>
                <p className={`text-[10px] font-black uppercase tracking-widest ${ventasGoal?.is_completed ? "text-white" : "text-neutral-600"}`}>Meta de Ventas</p>
              </div>
              {ventasGoal?.is_completed && (
                <span className="text-[9px] font-black uppercase bg-white text-neutral-900 px-2 py-0.5 rounded-lg">Cumplida</span>
              )}
            </div>
            <div className="grid grid-cols-2 gap-3">
              <div>
                <p className={`text-[8px] font-black uppercase ${ventasGoal?.is_completed ? "text-white/60" : "text-neutral-400"}`}>Umbral Semanal</p>
                <p className={`text-lg font-black ${ventasGoal?.is_completed ? "text-white" : "text-neutral-900"}`}>${parseFloat(ventasGoal?.ventas_threshold || "0").toFixed(0)}</p>
              </div>
              <div>
                <p className={`text-[8px] font-black uppercase ${ventasGoal?.is_completed ? "text-white/60" : "text-neutral-400"}`}>Bono</p>
                <p className={`text-lg font-black ${ventasGoal?.is_completed ? "text-white" : "text-neutral-900"}`}>{ventasGoal?.bonus_percentage || 0}%</p>
              </div>
            </div>
          </div>

          {/* Meta de Puntualidad */}
          <div
            className={`rounded-2xl p-5 border-2 transition-all ${puntualidadGoal?.is_completed ? "border-neutral-900 bg-neutral-900" : "border-neutral-200 bg-neutral-50"}`}
            onMouseEnter={() => setHoverPuntual(true)}
            onMouseLeave={() => setHoverPuntual(false)}
          >
            <div className="flex items-center justify-between mb-3">
              <div className="flex items-center gap-2">
                <span className="w-8 h-8 bg-white border border-neutral-200 rounded-xl flex items-center justify-center">
                  <IconoMorph icono={ICONO_RELOJ} iconoHover={ICONO_CALENDARIO} hover={hoverPuntual} size={16} strokeWidth={2.2} />
                </span>
                <p className={`text-[10px] font-black uppercase tracking-widest ${puntualidadGoal?.is_completed ? "text-white" : "text-neutral-600"}`}>Meta de Puntualidad</p>
              </div>
              {puntualidadGoal?.is_completed && (
                <span className="text-[9px] font-black uppercase bg-white text-neutral-900 px-2 py-0.5 rounded-lg">Cumplida</span>
              )}
            </div>
            <p className={`text-[9px] font-bold mb-2 ${puntualidadGoal?.is_completed ? "text-white/60" : "text-neutral-400"}`}>Registrar antes de 5 min del inicio del turno</p>
            <div>
              <p className={`text-[8px] font-black uppercase ${puntualidadGoal?.is_completed ? "text-white/60" : "text-neutral-400"}`}>Bono Fijo</p>
              <p className={`text-lg font-black ${puntualidadGoal?.is_completed ? "text-white" : "text-neutral-900"}`}>${puntualidadGoal?.bonus_amount || 0}</p>
            </div>
          </div>
        </div>
      </div>

      {/* METAS PERSONALIZADAS Y BONO */}
      <div className="space-y-6">
        {/* Bono */}
        <div
          className="bg-white rounded-[2rem] border border-neutral-200 p-8 shadow-sm"
          onMouseEnter={() => setHoverBono(true)}
          onMouseLeave={() => setHoverBono(false)}
        >
          <div className="flex items-center gap-3 mb-4">
            <div className="w-10 h-10 bg-white border-2 border-neutral-900 rounded-2xl flex items-center justify-center text-neutral-900">
              <IconoMorph icono={ICONO_REGALO} iconoHover={ICONO_BONO} hover={hoverBono} size={18} strokeWidth={2.2} />
            </div>
            <div>
              <h3 className="text-xs font-black text-neutral-900 uppercase tracking-widest">Bono</h3>
              <p className="text-[9px] text-neutral-400 font-bold uppercase tracking-tighter">Asignado por admin</p>
            </div>
          </div>
          <div className="bg-neutral-900 rounded-2xl p-6 text-center border-2 border-neutral-900">
            <p className="text-4xl font-black text-white">${profile.bono}</p>
            <p className="text-[9px] font-black text-white/60 uppercase tracking-widest mt-2">Bono Actual</p>
          </div>
        </div>

        {/* Metas Personalizadas */}
        <div
          className="bg-white rounded-[2rem] border border-neutral-200 p-8 shadow-sm"
          onMouseEnter={() => setHoverCustom(true)}
          onMouseLeave={() => setHoverCustom(false)}
        >
          <div className="flex items-center gap-3 mb-4">
            <div className="w-10 h-10 bg-white border-2 border-neutral-900 rounded-2xl flex items-center justify-center text-neutral-900">
              <IconoMorph icono={ICONO_MAS} iconoHover={ICONO_MAS_CIRCULO} hover={hoverCustom} size={18} strokeWidth={2.2} />
            </div>
            <div>
              <h3 className="text-xs font-black text-neutral-900 uppercase tracking-widest">Metas Personalizadas</h3>
              <p className="text-[9px] text-neutral-400 font-bold uppercase tracking-tighter">Objetivos adicionales</p>
            </div>
          </div>

          {customGoals.length > 0 ? (
            <div className="space-y-3">
              {customGoals.map((g, idx) => (
                <div
                  key={idx}
                  className={`flex items-center justify-between p-4 rounded-2xl border-2 transition-all ${g.is_completed ? "border-neutral-900 bg-neutral-900" : "border-neutral-200 bg-neutral-50"}`}
                >
                  <div className="flex items-center gap-3">
                    <div className={`w-3 h-3 rounded-full ${g.is_completed ? "bg-white" : "bg-neutral-300"}`}></div>
                    <span className={`text-xs font-black uppercase ${g.is_completed ? "text-white" : "text-neutral-700"}`}>{g.goal_name}</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className={`text-xs font-bold ${g.is_completed ? "text-white/70" : "text-neutral-500"}`}>${g.bonus_amount}</span>
                    {g.is_completed && (
                      <span className="w-5 h-5 bg-white rounded-full flex items-center justify-center">
                        <IconoMorph icono={ICONO_CHECK} iconoHover={ICONO_CHECK_CIRCULO} hover={hoverCustom} size={12} strokeWidth={2.5} />
                      </span>
                    )}
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
        <div
          className="bg-white rounded-[2rem] border border-neutral-200 p-8 shadow-sm"
          onMouseEnter={() => setHoverMensual(true)}
          onMouseLeave={() => setHoverMensual(false)}
        >
          <div className="flex items-center gap-3 mb-4">
            <div className="w-10 h-10 bg-white border-2 border-neutral-900 rounded-2xl flex items-center justify-center text-neutral-900">
              <IconoMorph icono={ICONO_TARGET} iconoHover={ICONO_TROFEO} hover={hoverMensual} size={18} strokeWidth={2.2} />
            </div>
            <div>
              <h3 className="text-xs font-black text-neutral-900 uppercase tracking-widest">Meta Mensual</h3>
              <p className="text-[9px] text-neutral-400 font-bold uppercase tracking-tighter">Objetivo de ventas del mes</p>
            </div>
          </div>
          <div className="bg-neutral-900 rounded-2xl p-6 text-center border-2 border-neutral-900">
            <p className="text-4xl font-black text-white">${profile.meta_mensual.toFixed(0)}</p>
            <p className="text-[9px] font-black text-white/60 uppercase tracking-widest mt-2">Meta Mensual</p>
          </div>
        </div>
      </div>
    </div>
  );
};

export default MetasBonos;
