// ═══════════════════════════════════════════════════════════════════════════
// MODAL NUEVO PRODUCTO — Alta rápida de inventario (Fase 0 barras).
// Rectangular apaisada, un solo formulario: nombre*, código de barras
// (opcional, el escáner HID escribe donde esté el foco), costo vs venta
// en tarjetas separadas con margen en vivo, comentario opcional.
// La validación dura vive en el backend (add_inventory_item_impl); aquí
// solo pre-chequeos de UX. Ver issue de barras Fase 0.
// ═══════════════════════════════════════════════════════════════════════════

import { useState } from "react";
import { MorphIcon } from "morphicons/react";
import {
  ICONO_ALERTA_CIRCULO,
  ICONO_BOLSA,
  ICONO_CHECK,
  ICONO_CHECK_CIRCULO,
  ICONO_CERRAR,
  ICONO_DOLAR,
  ICONO_EQUIS,
  ICONO_TRENDING,
} from "../../../icons";
import {
  agregarProductoInventario,
  type InventoryItem,
} from "../../../services/inventario";
import { notificarExito } from "../../../components/notificaciones";

interface ModalNuevoProductoProps {
  onClose: () => void;
  onSaved: () => void;
}

/// Dígito verificador EAN-8 / UPC-A / EAN-13. Solo pista visual: los
/// códigos internos de tienda ("BOLSA", "UNICO123") no son EAN y se
/// aceptan igual; el backend normaliza y valida lo suyo.
function eanValido(codigo: string): boolean | null {
  const d = codigo.replace(/[\s-]/g, "");
  if (d.length === 0) return null;
  if (!/^\d+$/.test(d) || (d.length !== 8 && d.length !== 12 && d.length !== 13)) {
    return null;
  }
  const digitos = d.split("").map(Number);
  const verificador = digitos.pop()!;
  let suma = 0;
  for (let i = 0; i < digitos.length; i++) {
    const desdeDerecha = digitos.length - 1 - i;
    suma += digitos[i] * (desdeDerecha % 2 === 0 ? 3 : 1);
  }
  return (10 - (suma % 10)) % 10 === verificador;
}

function aNumero(texto: string): number {
  const n = parseFloat(texto.replace(",", "."));
  return Number.isFinite(n) ? n : 0;
}

const ModalNuevoProducto = ({ onClose, onSaved }: ModalNuevoProductoProps) => {
  const [nombre, setNombre] = useState("");
  const [codigo, setCodigo] = useState("");
  const [costo, setCosto] = useState("");
  const [venta, setVenta] = useState("");
  const [stock, setStock] = useState("0");
  const [comentario, setComentario] = useState("");
  const [error, setError] = useState("");
  const [guardando, setGuardando] = useState(false);

  const costoNum = aNumero(costo);
  const ventaNum = aNumero(venta);
  const ganancia = ventaNum - costoNum;
  const margen = ventaNum > 0 ? (ganancia / ventaNum) * 100 : 0;
  const perdida = ventaNum > 0 && ganancia < 0;
  const ean = eanValido(codigo);

  const guardar = async () => {
    if (nombre.trim().length === 0) {
      setError("El nombre del producto es obligatorio.");
      return;
    }
    if (costoNum < 0 || ventaNum < 0) {
      setError("Los precios no pueden ser negativos.");
      return;
    }
    setError("");
    setGuardando(true);
    const item: InventoryItem = {
      nombre: nombre.trim().toUpperCase(),
      descripcion: comentario.trim() ? comentario.trim() : undefined,
      precio_costo: costoNum,
      precio_venta: ventaNum,
      stock: aNumero(stock),
      stock_minimo: 5,
      vendido: 0,
      codigo_barras: codigo.trim() ? codigo.trim() : undefined,
    };
    try {
      await agregarProductoInventario(item);
      notificarExito("Producto guardado con éxito");
      onSaved();
      onClose();
    } catch (e) {
      setError(String(e));
    } finally {
      setGuardando(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4" onClick={onClose}>
      <div
        className="w-full max-w-3xl rounded-[2rem] bg-white border border-neutral-200 shadow-2xl overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between px-8 py-5 border-b border-neutral-100">
          <div>
            <h3 className="text-lg font-black uppercase tracking-tight text-neutral-900">Nuevo producto</h3>
            <p className="text-[10px] font-black uppercase tracking-[0.3em] text-neutral-400">Alta al inventario</p>
          </div>
          <button onClick={onClose} className="flex h-9 w-9 items-center justify-center rounded-xl bg-neutral-100 text-neutral-500 hover:bg-neutral-200"><MorphIcon icon={ICONO_CERRAR} size={16} strokeWidth={2.5} /></button>
        </div>

        <div className="px-8 py-6 space-y-5">
          <div>
            <label className="text-[10px] font-black uppercase tracking-widest text-neutral-400">Nombre del producto *</label>
            <input
              autoFocus
              value={nombre}
              onChange={(e) => setNombre(e.target.value)}
              placeholder="Ej: Coca Cola Original 600ml"
              className="mt-2 w-full rounded-xl border border-neutral-200 px-4 py-3 text-sm font-bold text-neutral-900 outline-none focus:border-neutral-900"
            />
          </div>

          <div>
            <label className="text-[10px] font-black uppercase tracking-widest text-neutral-400">Código de barras</label>
            <input
              value={codigo}
              onChange={(e) => setCodigo(e.target.value)}
              placeholder="Pita el producto aquí o escríbelo a mano"
              className="mt-2 w-full rounded-xl border border-neutral-200 px-4 py-3 text-sm font-bold text-neutral-900 outline-none focus:border-neutral-900"
            />
            {ean === true && <p className="mt-1 flex items-center gap-1 text-[11px] font-bold text-emerald-600"><MorphIcon icon={ICONO_CHECK_CIRCULO} size={13} strokeWidth={2.5} /> Dígito verificador válido</p>}
            {ean === false && <p className="mt-1 flex items-center gap-1 text-[11px] font-bold text-red-500"><MorphIcon icon={ICONO_EQUIS} size={13} strokeWidth={2.5} /> Ese código no cuadra, revísalo (o déjalo vacío)</p>}
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="rounded-2xl border-2 border-red-200 bg-red-50/60 p-4">
              <p className="flex items-center gap-1.5 text-[10px] font-black uppercase tracking-widest text-red-500"><MorphIcon icon={ICONO_BOLSA} size={13} strokeWidth={2.5} /> Costo · me costó</p>
              <input
                type="text"
                inputMode="decimal"
                value={costo}
                onChange={(e) => setCosto(e.target.value)}
                placeholder="$ 0.00"
                className="mt-2 w-full rounded-xl border border-red-200 bg-white px-4 py-3 text-lg font-black text-neutral-900 outline-none focus:border-red-400"
              />
            </div>
            <div className="rounded-2xl border-2 border-emerald-200 bg-emerald-50/60 p-4">
              <p className="flex items-center gap-1.5 text-[10px] font-black uppercase tracking-widest text-emerald-600"><MorphIcon icon={ICONO_DOLAR} size={13} strokeWidth={2.5} /> Venta · lo vendo</p>
              <input
                type="text"
                inputMode="decimal"
                value={venta}
                onChange={(e) => setVenta(e.target.value)}
                placeholder="$ 0.00"
                className="mt-2 w-full rounded-xl border border-emerald-200 bg-white px-4 py-3 text-lg font-black text-neutral-900 outline-none focus:border-emerald-400"
              />
            </div>
          </div>

          <div className={`rounded-2xl px-5 py-3 text-xs font-black ${perdida ? "bg-red-100 text-red-700 border border-red-300" : "bg-neutral-100 text-neutral-700"}`}>
            {ventaNum > 0
              ? perdida
                ? <span className="flex items-center gap-1.5"><MorphIcon icon={ICONO_ALERTA_CIRCULO} size={14} strokeWidth={2.5} /> VENDES CON PÉRDIDA: -${Math.abs(ganancia).toFixed(2)} por pieza</span>
                : <span className="flex items-center gap-1.5"><MorphIcon icon={ICONO_TRENDING} size={14} strokeWidth={2.5} /> Ganas ${ganancia.toFixed(2)} por pieza · margen {margen.toFixed(1)}%</span>
              : "Escribe costo y venta para ver tu ganancia"}
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="text-[10px] font-black uppercase tracking-widest text-neutral-400">Stock inicial por unidad</label>
              <input
                type="text"
                inputMode="decimal"
                value={stock}
                onChange={(e) => setStock(e.target.value)}
                className="mt-2 w-full rounded-xl border border-neutral-200 px-4 py-3 text-sm font-bold text-neutral-900 outline-none focus:border-neutral-900"
              />
            </div>
            <div>
              <label className="text-[10px] font-black uppercase tracking-widest text-neutral-400">Comentario <span className="text-neutral-300">(opcional)</span></label>
              <input
                value={comentario}
                onChange={(e) => setComentario(e.target.value)}
                placeholder="Ej: llega los martes..."
                className="mt-2 w-full rounded-xl border border-neutral-200 px-4 py-3 text-sm font-bold text-neutral-900 outline-none focus:border-neutral-900"
              />
            </div>
          </div>

          {error && <p className="rounded-xl border border-red-200 bg-red-50 px-4 py-3 text-xs font-bold text-red-600">{error}</p>}
        </div>

        <div className="flex gap-3 border-t border-neutral-100 px-8 py-5">
          <button onClick={onClose} className="flex-1 rounded-xl bg-neutral-100 py-3 text-[10px] font-black uppercase tracking-widest text-neutral-500 hover:bg-neutral-200">Cancelar</button>
          <button onClick={guardar} disabled={guardando} className="flex flex-1 items-center justify-center gap-2 rounded-xl bg-neutral-900 py-3 text-[10px] font-black uppercase tracking-widest text-white hover:bg-neutral-700 disabled:opacity-50">
            <MorphIcon icon={ICONO_CHECK} size={14} strokeWidth={3} />{guardando ? "Guardando..." : "Guardar"}
          </button>
        </div>
      </div>
    </div>
  );
};

export default ModalNuevoProducto;
