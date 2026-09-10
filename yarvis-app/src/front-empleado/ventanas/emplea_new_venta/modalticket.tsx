import { useState, useEffect } from "react";
import { obtenerTiendaInfo, type TiendaInfo } from "../../../services/turno";
import { reportarError } from "../../../services/tauri";
import { notificarExito } from "../../../components/notificaciones";
import {
  listarImpresoras,
  imprimirTicketVenta,
  probarRed,
  type DestinoPrint,
  type ImpresoraInfo,
} from "../../../services/impresora";

interface CartItem {
  id?: number;
  nombre: string;
  precio_venta: number;
  cantidad: number;
  stock: number;
}

interface ModalTicketProps {
  onClose: () => void;
  cart: CartItem[];
  cartTotal: number;
  ticketNumber: number;
  ventaId: number;
  montoEfectivo: number;
  montoTarjeta: number;
  montoTransferencia: number;
}

export default function ModalTicket({
  onClose,
  cart,
  cartTotal,
  ticketNumber,
  montoEfectivo,
  montoTarjeta,
  montoTransferencia,
}: ModalTicketProps) {
  const [tienda, setTienda] = useState<TiendaInfo | null>(null);
  const [tabPrint, setTabPrint] = useState<"local" | "red">("local");
  const [impresoras, setImpresoras] = useState<ImpresoraInfo[]>([]);
  const [impresoraSel, setImpresoraSel] = useState("");
  const [ipRed, setIpRed] = useState("");
  const [puertoRed, setPuertoRed] = useState(9100);
  const [probando, setProbando] = useState(false);
  const [imprimiendo, setImprimiendo] = useState(false);

  useEffect(() => {
    obtenerTiendaInfo()
      .then(setTienda)
      .catch(() => setTienda({ nombre: null, ubicacion: null, cp: null }));
    listarImpresoras()
      .then((lista) => {
        setImpresoras(lista);
        const def = lista.find((i) => i.predeterminada) ?? lista[0];
        setImpresoraSel(def?.nombre ?? "");
      })
      .catch(() => setImpresoras([]));
  }, []);

  const cambio = (montoEfectivo + montoTarjeta + montoTransferencia) - cartTotal;

  const armarDestino = (): DestinoPrint | null => {
    if (tabPrint === "local") {
      if (!impresoraSel) {
        reportarError("Elige una impresora instalada", "Sin selección");
        return null;
      }
      return { Spooler: { nombre: impresoraSel } };
    }
    if (!ipRed.trim()) {
      reportarError("Escribe la IP de la térmica", "Sin IP");
      return null;
    }
    return { Red: { ip: ipRed.trim(), puerto: puertoRed || 9100 } };
  };

  const handleProbarRed = async () => {
    if (!ipRed.trim()) {
      reportarError("Escribe la IP de la térmica", "Sin IP");
      return;
    }
    setProbando(true);
    try {
      const msg = await probarRed(ipRed.trim(), puertoRed || 9100);
      notificarExito(msg);
    } catch (error) {
      reportarError("Sin conexión con la térmica", error);
    } finally {
      setProbando(false);
    }
  };

  const handleImprimir = async () => {
    const destino = armarDestino();
    if (!destino) return;
    const pagos = [
      montoEfectivo > 0 ? { metodo: "Efectivo", monto: montoEfectivo } : null,
      montoTarjeta > 0 ? { metodo: "Tarjeta", monto: montoTarjeta } : null,
      montoTransferencia > 0 ? { metodo: "Transferencia", monto: montoTransferencia } : null,
    ].filter((p): p is { metodo: string; monto: number } => p !== null);
    setImprimiendo(true);
    try {
      const msg = await imprimirTicketVenta(destino, {
        tienda: tienda?.nombre ?? "MI TIENDA",
        ubicacion: tienda?.ubicacion ?? null,
        folio: `#${ticketNumber}`,
        fecha: null,
        lineas: cart.map((item) => ({
          nombre: item.nombre,
          cantidad: item.cantidad,
          precio_unitario: item.precio_venta,
        })),
        total: cartTotal,
        pagos,
        qr: `YARVIS-${ticketNumber}`,
      });
      notificarExito(msg);
    } catch (error) {
      reportarError("No se pudo imprimir el ticket", error);
    } finally {
      setImprimiendo(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/40 backdrop-blur-sm z-50 flex items-center justify-center" onClick={onClose}>
      <div
        className="bg-white rounded-[2rem] shadow-2xl w-full max-w-sm p-8 space-y-5 animate-in zoom-in-95 duration-200"
        onClick={(e) => e.stopPropagation()}
      >
        <header className="text-center">
          <h2 className="text-lg font-black text-neutral-900 uppercase">Ticket</h2>
          <div className="h-0.5 w-8 bg-neutral-900 mx-auto mt-2 rounded-full"></div>
          <p className="text-[10px] font-bold text-neutral-400 uppercase tracking-widest mt-2">
            #{ticketNumber}
          </p>
        </header>

        <div className="bg-neutral-50 rounded-xl p-4 space-y-3">
          {tienda?.nombre && (
            <p className="text-xs font-black text-neutral-900 text-center uppercase">{tienda.nombre}</p>
          )}
          {tienda?.ubicacion && (
            <p className="text-[10px] font-bold text-neutral-500 text-center">{tienda.ubicacion}</p>
          )}
          {tienda?.cp && (
            <p className="text-[10px] font-bold text-neutral-500 text-center">CP: {tienda.cp}</p>
          )}

          <div className="h-px bg-neutral-200"></div>

          <div className="space-y-1">
            <p className="text-[8px] font-black text-neutral-400 uppercase tracking-widest">Productos</p>
            {cart.map((item, idx) => (
              <div key={idx} className="flex justify-between text-[10px] font-bold text-neutral-700">
                <span>{item.cantidad}x {item.nombre}</span>
                <span>${(item.precio_venta * item.cantidad).toFixed(2)}</span>
              </div>
            ))}
          </div>

          <div className="h-px bg-neutral-200"></div>

          <div className="flex justify-between text-xs font-black text-neutral-900">
            <span>TOTAL</span>
            <span>${cartTotal.toFixed(2)}</span>
          </div>

          <div className="space-y-1">
            <p className="text-[8px] font-black text-neutral-400 uppercase tracking-widest">Método de pago</p>
            {montoEfectivo > 0 && (
              <div className="flex justify-between text-[10px] font-bold text-neutral-600">
                <span>Efectivo</span>
                <span>${montoEfectivo.toFixed(2)}</span>
              </div>
            )}
            {montoTarjeta > 0 && (
              <div className="flex justify-between text-[10px] font-bold text-neutral-600">
                <span>Tarjeta</span>
                <span>${montoTarjeta.toFixed(2)}</span>
              </div>
            )}
            {montoTransferencia > 0 && (
              <div className="flex justify-between text-[10px] font-bold text-neutral-600">
                <span>Transferencia</span>
                <span>${montoTransferencia.toFixed(2)}</span>
              </div>
            )}
          </div>

          {cambio > 0 && (
            <>
              <div className="h-px bg-neutral-200"></div>
              <div className="flex justify-between text-xs font-black text-emerald-600">
                <span>Cambio</span>
                <span>${cambio.toFixed(2)}</span>
              </div>
            </>
          )}
        </div>

        <div className="pt-2 space-y-2">
          <div className="bg-neutral-900 rounded-2xl p-4 space-y-3">
            <div className="flex bg-white/10 rounded-xl p-1">
              <button
                onClick={() => setTabPrint("local")}
                className={`flex-1 py-2 text-[9px] font-black uppercase tracking-widest rounded-lg transition-all ${
                  tabPrint === "local" ? "bg-white text-neutral-900" : "text-neutral-400"
                }`}
              >
                Local
              </button>
              <button
                onClick={() => setTabPrint("red")}
                className={`flex-1 py-2 text-[9px] font-black uppercase tracking-widest rounded-lg transition-all ${
                  tabPrint === "red" ? "bg-white text-neutral-900" : "text-neutral-400"
                }`}
              >
                Red
              </button>
            </div>

            {tabPrint === "local" ? (
              impresoras.length === 0 ? (
                <p className="text-[9px] font-bold text-neutral-400 text-center uppercase tracking-widest py-2">
                  Sin impresoras instaladas en Windows
                </p>
              ) : (
                <select
                  value={impresoraSel}
                  onChange={(e) => setImpresoraSel(e.target.value)}
                  className="w-full px-3 py-2.5 bg-white/10 border border-white/10 rounded-xl text-[11px] font-bold text-white focus:outline-none"
                >
                  {impresoras.map((imp) => (
                    <option key={imp.nombre} value={imp.nombre} className="text-neutral-900">
                      {imp.nombre}{imp.predeterminada ? " (default)" : ""}
                    </option>
                  ))}
                </select>
              )
            ) : (
              <div className="flex gap-2">
                <input
                  value={ipRed}
                  onChange={(e) => setIpRed(e.target.value)}
                  placeholder="192.168.1.50"
                  className="flex-1 min-w-0 px-3 py-2.5 bg-white/10 border border-white/10 rounded-xl text-[11px] font-bold text-white placeholder:text-neutral-500 focus:outline-none"
                />
                <input
                  type="number"
                  value={puertoRed}
                  onChange={(e) => setPuertoRed(Number(e.target.value))}
                  className="w-20 px-3 py-2.5 bg-white/10 border border-white/10 rounded-xl text-[11px] font-bold text-white text-center focus:outline-none"
                />
                <button
                  onClick={handleProbarRed}
                  disabled={probando}
                  className="px-3 py-2.5 text-[9px] font-black uppercase tracking-widest rounded-xl bg-white/10 text-white hover:bg-white/20 transition-colors disabled:opacity-40"
                >
                  {probando ? "…" : "Probar"}
                </button>
              </div>
            )}

            <button
              onClick={handleImprimir}
              disabled={imprimiendo}
              className="w-full py-3 rounded-xl bg-white text-neutral-900 text-[10px] font-black uppercase tracking-[0.2em] hover:bg-neutral-200 transition-all disabled:opacity-40"
            >
              {imprimiendo ? "Imprimiendo…" : "🖨 Imprimir ticket"}
            </button>
          </div>

          <button
            onClick={onClose}
            className="w-full py-4 rounded-xl bg-neutral-900 text-white text-xs font-black uppercase tracking-[0.2em] hover:bg-neutral-800 transition-all shadow-xl shadow-neutral-200"
          >
            Cerrar
          </button>
        </div>
      </div>
    </div>
  );
}
