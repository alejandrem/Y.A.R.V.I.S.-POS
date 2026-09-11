// MODAL COMPRA/PAGO — Recepción + pago en un solo flujo de mostrador.
// Buscador con escáner embebido (Enter) → unidad/paquete → cantidad →
// sugerencia o "sin recomendación" → monto editable → comentario →
// confirmar. Sin proveedor elegido se genera MOSTRADOR 00001 al confirmar.
//
// Modo "rectificar": precarga la factura original (proveedor BLOQUEADO,
// no se puede mover el pago a otro), y al confirmar crea una factura
// NUEVA que apunta a la original. La original jamás se toca.
// ═══════════════════════════════════════════════════════════════════════════

import { useEffect, useMemo, useRef, useState } from "react";
import { MorphIcon } from "morphicons/react";
import { ICONO_BUSCAR, ICONO_BOLSA, ICONO_CODIGO_BARRAS, ICONO_EDITAR, ICONO_BORRAR } from "../../../../components/ui";
import { reportarError } from "../../../../services/tauri";
import { notificarExito } from "../../../../components/notificaciones";
import { obtenerInventario, type InventoryItem } from "../../../../services/inventario";
import {
  crearProveedorGenerico, registrarCompra, rectificarCompra, sugerirPago,
  type CompraDetalle, type ItemCompraEnvio, type Proveedor,
} from "../../../../services/proveedores";

interface Renglon {
  producto_id: number | null;
  nombre: string;
  presentacion: "unidad" | "paquete";
  cantidad: number;
  piezasPorPaquete: number | null;
  paquetes: number | null;
  sugerido: number | null;
}

interface ModalCompraProps {
  proveedor: Proveedor | null;
  proveedores: Proveedor[];
  onProveedor: (p: Proveedor | null) => void;
  onCerrar: () => void;
  onRegistrada: (compraId: number) => void;
  /** Rectificar una factura existente (proveedor bloqueado, crea nueva). */
  modo?: "nueva" | "rectificar";
  compraOriginal?: CompraDetalle | null;
}

const normCodigo = (s: string) => s.trim().toUpperCase().replace(/[\s-]/g, "");

const ModalCompra = ({ proveedor, proveedores, onProveedor, onCerrar, onRegistrada, modo = "nueva", compraOriginal = null }: ModalCompraProps) => {
  const esRectificar = modo === "rectificar" && compraOriginal !== null;
  const [inventario, setInventario] = useState<InventoryItem[]>([]);
  const [query, setQuery] = useState("");
  const [selIdx, setSelIdx] = useState(0);
  const [showDrop, setShowDrop] = useState(false);
  const [lineaId, setLineaId] = useState<number | null>(null);
  const [lineaNombre, setLineaNombre] = useState("");
  const [presentacion, setPresentacion] = useState<"unidad" | "paquete">("unidad");
  const [cantidad, setCantidad] = useState("1");
  const [piezas, setPiezas] = useState("");
  const [paquetes, setPaquetes] = useState("");
  const [sugerido, setSugerido] = useState<number | null>(null);
  const [cargandoSug, setCargandoSug] = useState(false);
  const [renglones, setRenglones] = useState<Renglon[]>([]);
  // Índice en edición (null = agregando nuevo). El lápiz o tocar la fila
  // cargan el renglón al editor; + Agregar se vuelve Guardar cambios.
  const [editIdx, setEditIdx] = useState<number | null>(null);
  const [monto, setMonto] = useState("");
  const [montoDirty, setMontoDirty] = useState(false);
  const [metodo, setMetodo] = useState("efectivo");
  const [comentarioOpen, setComentarioOpen] = useState(false);
  const [comentario, setComentario] = useState("");
  const [busy, setBusy] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    obtenerInventario().then(setInventario).catch((e) => reportarError("No se pudo cargar el inventario", e));
    inputRef.current?.focus();
  }, []);

  // Legado: renglones paquete guardados sin desglose (NULL, de antes
  // del soporte de paquetes). Se conserva el total original tal cual
  // como piezas × 1: no se borra nada ni se bloquea la edición.
  const desgloseOPiezas = (
    presentacion: "unidad" | "paquete",
    cantidad: number,
    piezasPorPaquete: number | null,
    paquetes: number | null,
  ): { piezasPorPaquete: number | null; paquetes: number | null } => {
    if (presentacion !== "paquete") return { piezasPorPaquete: null, paquetes: null };
    if (piezasPorPaquete !== null && paquetes !== null) return { piezasPorPaquete, paquetes };
    return { piezasPorPaquete: cantidad, paquetes: 1 };
  };

  // Precarga de rectificativa: renglones, monto, método y comentario
  // vienen de la factura original. El proveedor NO se toca.
  useEffect(() => {
    if (!esRectificar || !compraOriginal) return;
    setRenglones(
      compraOriginal.items.map((it) => {
        const pres = (it.presentacion === "paquete" ? "paquete" : "unidad") as "unidad" | "paquete";
        const d = desgloseOPiezas(pres, it.cantidad, it.piezas_por_paquete, it.paquetes);
        return {
          producto_id: it.producto_id,
          nombre: it.nombre,
          presentacion: pres,
          cantidad: it.cantidad,
          piezasPorPaquete: d.piezasPorPaquete,
          paquetes: d.paquetes,
          sugerido: it.cantidad * it.precio_sugerido,
        };
      }),
    );
    setMonto(compraOriginal.pagado.toFixed(2));
    setMontoDirty(true);
    setMetodo(compraOriginal.metodo_pago);
    if (compraOriginal.comentario) {
      setComentario(compraOriginal.comentario);
      setComentarioOpen(true);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    const tecla = (e: KeyboardEvent) => {
      if (e.key === "Escape") onCerrar();
    };
    document.addEventListener("keydown", tecla);
    return () => document.removeEventListener("keydown", tecla);
  }, [onCerrar]);

  const resultados = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return [];
    return inventario
      .filter((it) =>
        it.nombre.toLowerCase().includes(q) ||
        (it.codigo_barras ?? "").toLowerCase().includes(q) ||
        (it.categoria ?? "").toLowerCase().includes(q),
      )
      .slice(0, 8);
  }, [query, inventario]);

  useEffect(() => setSelIdx(0), [query]);

  // Total de unidades: directo en unidad, piezas × paquetes en paquete.
  const numCantidad = Number(cantidad);
  const numPiezas = Number(piezas);
  const numPaquetes = Number(paquetes);
  const unidadesTotales =
    presentacion === "paquete"
      ? Number.isFinite(numPiezas) && Number.isFinite(numPaquetes) ? numPiezas * numPaquetes : NaN
      : numCantidad;

  // Sugerencia del renglón activo (debounce para no spamear).
  useEffect(() => {
    if (!lineaNombre) {
      setSugerido(null);
      return;
    }
    if (!Number.isFinite(unidadesTotales) || unidadesTotales <= 0) {
      setSugerido(null);
      return;
    }
    setCargandoSug(true);
    const t = window.setTimeout(() => {
      sugerirPago(lineaNombre, unidadesTotales)
        .then((s) => setSugerido(s.sugerido))
        .catch(() => setSugerido(null))
        .finally(() => setCargandoSug(false));
    }, 350);
    return () => window.clearTimeout(t);
  }, [lineaNombre, unidadesTotales]);

  const sumaSugerida = renglones.reduce((a, r) => a + (r.sugerido ?? 0), 0);
  useEffect(() => {
    if (!montoDirty) setMonto(sumaSugerida > 0 ? sumaSugerida.toFixed(2) : "");
  }, [sumaSugerida, montoDirty]);

  const elegir = (item: InventoryItem) => {
    setLineaId(item.id ?? null);
    setLineaNombre(item.nombre);
    setPresentacion("unidad");
    setCantidad("1");
    setPiezas("");
    setPaquetes("");
    setQuery("");
    setShowDrop(false);
  };

  const onEnterBuscador = () => {
    const q = normCodigo(query);
    if (!q) return;
    // Escáner: código exacto entra directo sin menús.
    const porCodigo = inventario.find((it) => normCodigo(it.codigo_barras ?? "") === q && q.length > 0);
    if (porCodigo) {
      elegir(porCodigo);
      return;
    }
    const elegido = resultados[selIdx] ?? resultados[0];
    if (elegido) elegir(elegido);
  };

  const limpiarEditor = () => {
    setLineaId(null);
    setLineaNombre("");
    setPresentacion("unidad");
    setCantidad("1");
    setPiezas("");
    setPaquetes("");
    setSugerido(null);
    setEditIdx(null);
  };

  const iniciarEdicion = (i: number) => {
    // Tocar el que ya se edita cancela y limpia.
    if (editIdx === i) {
      limpiarEditor();
      inputRef.current?.focus();
      return;
    }
    const r = renglones[i];
    setLineaId(r.producto_id);
    setLineaNombre(r.nombre);
    setPresentacion(r.presentacion);
    if (r.presentacion === "paquete") {
      const d = desgloseOPiezas(r.presentacion, r.cantidad, r.piezasPorPaquete, r.paquetes);
      setPiezas(d.piezasPorPaquete !== null ? String(d.piezasPorPaquete) : "");
      setPaquetes(d.paquetes !== null ? String(d.paquetes) : "");
    } else {
      setCantidad(String(r.cantidad));
    }
    setSugerido(r.sugerido);
    setEditIdx(i);
    inputRef.current?.focus();
  };

  const agregarRenglon = () => {
    if (!lineaNombre.trim()) {
      reportarError("Elige un producto primero", "Renglón incompleto");
      return;
    }
    if (presentacion === "paquete") {
      if (!Number.isFinite(numPiezas) || numPiezas <= 0 || !Number.isFinite(numPaquetes) || numPaquetes <= 0) {
        reportarError("Dime cuántas piezas trae el paquete y cuántos paquetes son", "Faltan datos del paquete");
        return;
      }
    } else if (!Number.isFinite(numCantidad) || numCantidad <= 0) {
      reportarError("Escribe una cantidad válida", "Renglón incompleto");
      return;
    }
    const total = presentacion === "paquete" ? (numPiezas * numPaquetes) : numCantidad;
    const nuevo: Renglon = {
      producto_id: lineaId, nombre: lineaNombre.trim(), presentacion,
      cantidad: total,
      piezasPorPaquete: presentacion === "paquete" ? numPiezas : null,
      paquetes: presentacion === "paquete" ? numPaquetes : null,
      sugerido,
    };
    if (editIdx !== null) {
      setRenglones((prev) => prev.map((r, j) => (j === editIdx ? nuevo : r)));
    } else {
      setRenglones((prev) => [...prev, nuevo]);
    }
    limpiarEditor();
    inputRef.current?.focus();
  };

  const confirmar = async () => {
    if (!renglones.length) {
      reportarError("Agrega al menos un producto", "Compra vacía");
      return;
    }
    const montoNum = Number(monto);
    if (!Number.isFinite(montoNum) || montoNum < 0) {
      reportarError("Escribe un monto válido", "Monto inválido");
      return;
    }
    setBusy(true);
    try {
      const items: ItemCompraEnvio[] = renglones.map((r) => ({
        producto_id: r.producto_id,
        nombre: r.nombre,
        presentacion: r.presentacion,
        cantidad: r.cantidad,
        piezasPorPaquete: r.piezasPorPaquete,
        paquetes: r.paquetes,
      }));
      if (esRectificar && compraOriginal) {
        // Rectificativa: mismo proveedor SIEMPRE (viene de la factura
        // original, bloqueado en la UI). Crea factura NUEVA.
        const r = await rectificarCompra({
          compraOriginalId: compraOriginal.id,
          items,
          montoPagado: montoNum,
          metodoPago: metodo,
          comentario: comentario.trim() || null,
        });
        notificarExito(`Rectificativa #${r.compra_id} guardada (la #${compraOriginal.id} queda intacta)`);
        onRegistrada(r.compra_id);
        return;
      }
      let prov = proveedor;
      if (!prov) {
        prov = await crearProveedorGenerico();
        onProveedor(prov);
        notificarExito(`Se generó ${prov.nombre}`);
      }
      const r = await registrarCompra({
        proveedorId: prov.id,
        items,
        montoPagado: montoNum,
        metodoPago: metodo,
        comentario: comentario.trim() || null,
      });
      notificarExito(`Compra #${r.compra_id} guardada${r.movimiento_pendiente ? " (egreso pendiente de corte)" : ""}`);
      onRegistrada(r.compra_id);
    } catch (e) {
      reportarError(esRectificar ? "No se pudo guardar la rectificación" : "No se pudo registrar la compra", e);
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4 backdrop-blur-sm" onClick={onCerrar}>
      <div
        className="w-full max-w-2xl bg-white rounded-[2rem] shadow-2xl max-h-[92vh] overflow-y-auto custom-scrollbar animate-in fade-in zoom-in-95 duration-200"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Proveedor fijo arriba (bloqueado al rectificar) */}
        <div className="px-6 sm:px-8 pt-6">
          {esRectificar && compraOriginal ? (
            <div className="rounded-2xl px-5 py-4 bg-amber-50 border-2 border-amber-200">
              <p className="text-[9px] font-black uppercase tracking-[0.25em] text-amber-600">Rectificando factura #{compraOriginal.id} · proveedor bloqueado</p>
              <p className="text-lg font-black uppercase truncate text-neutral-900">{proveedor ? proveedor.nombre : compraOriginal.proveedor}</p>
              <p className="text-[10px] font-bold text-amber-700 mt-1">Se crea una factura NUEVA; la #{compraOriginal.id} queda intacta.</p>
            </div>
          ) : (
            <>
              <div className="flex items-center gap-3 bg-neutral-950 text-white rounded-2xl px-5 py-4">
                <div className="flex-1 min-w-0">
                  <p className="text-[9px] font-black uppercase tracking-[0.25em] text-neutral-400">Pagar a</p>
                  <p className="text-lg font-black uppercase truncate">{proveedor ? proveedor.nombre : "Sin nombre"}</p>
                </div>
                <select
                  value={proveedor?.id ?? ""}
                  onChange={(e) => {
                    const id = Number(e.target.value);
                    onProveedor(proveedores.find((p) => p.id === id) ?? null);
                  }}
                  className="bg-white/10 border border-white/20 rounded-xl px-3 py-2.5 text-[10px] font-black uppercase tracking-widest outline-none focus:border-white max-w-[180px]"
                >
                  <option value="">Sin nombre (genera ID)</option>
                  {proveedores.map((p) => (
                    <option key={p.id} value={p.id} className="text-neutral-900">{p.nombre}</option>
                  ))}
                </select>
              </div>
              {!proveedor && (
                <p className="text-[10px] font-bold text-amber-600 mt-2 ml-1">Sin proveedor elegido: al confirmar se genera MOSTRADOR 00001 automático.</p>
              )}
            </>
          )}
        </div>

        <div className="px-6 sm:px-8 py-6 space-y-5">
          {/* Buscador embebido (escáner + texto) */}
          <div className="relative">
            <div className="absolute inset-y-0 left-0 pl-5 flex items-center pointer-events-none">
              <MorphIcon icon={ICONO_BUSCAR} size={20} strokeWidth={2.5} spring="smooth" className="text-neutral-300" />
            </div>
            <input
              ref={inputRef}
              value={query}
              onChange={(e) => {
                setQuery(e.target.value);
                setShowDrop(true);
              }}
              onKeyDown={(e) => {
                if (e.key === "Enter") onEnterBuscador();
                if (e.key === "ArrowDown") {
                  e.preventDefault();
                  setSelIdx((i) => Math.min(i + 1, Math.max(resultados.length - 1, 0)));
                }
                if (e.key === "ArrowUp") {
                  e.preventDefault();
                  setSelIdx((i) => Math.max(i - 1, 0));
                }
              }}
              onFocus={() => setShowDrop(true)}
              placeholder="Escanea o busca por nombre, código o categoría..."
              className="w-full pl-14 pr-6 py-5 bg-white border-2 border-neutral-100 rounded-[1.75rem] shadow-xl shadow-neutral-100/60 text-base font-black text-neutral-900 placeholder:text-neutral-300 placeholder:font-bold placeholder:text-sm focus:outline-none focus:border-neutral-900 focus:ring-8 focus:ring-neutral-900/5 transition-all"
            />
            {showDrop && resultados.length > 0 && (
              <div className="absolute top-full left-0 right-0 mt-3 bg-white border border-neutral-200 rounded-[2rem] shadow-2xl overflow-hidden z-50">
                <div className="p-2.5 max-h-72 overflow-y-auto custom-scrollbar">
                  {resultados.map((item, idx) => (
                    <button
                      key={item.id ?? idx}
                      onClick={() => elegir(item)}
                      className={`w-full flex items-center gap-4 p-3.5 rounded-2xl text-left transition-all ${idx === selIdx ? "bg-neutral-950 text-white" : "hover:bg-neutral-50 text-neutral-900"}`}
                    >
                      <div className={`w-11 h-11 rounded-2xl flex items-center justify-center shrink-0 ${idx === selIdx ? "bg-white/15" : "bg-neutral-100"}`}>
                        <MorphIcon icon={item.codigo_barras ? ICONO_CODIGO_BARRAS : ICONO_BOLSA} size={17} strokeWidth={2.2} spring="smooth" className={idx === selIdx ? "text-white" : "text-neutral-400"} />
                      </div>
                      <div className="flex-1 min-w-0">
                        <p className={`text-sm font-black truncate ${idx === selIdx ? "text-white" : "text-neutral-900"}`}>{item.nombre}</p>
                        <p className={`text-[10px] font-bold uppercase truncate ${idx === selIdx ? "text-white/50" : "text-neutral-400"}`}>Costo: ${item.precio_costo.toFixed(2)}</p>
                      </div>
                    </button>
                  ))}
                </div>
              </div>
            )}
          </div>

          {/* Renglón activo: unidad/paquete + cantidad */}
          {lineaNombre && (
            <div className="rounded-2xl bg-neutral-950 text-white p-5 space-y-4 animate-in fade-in slide-in-from-top-2">
              <p className="text-sm font-black uppercase truncate">{lineaNombre}</p>
              <div className="flex flex-wrap items-center gap-3">
                <div className="flex bg-white/10 rounded-xl p-1">
                  {(["unidad", "paquete"] as const).map((op) => (
                    <button
                      key={op}
                      onClick={() => setPresentacion(op)}
                      className={`px-5 py-2.5 rounded-lg text-[10px] font-black uppercase tracking-widest transition-all ${presentacion === op ? "bg-white text-neutral-900" : "text-neutral-400"}`}
                    >
                      {op}
                    </button>
                  ))}
                </div>
                {presentacion === "unidad" && (
                  <div className="flex items-center gap-2">
                    <span className="text-[10px] font-black uppercase tracking-widest text-neutral-400">Cant.</span>
                    <input
                      type="number"
                      min={0}
                      value={cantidad}
                      onChange={(e) => setCantidad(e.target.value)}
                      onKeyDown={(e) => {
                        if (e.key === "Enter") agregarRenglon();
                      }}
                      className="w-24 px-3 py-2.5 bg-white/10 border border-white/20 rounded-xl text-base font-black text-center outline-none focus:border-white"
                    />
                  </div>
                )}
              </div>
              {presentacion === "paquete" && (
                <div className="flex flex-wrap items-center gap-3 bg-white/5 border border-white/10 rounded-xl p-3">
                  <div className="flex items-center gap-2 flex-1 min-w-[140px]">
                    <span className="text-[10px] font-black uppercase tracking-widest text-neutral-400">Piezas por paquete</span>
                    <input
                      type="number"
                      min={0}
                      value={piezas}
                      onChange={(e) => setPiezas(e.target.value)}
                      onKeyDown={(e) => {
                        if (e.key === "Enter") agregarRenglon();
                      }}
                      placeholder="12"
                      className="w-full px-3 py-2.5 bg-white/10 border border-white/20 rounded-xl text-base font-black text-center outline-none focus:border-white placeholder:text-neutral-600"
                    />
                  </div>
                  <span className="text-white font-black">×</span>
                  <div className="flex items-center gap-2 flex-1 min-w-[140px]">
                    <span className="text-[10px] font-black uppercase tracking-widest text-neutral-400">Paquetes</span>
                    <input
                      type="number"
                      min={0}
                      value={paquetes}
                      onChange={(e) => setPaquetes(e.target.value)}
                      onKeyDown={(e) => {
                        if (e.key === "Enter") agregarRenglon();
                      }}
                      placeholder="3"
                      className="w-full px-3 py-2.5 bg-white/10 border border-white/20 rounded-xl text-base font-black text-center outline-none focus:border-white placeholder:text-neutral-600"
                    />
                  </div>
                  <span className="w-full text-center text-sm font-black text-emerald-400">
                    {Number.isFinite(unidadesTotales) && unidadesTotales > 0 ? `= ${unidadesTotales} uds en total` : "escribe piezas y paquetes"}
                  </span>
                </div>
              )}
              <div className="flex justify-end">
                <button onClick={agregarRenglon} className="px-6 py-2.5 bg-white text-neutral-900 rounded-xl text-[10px] font-black uppercase tracking-widest hover:scale-105 active:scale-95 transition-all">
                  {editIdx !== null ? "✓ Guardar cambios" : "+ Agregar"}
                </button>
              </div>
              <p className="text-[11px] font-bold text-neutral-400">
                {cargandoSug ? "Calculando sugerencia..." : sugerido !== null ? <>Sugerido: <span className="text-emerald-400 font-black">${sugerido.toFixed(2)}</span> (costo registrado)</> : "sin recomendación."}
              </p>
            </div>
          )}

          {/* Renglones */}
          {renglones.length > 0 && (
            <div className="space-y-2">
              {renglones.map((r, i) => (
                <div
                  key={i}
                  onClick={() => iniciarEdicion(i)}
                  title="Toca para editar"
                  className={`flex items-center gap-3 p-4 rounded-2xl border transition-all cursor-pointer ${editIdx === i ? "bg-neutral-950 text-white border-neutral-950 shadow-lg" : "bg-neutral-50 border-neutral-100 hover:border-neutral-900"}`}
                >
                  <div className="flex-1 min-w-0">
                    <p className={`text-xs font-black uppercase truncate ${editIdx === i ? "text-white" : "text-neutral-900"}`}>{r.nombre}</p>
                    <p className={`text-[10px] font-bold ${editIdx === i ? "text-white/60" : "text-neutral-400"}`}>
                      {r.presentacion === "paquete" && r.piezasPorPaquete !== null && r.paquetes !== null
                        ? `${r.paquetes} paq × ${r.piezasPorPaquete} pzas = ${r.cantidad} uds`
                        : `${r.cantidad} ${r.presentacion}`} · {r.sugerido !== null ? `sug. $${r.sugerido.toFixed(2)}` : "sin recomendación"}
                    </p>
                  </div>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      iniciarEdicion(i);
                    }}
                    title="Editar renglón"
                    aria-label="Editar renglón"
                    className={`p-2.5 rounded-xl border-2 transition-all shrink-0 ${editIdx === i ? "border-white text-white" : "border-neutral-900 text-neutral-900 hover:bg-neutral-900 hover:text-white"}`}
                  >
                    <MorphIcon icon={ICONO_EDITAR} size={15} strokeWidth={2.4} spring="snappy" reducedMotion="user" />
                  </button>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      // Borrar desfasa índices: se cancela cualquier edición.
                      limpiarEditor();
                      setRenglones((prev) => prev.filter((_, j) => j !== i));
                    }}
                    title="Quitar renglón"
                    aria-label="Quitar renglón"
                    className="p-2.5 rounded-xl border-2 border-neutral-900 text-neutral-900 hover:bg-red-500 hover:border-red-500 hover:text-white transition-all shrink-0"
                  >
                    <MorphIcon icon={ICONO_BORRAR} size={15} strokeWidth={2.4} spring="snappy" reducedMotion="user" />
                  </button>
                </div>
              ))}
            </div>
          )}

          {/* Método + monto */}
          <div className="flex gap-2">
            {["efectivo", "tarjeta", "transferencia"].map((m) => (
              <button
                key={m}
                onClick={() => setMetodo(m)}
                className={`flex-1 py-3.5 rounded-2xl text-[10px] font-black uppercase tracking-widest border-2 transition-all ${metodo === m ? "bg-neutral-950 text-white border-neutral-950" : "bg-white text-neutral-400 border-neutral-100"}`}
              >
                {m}
              </button>
            ))}
          </div>
          <div>
            <label className="text-[10px] font-black uppercase tracking-widest text-neutral-400 ml-1">Monto a pagar</label>
            <input
              type="number"
              min={0}
              value={monto}
              onChange={(e) => {
                setMonto(e.target.value);
                setMontoDirty(true);
              }}
              placeholder="0.00"
              className="mt-2 w-full px-5 py-4 bg-neutral-50 border-2 border-neutral-100 rounded-2xl text-xl font-black text-neutral-900 outline-none focus:border-neutral-900 transition-all"
            />
          </div>

          {/* Comentario colapsable */}
          {!comentarioOpen ? (
            <button onClick={() => setComentarioOpen(true)} className="w-full py-4 rounded-2xl border-2 border-dashed border-neutral-200 text-[10px] font-black uppercase tracking-widest text-neutral-400 hover:text-neutral-900 hover:border-neutral-900 transition-all">
              da click para agregar un comentario
            </button>
          ) : (
            <textarea
              autoFocus
              value={comentario}
              onChange={(e) => setComentario(e.target.value)}
              placeholder="Ej. Llegaron 2 rotas, se descontaron..."
              rows={2}
              className="w-full px-5 py-4 bg-neutral-50 border-2 border-neutral-100 rounded-2xl text-sm font-bold text-neutral-900 outline-none focus:border-neutral-900 transition-all resize-none"
            />
          )}

          <div className="flex gap-3">
            <button onClick={onCerrar} className="flex-1 py-4 rounded-2xl border-2 border-neutral-200 text-neutral-500 text-xs font-black uppercase tracking-widest hover:border-neutral-900 hover:text-neutral-900 transition-all">
              Cancelar
            </button>
            <button
              disabled={busy}
              onClick={confirmar}
              className="flex-[2] py-4 rounded-2xl bg-neutral-950 text-white text-xs font-black uppercase tracking-widest shadow-xl shadow-neutral-300 hover:scale-[1.01] active:scale-95 transition-all disabled:opacity-40"
            >
              {busy ? "Guardando..." : esRectificar ? "Guardar rectificación" : "Confirmar"}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default ModalCompra;
