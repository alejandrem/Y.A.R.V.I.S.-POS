// ═══════════════════════════════════════════════════════════════════════════
// VENTANA COBRO — Cobro en UNA sola ventana (sin segundo modal).
// Tarea única: cobrar la venta del empleado y cerrar.
//
//   · Botón principal: cobra + imprime el ticket automáticamente.
//   · Botón secundario: cobra sin ticket + abre el cajón (misma lógica
//     que F6: pulso sin imprimir nada).
//   · Debajo: destino de impresión (Local/Red, ancho) + cajón manual.
//
// El descuento de inventario ocurre SOLO al confirmar (completar_venta
// es transaccional en el backend). La venta manda: si el print o el
// cajón fallan, la venta YA quedó cobrada (F2 reimprime) y la ventana
// se cierra igual. Escape/clic-fuera cancela (carrito intacto).
// ═══════════════════════════════════════════════════════════════════════════

import { useEffect, useRef, useState } from "react";
import { MorphIcon } from "morphicons/react";
import { completarVenta } from "../../../../services/venta";
import { obtenerTiendaInfo } from "../../../../services/turno";
import { reportarError } from "../../../../services/tauri";
import { notificarExito } from "../../../../components/notificaciones";
import {
  listarImpresoras,
  imprimirTicketVenta,
  abrirCajon,
  esImpresoraVirtual,
  type ImpresoraInfo,
} from "../../../../services/impresora";
import { ICONO_CAJA, ICONO_CHECK, ICONO_EQUIS, ICONO_DOCUMENTO } from "../../../../components/ui";
import type { CartItem } from "../hooks/useCarrito";
import SeccionProductos from "./seccion-productos";
import SeccionPago from "./seccion-pago";
import SeccionDestino, { armarDestinoPrint } from "./seccion-destino";

interface VentanaCobroProps {
  cart: CartItem[];
  cartTotal: number;
  /** Salir sin cobrar: el carrito queda intacto. */
  onCancelar: () => void;
  /** La venta quedó cobrada: el padre limpia el carrito y cierra. */
  onTerminar: () => void;
}

export default function VentanaCobro({ cart, cartTotal, onCancelar, onTerminar }: VentanaCobroProps) {
  const [efectivo, setEfectivo] = useState("");
  const [tarjeta, setTarjeta] = useState("");
  const [transferencia, setTransferencia] = useState("");
  const [procesando, setProcesando] = useState(false);
  const [error, setError] = useState("");
  // Destino de impresión (también lo usa el auto-print al cobrar).
  const [tabPrint, setTabPrint] = useState<"local" | "red">("local");
  const [impresoras, setImpresoras] = useState<ImpresoraInfo[]>([]);
  const [impresoraSel, setImpresoraSel] = useState("");
  const [anchoMm, setAnchoMm] = useState<80 | 58>(80);
  const [ipRed, setIpRed] = useState("");
  const [puertoRed, setPuertoRed] = useState(9100);
  // Candado anti-doble-cobro: ref (síncrono) en vez de state (async).
  // Dos Enters/clics en el mismo tick verían el mismo `procesando=false`;
  // el ref ya está en true para el segundo. Una venta = un cobro.
  const cobrandoRef = useRef(false);

  const montoEfectivo = parseFloat(efectivo) || 0;
  const montoTarjeta = parseFloat(tarjeta) || 0;
  const montoTransferencia = parseFloat(transferencia) || 0;
  const totalPagado = montoEfectivo + montoTarjeta + montoTransferencia;
  const cambio = totalPagado - cartTotal;
  const esValido = totalPagado >= cartTotal && cartTotal > 0;

  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onCancelar();
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [onCancelar]);

  useEffect(() => {
    listarImpresoras()
      .then((lista) => {
        setImpresoras(lista);
        // Preselección inteligente: primera física (las virtuales PDF/XPS
        // aceptan el trabajo pero solo generan archivos vacíos de 0 bytes).
        const fisicas = lista.filter((i) => !esImpresoraVirtual(i.nombre));
        const def = fisicas.find((i) => i.predeterminada) ?? fisicas[0] ?? lista.find((i) => i.predeterminada) ?? lista[0];
        setImpresoraSel(def?.nombre ?? "");
      })
      .catch(() => setImpresoras([]));
  }, []);

  const descPorLinea = (it: CartItem) => {
    const bruto = it.precio_venta * it.cantidad;
    const d = it.descuento ?? 0;
    return Number.isFinite(d) ? Math.min(Math.max(0, d), bruto) : 0;
  };

  const cobrarEnBackend = async () => {
    const subtotalBruto = cart.reduce((acc, it) => acc + it.precio_venta * it.cantidad, 0);
    return completarVenta({
      items: cart.map((item) => ({
        id: item.id ?? null,
        nombre: item.nombre,
        precio_venta: item.precio_venta,
        cantidad: item.cantidad,
        descuento: descPorLinea(item),
      })),
      total: cartTotal,
      subtotal: subtotalBruto,
      descuento: 0,
      monto_efectivo: montoEfectivo,
      monto_tarjeta: montoTarjeta,
      monto_transferencia: montoTransferencia,
      cliente_id: null,
    });
  };

  const imprimirTicket = async (ticketNumber: number) => {
    const destino = armarDestinoPrint(tabPrint, impresoraSel, ipRed, puertoRed);
    if (!destino) return false;
    const pagos = [
      montoEfectivo > 0 ? { metodo: "Efectivo", monto: montoEfectivo } : null,
      montoTarjeta > 0 ? { metodo: "Tarjeta", monto: montoTarjeta } : null,
      montoTransferencia > 0 ? { metodo: "Transferencia", monto: montoTransferencia } : null,
    ].filter((p): p is { metodo: string; monto: number } => p !== null);
    const tienda = await obtenerTiendaInfo().catch(() => ({ nombre: null, ubicacion: null, cp: null }));
    await imprimirTicketVenta(destino, {
      tienda: tienda?.nombre ?? "MI TIENDA",
      ubicacion: tienda?.ubicacion ?? null,
      folio: `#${ticketNumber}`,
      fecha: null,
      ancho_mm: anchoMm,
      lineas: cart.map((item) => ({
        nombre: item.nombre,
        cantidad: item.cantidad,
        precio_unitario: item.precio_venta,
        descuento: item.descuento ?? 0,
      })),
      total: cartTotal,
      pagos,
      qr: `YARVIS-${ticketNumber}`,
    });
    return true;
  };

  /** Principal: cobra e imprime el ticket automáticamente. */
  const handleCobrar = async () => {
    if (cobrandoRef.current) return;
    if (!esValido) {
      setError("El monto total no coincide con el cobro");
      return;
    }
    cobrandoRef.current = true;
    setProcesando(true);
    setError("");
    try {
      const result = await cobrarEnBackend();
      try {
        const impreso = await imprimirTicket(result.ticket_number);
        if (impreso) notificarExito(`Ticket #${result.ticket_number} · Cambio $${cambio.toFixed(2)}`);
        else reportarError("Cobrada sin ticket: configura impresora o reimprime con F2", `Ticket #${result.ticket_number}`);
      } catch (error) {
        reportarError("Se cobró pero no se pudo imprimir (F2 reimprime)", error);
      }
      onTerminar();
    } catch (err) {
      setError(String(err));
    } finally {
      setProcesando(false);
      cobrandoRef.current = false;
    }
  };

  /** Secundario: cobra SIN ticket pero abre el cajón (misma lógica que
   * F6: pulso sin imprimir nada). Ideal para ventas que no piden papel. */
  const handleCobrarSinTicket = async () => {
    if (cobrandoRef.current) return;
    if (!esValido) {
      setError("El monto total no coincide con el cobro");
      return;
    }
    cobrandoRef.current = true;
    setProcesando(true);
    setError("");
    try {
      const result = await cobrarEnBackend();
      const destino = armarDestinoPrint(tabPrint, impresoraSel, ipRed, puertoRed);
      if (destino) {
        try {
          await abrirCajon(destino);
        } catch (error) {
          reportarError("Se cobró pero no se pudo abrir el cajón", error);
        }
      }
      notificarExito(`Cobrada sin ticket · Folio #${result.ticket_number} · Cambio $${cambio.toFixed(2)}`);
      onTerminar();
    } catch (err) {
      setError(String(err));
    } finally {
      setProcesando(false);
      cobrandoRef.current = false;
    }
  };

  const limpiarError = () => setError("");

  return (
    <div
      className="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4"
      onClick={onCancelar}
    >
      <div
        className="bg-white rounded-[2.5rem] shadow-2xl w-full max-w-4xl overflow-hidden animate-in zoom-in-95 fade-in duration-200 max-h-[92vh] flex flex-col"
        onClick={(e) => e.stopPropagation()}
      >
        {/* HEADER OSCURO */}
        <div className="bg-neutral-950 px-8 pt-6 pb-5 text-center relative overflow-hidden shrink-0">
          <div className="absolute -top-10 -right-10 w-40 h-40 bg-white/[0.04] rounded-full blur-2xl" />
          <button
            onClick={onCancelar}
            className="absolute top-5 right-5 p-2 rounded-xl hover:bg-white/10 text-neutral-500 hover:text-white transition-all"
            title="Cerrar (Esc)"
          >
            <MorphIcon icon={ICONO_EQUIS} size={16} strokeWidth={2.5} spring="snappy" reducedMotion="user" />
          </button>
          <div className="w-12 h-12 mx-auto bg-white/10 rounded-2xl flex items-center justify-center mb-3">
            <MorphIcon icon={ICONO_CAJA} size={20} strokeWidth={2.2} spring="smooth" className="text-white" />
          </div>
          <h2 className="text-xl font-black text-white uppercase tracking-tight">Cobrar Venta</h2>
          <p className="text-xs font-black text-white uppercase tracking-[0.25em] mt-1 tabular-nums">
            Total a cobrar: ${cartTotal.toFixed(2)}
          </p>
        </div>

        {/* CUERPO EN DOS COLUMNAS */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-5 p-6 sm:p-7 overflow-y-auto custom-scrollbar">
          <SeccionProductos cart={cart} cartTotal={cartTotal} />
          <div className="space-y-4 min-h-0">
            <SeccionPago
              cart={cart}
              cartTotal={cartTotal}
              efectivo={efectivo}
              tarjeta={tarjeta}
              transferencia={transferencia}
              onEfectivo={(v) => { setEfectivo(v); limpiarError(); }}
              onTarjeta={(v) => { setTarjeta(v); limpiarError(); }}
              onTransferencia={(v) => { setTransferencia(v); limpiarError(); }}
              procesando={procesando}
              error={error}
              onConfirmar={handleCobrar}
            />
            <button
              onClick={handleCobrar}
              disabled={!esValido || procesando}
              className="w-full py-5 rounded-3xl bg-neutral-950 text-white text-sm font-black uppercase tracking-[0.2em] hover:bg-neutral-800 transition-all shadow-xl shadow-neutral-300 active:scale-[0.98] disabled:opacity-30 disabled:cursor-not-allowed flex items-center justify-center gap-3"
            >
              <MorphIcon icon={ICONO_CHECK} size={18} strokeWidth={3} spring="snappy" reducedMotion="user" />
              {procesando ? "Procesando..." : `Cobrar $${cartTotal.toFixed(2)}`}
            </button>
            <button
              onClick={handleCobrarSinTicket}
              disabled={!esValido || procesando}
              className="w-full py-3.5 rounded-2xl border-2 border-dashed border-neutral-300 text-neutral-500 text-[11px] font-black uppercase tracking-[0.2em] hover:border-neutral-950 hover:text-neutral-950 transition-all disabled:opacity-30 disabled:cursor-not-allowed flex items-center justify-center gap-2"
            >
              <MorphIcon icon={ICONO_DOCUMENTO} size={15} strokeWidth={2.2} spring="snappy" reducedMotion="user" />
              Cobrar sin ticket
            </button>
            <SeccionDestino
              tabPrint={tabPrint}
              onTabPrint={setTabPrint}
              impresoras={impresoras}
              impresoraSel={impresoraSel}
              onImpresoraSel={setImpresoraSel}
              anchoMm={anchoMm}
              onAnchoMm={setAnchoMm}
              ipRed={ipRed}
              onIpRed={setIpRed}
              puertoRed={puertoRed}
              onPuertoRed={setPuertoRed}
              ocupado={procesando}
            />
          </div>
        </div>

        <div className="px-6 sm:px-7 pb-5 shrink-0">
          <button
            onClick={onCancelar}
            className="w-full py-3 text-[10px] font-black text-neutral-400 uppercase tracking-widest hover:text-neutral-950 transition-colors"
          >
            Cancelar
          </button>
        </div>
      </div>
    </div>
  );
}
