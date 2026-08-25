// ═══════════════════════════════════════════════════════════════════════════
// BUSCADOR DE PRODUCTOS — Sección de búsqueda del punto de venta.
// Tarea única: renderizar el input con spinner/icono, badge de estado IA,
// dropdown de resultados (×N en carrito, SIN STOCK, resaltado por teclado)
// y empty state "Sin resultados". 100% presentacional: recibe estado y
// callbacks por props.
// ═══════════════════════════════════════════════════════════════════════════

import { MorphIcon } from "morphicons/react";
import type { InventoryItem } from "../../../../services/inventario";
import type { CartItem } from "../hooks/useCarrito";
import {
  ICONO_BUSCAR, ICONO_EQUIS, ICONO_ESTRELLA,
  ICONO_CODIGO_BARRAS, ICONO_BOLSA,
} from "../../../../components/ui";

interface BuscadorProductosProps {
  searchQuery: string;
  onSearchChange: (value: string) => void;
  searchResults: InventoryItem[];
  selectedIndex: number;
  isSearching: boolean;
  showDropdown: boolean;
  iaStatus: "idle" | "loading" | "ready" | "error";
  iaSuggestion: string;
  cart: CartItem[];
  onKeyDown: (e: React.KeyboardEvent) => void;
  onSeleccionar: (item: InventoryItem) => void;
  onFocusInput: () => void;
  onLimpiar: () => void;
  searchRef: React.RefObject<HTMLDivElement | null>;
  inputRef: React.RefObject<HTMLInputElement | null>;
  dropdownRef: React.RefObject<HTMLDivElement | null>;
}

export default function BuscadorProductos({
  searchQuery,
  onSearchChange,
  searchResults,
  selectedIndex,
  isSearching,
  showDropdown,
  iaStatus,
  iaSuggestion,
  cart,
  onKeyDown,
  onSeleccionar,
  onFocusInput,
  onLimpiar,
  searchRef,
  inputRef,
  dropdownRef,
}: BuscadorProductosProps) {
  return (
    <div ref={searchRef} className="relative group">
      <div className="absolute inset-y-0 left-0 pl-5 flex items-center pointer-events-none z-10">
        {isSearching ? (
          <svg className="animate-spin h-5 w-5 text-neutral-400" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
            <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
            <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
          </svg>
        ) : (
          <MorphIcon icon={ICONO_BUSCAR} size={20} strokeWidth={2.5} spring="smooth" className="text-neutral-300 group-focus-within:text-neutral-900 transition-colors duration-200" />
        )}
      </div>
      <input
        ref={inputRef}
        type="text"
        value={searchQuery}
        onChange={(e) => onSearchChange(e.target.value)}
        onKeyDown={onKeyDown}
        onFocus={onFocusInput}
        placeholder="Escanea o busca por nombre, código o categoría..."
        className="w-full pl-14 pr-32 py-5 bg-white border-2 border-neutral-100 rounded-[1.75rem] shadow-xl shadow-neutral-100/60 text-base font-black text-neutral-900 placeholder:text-neutral-300 placeholder:font-bold placeholder:text-sm focus:outline-none focus:border-neutral-900 focus:ring-8 focus:ring-neutral-900/5 transition-all duration-300"
      />
      <div className="absolute right-4 top-1/2 -translate-y-1/2 flex items-center gap-2">
        {searchQuery && (
          <button
            onClick={onLimpiar}
            className="p-2 rounded-xl hover:bg-neutral-100 text-neutral-300 hover:text-neutral-900 transition-all"
            title="Limpiar búsqueda"
          >
            <MorphIcon icon={ICONO_EQUIS} size={16} strokeWidth={2.5} spring="snappy" reducedMotion="user" />
          </button>
        )}
        <span className={`inline-flex items-center gap-1.5 px-3 py-1.5 text-[9px] font-black rounded-xl border uppercase tracking-widest transition-all duration-300 ${
          iaStatus === "loading"
            ? "bg-amber-50 text-amber-600 border-amber-200"
            : iaStatus === "ready"
            ? "bg-emerald-50 text-emerald-600 border-emerald-200"
            : iaStatus === "error"
            ? "bg-neutral-50 text-neutral-400 border-neutral-200"
            : "bg-neutral-50 text-neutral-400 border-neutral-200 group-focus-within:border-neutral-900 group-focus-within:text-neutral-900"
        }`}>
          <MorphIcon icon={ICONO_ESTRELLA} size={11} strokeWidth={2.5} spring="snappy" reducedMotion="user" />
          {iaStatus === "loading" ? "BUSCANDO" : iaStatus === "ready" ? "IA OK" : "IA"}
        </span>
      </div>

      {/* ── DROPDOWN DE RESULTADOS ──────────────────────────────── */}
      {showDropdown && searchResults.length > 0 && (
        <div
          ref={dropdownRef}
          className="absolute top-full left-0 right-0 mt-3 bg-white border border-neutral-200 rounded-[2rem] shadow-2xl shadow-neutral-300/40 overflow-hidden z-50 animate-in fade-in slide-in-from-top-2 duration-200"
        >
          <div className="p-2.5 max-h-96 overflow-y-auto custom-scrollbar">
            {searchResults.map((item, idx) => {
              const inCart = cart.find((c) => c.id === item.id);
              return (
                <button
                  key={item.id || idx}
                  onClick={() => onSeleccionar(item)}
                  className={`w-full flex items-center gap-4 p-3.5 rounded-2xl text-left transition-all duration-150 ${
                    idx === selectedIndex
                      ? "bg-neutral-950 text-white scale-[1.01]"
                      : "hover:bg-neutral-50 text-neutral-900"
                  }`}
                >
                  <div className={`w-11 h-11 rounded-2xl flex items-center justify-center flex-shrink-0 ${
                    idx === selectedIndex ? "bg-white/15" : "bg-neutral-100"
                  }`}>
                    <MorphIcon icon={item.codigo_barras ? ICONO_CODIGO_BARRAS : ICONO_BOLSA} size={17} strokeWidth={2.2} spring="smooth" className={idx === selectedIndex ? "text-white" : "text-neutral-400"} />
                  </div>
                  <div className="flex-1 min-w-0">
                    <p className={`text-sm font-black truncate ${idx === selectedIndex ? "text-white" : "text-neutral-900"}`}>
                      {item.nombre}
                    </p>
                    <p className={`text-[10px] font-bold uppercase tracking-wider truncate mt-0.5 ${idx === selectedIndex ? "text-white/50" : "text-neutral-400"}`}>
                      {item.categoria || "Sin categoría"} · Stock: {item.stock}
                    </p>
                  </div>
                  <div className="text-right flex-shrink-0">
                    <p className={`text-sm font-black ${idx === selectedIndex ? "text-white" : "text-neutral-900"}`}>
                      ${item.precio_venta.toFixed(2)}
                    </p>
                    {inCart && (
                      <p className={`text-[9px] font-black uppercase tracking-widest ${idx === selectedIndex ? "text-emerald-300" : "text-emerald-600"}`}>
                        ×{inCart.cantidad} en carrito
                      </p>
                    )}
                  </div>
                  {item.stock <= 0 && (
                    <span className={`text-[8px] font-black px-2 py-1 rounded-lg uppercase tracking-widest ${idx === selectedIndex ? "bg-red-500/30 text-red-200" : "bg-red-50 text-red-500"}`}>
                      SIN STOCK
                    </span>
                  )}
                </button>
              );
            })}
          </div>
          {iaStatus === "ready" && (
            <div className="px-5 py-3 bg-emerald-50 border-t border-emerald-100 flex items-center gap-2.5">
              <MorphIcon icon={ICONO_ESTRELLA} size={13} strokeWidth={2.5} spring="smooth" className="text-emerald-500" />
              <span className="text-[11px] font-black text-emerald-700">{iaSuggestion}</span>
            </div>
          )}
        </div>
      )}

      {showDropdown && searchQuery && searchResults.length === 0 && !isSearching && (
        <div className="absolute top-full left-0 right-0 mt-3 bg-white border border-neutral-200 rounded-[2rem] shadow-2xl shadow-neutral-300/40 overflow-hidden z-50 animate-in fade-in slide-in-from-top-2 duration-200">
          <div className="py-10 text-center">
            <div className="w-14 h-14 bg-neutral-950 rounded-2xl flex items-center justify-center mx-auto mb-4 shadow-lg">
              <MorphIcon icon={ICONO_BUSCAR} size={22} strokeWidth={2} spring="smooth" className="text-white" />
            </div>
            <p className="text-xs font-black uppercase tracking-widest text-neutral-400">Sin resultados para "{searchQuery}"</p>
            <p className="text-[10px] font-bold text-neutral-300 mt-1.5">Intenta con otro nombre o código</p>
          </div>
        </div>
      )}
    </div>
  );
}
