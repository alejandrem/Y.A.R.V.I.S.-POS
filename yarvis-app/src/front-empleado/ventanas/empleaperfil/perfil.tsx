import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { MorphIcon } from "morphicons/react";
import { ICONO_RELOJ } from "../../../components/ui";
import {
  geometriaBarra, fmtHM, MiniBarraDia,
  type MiTurno, type DiaExtra,
} from "../../../components/turno";

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
  const [turno, setTurno] = useState<MiTurno | null>(null);
  const [extras, setExtras] = useState<DiaExtra[] | null>(null);
  const [expandidas, setExpandidas] = useState<Set<string>>(new Set());
  const [ahora, setAhora] = useState(() => new Date());

  useEffect(() => {
    if (activeTab === "perfil" && operatorName) {
      loadProfile();
    }
  }, [activeTab, operatorName]);

  // Reloj vivo: la barra avanza sola cada 30 segundos.
  useEffect(() => {
    const t = window.setInterval(() => setAhora(new Date()), 30000);
    return () => window.clearInterval(t);
  }, []);

  useEffect(() => {
    if (activeTab === "perfil") {
      invoke<MiTurno>("get_mi_turno").then(setTurno).catch((e) => console.error("Error al cargar turno:", e));
      invoke<DiaExtra[]>("get_mis_horas_extra").then(setExtras).catch((e) => console.error("Error al cargar extras:", e));
    }
  }, [activeTab]);

  const toggleExtra = (fecha: string) => {
    setExpandidas((prev) => {
      const next = new Set(prev);
      if (next.has(fecha)) next.delete(fecha); else next.add(fecha);
      return next;
    });
  };

  const fmtMin = (m: number) => `${Math.floor(m / 60)}h ${m % 60}m`;


  const loadProfile = async () => {
    try {
      const result = await invoke<EmployeeProfileFull>("get_employee_profile", { nombre: operatorName });
      setData(result);
    } catch (error) {
      console.error("Error al cargar perfil:", error);
    }
  };

  // Geometría de la barra de asistencia para HOY.
  const barra = geometriaBarra(turno, ahora);

  if (!data) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <p className="text-neutral-400 text-xs font-bold uppercase tracking-widest">Cargando perfil...</p>
      </div>
    );
  }

  const { profile, goals } = data;

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

      {/* TURNO - BARRA DE ASISTENCIA CON PUNTOS Y EXTRAS */}
      <div className="bg-white rounded-[2rem] border border-neutral-200 p-8 shadow-sm">
        <div className="flex items-center gap-3 mb-6">
          <div className="w-10 h-10 bg-neutral-950 rounded-2xl flex items-center justify-center shadow-md">
            <MorphIcon icon={ICONO_RELOJ} size={17} strokeWidth={2.2} spring="smooth" className="text-white" />
          </div>
          <div>
            <h3 className="text-xs font-black text-neutral-900 uppercase tracking-widest">Mi Turno</h3>
            <p className="text-[9px] text-neutral-400 font-bold uppercase tracking-tighter">Horario de trabajo</p>
          </div>
          {barra?.enExtra && (
            <span className="ml-auto inline-flex items-center gap-1.5 px-3 py-1.5 bg-emerald-50 border border-emerald-200 rounded-xl text-[10px] font-black uppercase tracking-widest text-emerald-600 animate-pulse">
              Extra: +{Math.floor(barra.extraMinutos / 60)}h {barra.extraMinutos % 60}m
            </span>
          )}
          {!barra?.enExtra && barra?.llegoPuntual && (
            <span className="ml-auto inline-flex items-center gap-1.5 px-3 py-1.5 bg-sky-50 border border-sky-200 rounded-xl text-[10px] font-black uppercase tracking-widest text-sky-600">
              ¡Felicidades, llegaste temprano!
            </span>
          )}
        </div>

        {!barra ? (
          /* Día de descanso o sin horario hoy */
          <div className="py-10 text-center bg-neutral-50 rounded-2xl border border-dashed border-neutral-200">
            <p className="text-sm font-black uppercase tracking-widest text-neutral-400">Hoy no tienes turno asignado</p>
            <p className="text-[10px] font-bold text-neutral-300 mt-1.5">Día de descanso · disfruta uwu</p>
          </div>
        ) : (
          <>
            <div className="flex items-center gap-4 mb-4">
              <div className="text-center shrink-0">
                <p className="text-3xl font-black text-neutral-900">{fmtHM(barra.inicio)}</p>
                <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest mt-1">Entrada</p>
              </div>

              {/* PISTA DE LA BARRA */}
              <div className="flex-1">
                <div className="relative h-4 bg-neutral-100 rounded-full border border-neutral-200 overflow-visible">
                  {/* EXTRA tempranero (verde antes de la entrada oficial) */}
                  {barra.preExtraActivo && (
                    <div
                      className="absolute inset-y-0 bg-emerald-400 transition-all duration-700 ease-out"
                      style={{ left: `${barra.loginPct}%`, width: `${Math.max(0, barra.preExtraPct)}%`, borderRadius: "999px 0 0 999px" }}
                    />
                  )}
                  {/* Trabajo dentro del horario (negro) */}
                  <div
                    className="absolute inset-y-0 bg-neutral-900 rounded-full transition-all duration-700 ease-out"
                    style={{ left: `${barra.inicioPct}%`, width: `${Math.max(0, barra.trabajoPct)}%` }}
                  />
                  {/* Extra post-turno (verde después de la salida) */}
                  {barra.enExtraPost && (
                    <div
                      className="absolute inset-y-0 bg-emerald-500 transition-all duration-700 ease-out"
                      style={{ left: `${barra.finPct}%`, width: `${Math.max(0, barra.postExtraPct)}%`, borderRadius: "0 999px 999px 0" }}
                    />
                  )}
                  {/* ● Bolita en la ENTRADA OFICIAL cuando hubo extra tempranero */}
                  {barra.preExtraActivo && (
                    <div
                      className="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-3.5 h-3.5 bg-white border-[3px] border-neutral-900 rounded-full shadow-md z-10"
                      style={{ left: `${barra.inicioPct}%` }}
                      title="Entrada oficial — lo trabajado antes cuenta como extra"
                    />
                  )}
                  {/* ● Bolita: PRIMER LOGIN del día (llegada real) */}
                  {barra.loginPct !== null && (
                    <div
                      className="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-3.5 h-3.5 bg-white border-[3px] border-neutral-900 rounded-full shadow-md z-10"
                      style={{ left: `${barra.loginPct}%` }}
                      title={`Primer login: ${turno?.primer_login ?? ""}`}
                    />
                  )}
                  {/* ● Bolita blanca en la frontera del extra post-turno */}
                  {barra.enExtraPost && (
                    <div
                      className="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-3.5 h-3.5 bg-white border-[3px] border-emerald-500 rounded-full shadow-md z-10"
                      style={{ left: `${barra.finPct}%` }}
                      title="Fin de tu horario — desde aquí cuenta como extra"
                    />
                  )}
                </div>

                <div className="flex justify-between mt-2">
                  <span className="text-[8px] font-black text-neutral-300 uppercase">
                    {turno?.primer_login
                      ? `Llegaste ${turno.primer_login}${
                          barra.minutosTarde > 0
                            ? ` · ${barra.minutosTarde} min tarde`
                            : barra.llegoPuntual
                              ? " · ¡Felicidades, llegaste puntual!"
                              : ` · ${barra.minutosTemprano} min temprano (extra)`
                        }`
                      : "Sin registro de entrada"}
                  </span>
                  <span className={`text-[8px] font-black uppercase ${barra.enExtra ? "text-emerald-500" : "text-neutral-300"}`}>
                    {barra.enExtra ? `Progreso: ${Math.round(barra.trabajoPct)}% + extra` : `Progreso: ${Math.round(barra.trabajoPct)}%`}
                  </span>
                </div>
              </div>

              <div className="text-center shrink-0">
                <p className={`text-3xl font-black ${barra.enExtra ? "text-emerald-600" : "text-neutral-900"}`}>{fmtHM(barra.fin)}</p>
                <p className={`text-[9px] font-black uppercase tracking-widest mt-1 ${barra.enExtra ? "text-emerald-500" : "text-neutral-400"}`}>Salida</p>
              </div>
            </div>

            {turno && turno.primer_login === null && (
              <p className="text-[10px] font-bold text-amber-500 bg-amber-50 border border-amber-200 rounded-xl px-4 py-2.5 mt-4">
                Aún no registras tu primer login de hoy — este marca tu hora de entrada real.
              </p>
            )}
          </>
        )}

        <div className="grid grid-cols-1 sm:grid-cols-3 gap-3 mt-6">
          <div className="bg-neutral-50 rounded-xl p-4 text-center border border-neutral-100">
            <p className="text-[8px] font-black text-neutral-400 uppercase tracking-widest">Horas/Día</p>
            <p className="text-xl font-black text-neutral-900 mt-1">{(turno?.horas_por_dia ?? profile.horas_por_dia).toFixed(1)}h</p>
          </div>
          <div className="bg-neutral-50 rounded-xl p-4 text-center border border-neutral-100">
            <p className="text-[8px] font-black text-neutral-400 uppercase tracking-widest">Días/Semana</p>
            <p className="text-xl font-black text-neutral-900 mt-1">{turno?.dias_semana ?? profile.dias_semana}</p>
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

      {/* HORAS EXTRAS INDEFINIDAS — historial completo (oculto si no hay) */}
      {extras && extras.length > 0 && (
        <div className="bg-white rounded-[2rem] border border-neutral-200 p-8 shadow-sm">
          <div className="flex items-center gap-3 mb-6">
            <div className="w-10 h-10 bg-emerald-50 text-emerald-500 rounded-2xl flex items-center justify-center">
              <MorphIcon icon={ICONO_RELOJ} size={17} strokeWidth={2.2} spring="smooth" />
            </div>
            <div>
              <h3 className="text-xs font-black text-neutral-900 uppercase tracking-widest">Horas Extras Indefinidas</h3>
              <p className="text-[9px] text-neutral-400 font-bold uppercase tracking-tighter">Cada día que trabajaste fuera de tu horario</p>
            </div>
            <span className="ml-auto px-3 py-1 bg-neutral-950 text-white text-[9px] font-black rounded-lg uppercase tracking-widest">
              {extras.length} {extras.length === 1 ? "DÍA" : "DÍAS"}
            </span>
          </div>

          <div className={`space-y-2 pr-1 custom-scrollbar ${extras.length > 8 ? "max-h-[420px] overflow-y-auto" : ""}`}>
            {extras.map((d) => {
              const totalExtra = d.extra_pre_min + d.extra_post_min;
              const abierta = expandidas.has(d.fecha);
              return (
                <div key={d.fecha} className={`rounded-2xl border transition-all ${abierta ? "border-neutral-300 bg-neutral-50/60" : "border-neutral-100 hover:border-neutral-200"}`}>
                  {/* FILA RESUMEN */}
                  <button
                    onClick={() => toggleExtra(d.fecha)}
                    className="w-full flex items-center gap-3 p-3.5 text-left"
                  >
                    <div className="shrink-0 w-24">
                      <p className="text-[11px] font-black text-neutral-900 uppercase leading-none">{d.dia_label}</p>
                      <p className="text-[9px] font-bold text-neutral-400 mt-1">{d.fecha.slice(5).split("-").reverse().join("/")}</p>
                    </div>

                    {/* Mini barra histórica */}
                    <div className="flex-1 flex items-center gap-3 min-w-0" title={`Entrada oficial ${d.entrada_oficial} · Salida ${d.salida_oficial}`}>
                      <span className="text-[9px] font-black uppercase tracking-widest whitespace-nowrap text-neutral-500">{d.primer_login}</span>
                      <MiniBarraDia d={d} />
                      <span className="text-[9px] font-black uppercase tracking-widest whitespace-nowrap text-emerald-600">{d.ultimo_login}</span>
                    </div>

                    <span className="px-2 py-0.5 bg-emerald-100 text-emerald-600 rounded-lg text-[8px] font-black uppercase tracking-widest whitespace-nowrap shrink-0">
                      +{fmtMin(totalExtra)}
                    </span>

                    <MorphIcon
                      icon={ICONO_RELOJ}
                      size={13}
                      strokeWidth={2.4}
                      spring="snappy"
                      reducedMotion="user"
                      className={`shrink-0 text-neutral-300 transition-transform duration-200 ${abierta ? "rotate-180" : ""}`}
                    />
                  </button>

                  {/* DESGLOSE AL PICARLE A VER */}
                  {abierta && (
                    <div className="px-4 pb-4 animate-in fade-in slide-in-from-top-1 duration-200">
                      <div className="bg-white rounded-xl border border-neutral-200 p-4 space-y-2 text-[10px] font-bold text-neutral-500 grid grid-cols-1 sm:grid-cols-2 gap-x-6 gap-y-2">
                        <p><span className="font-black text-neutral-400 uppercase tracking-wider">Llegó:</span> <span className="text-neutral-900 font-black">{d.primer_login}</span>{d.extra_pre_min > 0 && <> · <span className="text-emerald-600 font-black">{fmtMin(d.extra_pre_min)} antes de su entrada</span></>}</p>
                        <p><span className="font-black text-neutral-400 uppercase tracking-wider">Se fue:</span> <span className="text-neutral-900 font-black">{d.ultimo_login}</span>{d.extra_post_min > 0 && <> · <span className="text-emerald-600 font-black">{fmtMin(d.extra_post_min)} después de su salida</span></>}</p>
                        <p><span className="font-black text-neutral-400 uppercase tracking-wider">Horario ese día:</span> <span className="text-neutral-900 font-black">{d.entrada_oficial} — {d.salida_oficial}</span></p>
                        <p><span className="font-black text-neutral-400 uppercase tracking-wider">Total trabajado:</span> <span className="text-neutral-900 font-black">{fmtMin(d.trabajo_min)}</span> · <span className="text-emerald-600 font-black">Extra total: {fmtMin(totalExtra)}</span></p>
                      </div>
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </div>
      )}
    </div>
  );
};

export default Perfil;
export { perfilNav };
