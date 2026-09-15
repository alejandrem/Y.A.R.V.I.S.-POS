// ═══════════════════════════════════════════════════════════════════════════
// TURNO-EXTRA · BARRA DE EXTRA (pista 2) — Llegada → salida oficial →
// ahora/corte Z. Solo se pinta cuando hay extra REAL (enExtra): el verde
// sale únicamente en el tramo genuino. La bolita verde marca la salida
// oficial (frontera del extra) y la negra la llegada real. El "ahora"
// avanza solo; al hacer corte Z, ultimo_login lo congela como salida real.
// ═══════════════════════════════════════════════════════════════════════════

import { fmtHM, type BarraTurno, type MiTurno } from "./index";

interface BarraExtraProps {
  barra: BarraTurno;
  turno: MiTurno | null;
}

export function BarraExtra({ barra, turno }: BarraExtraProps) {
  if (!barra.enExtra) return null;
  return (
    <div className="mt-4">
      <p className="text-[8px] font-black text-emerald-500 uppercase tracking-widest mb-1.5">
        {barra.fueraDeTurno
          ? `Extra nocturno · ${turno?.primer_login ?? ""} → hasta tu corte Z`
          : `Extra · ${turno?.primer_login ?? ""} → ${fmtHM(barra.fin)} → ${turno?.ultimo_login ?? "ahora"}`}
      </p>
      <div className="relative h-4 bg-neutral-100 rounded-full border border-emerald-200 overflow-visible">
        {/* Extra tempranero (verde antes de la entrada oficial) */}
        {barra.preExtraActivo && (
          <div
            className="absolute inset-y-0 bg-emerald-400 transition-all duration-700 ease-out"
            style={{ left: `${barra.xLoginPct}%`, width: `${Math.max(0, barra.xIniPct - barra.xLoginPct)}%`, borderRadius: "999px 0 0 999px" }}
          />
        )}
        {/* Turno trabajado (negro, referencia; de noche queda bajo el verde) */}
        <div
          className="absolute inset-y-0 bg-neutral-900 rounded-full transition-all duration-700 ease-out"
          style={{ left: `${barra.xNegroIzq}%`, width: `${barra.xNegroAncho}%` }}
        />
        {/* Extra genuino (verde: posturno o tramo nocturno completo) */}
        {(barra.enExtraPost || barra.fueraDeTurno) && (
          <div
            className="absolute inset-y-0 bg-emerald-500 transition-all duration-700 ease-out"
            style={{ left: `${barra.xVerdeIzq}%`, width: `${barra.xVerdeAncho}%`, borderRadius: barra.fueraDeTurno ? "999px" : "0 999px 999px 0" }}
          />
        )}
        {/* ● Bolita en la llegada real */}
        <div
          className="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-3.5 h-3.5 bg-white border-[3px] border-neutral-900 rounded-full shadow-md z-10"
          style={{ left: `${barra.xLoginPct}%` }}
          title={`Llegaste ${turno?.primer_login ?? ""}`}
        />
        {/* ● Bolita blanca en la salida oficial (frontera del extra).
            De noche no hay frontera: todo el tramo es nocturno. */}
        {!barra.fueraDeTurno && (
          <div
            className="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-3.5 h-3.5 bg-white border-[3px] border-emerald-500 rounded-full shadow-md z-10"
            style={{ left: `${barra.xFinPct}%` }}
            title="Fin de tu horario — desde aquí cuenta como extra"
          />
        )}
      </div>
    </div>
  );
}
