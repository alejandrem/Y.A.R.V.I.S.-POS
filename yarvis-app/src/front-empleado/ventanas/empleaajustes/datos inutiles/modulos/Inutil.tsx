import { ICONO_USUARIO } from "../../../../../components/ui";

export const inutilSeparador = {
  id: "inutil",
  label: "INUTIL",
  icon: ICONO_USUARIO,
  left: "86%",
};

export const InutilIzq = () => (
  <div className="flex-1 bg-white border-r-[3px] border-neutral-900 p-10 flex flex-col relative overflow-hidden">
    <div className="absolute top-8 left-10 right-10 h-[1px] bg-neutral-900/10" />
    <div className="absolute top-12 left-10 right-10 h-[1px] bg-neutral-900/10" />
    <div className="flex-1 flex flex-col items-center justify-center text-center">
      <p className="font-mono text-[11px] font-black tracking-[0.35em] text-neutral-900 border-2 border-neutral-900 px-3 py-1.5 rounded-lg">
        MODULO 07 // CREADOR
      </p>
      <h3 className="font-mono text-[38px] font-black tracking-[0.12em] text-neutral-900 leading-none mt-6">INUTIL</h3>
      <div className="w-20 h-[4px] bg-neutral-900 mt-6" />
      <p className="font-mono text-[13px] font-black tracking-[0.12em] text-neutral-900 mt-4">ALEJANDRO ELIOSA MORALES</p>
      <p className="font-mono text-[11px] font-bold tracking-widest text-neutral-500 mt-4 max-w-[320px] leading-relaxed">
        Quien hizo este POS
        <br />y donde encontrarlo.
      </p>
      <div className="mt-6 flex gap-2">
        <span className="font-mono text-[8px] font-black tracking-widest bg-neutral-900 text-white px-2 py-1 rounded">PAG. 15 — 16</span>
      </div>
    </div>
    <p className="font-mono text-[11px] font-black tracking-widest text-neutral-400 text-center">— 15 —</p>
    <div className="absolute bottom-0 left-0 w-6 h-6 border-t-[2px] border-r-[2px] border-neutral-900/20 rounded-tr-xl" />
  </div>
);

export const InutilDer = () => (
  <div className="flex-1 bg-white p-8 flex flex-col relative overflow-hidden">
    <div className="absolute top-8 left-10 right-10 h-[1px] bg-neutral-900/10" />
    <div className="absolute top-12 left-10 right-10 h-[1px] bg-neutral-900/10" />
    <div className="flex-1 flex flex-col justify-center gap-3 mt-2">
      <div className="flex gap-3 items-center bg-white border-2 border-neutral-900 rounded-2xl p-3">
        <div className="w-10 h-10 bg-white border-2 border-neutral-900 rounded-xl flex items-center justify-center shrink-0">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="black" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
            <path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72c.127.96.361 1.903.7 2.81a2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45c.907.339 1.85.573 2.81.7A2 2 0 0 1 22 16.92z" />
          </svg>
        </div>
        <div className="flex-1 min-w-0">
          <p className="font-mono text-[9px] font-black tracking-widest text-neutral-500">TELEFONO</p>
          <p className="font-mono text-[11px] font-black text-neutral-900">+52 246 295 295 5734</p>
        </div>
      </div>
      <div className="flex gap-3 items-center bg-white border-2 border-neutral-900 rounded-2xl p-3">
        <div className="w-10 h-10 bg-white border-2 border-neutral-900 rounded-xl flex items-center justify-center shrink-0">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="black" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
            <path d="M4 4h16a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2z" />
            <polyline points="22 6 12 13 2 6" />
          </svg>
        </div>
        <div className="flex-1 min-w-0">
          <p className="font-mono text-[9px] font-black tracking-widest text-neutral-500">CORREO</p>
          <p className="font-mono text-[11px] font-black text-neutral-900 truncate">alejandroeliosa28@gmail.com</p>
        </div>
      </div>
      <div className="flex gap-3 items-center bg-white border-2 border-neutral-900 rounded-2xl p-3">
        <div className="w-10 h-10 bg-white border-2 border-neutral-900 rounded-xl flex items-center justify-center shrink-0">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="black" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
            <rect x="2" y="2" width="20" height="20" rx="5" />
            <circle cx="12" cy="12" r="5" />
            <circle cx="17.5" cy="6.5" r="1.5" />
          </svg>
        </div>
        <div className="flex-1 min-w-0">
          <p className="font-mono text-[9px] font-black tracking-widest text-neutral-500">INSTAGRAM</p>
          <p className="font-mono text-[11px] font-black text-neutral-900">@i.amtrak</p>
        </div>
      </div>
      <div className="flex gap-3 items-center bg-white border-2 border-neutral-900 rounded-2xl p-3">
        <div className="w-10 h-10 bg-white border-2 border-neutral-900 rounded-xl flex items-center justify-center shrink-0">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="black" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
            <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22" />
          </svg>
        </div>
        <div className="flex-1 min-w-0">
          <p className="font-mono text-[9px] font-black tracking-widest text-neutral-500">GITHUB</p>
          <p className="font-mono text-[11px] font-black text-neutral-900 truncate">github.com/alejandrem</p>
        </div>
      </div>
      <div className="flex gap-3 items-center bg-neutral-900 rounded-2xl p-3 border-2 border-neutral-900">
        <div className="w-10 h-10 bg-white rounded-xl flex items-center justify-center shrink-0">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="black" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
            <polyline points="14 2 14 8 20 8" />
          </svg>
        </div>
        <div className="flex-1 min-w-0">
          <p className="font-mono text-[9px] font-black tracking-widest text-white/60">REPO DEL PROYECTO</p>
          <p className="font-mono text-[11px] font-black text-white truncate">github.com/alejandrem/Y.A.R.V.I.S.-POS</p>
        </div>
      </div>
    </div>
    <p className="font-mono text-[11px] font-black tracking-widest text-neutral-400 text-center mt-2">— 16 —</p>
    <div className="absolute bottom-0 right-0 w-6 h-6 border-t-[2px] border-l-[2px] border-neutral-900/20 rounded-tl-xl" />
  </div>
);
