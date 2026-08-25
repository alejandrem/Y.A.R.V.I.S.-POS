// ═══════════════════════════════════════════════════════════════════════════
// NUEVA VENTA — Punto de venta del operador (ORQUESTADOR).
// Tarea única: poseer TODO el estado (inventario, búsqueda, IA, modales),
// los handlers y el listener de F5, y componer los componentes
// presentacionales (buscador, tabla de carrito, pie de cobro). La lógica no
// cambia: el carrito vive solo en memoria y el inventario se descuenta
// ÚNICAMENTE al confirmar el cobro (F5 → modal → Confirmar).
// ═══════════════════════════════════════════════════════════════════════════

import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { obtenerInventario, type InventoryItem } from "../../../services/inventario";
import ModalVenta from "./modalventa";
import ModalTicket from "./modalticket";
import { useCarrito } from "./hooks/useCarrito";
import BuscadorProductos from "./componentes/buscador-productos";
import TablaCarrito from "./componentes/tabla-carrito";
import PieCobro from "./componentes/pie-cobro";

const nuevaVentaNav = {
  id: "nueva_venta",
  label: "NUEVA VENTA",
  icon: (
    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="9" cy="21" r="1" /><circle cx="20" cy="21" r="1" />
      <path d="M1 1h4l2.68 13.39a2 2 0 0 0 2 1.61h9.72a2 2 0 0 0 2-1.61L23 6H6" />
    </svg>
  ),
};

interface NuevaVentaProps { activeTab: string }

export default function NuevaVenta({ activeTab }: NuevaVentaProps) {
  const [inventory, setInventory] = useState<InventoryItem[]>([]);
  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<InventoryItem[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [showDropdown, setShowDropdown] = useState(false);
  const [selectedIndex, setSelectedIndex] = useState(-1);
  const [iaStatus, setIaStatus] = useState<"idle" | "loading" | "ready" | "error">("idle");
  const [iaSuggestion, setIaSuggestion] = useState("");
  const [showModalVenta, setShowModalVenta] = useState(false); const [showModalTicket, setShowModalTicket] = useState(false);
  const [lastVentaId, setLastVentaId] = useState(0); const [lastTicketNumber, setLastTicketNumber] = useState(0);
  const [lastMontoEfectivo, setLastMontoEfectivo] = useState(0); const [lastMontoTarjeta, setLastMontoTarjeta] = useState(0);
  const [lastMontoTransferencia, setLastMontoTransferencia] = useState(0);
  const searchRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);

  const { cart, addToCart, updateQuantity, removeFromCart, limpiarCarrito, cartTotal } =
    useCarrito({ inputRef });

  useEffect(() => {
    if (activeTab === "nueva_venta") loadInventory();
  }, [activeTab]);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (searchRef.current && !searchRef.current.contains(e.target as Node)) setShowDropdown(false);
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  useEffect(() => {
    const handleF5 = (e: KeyboardEvent) => {
      if (e.key === "F5" && cart.length > 0 && !showModalVenta && !showModalTicket) {
        e.preventDefault(); setShowModalVenta(true);
      }
    };
    window.addEventListener("keydown", handleF5);
    return () => window.removeEventListener("keydown", handleF5);
  }, [cart, showModalVenta, showModalTicket]);

  const loadInventory = async () => {
    try { setInventory(await obtenerInventario()); }
    catch (error) { console.error("Error al cargar inventario:", error); }
  };

  const searchProducts = useCallback(
    async (query: string) => {
      if (!query.trim()) { setSearchResults([]); setShowDropdown(false); return; }

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
            const matched = aiResults.map((r) => inventory.find((p) => p.id === r.id)).filter(Boolean) as InventoryItem[];
            if (matched.length > 0) {
              setSearchResults(matched); setIaStatus("ready");
              setIaSuggestion(`Encontré "${matched[0].nombre}" por similitud`);
            } else setIaStatus("error");
          } else setIaStatus("error");
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
    const debounce = setTimeout(() => searchProducts(searchQuery), 200);
    return () => clearTimeout(debounce);
  }, [searchQuery, searchProducts]);

  const seleccionarProducto = (product: InventoryItem) => {
    addToCart(product); setSearchQuery(""); setShowDropdown(false);
  };

  const limpiarBusqueda = () => {
    setSearchQuery(""); setSearchResults([]); setShowDropdown(false);
    inputRef.current?.focus();
  };

  const handleAbrirCobro = () => { if (cart.length === 0) return; setShowModalVenta(true); };

  const handleVentaCompletada = (ventaId: number, ticketNumber: number, efectivo: number, tarjeta: number, transferencia: number) => {
    setLastVentaId(ventaId); setLastTicketNumber(ticketNumber);
    setLastMontoEfectivo(efectivo); setLastMontoTarjeta(tarjeta);
    setLastMontoTransferencia(transferencia);
    setShowModalVenta(false); setShowModalTicket(true);
  };

  const handleCerrarTicket = () => { setShowModalTicket(false); limpiarCarrito(); };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (!showDropdown || searchResults.length === 0) return;
    if (e.key === "ArrowDown") {
      e.preventDefault(); setSelectedIndex((prev) => (prev < searchResults.length - 1 ? prev + 1 : 0));
    } else if (e.key === "ArrowUp") {
      e.preventDefault(); setSelectedIndex((prev) => (prev > 0 ? prev - 1 : searchResults.length - 1));
    } else if (e.key === "Enter") {
      e.preventDefault();
      if (selectedIndex >= 0 && selectedIndex < searchResults.length) seleccionarProducto(searchResults[selectedIndex]);
    } else if (e.key === "Escape") { setShowDropdown(false); }
  };

  // ── RENDER ──────────────────────────────────────────────────────────────

  return (
    <>
    <div className="flex-1 flex flex-col gap-6 animate-in fade-in slide-in-from-bottom-2 duration-500 max-w-5xl mx-auto w-full">

      {/* ═══ BÚSQUEDA ═════════════════════════════════════════════════ */}
      <BuscadorProductos
        searchQuery={searchQuery} onSearchChange={setSearchQuery}
        searchResults={searchResults} selectedIndex={selectedIndex}
        isSearching={isSearching} showDropdown={showDropdown}
        iaStatus={iaStatus} iaSuggestion={iaSuggestion} cart={cart}
        onKeyDown={handleKeyDown} onSeleccionar={seleccionarProducto}
        onFocusInput={() => searchQuery && searchResults.length > 0 && setShowDropdown(true)}
        onLimpiar={limpiarBusqueda}
        searchRef={searchRef} inputRef={inputRef} dropdownRef={dropdownRef}
      />

      {/* ═══ CARRITO ══════════════════════════════════════════════════ */}
      <TablaCarrito cart={cart} onUpdateCantidad={updateQuantity} onEliminar={removeFromCart} onLimpiar={limpiarCarrito}>
        <PieCobro iaSuggestion={iaSuggestion} total={cartTotal} onCobrar={handleAbrirCobro} disabled={cart.length === 0} />
      </TablaCarrito>
    </div>

    {showModalVenta && (
      <ModalVenta onClose={() => setShowModalVenta(false)} onVentaCompletada={handleVentaCompletada} cart={cart} cartTotal={cartTotal} />
    )}

    {showModalTicket && (
      <ModalTicket
        onClose={handleCerrarTicket} cart={cart} cartTotal={cartTotal}
        ticketNumber={lastTicketNumber} ventaId={lastVentaId}
        montoEfectivo={lastMontoEfectivo} montoTarjeta={lastMontoTarjeta}
        montoTransferencia={lastMontoTransferencia}
      />
    )}
    </>
  );
}

export { nuevaVentaNav };
