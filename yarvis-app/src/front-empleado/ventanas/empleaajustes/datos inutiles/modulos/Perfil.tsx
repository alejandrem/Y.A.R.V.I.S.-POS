import { ICONO_USUARIO } from "../../../../../components/ui";

export const perfilSeparador = {
  id: "perfil",
  label: "PERFIL",
  icon: ICONO_USUARIO,
  left: "72%",
};

export const PerfilIzq = () => (
  <div className="flex-1 bg-white border-r-[3px] border-neutral-900 p-10 flex flex-col relative overflow-hidden">
    <div className="absolute top-8 left-10 right-10 h-[1px] bg-neutral-900/10" />
    <div className="absolute top-12 left-10 right-10 h-[1px] bg-neutral-900/10" />
    <div className="flex-1 flex flex-col items-center justify-center text-center">
      <p className="font-mono text-[11px] font-black tracking-[0.35em] text-neutral-900 border-2 border-neutral-900 px-3 py-1.5 rounded-lg">
        MODULO 05 // EMPLEADO
      </p>
      <h3 className="font-mono text-[38px] font-black tracking-[0.12em] text-neutral-900 leading-none mt-6">MI</h3>
      <h3 className="font-mono text-[38px] font-black tracking-[0.12em] text-neutral-900 leading-none">PERFIL</h3>
      <div className="w-20 h-[4px] bg-neutral-900 mt-6" />
      <p className="font-mono text-[11px] font-bold tracking-widest text-neutral-500 mt-4 max-w-[320px] leading-relaxed">
        Tu turno, salario y metas
        <br />
        todo en un lugar.
      </p>
      <div className="mt-6 flex gap-2">
        <span className="font-mono text-[8px] font-black tracking-widest bg-neutral-900 text-white px-2 py-1 rounded">PAG. 11 — 12</span>
      </div>
    </div>
    <p className="font-mono text-[11px] font-black tracking-widest text-neutral-400 text-center">— 11 —</p>
    <div className="absolute bottom-0 left-0 w-6 h-6 border-t-[2px] border-r-[2px] border-neutral-900/20 rounded-tr-xl" />
  </div>
);

export const PerfilDer = () => (
  <div className="flex-1 bg-white p-8 flex flex-col relative overflow-hidden">
    <div className="absolute top-8 left-10 right-10 h-[1px] bg-neutral-900/10" />
    <div className="absolute top-12 left-10 right-10 h-[1px] bg-neutral-900/10" />
    <div className="flex-1 flex flex-col justify-center gap-4 mt-2">
      <div className="flex gap-4 items-center bg-white border-2 border-neutral-900 rounded-2xl p-4">
        <div className="w-14 h-14 bg-white border-2 border-neutral-900 rounded-xl flex items-center justify-center shrink-0">
          <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="black" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="12" cy="12" r="9" />
            <path d="M12 7v5l3 2" />
          </svg>
        </div>
        <div className="flex-1 min-w-0">
          <p className="font-mono text-[10px] font-black tracking-widest text-neutral-900">01 — MI TURNO</p>
          <p className="font-mono text-[9px] font-bold text-neutral-500 leading-relaxed mt-1">
            Barra negra = tu horario. Verde = horas extra. Bolita blanca = tu login real.
          </p>
        </div>
      </div>
      <div className="flex gap-4 items-center bg-white border-2 border-neutral-900 rounded-2xl p-4">
        <div className="w-14 h-14 bg-white border-2 border-neutral-900 rounded-xl flex items-center justify-center shrink-0">
          <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="black" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
            <path d="M12 1v22" />
            <path d="M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6" />
          </svg>
        </div>
        <div className="flex-1 min-w-0">
          <p className="font-mono text-[10px] font-black tracking-widest text-neutral-900">02 — SALARIO</p>
          <p className="font-mono text-[9px] font-bold text-neutral-500 leading-relaxed mt-1">
            Por hora, día, semana y mes. Cálculo automático según tus horas y días.
          </p>
        </div>
      </div>
      <div className="flex gap-4 items-center bg-neutral-900 rounded-2xl p-4 border-2 border-neutral-900">
        <div className="w-14 h-14 bg-white rounded-xl flex items-center justify-center shrink-0">
          <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="black" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="12" cy="8" r="6" />
            <path d="M15.477 12.89L17 22l-5-3-5 3 1.523-9.11" />
          </svg>
        </div>
        <div className="flex-1 min-w-0">
          <p className="font-mono text-[10px] font-black tracking-widest text-white">03 — METAS Y BONOS</p>
          <p className="font-mono text-[9px] font-bold text-white/60 leading-relaxed mt-1">
            Verde = cumplida. Bono fijo o porcentaje según ventas y puntualidad.
          </p>
        </div>
      </div>
    </div>
    <p className="font-mono text-[11px] font-black tracking-widest text-neutral-400 text-center mt-2">— 12 —</p>
    <div className="absolute bottom-0 right-0 w-6 h-6 border-t-[2px] border-l-[2px] border-neutral-900/20 rounded-tl-xl" />
  </div>
);
