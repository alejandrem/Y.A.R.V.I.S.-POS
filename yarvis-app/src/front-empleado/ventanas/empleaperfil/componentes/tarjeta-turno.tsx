// ══════════════════════════════════════════════════════════════════
// TAREA: Card "Mi Turno" — compone las 2 pistas de turno-extra:
// BarraTurnoNormal (ventana oficial) + BarraExtra (extra real).
// Presentacional: recibe la barra ya calculada y el turno.
// ══════════════════════════════════════════════════════════════════
import { MorphIcon } from "morphicons/react";
import { ICONO_RELOJ } from "../../../../components/ui";
import { fmtHM } from "../../../../components/turno-extra";
import type { MiTurno, BarraTurno } from "../../../../components/turno-extra";
import { BarraTurnoNormal } from "../../../../components/turno-extra/BarraTurnoNormal";
import { BarraExtra } from "../../../../components/turno-extra/BarraExtra";
import type { EmployeeProfile } from "../utilidades/tipos";

interface TarjetaTurnoProps {
  profile: EmployeeProfile;
  turno: MiTurno | null;
  // Resultado de geometriaBarra(turno, ahora)
  barra: BarraTurno | null;
}

const TarjetaTurno = ({ profile, turno, barra }: TarjetaTurnoProps) => {
  return (
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
        {barra?.fueraDeTurno && !barra?.enExtra && (
          <span className="ml-auto inline-flex items-center gap-1.5 px-3 py-1.5 bg-amber-50 border border-amber-200 rounded-xl text-[10px] font-black uppercase tracking-widest text-amber-600">
            Fuera de turno
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

            {/* PISTA 1: BARRA DE TURNO + PISTA 2: BARRA DE EXTRA */}
            <div className="flex-1">
              <BarraTurnoNormal barra={barra} turno={turno} />
              <BarraExtra barra={barra} turno={turno} />
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
  );
};

export default TarjetaTurno;
