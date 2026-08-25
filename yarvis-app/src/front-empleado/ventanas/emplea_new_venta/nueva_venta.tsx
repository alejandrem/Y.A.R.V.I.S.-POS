// ═══════════════════════════════════════════════════════════════════════════
// NUEVA VENTA — Punto de venta del operador.
// Tarea única: búsqueda de productos (local + IA por similitud), carrito y
// cobro. La lógica no cambia: el carrito vive solo en memoria y el inventario
// se descuenta ÚNICAMENTE al confirmar el cobro (F5 → modal → Confirmar).
// ═══════════════════════════════════════════════════════════════════════════

import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { MorphIcon } from "morphicons/react";
import { obtenerInventario, type InventoryItem } from "../../../services/inventario";
import ModalVenta from "./modalventa";
import ModalTicket from "./modalticket";
import {
  BotonAnimado,
  ICONO_BUSCAR, ICONO_EQUIS, ICONO_MAS, ICONO_RESTA,
  ICONO_CARRITO, ICONO_BILLETE, ICONO_ESTRELLA, ICONO_ESCANER,
  ICONO_CODIGO_BARRAS, ICONO_BOLSA,
} from "../../../components/ui";

const nuevaVentaNav = {
  id: "nueva_venta",
  label: "NUEVA VENTA",
  icon: (
    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="9" cy="21" r="1" />
      <circle cx="20" cy="21" r="1" />
      <path d="M1 1h4l2.68 13.39a2 2 0 0 0 2 1.61h9.72a2 2 0 0 0 2-1.61L23 6H6" />
    </svg>
  ),
};

interface CartItem {
  id?: number;
  nombre: string;
  precio_venta: number;
  cantidad: number;
  stock: number;
}

interface NuevaVentaProps {
  activeTab: string;
}

export default function NuevaVenta({ activeTab }: NuevaVentaProps) {
  const [inventory, setInventory] = useState<InventoryItem[]>([]);
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<InventoryItem[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [showDropdown, setShowDropdown] = useState(false);
  const [cart, setCart] = useState<CartItem[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(-1);
  const [iaStatus, setIaStatus] = useState<"idle" | "loading" | "ready" | "error">("idle");
  const [iaSuggestion, setIaSuggestion] = useState("");
  const [showModalVenta, setShowModalVenta] = useState(false);
  const [showModalTicket, setShowModalTicket] = useState(false);
  const [lastVentaId, setLastVentaId] = useState(0);
  const [lastTicketNumber, setLastTicketNumber] = useState(0);
  const [lastMontoEfectivo, setLastMontoEfectivo] = useState(0);
  const [lastMontoTarjeta, setLastMontoTarjeta] = useState(0);
  const [lastMontoTransferencia, setLastMontoTransferencia] = useState(0);
  const searchRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (activeTab === "nueva_venta") {
      loadInventory();
    }
  }, [activeTab]);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (searchRef.current && !searchRef.current.contains(e.target as Node)) {
        setShowDropdown(false);
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  useEffect(() => {
    const handleF5 = (e: KeyboardEvent) => {
      if (e.key === "F5" && cart.length > 0 && !showModalVenta && !showModalTicket) {
        e.preventDefault();
        setShowModalVenta(true);
      }
    };
    window.addEventListener("keydown", handleF5);
    return () => window.removeEventListener("keydown", handleF5);
  }, [cart, showModalVenta, showModalTicket]);

  const loadInventory = async () => {
    try {
      const items = await obtenerInventario();
      setInventory(items);
    } catch (error) {
      console.error("Error al cargar inventario:", error);
    }
  };

  const searchProducts = useCallback(
    async (query: string) => {
      if (!query.trim()) {
        setSearchResults([]);
        setShowDropdown(false);
        return;
      }

      setIsSearching(true);
      setShowDropdown(true);

      const q = query.toLowerCase().trim();
      const localResults = inventory.filter(
        (item) =>
          item.nombre.toLowerCase().includes(q) ||
          (item.codigo_barras && item.codigo_barras.toLowerCase().includes(q)) ||
          (item.categoria && item.categoria.toLowerCase().includes(q))
      );

      setSearchResults(localResults.slice(0, 8));
      setSelectedIndex(-1);
      setIsSearching(false);

      if (localResults.length === 0 && query.length > 2) {
        setIaStatus("loading");
        try {
          const aiResults = await invoke<{ id: number; contenido: string; score: number }[]>("buscar_producto_similar", { query, topK: 5 });
          if (aiResults && aiResults.length > 0) {
            const matched = aiResults
              .map((r) => inventory.find((p) => p.id === r.id))
              .filter(Boolean) as InventoryItem[];
            if (matched.length > 0) {
              setSearchResults(matched);
              setIaSuggestion(`Encontré "${matched[0].nombre}" por similitud`);
              setIaStatus("ready");
            } else {
              setIaStatus("error");
            }
          } else {
            setIaStatus("error");
          }
        } catch {
          setIaStatus("error");
        }
      } else {
        setIaStatus("idle");
        setIaSuggestion("");
      }
    },
    [inventory]
  );

  useEffect(() => {
    const debounce = setTimeout(() => {
      searchProducts(searchQuery);
    }, 200);
    return () => clearTimeout(debounce);
  }, [searchQuery, searchProducts]);

  const addToCart = (product: InventoryItem) => {
    setCart((prev) => {
      const existing = prev.find((item) => item.id === product.id);
      if (existing) {
        if (existing.cantidad >= product.stock) return prev;
        return prev.map((item) =>
          item.id === product.id ? { ...item, cantidad: item.cantidad + 1 } : item
        );
      }
      return [
        ...prev,
        {
          id: product.id,
          nombre: product.nombre,
          precio_venta: product.precio_venta,
          cantidad: 1,
          stock: product.stock,
        },
      ];
    });
    setSearchQuery("");
    setShowDropdown(false);
    inputRef.current?.focus();
  };

  const updateQuantity = (id: number | undefined, delta: number) => {
    if (id === undefined) return;
    setCart((prev) =>
      prev
        .map((item) => {
          if (item.id === id) {
            const newQty = item.cantidad + delta;
            if (newQty <= 0) return null;
            if (newQty > item.stock) return item;
            return { ...item, cantidad: newQty };
          }
          return item;
        })
        .filter(Boolean) as CartItem[]
    );
  };

  const removeFromCart = (id: number | undefined) => {
    if (id === undefined) return;
    setCart((prev) => prev.filter((item) => item.id !== id));
  };

  const cartTotal = cart.reduce((acc, item) => acc + item.precio_venta * item.cantidad, 0);

  const handleAbrirCobro = () => {
    if (cart.length === 0) return;
    setShowModalVenta(true);
  };

  const handleVentaCompletada = (ventaId: number, ticketNumber: number, efectivo: number, tarjeta: number, transferencia: number) => {
    setLastVentaId(ventaId);
    setLastTicketNumber(ticketNumber);
    setLastMontoEfectivo(efectivo);
    setLastMontoTarjeta(tarjeta);
    setLastMontoTransferencia(transferencia);
    setShowModalVenta(false);
    setShowModalTicket(true);
  };

  const handleCerrarTicket = () => {
    setShowModalTicket(false);
    setCart([]);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (!showDropdown || searchResults.length === 0) return;

    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelectedIndex((prev) => (prev < searchResults.length - 1 ? prev + 1 : 0));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelectedIndex((prev) => (prev > 0 ? prev - 1 : searchResults.length - 1));
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (selectedIndex >= 0 && selectedIndex < searchResults.length) {
        addToCart(searchResults[selectedIndex]);
      }
    } else if (e.key === "Escape") {
      setShowDropdown(false);
    }
  };

  // ── RENDER ──────────────────────────────────────────────────────────────

  return (
    <>
    <div className="flex-1 flex flex-col gap-6 animate-in fade-in slide-in-from-bottom-2 duration-500 max-w-5xl mx-auto w-full">

      {/* ═══ BÚSQUEDA ═════════════════════════════════════════════════ */}
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
          onChange={(e) => setSearchQuery(e.target.value)}
          onKeyDown={handleKeyDown}
          onFocus={() => searchQuery && searchResults.length > 0 && setShowDropdown(true)}
          placeholder="Escanea o busca por nombre, código o categoría..."
          className="w-full pl-14 pr-32 py-5 bg-white border-2 border-neutral-100 rounded-[1.75rem] shadow-xl shadow-neutral-100/60 text-base font-black text-neutral-900 placeholder:text-neutral-300 placeholder:font-bold placeholder:text-sm focus:outline-none focus:border-neutral-900 focus:ring-8 focus:ring-neutral-900/5 transition-all duration-300"
        />
        <div className="absolute right-4 top-1/2 -translate-y-1/2 flex items-center gap-2">
          {searchQuery && (
            <button
              onClick={() => { setSearchQuery(""); setSearchResults([]); setShowDropdown(false); inputRef.current?.focus(); }}
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
                    onClick={() => addToCart(item)}
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

      {/* ═══ CARRITO ══════════════════════════════════════════════════ */}
      <div className="flex-1 bg-white rounded-[2.5rem] border border-neutral-200 shadow-sm overflow-hidden flex flex-col">
        <div className="px-8 py-5 flex justify-between items-center">
          <h3 className="text-sm font-black text-neutral-950 uppercase tracking-tight flex items-center gap-3">
            <div className="w-10 h-10 bg-neutral-950 rounded-2xl flex items-center justify-center shadow-md">
              <MorphIcon icon={ICONO_CARRITO} size={16} strokeWidth={2.2} spring="smooth" className="text-white" />
            </div>
            Detalle de Venta
          </h3>
          <div className="flex items-center gap-4">
            <span className="px-3 py-1.5 bg-neutral-950 text-white text-[9px] font-black rounded-lg uppercase tracking-widest">
              {cart.reduce((acc, item) => acc + item.cantidad, 0)} ARTÍCULOS
            </span>
            {cart.length > 0 && (
              <button
                onClick={() => setCart([])}
                className="text-[9px] font-black text-red-400 hover:text-red-600 uppercase tracking-widest transition-colors"
              >
                LIMPIAR
              </button>
            )}
          </div>
        </div>

        <div className="flex-1 overflow-y-auto px-8 pb-4 custom-scrollbar">
          {cart.length === 0 ? (
            <div className="py-20 text-center">
              <div className="w-16 h-16 mx-auto bg-neutral-100 rounded-3xl flex items-center justify-center mb-5">
                <MorphIcon icon={ICONO_CARRITO} size={26} strokeWidth={1.8} spring="smooth" className="text-neutral-300" />
              </div>
              <p className="text-[11px] font-black uppercase tracking-[0.25em] text-neutral-300">Esperando productos...</p>
              <p className="text-[10px] font-bold text-neutral-200 mt-2">Escanea un código o busca arriba para empezar</p>
            </div>
          ) : (
            <table className="w-full text-left border-collapse">
              <thead>
                <tr className="text-[9px] font-black text-neutral-400 uppercase tracking-widest border-b-2 border-neutral-100">
                  <th className="pb-3 px-2">Cantidad</th>
                  <th className="pb-3 px-2">Producto</th>
                  <th className="pb-3 px-2">P. Unitario</th>
                  <th className="pb-3 px-2 text-right">Subtotal</th>
                  <th className="pb-3 px-2 w-12"></th>
                </tr>
              </thead>
              <tbody className="divide-y divide-neutral-100">
                {cart.map((item) => (
                  <tr key={item.id} className="group hover:bg-neutral-50/60 transition-colors">
                    <td className="py-3.5 px-2">
                      <div className="flex items-center gap-2 bg-neutral-50 rounded-2xl p-1 w-fit border border-neutral-100">
                        <button
                          onClick={() => updateQuantity(item.id, -1)}
                          className="w-8 h-8 rounded-xl bg-white hover:bg-neutral-950 hover:text-white flex items-center justify-center shadow-sm transition-all active:scale-90"
                          title="Quitar uno"
                        >
                          <MorphIcon icon={ICONO_RESTA} size={13} strokeWidth={3} spring="snappy" reducedMotion="user" />
                        </button>
                        <span className="w-8 text-center text-sm font-black text-neutral-900">{item.cantidad}</span>
                        <button
                          onClick={() => updateQuantity(item.id, 1)}
                          disabled={item.cantidad >= item.stock}
                          className="w-8 h-8 rounded-xl bg-white hover:bg-neutral-950 hover:text-white flex items-center justify-center shadow-sm transition-all active:scale-90 disabled:opacity-30 disabled:cursor-not-allowed disabled:hover:bg-white disabled:hover:text-black"
                          title="Agregar uno"
                        >
                          <MorphIcon icon={ICONO_MAS} size={13} strokeWidth={3} spring="snappy" reducedMotion="user" />
                        </button>
                      </div>
                    </td>
                    <td className="py-3.5 px-2 font-black text-neutral-800 text-xs uppercase">{item.nombre}</td>
                    <td className="py-3.5 px-2 font-bold text-neutral-400 text-xs">${item.precio_venta.toFixed(2)}</td>
                    <td className="py-3.5 px-2 text-right font-black text-neutral-950 text-base">
                      ${(item.precio_venta * item.cantidad).toFixed(2)}
                    </td>
                    <td className="py-3.5 px-2 text-right">
                      <button
                        onClick={() => removeFromCart(item.id)}
                        className="p-2 rounded-xl bg-neutral-100 text-neutral-400 hover:bg-red-50 hover:text-red-500 transition-all opacity-0 group-hover:opacity-100 active:scale-90"
                        title="Eliminar del carrito"
                      >
                        <MorphIcon icon={ICONO_EQUIS} size={13} strokeWidth={2.5} spring="snappy" reducedMotion="user" />
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>

        {/* ── FOOTER OSCURO: IA + TOTAL ───────────────────────────── */}
        <div className="p-5 sm:p-6 bg-neutral-950 flex flex-col sm:flex-row items-stretch sm:items-center gap-4">
          <div className="flex-1 flex items-center gap-4 bg-white/5 p-4 rounded-3xl border border-white/10">
            <div className="w-10 h-10 bg-white/10 rounded-2xl flex items-center justify-center shrink-0">
              <MorphIcon icon={ICONO_ESTRELLA} size={17} strokeWidth={2} spring="smooth" className="text-amber-300" />
            </div>
            <div className="min-w-0">
              <p className="text-[8px] font-black text-neutral-500 uppercase tracking-[0.2em] mb-1">Sugerencia IA</p>
              <p className="text-[11px] font-bold text-neutral-200 leading-tight truncate">
                {iaSuggestion || <span className="opacity-30 italic">Sin recomendaciones...</span>}
              </p>
            </div>
          </div>
          <BotonAnimado
            icono={ICONO_BILLETE}
            iconoHover={ICONO_ESCANER}
            onClick={handleAbrirCobro}
            disabled={cart.length === 0}
            className="bg-white hover:bg-neutral-50 text-neutral-950 shadow-xl shadow-black/30 sm:min-w-[220px] justify-center !rounded-3xl !py-5 !text-lg"
          >
            Cobrar ${cartTotal.toFixed(2)}
          </BotonAnimado>
        </div>
      </div>
    </div>

    {showModalVenta && (
      <ModalVenta
        onClose={() => setShowModalVenta(false)}
        onVentaCompletada={handleVentaCompletada}
        cart={cart}
        cartTotal={cartTotal}
      />
    )}

    {showModalTicket && (
      <ModalTicket
        onClose={handleCerrarTicket}
        cart={cart}
        cartTotal={cartTotal}
        ticketNumber={lastTicketNumber}
        ventaId={lastVentaId}
        montoEfectivo={lastMontoEfectivo}
        montoTarjeta={lastMontoTarjeta}
        montoTransferencia={lastMontoTransferencia}
      />
    )}
    </>
  );
}

export { nuevaVentaNav };
