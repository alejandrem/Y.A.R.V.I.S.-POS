// ══════════════════════════════════════════════════════════════════
// TAREA: Orquestador de la ventana Perfil del empleado.
// Posee el estado y las cargas (perfil, turno, horas extra), el
// reloj vivo y compone las cards visuales de ./componentes.
// ══════════════════════════════════════════════════════════════════
import { useState, useEffect } from "react";
import { invokeTauri, reportarError } from "../../../services/tauri";
import { obtenerMiTurno, obtenerMisHorasExtra } from "../../../services/turno";
import {
  geometriaBarra,
  type MiTurno, type DiaExtra,
} from "../../../components/turno";
import type { EmployeeProfileFull } from "./utilidades/tipos";
import TarjetaTurno from "./componentes/tarjeta-turno";
import TarjetasSalario from "./componentes/tarjetas-salario";
import MetasBonos from "./componentes/metas-bonos";
import HorasExtras from "./componentes/horas-extras";

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
      obtenerMiTurno().then(setTurno).catch((e) => reportarError("No se pudo cargar tu turno", e));
      obtenerMisHorasExtra().then(setExtras).catch((e) => reportarError("No se pudieron cargar tus horas extra", e));
    }
  }, [activeTab]);

  const toggleExtra = (fecha: string) => {
    setExpandidas((prev) => {
      const next = new Set(prev);
      if (next.has(fecha)) next.delete(fecha); else next.add(fecha);
      return next;
    });
  };

  const loadProfile = async () => {
    try {
      setData(await invokeTauri<EmployeeProfileFull>("get_employee_profile", { nombre: operatorName }));
    } catch (error) {
      reportarError("No se pudo cargar tu perfil", error);
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

  return (
    <div className="w-full max-w-[1200px] mx-auto space-y-8">
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
      <TarjetaTurno profile={profile} turno={turno} barra={barra} />

      {/* SALARIO - CUADROS GRANDES */}
      <TarjetasSalario profile={profile} />

      {/* METAS Y BONOS */}
      <MetasBonos goals={goals} profile={profile} />

      {/* HORAS EXTRAS INDEFINIDAS — historial completo (oculto si no hay) */}
      {extras && extras.length > 0 && (
        <HorasExtras extras={extras} expandidas={expandidas} onToggle={toggleExtra} />
      )}
    </div>
  );
};

export default Perfil;
export { perfilNav };
