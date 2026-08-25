// ══════════════════════════════════════════════════════════════════
// TAREA: Grid de tarjetas de salario (por hora, día, semana y mes).
// Presentacional puro: solo recibe el perfil del empleado.
// ══════════════════════════════════════════════════════════════════
import type { EmployeeProfile } from "../utilidades/tipos";

const TarjetasSalario = ({ profile }: { profile: EmployeeProfile }) => {
  return (
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
  );
};

export default TarjetasSalario;
