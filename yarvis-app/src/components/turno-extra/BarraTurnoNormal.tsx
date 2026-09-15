// ═══════════════════════════════════════════════════════════════════════════
// TURNO-EXTRA · BARRA DE TURNO (pista 1) — Ventana oficial [entrada,
// salida]: extremos SIEMPRE el horario que puso el admin. La bolita
// blanca marca a qué hora entró el empleado; si entró antes, la bolita
// se recorre hacia atrás. El negro es trabajo real (arranca en su
// llegada, no pinta lo no trabajado). Nunca lleva verde.
// ═══════════════════════════════════════════════════════════════════════════

import { fmtHM } from "./index";
import type { BarraTurno, MiTurno } from "./index";

interface BarraTurnoNormalProps {
  barra: BarraTurno;
  turno: MiTurno | null;
}

export function BarraTurnoNormal({ barra, turno }: BarraTurnoNormalProps) {
  return (
    <>
      <p className="text-[8px] font-black text-neutral-400 uppercase tracking-widest mb-1.5">Turno</p>
      <div className="relative h-4 bg-neutral-100 rounded-full border border-neutral-200 overflow-visible">
        {/* Trabajo dentro del horario (negro, arranca en la bolita) */}
        <div
          className="absolute inset-y-0 bg-neutral-900 rounded-full transition-all duration-700 ease-out"
          style={{ left: `${barra.tTrabIniPct}%`, width: `${Math.max(0, barra.tTrabFinPct - barra.tTrabIniPct)}%` }}
        />
        {/* Palito verde: hora de entrada oficial (solo si vino extra-temprano) */}
        {barra.preExtraActivo && (
          <div
            className="absolute -top-0.5 w-1.5 h-5 bg-emerald-500 rounded-full shadow-sm z-10"
            style={{ left: `calc(${barra.tIniPct}% - 3px)` }}
            title={`Entrada ${fmtHM(barra.inicio)} — lo de antes cuenta como extra`}
          />
        )}
        {/* ● Bolita: PRIMER LOGIN del día (llegada real) */}
        {barra.tLoginPct !== null && (
          <div
            className="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-3.5 h-3.5 bg-white border-[3px] border-neutral-900 rounded-full shadow-md z-10"
            style={{ left: `${barra.tLoginPct}%` }}
            title={`Primer login: ${turno?.primer_login ?? ""}`}
          />
        )}
      </div>

      <div className="flex justify-between mt-2">
        <span className="text-[8px] font-black text-neutral-300 uppercase">
                  {turno?.primer_login
                    ? barra.fueraDeTurno
                      ? `Llegaste ${turno.primer_login} · turno nocturno (cuenta como extra)`
                      : `Llegaste ${turno.primer_login}${
                        barra.minutosTarde > 0
                          ? ` · ${barra.minutosTarde} min tarde`
                          : barra.llegoPuntual
                            ? " · ¡Felicidades, llegaste puntual!"
                            : ` · ${barra.minutosTemprano} min temprano (${barra.extraPreMinutos} extra)`
                      }`
                    : "Sin registro de entrada"}
        </span>
        <span className={`text-[8px] font-black uppercase ${barra.enExtra ? "text-emerald-500" : "text-neutral-300"}`}>
          {barra.enExtra ? `Progreso: ${Math.round(barra.tTrabFinPct)}% + extra` : `Progreso: ${Math.round(barra.tTrabFinPct)}%`}
        </span>
      </div>
    </>
  );
}
