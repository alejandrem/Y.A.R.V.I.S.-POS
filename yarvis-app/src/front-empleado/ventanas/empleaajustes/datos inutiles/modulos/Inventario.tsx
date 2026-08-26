import { ICONO_CAJA } from "../../../../../components/ui";

export const inventarioSeparador = {
  id: "inventario",
  label: "INVENTARIO",
  icon: ICONO_CAJA,
  left: "52%",
};

export const InventarioIzq = () => (
  <div className="flex-1 bg-white border-r-[3px] border-neutral-900 p-10 flex flex-col relative overflow-hidden">
    <div className="absolute top-8 left-10 right-10 h-[1px] bg-neutral-900/10" />
    <div className="absolute top-12 left-10 right-10 h-[1px] bg-neutral-900/10" />

    <div className="flex-1 flex flex-col items-center justify-center text-center">
      <p className="font-mono text-[11px] font-black tracking-[0.35em] text-neutral-900 border-2 border-neutral-900 px-3 py-1.5 rounded-lg">
        MODULO 02 // STOCK
      </p>
      <h3 className="font-mono text-[36px] font-black tracking-[0.10em] text-neutral-900 leading-none mt-6">
        INVENTARIO
      </h3>
      <div className="w-20 h-[4px] bg-neutral-900 mt-6" />
      <p className="font-mono text-[11px] font-bold tracking-widest text-neutral-500 mt-4 max-w-[320px] leading-relaxed">
        Consulta, busca y verifica
        <br />
        stock sin editar.
      </p>
      <div className="mt-6 flex gap-2">
        <span className="font-mono text-[8px] font-black tracking-widest bg-neutral-900 text-white px-2 py-1 rounded">
          PAG. 05 — 06
        </span>
      </div>
    </div>

    <p className="font-mono text-[11px] font-black tracking-widest text-neutral-400 text-center">— 05 —</p>
    <div className="absolute bottom-0 left-0 w-6 h-6 border-t-[2px] border-r-[2px] border-neutral-900/20 rounded-tr-xl" />
  </div>
);

export const InventarioDer = () => (
  <div className="flex-1 bg-white p-8 flex flex-col relative overflow-hidden">
    <div className="absolute top-8 left-10 right-10 h-[1px] bg-neutral-900/10" />
    <div className="absolute top-12 left-10 right-10 h-[1px] bg-neutral-900/10" />

    <div className="flex-1 flex flex-col justify-center gap-4 mt-2">
      {/* Buscar */}
      <div className="flex gap-4 items-center bg-white border-2 border-neutral-900 rounded-2xl p-4">
        <div className="w-14 h-14 bg-white border-2 border-neutral-900 rounded-xl flex items-center justify-center shrink-0">
          <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="black" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
            <path d="M3 7V5a2 2 0 0 1 2-2h2" />
            <path d="M17 3h2a2 2 0 0 1 2 2v2" />
            <path d="M21 17v2a2 2 0 0 1-2 2h-2" />
            <path d="M7 21H5a2 2 0 0 1-2-2v-2" />
            <circle cx="12" cy="12" r="4" />
          </svg>
        </div>
        <div className="flex-1 min-w-0">
          <p className="font-mono text-[10px] font-black tracking-widest text-neutral-900">01 — BUSCAR</p>
          <p className="font-mono text-[9px] font-bold text-neutral-500 leading-relaxed mt-1">
            Barra superior con lupa. Filtra por nombre, código o categoría.
          </p>
        </div>
      </div>

      {/* Stock */}
      <div className="flex gap-4 items-center bg-white border-2 border-neutral-900 rounded-2xl p-4">
        <div className="w-14 h-14 bg-white border-2 border-neutral-900 rounded-xl flex items-center justify-center shrink-0">
          <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="black" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
            <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" />
            <path d="M3.27 6.96 12 12.01l8.73-5.05" />
            <path d="M12 22.08V12" />
          </svg>
        </div>
        <div className="flex-1 min-w-0">
          <p className="font-mono text-[10px] font-black tracking-widest text-neutral-900">02 — VER STOCK</p>
          <p className="font-mono text-[9px] font-bold text-neutral-500 leading-relaxed mt-1">
            Colores: verde = ok, rojo = bajo mínimo, barra = nivel visual.
          </p>
        </div>
      </div>

      {/* Alerta */}
      <div className="flex gap-4 items-center bg-neutral-900 rounded-2xl p-4 border-2 border-neutral-900">
        <div className="w-14 h-14 bg-white rounded-xl flex items-center justify-center shrink-0">
          <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="black" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
            <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
            <path d="M12 9v4" />
            <path d="M12 17h.01" />
          </svg>
        </div>
        <div className="flex-1 min-w-0">
          <p className="font-mono text-[10px] font-black tracking-widest text-white">03 — ALERTA BAJO</p>
          <p className="font-mono text-[9px] font-bold text-white/60 leading-relaxed mt-1">
            Si ves punto rojo parpadeando, avisa al admin para reponer.
          </p>
        </div>
      </div>
    </div>

    <p className="font-mono text-[11px] font-black tracking-widest text-neutral-400 text-center mt-2">— 06 —</p>
    <div className="absolute bottom-0 right-0 w-6 h-6 border-t-[2px] border-l-[2px] border-neutral-900/20 rounded-tl-xl" />
  </div>
);
