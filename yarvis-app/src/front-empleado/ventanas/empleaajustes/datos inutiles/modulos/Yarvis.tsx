import { ICONO_ROBOT } from "../../../../../components/ui";

export const yarvisSeparador = {
  id: "yarvis",
  label: "Y.A.R.V.I.S.",
  icon: ICONO_ROBOT,
  left: "79%",
};

export const YarvisIzq = () => (
  <div className="flex-1 bg-white border-r-[3px] border-neutral-900 p-10 flex flex-col relative overflow-hidden">
    <div className="absolute top-8 left-10 right-10 h-[1px] bg-neutral-900/10" />
    <div className="absolute top-12 left-10 right-10 h-[1px] bg-neutral-900/10" />
    <div className="flex-1 flex flex-col items-center justify-center text-center">
      <p className="font-mono text-[11px] font-black tracking-[0.35em] text-neutral-900 border-2 border-neutral-900 px-3 py-1.5 rounded-lg">
        MODULO 06 // IA LOCAL
      </p>
      <h3 className="font-mono text-[34px] font-black tracking-[0.10em] text-neutral-900 leading-none mt-6">Y.A.R.V.I.S.</h3>
      <div className="w-20 h-[4px] bg-neutral-900 mt-6" />
      <p className="font-mono text-[11px] font-bold tracking-widest text-neutral-500 mt-4 max-w-[320px] leading-relaxed">
        Tu asistente sin internet.
        <br />
        Pregunta lo que quieras.
      </p>
      <div className="mt-6 flex gap-2">
        <span className="font-mono text-[8px] font-black tracking-widest bg-neutral-900 text-white px-2 py-1 rounded">PAG. 13 — 14</span>
      </div>
    </div>
    <p className="font-mono text-[11px] font-black tracking-widest text-neutral-400 text-center">— 13 —</p>
    <div className="absolute bottom-0 left-0 w-6 h-6 border-t-[2px] border-r-[2px] border-neutral-900/20 rounded-tr-xl" />
  </div>
);

export const YarvisDer = () => (
  <div className="flex-1 bg-white p-8 flex flex-col relative overflow-hidden">
    <div className="absolute top-8 left-10 right-10 h-[1px] bg-neutral-900/10" />
    <div className="absolute top-12 left-10 right-10 h-[1px] bg-neutral-900/10" />
    <div className="flex-1 flex flex-col justify-center gap-4 mt-2">
      <div className="flex gap-4 items-center bg-white border-2 border-neutral-900 rounded-2xl p-4">
        <div className="w-14 h-14 bg-white border-2 border-neutral-900 rounded-xl flex items-center justify-center shrink-0">
          <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="black" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
            <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
            <circle cx="12" cy="12" r="1" />
            <circle cx="9" cy="12" r="1" />
            <circle cx="15" cy="12" r="1" />
          </svg>
        </div>
        <div className="flex-1 min-w-0">
          <p className="font-mono text-[10px] font-black tracking-widest text-neutral-900">01 — PREGUNTA LIBRE</p>
          <p className="font-mono text-[9px] font-bold text-neutral-500 leading-relaxed mt-1">
            &quot;¿Cuánto vendí hoy?&quot; &quot;¿Qué producto falta?&quot; — sin internet, con tu DB.
          </p>
        </div>
      </div>
      <div className="flex gap-4 items-center bg-white border-2 border-neutral-900 rounded-2xl p-4">
        <div className="w-14 h-14 bg-white border-2 border-neutral-900 rounded-xl flex items-center justify-center shrink-0">
          <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="black" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
            <rect x="4" y="4" width="16" height="16" rx="2" />
            <rect x="9" y="9" width="6" height="6" />
            <path d="M9 1v3M15 1v3M9 20v3M15 20v3M1 9h3M1 15h3M20 9h3M20 15h3" />
          </svg>
        </div>
        <div className="flex-1 min-w-0">
          <p className="font-mono text-[10px] font-black tracking-widest text-neutral-900">02 — HERRAMIENTAS INCLUIDAS</p>
          <p className="font-mono text-[9px] font-bold text-neutral-500 leading-relaxed mt-1">
            Qwen 1.5B Coder usa 10 herramientas reales. No inventa cifras.
          </p>
        </div>
      </div>
      <div className="flex gap-4 items-center bg-neutral-900 rounded-2xl p-4 border-2 border-neutral-900">
        <div className="w-14 h-14 bg-white rounded-xl flex items-center justify-center shrink-0">
          <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="black" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
            <path d="M12 8V4H8" />
            <rect x="4" y="8" width="16" height="12" rx="2" />
            <path d="M9 13v2" />
            <path d="M15 13v2" />
          </svg>
        </div>
        <div className="flex-1 min-w-0">
          <p className="font-mono text-[10px] font-black tracking-widest text-white">03 — OFFLINE</p>
          <p className="font-mono text-[9px] font-bold text-white/60 leading-relaxed mt-1">
            Funciona sin nube. Si hay internet, usa Gemini/OpenCode como refuerzo.
          </p>
        </div>
      </div>
    </div>
    <p className="font-mono text-[11px] font-black tracking-widest text-neutral-400 text-center mt-2">— 14 —</p>
    <div className="absolute bottom-0 right-0 w-6 h-6 border-t-[2px] border-l-[2px] border-neutral-900/20 rounded-tl-xl" />
  </div>
);
