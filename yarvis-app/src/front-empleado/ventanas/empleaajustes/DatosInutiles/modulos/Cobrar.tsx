import { ICONO_BILLETE } from "../../../../../components/ui";

// Separador de este modulo - se renderiza en el lomo superior del libro
export const cobrarSeparador = {
  id: "cobrar",
  label: "COBRAR",
  icon: ICONO_BILLETE,
  left: "44%",
};

// Pagina izquierda del spread COBRAR - titulo del modulo
export const CobrarIzq = () => (
  <div className="flex-1 bg-white border-r-[3px] border-neutral-900 p-10 flex flex-col relative overflow-hidden">
    <div className="absolute top-8 left-10 right-10 h-[1px] bg-neutral-900/10" />
    <div className="absolute top-12 left-10 right-10 h-[1px] bg-neutral-900/10" />

    <div className="flex-1 flex flex-col items-center justify-center text-center">
      <p className="font-mono text-[11px] font-black tracking-[0.35em] text-neutral-900 border-2 border-neutral-900 px-3 py-1.5 rounded-lg">
        MODULO 01 // COBRANZAS
      </p>
      <h3 className="font-mono text-[38px] font-black tracking-[0.12em] text-neutral-900 leading-none mt-6">
        COMO
      </h3>
      <h3 className="font-mono text-[38px] font-black tracking-[0.12em] text-neutral-900 leading-none">
        COBRAR
      </h3>
      <div className="w-20 h-[4px] bg-neutral-900 mt-6" />
      <p className="font-mono text-[11px] font-bold tracking-widest text-neutral-500 mt-4 max-w-[320px] leading-relaxed">
        Del carrito al ticket en 3 pasos.
        <br />F5 es tu mejor amigo.
      </p>
      <div className="mt-6 flex gap-2">
        <span className="font-mono text-[8px] font-black tracking-widest bg-neutral-900 text-white px-2 py-1 rounded">
          PAG. 03 — 04
        </span>
      </div>
    </div>

    <p className="font-mono text-[11px] font-black tracking-widest text-neutral-400 text-center">— 03 —</p>
    <div className="absolute bottom-0 left-0 w-6 h-6 border-t-[2px] border-r-[2px] border-neutral-900/20 rounded-tr-xl" />
  </div>
);

// Pagina derecha del spread COBRAR - pasos con palabras + SVG
export const CobrarDer = () => (
  <div className="flex-1 bg-white p-8 flex flex-col relative overflow-hidden">
    <div className="absolute top-8 left-10 right-10 h-[1px] bg-neutral-900/10" />
    <div className="absolute top-12 left-10 right-10 h-[1px] bg-neutral-900/10" />

    <div className="flex-1 flex flex-col justify-center gap-4 mt-2">
      {/* Paso 1 */}
      <div className="flex gap-4 items-center bg-white border-2 border-neutral-900 rounded-2xl p-4">
        <div className="w-14 h-14 bg-white border-2 border-neutral-900 rounded-xl flex items-center justify-center shrink-0">
          <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="black" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="11" cy="11" r="7" />
            <path d="M21 21l-3.2-3.2" />
            <path d="M8 11h6" />
          </svg>
        </div>
        <div className="flex-1 min-w-0">
          <p className="font-mono text-[10px] font-black tracking-widest text-neutral-900">01 — BUSCAR PRODUCTO</p>
          <p className="font-mono text-[9px] font-bold text-neutral-500 leading-relaxed mt-1">
            Teclea o escanea (F7). El buscador filtra por nombre, código o categoría.
          </p>
        </div>
      </div>

      {/* Paso 2 */}
      <div className="flex gap-4 items-center bg-white border-2 border-neutral-900 rounded-2xl p-4">
        <div className="w-14 h-14 bg-white border-2 border-neutral-900 rounded-xl flex items-center justify-center shrink-0">
          <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="black" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
            <path d="M6 6h15l-1.5 9h-13z" />
            <path d="M6 6l-2-2H2" />
            <circle cx="9" cy="20" r="1.5" />
            <circle cx="18" cy="20" r="1.5" />
            <path d="M9 12h6" />
            <path d="M12 9v6" />
          </svg>
        </div>
        <div className="flex-1 min-w-0">
          <p className="font-mono text-[10px] font-black tracking-widest text-neutral-900">02 — AGREGAR AL CARRITO</p>
          <p className="font-mono text-[9px] font-bold text-neutral-500 leading-relaxed mt-1">
            Click en el producto o Enter. Ajusta cantidad con + / -.
          </p>
        </div>
      </div>

      {/* Paso 3 */}
      <div className="flex gap-4 items-center bg-neutral-900 rounded-2xl p-4 border-2 border-neutral-900">
        <div className="w-14 h-14 bg-white rounded-xl flex items-center justify-center shrink-0">
          <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="black" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
            <rect x="2" y="6" width="20" height="12" rx="2" />
            <circle cx="12" cy="12" r="3" />
            <path d="M6 9h.01 M18 15h.01" />
          </svg>
        </div>
        <div className="flex-1 min-w-0">
          <p className="font-mono text-[10px] font-black tracking-widest text-white">03 — COBRAR [F5]</p>
          <p className="font-mono text-[9px] font-bold text-white/60 leading-relaxed mt-1">
            Presiona F5 o el botón negro. Elige efectivo, tarjeta o transferencia.
          </p>
        </div>
        <span className="font-mono text-[8px] font-black bg-white text-neutral-900 px-2 py-1 rounded-lg">F5</span>
      </div>
    </div>

    <p className="font-mono text-[11px] font-black tracking-widest text-neutral-400 text-center mt-2">— 04 —</p>
    <div className="absolute bottom-0 right-0 w-6 h-6 border-t-[2px] border-l-[2px] border-neutral-900/20 rounded-tl-xl" />
  </div>
);
