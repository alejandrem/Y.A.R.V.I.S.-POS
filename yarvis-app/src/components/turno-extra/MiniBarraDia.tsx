// ═══════════════════════════════════════════════════════════════════════════
// TURNO-EXTRA · MINI BARRA HISTÓRICA — verde pre + negro trabajo + verde
// post por día pasado. La usan el perfil del empleado y el detalle de
// personal del admin. Los números ya vienen corregidos del backend
// (asistencia.rs::calcular_extras): el verde solo sale con extra real.
// De noche la ventana es fija (8h de referencia): si terminara en la
// salida, el verde siempre daría 100%.
// ═══════════════════════════════════════════════════════════════════════════

import type { DiaExtra } from "./tipos";

/** Escala fija de la mini nocturna (8h de referencia). */
export const ESCALA_NOCTURNA_MIN = 480;

const mDe = (t: string): number => {
  const p = t.split(":").map(Number);
  return p[0] * 60 + p[1];
};

export interface GeometriaMini {
  ini: number;
  fin: number;
  login: number;
  salida: number;
  finOficial: number;
  nocturno: boolean;
  ventanaIni: number;
  ventanaFin: number;
}

/** Ventana y puntos de la mini barra para un día dado (puro, testeable). */
export function geometriaMiniBarra(d: DiaExtra): GeometriaMini {
  const ini = mDe(d.entrada_oficial);
  let fin = mDe(d.salida_oficial);
  if (fin <= ini) fin += 24 * 60; // nocturno cruza medianoche
  const login = mDe(d.primer_login);
  let salida = Math.max(mDe(d.ultimo_login), login);
  if (salida <= login) salida = login + 1;
  // Regla B: si entró de noche tras el fin, la ventana arranca en la
  // llegada (bolita a la izquierda) y termina en escala fija.
  const nocturno = login > fin;
  const ventanaIni = d.extra_pre_min > 0 || nocturno ? login : ini;
  const ventanaFin = nocturno ? login + ESCALA_NOCTURNA_MIN : Math.max(fin, salida);
  return { ini, fin, login, salida, finOficial: fin, nocturno, ventanaIni, ventanaFin };
}

/** Mini barra histórica: verde pre + negro trabajo + verde post. */
export function MiniBarraDia({ d }: { d: DiaExtra }) {
  const g = geometriaMiniBarra(d);
  const span = Math.max(1, g.ventanaFin - g.ventanaIni);
  const pct = (m: number) => Math.min(100, Math.max(0, ((m - g.ventanaIni) / span) * 100));
  const verdeIzq = g.nocturno ? pct(g.login) : pct(g.finOficial);

  return (
    <div className="relative flex-1 h-2.5 bg-neutral-100 rounded-full overflow-visible min-w-[80px]">
      {d.extra_pre_min > 0 && (
        <div className="absolute inset-y-0 bg-emerald-400" style={{ left: `${pct(g.login)}%`, width: `${pct(g.ini) - pct(g.login)}%`, borderRadius: "999px 0 0 999px" }} />
      )}
      <div className="absolute inset-y-0 bg-neutral-950 rounded-full transition-all duration-700 ease-out" style={g.nocturno ? { left: `${pct(g.login)}%`, width: "0.5%" } : { left: `${pct(g.ini)}%`, width: `${Math.max(0.5, pct(Math.min(g.salida, g.finOficial)) - pct(g.ini))}%` }} />
      {d.extra_post_min > 0 && (
        <div className="absolute inset-y-0 bg-emerald-500" style={{ left: `${verdeIzq}%`, width: `${Math.max(0, pct(g.salida) - verdeIzq)}%`, borderRadius: g.nocturno ? "999px" : "0 999px 999px 0" }} />
      )}
      <div className="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-2 h-2 bg-white border-2 border-neutral-900 rounded-full shadow-sm z-10" style={{ left: `${pct(g.login)}%` }} title={`Entró ${d.primer_login}`} />
      {d.extra_pre_min > 0 && (
        <div className="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-2 h-2 bg-white border-2 border-neutral-900 rounded-full shadow-sm z-10" style={{ left: `${pct(g.ini)}%` }} title={`Entrada oficial ${d.entrada_oficial}`} />
      )}
      {d.extra_post_min > 0 && !g.nocturno && (
        <div className="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-2 h-2 bg-white border-2 border-emerald-500 rounded-full shadow-sm z-10" style={{ left: `${pct(g.finOficial)}%` }} title={`Salida oficial ${d.salida_oficial}`} />
      )}
    </div>
  );
}
