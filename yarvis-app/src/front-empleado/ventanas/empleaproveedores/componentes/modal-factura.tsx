// MODAL FACTURA — La "factura de que pagamos": cabecera, renglones,
// totales, botón de imprimir y botón de EDITAR (abre rectificativa).
// Las facturas NO se borran: no existe ese botón en ningún lado.
// Tarea única: mostrar, imprimir y encadenar rectificación.
// ═══════════════════════════════════════════════════════════════════════════

import { useEffect, useState } from "react";
import { MorphIcon } from "morphicons/react";
import { ICONO_EDITAR } from "../../../../components/ui";
import { reportarError } from "../../../../services/tauri";
import { notificarExito } from "../../../../components/notificaciones";
import { obtenerCompraDetalle, type CompraDetalle } from "../../../../services/proveedores";
import { obtenerTiendaInfo } from "../../../../services/turno";
import {
  listarImpresoras, imprimirTicketVenta, type ImpresoraInfo,
} from "../../../../services/impresora";

interface ModalFacturaProps {
  compraId: number;
  onCerrar: () => void;
  onEditar: (detalle: CompraDetalle) => void;
  onVerFactura: (compraId: number) => void;
}

const ModalFactura = ({ compraId, onCerrar, onEditar, onVerFactura }: ModalFacturaProps) => {
  const [detalle, setDetalle] = useState<CompraDetalle | null>(null);
  const [impresoras, setImpresoras] = useState<ImpresoraInfo[]>([]);
  const [impresoraSel, setImpresoraSel] = useState("");
  const [imprimiendo, setImprimiendo] = useState(false);

  useEffect(() => {
    let viva = true;
    obtenerCompraDetalle(compraId)
      .then((d) => {
        if (viva) setDetalle(d);
      })
      .catch((e) => reportarError("No se pudo abrir la factura", e));
    listarImpresoras()
      .then((lista) => {
        if (!viva) return;
        setImpresoras(lista);
        setImpresoraSel((lista.find((i) => i.predeterminada) ?? lista[0])?.nombre ?? "");
      })
      .catch(() => {
        if (viva) setImpresoras([]);
      });
    return () => {
      viva = false;
    };
  }, [compraId]);

  useEffect(() => {
    const tecla = (e: KeyboardEvent) => {
      if (e.key === "Escape") onCerrar();
    };
    document.addEventListener("keydown", tecla);
    return () => document.removeEventListener("keydown", tecla);
  }, [onCerrar]);

  const imprimir = async () => {
    if (!detalle) return;
    if (!impresoraSel) {
      reportarError("Elige una impresora instalada", "Sin selección");
      return;
    }
    setImprimiendo(true);
    try {
      const tienda = await obtenerTiendaInfo().catch(() => ({ nombre: null, ubicacion: null, cp: null }));
      const msg = await imprimirTicketVenta(
        { Spooler: { nombre: impresoraSel } },
        {
          tienda: tienda.nombre ?? "MI TIENDA",
          ubicacion: tienda.ubicacion ?? null,
          folio: `COMPRA-${detalle.id}`,
          fecha: detalle.fecha,
          lineas: detalle.items.map((it) => ({
            nombre: `${it.nombre} (${it.presentacion})`,
            cantidad: it.cantidad,
            precio_unitario: it.precio_sugerido,
          })),
          total: detalle.pagado,
          pagos: [{ metodo: detalle.metodo_pago, monto: detalle.pagado }],
          qr: `COMPRA-${detalle.id}`,
        },
      );
      notificarExito(msg);
    } catch (e) {
      reportarError("No se pudo imprimir la factura", e);
    } finally {
      setImprimiendo(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4 backdrop-blur-sm" onClick={onCerrar}>
      <div
        className="w-full max-w-lg bg-white rounded-[2rem] shadow-2xl overflow-hidden animate-in fade-in zoom-in-95 duration-200"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="bg-neutral-950 text-white px-6 sm:px-8 py-6 text-center">
          <p className="text-[9px] font-black uppercase tracking-[0.3em] text-neutral-400">Factura de compra</p>
          <h3 className="text-3xl font-black mt-1">#{detalle?.id ?? compraId}</h3>
          <p className="text-[11px] font-bold text-neutral-400 mt-1 uppercase">{detalle?.proveedor ?? "…"}</p>
        </div>

        <div className="px-6 sm:px-8 py-6 max-h-[50vh] overflow-y-auto custom-scrollbar">
          {detalle === null ? (
            <div className="py-10 flex justify-center">
              <div className="w-8 h-8 rounded-full border-2 border-neutral-900 border-t-transparent animate-spin" />
            </div>
          ) : (
            <div className="space-y-4">
              {detalle.rectifica_a != null && (
                <button
                  onClick={() => onVerFactura(detalle.rectifica_a!)}
                  className="w-full px-4 py-3 rounded-2xl bg-sky-50 border border-sky-200 text-sky-700 text-[10px] font-black uppercase tracking-widest text-center hover:bg-sky-100 transition-colors"
                >
                  ↩ Rectifica a la factura #{detalle.rectifica_a} (tócala para verla)
                </button>
              )}
              {(detalle.rectificada_por ?? []).length > 0 && (
                <div className="px-4 py-3 rounded-2xl bg-amber-50 border border-amber-200 text-center">
                  <p className="text-amber-700 text-[10px] font-black uppercase tracking-widest">
                    ! Esta factura fue rectificada (intacta, solo lectura)
                  </p>
                  <div className="flex flex-wrap justify-center gap-2 mt-2">
                    {(detalle.rectificada_por ?? []).map((id) => (
                      <button
                        key={id}
                        onClick={() => onVerFactura(id)}
                        className="px-3 py-1.5 rounded-xl bg-white border border-amber-300 text-amber-700 text-[10px] font-black hover:bg-amber-100 transition-colors"
                      >
                        Ver #{id}
                      </button>
                    ))}
                  </div>
                </div>
              )}
              {detalle.movimiento_id === null && detalle.pagado > 0 && (
                <p className="px-4 py-3 rounded-2xl bg-amber-50 border border-amber-200 text-amber-700 text-[10px] font-black uppercase tracking-widest text-center">
                  ! Egreso pendiente de corte
                </p>
              )}
              <div className="space-y-2">
                {detalle.items.map((it, i) => (
                  <div key={i} className="flex items-center justify-between gap-3 p-4 bg-neutral-50 rounded-2xl border border-neutral-100">
                    <div className="min-w-0">
                      <p className="text-xs font-black text-neutral-900 uppercase truncate">{it.nombre}</p>
                      <p className="text-[10px] font-bold text-neutral-400">
                        {it.presentacion === "paquete" && it.piezas_por_paquete !== null && it.paquetes !== null
                          ? `${it.paquetes} paq × ${it.piezas_por_paquete} pzas = ${it.cantidad} unidades`
                          : `${it.cantidad} ${it.presentacion}`} · sug. ${it.precio_sugerido.toFixed(2)}
                      </p>
                    </div>
                  </div>
                ))}
              </div>
              <div className="rounded-2xl bg-neutral-950 text-white p-5 space-y-1.5">
                <div className="flex justify-between text-[11px] font-bold text-neutral-400 uppercase">
                  <span>Sugerido</span><span>${detalle.sugerido.toFixed(2)}</span>
                </div>
                <div className="flex justify-between text-[11px] font-bold text-neutral-400 uppercase">
                  <span>Método</span><span>{detalle.metodo_pago}</span>
                </div>
                <div className="flex justify-between text-xl font-black pt-2 border-t border-white/10">
                  <span className="uppercase">Pagado</span><span>${detalle.pagado.toFixed(2)}</span>
                </div>
                {detalle.comentario && <p className="text-[11px] text-neutral-400 italic pt-1">"{detalle.comentario}"</p>}
              </div>
              <div className="flex items-center gap-2">
                <select
                  value={impresoraSel}
                  onChange={(e) => setImpresoraSel(e.target.value)}
                  className="flex-1 px-4 py-3.5 bg-neutral-50 border-2 border-neutral-100 rounded-2xl text-[10px] font-black uppercase tracking-widest outline-none focus:border-neutral-900"
                >
                  <option value="">Elige impresora…</option>
                  {impresoras.map((im) => (
                    <option key={im.nombre} value={im.nombre}>{im.nombre}</option>
                  ))}
                </select>
                <button
                  disabled={imprimiendo}
                  onClick={imprimir}
                  className="px-6 py-3.5 rounded-2xl bg-neutral-950 text-white text-[10px] font-black uppercase tracking-widest hover:scale-105 active:scale-95 transition-all disabled:opacity-40"
                >
                  {imprimiendo ? "…" : "Imprimir"}
                </button>
              </div>
            </div>
          )}
        </div>

        <div className="px-6 sm:px-8 pb-6 space-y-3">
          <button
            onClick={() => {
              if (detalle) onEditar(detalle);
            }}
            disabled={!detalle}
            className="w-full py-4 rounded-2xl bg-neutral-950 text-white text-xs font-black uppercase tracking-widest hover:scale-[1.01] active:scale-95 transition-all disabled:opacity-40 flex items-center justify-center gap-2"
          >
            <MorphIcon icon={ICONO_EDITAR} size={15} strokeWidth={2.4} spring="snappy" reducedMotion="user" />
            Editar (crea rectificativa, no borra nada)
          </button>
          <button onClick={onCerrar} className="w-full py-4 rounded-2xl border-2 border-neutral-200 text-neutral-500 text-xs font-black uppercase tracking-widest hover:border-neutral-900 hover:text-neutral-900 transition-all">
            Cerrar
          </button>
        </div>
      </div>
    </div>
  );
};

export default ModalFactura;
