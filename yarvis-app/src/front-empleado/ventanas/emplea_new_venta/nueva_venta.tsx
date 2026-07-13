import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import ModalVenta from "./modalventa";
import ModalTicket from "./modalticket";

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

interface InventoryItem {
  id?: number;
  nombre: string;
  descripcion?: string;
  precio_costo: number;
  precio_venta: number;
  stock: number;
  stock_minimo: number;
  vendido: number;
  codigo_barras?: string;
  categoria?: string;
}

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
      const items = await invoke<InventoryItem[]>("get_inventory");
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

  return (
    <>
    <div className="flex-1 flex flex-col gap-4 animate-in fade-in duration-500 max-w-5xl mx-auto w-full">
      {/* SEARCH BAR */}
      <div ref={searchRef} className="relative group">
        <div className="absolute inset-y-0 left-0 pl-5 flex items-center pointer-events-none">
          {isSearching ? (
            <svg className="animate-spin h-4 w-4 text-neutral-400" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
              <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
              <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
            </svg>
          ) : (
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" className="text-neutral-300 group-focus-within:text-neutral-900 transition-colors duration-200">
              <circle cx="11" cy="11" r="8" />
              <path d="m21 21-4.3-4.3" />
            </svg>
          )}
        </div>
        <input
          ref={inputRef}
          type="text"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          onKeyDown={handleKeyDown}
          onFocus={() => searchQuery && searchResults.length > 0 && setShowDropdown(true)}
          placeholder="Buscar producto por nombre, código o categoría..."
          className="w-full pl-12 pr-28 py-3.5 bg-white border border-neutral-200 rounded-2xl shadow-sm text-sm font-medium text-neutral-900 placeholder:text-neutral-300 focus:outline-none focus:ring-4 focus:ring-neutral-900/5 focus:border-neutral-300 focus:shadow-[0_8px_30px_rgba(0,0,0,0.04)] transition-all duration-300"
        />
        <div className="absolute right-3 top-1/2 -translate-y-1/2 flex items-center gap-2">
          {searchQuery && (
            <button
              onClick={() => { setSearchQuery(""); setSearchResults([]); setShowDropdown(false); inputRef.current?.focus(); }}
              className="p-1 rounded-lg hover:bg-neutral-100 text-neutral-300 hover:text-neutral-600 transition-all"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
            </button>
          )}
          <span className={`px-2 py-1 text-[9px] font-black rounded-lg border uppercase tracking-wider transition-all duration-300 ${
            iaStatus === "loading"
              ? "bg-amber-50 text-amber-600 border-amber-200"
              : iaStatus === "ready"
              ? "bg-emerald-50 text-emerald-600 border-emerald-200"
              : iaStatus === "error"
              ? "bg-neutral-50 text-neutral-400 border-neutral-200"
              : "bg-neutral-50 text-neutral-400 border-neutral-200 group-focus-within:border-neutral-900 group-focus-within:text-neutral-900"
          }`}>
            {iaStatus === "loading" ? "BUSCANDO..." : iaStatus === "ready" ? "IA OK" : "IA"}
          </span>
        </div>

        {/* DROPDOWN */}
        {showDropdown && searchResults.length > 0 && (
          <div
            ref={dropdownRef}
            className="absolute top-full left-0 right-0 mt-2 bg-white border border-neutral-200 rounded-2xl shadow-2xl shadow-neutral-200/50 overflow-hidden z-50 animate-in fade-in slide-in-from-top-2 duration-200"
          >
            <div className="p-2 max-h-80 overflow-y-auto">
              {searchResults.map((item, idx) => {
                const inCart = cart.find((c) => c.id === item.id);
                return (
                  <button
                    key={item.id || idx}
                    onClick={() => addToCart(item)}
                    className={`w-full flex items-center gap-3 p-3 rounded-xl text-left transition-all duration-150 ${
                      idx === selectedIndex
                        ? "bg-neutral-900 text-white"
                        : "hover:bg-neutral-50 text-neutral-900"
                    }`}
                  >
                    <div className={`w-10 h-10 rounded-xl flex items-center justify-center text-[10px] font-black flex-shrink-0 ${
                      idx === selectedIndex ? "bg-white/20 text-white" : "bg-neutral-100 text-neutral-400"
                    }`}>
                      {item.categoria?.charAt(0)?.toUpperCase() || "P"}
                    </div>
                    <div className="flex-1 min-w-0">
                      <p className={`text-xs font-bold truncate ${idx === selectedIndex ? "text-white" : "text-neutral-900"}`}>
                        {item.nombre}
                      </p>
                      <p className={`text-[10px] font-medium truncate ${idx === selectedIndex ? "text-white/60" : "text-neutral-400"}`}>
                        {item.categoria || "Sin categoría"} · Stock: {item.stock}
                      </p>
                    </div>
                    <div className="text-right flex-shrink-0">
                      <p className={`text-xs font-black ${idx === selectedIndex ? "text-white" : "text-neutral-900"}`}>
                        ${item.precio_venta.toFixed(2)}
                      </p>
                      {inCart && (
                        <p className={`text-[9px] font-bold ${idx === selectedIndex ? "text-white/60" : "text-emerald-600"}`}>
                          En carrito: {inCart.cantidad}
                        </p>
                      )}
                    </div>
                    {item.stock <= 0 && (
                      <span className={`text-[8px] font-black px-1.5 py-0.5 rounded-full ${idx === selectedIndex ? "bg-white/20 text-white" : "bg-red-50 text-red-500"}`}>
                        SIN STOCK
                      </span>
                    )}
                  </button>
                );
              })}
            </div>
            {iaStatus === "ready" && (
              <div className="px-4 py-2.5 bg-emerald-50 border-t border-emerald-100 flex items-center gap-2">
                <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" className="text-emerald-500"><path d="m12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275L12 3Z"/></svg>
                <span className="text-[10px] font-bold text-emerald-700">{iaSuggestion}</span>
              </div>
            )}
          </div>
        )}

        {showDropdown && searchQuery && searchResults.length === 0 && !isSearching && (
          <div className="absolute top-full left-0 right-0 mt-2 bg-white border border-neutral-200 rounded-2xl shadow-2xl shadow-neutral-200/50 overflow-hidden z-50 animate-in fade-in slide-in-from-top-2 duration-200">
            <div className="p-8 text-center">
              <div className="w-12 h-12 bg-neutral-50 rounded-2xl flex items-center justify-center mx-auto mb-3">
                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" className="text-neutral-300"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
              </div>
              <p className="text-xs font-bold text-neutral-400">Sin resultados para "{searchQuery}"</p>
              <p className="text-[10px] text-neutral-300 mt-1">Intenta con otro nombre o código</p>
            </div>
          </div>
        )}
      </div>

      {/* CART TABLE */}
      <div className="flex-1 bg-white rounded-2xl border border-neutral-200 shadow-sm overflow-hidden flex flex-col">
        <div className="px-6 py-4 border-b border-neutral-100 bg-neutral-50/50 flex justify-between items-center">
          <h3 className="text-[10px] font-black text-neutral-900 uppercase tracking-widest flex items-center gap-2">
            <div className="w-1 h-4 bg-neutral-900 rounded-full"></div>
            Detalle de Venta
          </h3>
          <div className="flex items-center gap-3">
            <span className="text-[9px] font-bold text-neutral-400 uppercase tracking-tighter">
              {cart.reduce((acc, item) => acc + item.cantidad, 0)} ARTÍCULOS
            </span>
            {cart.length > 0 && (
              <button
                onClick={() => setCart([])}
                className="text-[9px] font-bold text-red-400 hover:text-red-600 uppercase tracking-wider transition-colors"
              >
                LIMPIAR
              </button>
            )}
          </div>
        </div>

        <div className="flex-1 overflow-y-auto px-6">
          <table className="w-full text-left border-collapse">
            <thead className="sticky top-0 bg-white z-10">
              <tr className="text-[9px] font-black text-neutral-400 uppercase tracking-widest border-b border-neutral-100">
                <th className="py-3 px-2">Cant.</th>
                <th className="py-3 px-2">Producto</th>
                <th className="py-3 px-2">P. Unitario</th>
                <th className="py-3 px-2 text-right">Subtotal</th>
                <th className="py-3 px-2 w-10"></th>
              </tr>
            </thead>
            <tbody className="divide-y divide-neutral-50">
              {cart.length === 0 ? (
                <tr>
                  <td colSpan={5} className="py-24 text-center">
                    <div className="flex flex-col items-center gap-3 opacity-10">
                      <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"><circle cx="8" cy="21" r="1" /><circle cx="19" cy="21" r="1" /><path d="M2.05 2.05h2l2.66 12.42a2 2 0 0 0 2 1.58h9.78a2 2 0 0 0 1.95-1.57l1.56-7.43H5.12" /></svg>
                      <p className="text-[10px] font-black uppercase tracking-[0.3em]">Esperando productos...</p>
                    </div>
                  </td>
                </tr>
              ) : (
                cart.map((item) => (
                  <tr key={item.id} className="group hover:bg-neutral-50/50 transition-colors">
                    <td className="py-3 px-2">
                      <div className="flex items-center gap-1">
                        <button
                          onClick={() => updateQuantity(item.id, -1)}
                          className="w-6 h-6 rounded-lg bg-neutral-100 hover:bg-neutral-200 flex items-center justify-center text-neutral-500 hover:text-neutral-900 transition-all text-[10px] font-black"
                        >
                          -
                        </button>
                        <span className="w-8 text-center text-xs font-black text-neutral-900">{item.cantidad}</span>
                        <button
                          onClick={() => updateQuantity(item.id, 1)}
                          disabled={item.cantidad >= item.stock}
                          className="w-6 h-6 rounded-lg bg-neutral-100 hover:bg-neutral-200 flex items-center justify-center text-neutral-500 hover:text-neutral-900 transition-all text-[10px] font-black disabled:opacity-30 disabled:cursor-not-allowed"
                        >
                          +
                        </button>
                      </div>
                    </td>
                    <td className="py-3 px-2 font-bold text-neutral-700 text-xs">{item.nombre}</td>
                    <td className="py-3 px-2 font-medium text-neutral-400 text-xs">${item.precio_venta.toFixed(2)}</td>
                    <td className="py-3 px-2 text-right font-black text-neutral-900 text-sm">
                      ${(item.precio_venta * item.cantidad).toFixed(2)}
                    </td>
                    <td className="py-3 px-2 text-right">
                      <button
                        onClick={() => removeFromCart(item.id)}
                        className="p-1 rounded-lg text-neutral-300 hover:text-red-500 hover:bg-red-50 transition-all opacity-0 group-hover:opacity-100"
                      >
                        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
                      </button>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>

        {/* FOOTER */}
        <div className="p-4 bg-neutral-900 text-white flex items-center gap-4">
          <div className="flex-1 flex items-center gap-3 bg-white/5 p-3 rounded-xl border border-white/10">
            <div className="w-8 h-8 bg-white/10 rounded-lg flex items-center justify-center text-sm">✨</div>
            <div>
              <p className="text-[8px] font-black text-neutral-400 uppercase tracking-widest mb-0.5">Sugerencia IA</p>
              <p className="text-[11px] font-medium text-neutral-200 leading-tight">
                {iaSuggestion || <span className="opacity-30 italic">Sin recomendaciones...</span>}
              </p>
            </div>
          </div>
          <div className="flex flex-col items-end gap-1">
            <p className="text-[8px] font-black text-neutral-500 uppercase tracking-widest leading-none">Total a cobrar</p>
            <button
              disabled={cart.length === 0}
              onClick={handleAbrirCobro}
              className="px-8 py-3.5 bg-white hover:bg-neutral-50 text-black rounded-xl font-black text-base shadow-lg hover:shadow-white/5 transition-all hover:scale-[1.05] active:scale-95 flex items-center gap-2 leading-none border border-transparent hover:border-white/20 disabled:opacity-30 disabled:cursor-not-allowed disabled:hover:scale-100"
            >
              ${cartTotal.toFixed(2)}
              <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"><polyline points="20 6 9 17 4 12" /></svg>
            </button>
          </div>
        </div>
      </div>
    </div>

    {showModalVenta && (
      <ModalVenta
        onClose={() => setShowModalVenta(false)}
        onVentaCompletada={handleVentaCompletada}
        cart={cart}
        cartTotal={cartTotal}
        cajero="Empleado"
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
