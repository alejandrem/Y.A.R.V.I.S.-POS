// ═══════════════════════════════════════════════════════════════════════════
// PROVEEDORES DEL EMPLEADO — Recepción de mercancía y pagos.
// Header + 2 botones gordos (alta y pago) + historial de facturas.
// Cada modal vive en ./componentes (1 archivo = 1 tarea).
// ═══════════════════════════════════════════════════════════════════════════

import { useEffect, useState } from "react";
import { MorphIcon } from "morphicons/react";
import { ICONO_EDITAR } from "../../../components/ui";
import { reportarError } from "../../../services/tauri";
import {
  historialCompras, listarProveedores, obtenerCompraDetalle,
  type CompraDetalle, type CompraRow, type Proveedor,
} from "../../../services/proveedores";
import ModalAlta from "./componentes/modal-alta";
import ModalCompra from "./componentes/modal-compra";
import ModalFactura from "./componentes/modal-factura";

const proveedoresNav = {
  id: "proveedores",
  label: "PROVEEDORES",
  icon: (
    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M10 17h4V5H2v12h3" />
      <path d="M20 17h2v-3.34a4 4 0 0 0-1.17-2.83L19 9h-5v8h1" />
      <circle cx="7.5" cy="17.5" r="2.5" />
      <circle cx="17.5" cy="17.5" r="2.5" />
    </svg>
  ),
};

interface ProveedoresProps {
  activeTab: string;
}

const Proveedores = ({ activeTab }: ProveedoresProps) => {
  const [proveedores, setProveedores] = useState<Proveedor[]>([]);
  const [compras, setCompras] = useState<CompraRow[] | null>(null);
  const [altaOpen, setAltaOpen] = useState(false);
  const [compraProv, setCompraProv] = useState<Proveedor | null | undefined>(undefined);
  const [facturaId, setFacturaId] = useState<number | null>(null);
  // Rectificativa en curso: detalle origen + proveedor bloqueado.
  const [rectificando, setRectificando] = useState<CompraDetalle | null>(null);

  const recargar = async () => {
    try {
      const [provs, hist] = await Promise.all([
        listarProveedores().catch(() => [] as Proveedor[]),
        historialCompras(null, 100, 0).catch(() => [] as CompraRow[]),
      ]);
      setProveedores(provs || []);
      setCompras(hist || []);
    } catch (e) {
      reportarError("No se pudieron cargar los proveedores", e);
    }
  };

  useEffect(() => {
    if (activeTab === "proveedores") recargar();
  }, [activeTab]);

  if (activeTab !== "proveedores") return null;

  const abrirCompra = (prov: Proveedor | null) => {
    setAltaOpen(false);
    setRectificando(null);
    setCompraProv(prov);
  };

  // Editar directo desde el historial: carga la factura y abre la
  // rectificativa precargada (proveedor bloqueado, crea factura nueva).
  const editarDirecto = async (id: number) => {
    try {
      const det = await obtenerCompraDetalle(id);
      iniciarRectificacion(det);
    } catch (e) {
      reportarError("No se pudo abrir la factura para editar", e);
    }
  };
  // Editar = abrir rectificativa precargada (proveedor bloqueado).
  // La factura original jamás se toca: se crea una NUEVA que apunta.
  const iniciarRectificacion = async (detalle: CompraDetalle) => {
    const prov = proveedores.find((p) => p.nombre === detalle.proveedor)
      ?? await listarProveedores()
        .then((todos) => {
          setProveedores(todos || []);
          return (todos || []).find((p) => p.nombre === detalle.proveedor) ?? null;
        })
        .catch(() => null);
    setFacturaId(null);
    setCompraProv(prov);
    setRectificando(detalle);
  };

  return (
    <div className="w-full max-w-[1200px] mx-auto space-y-6 animate-in fade-in slide-in-from-bottom-2 duration-500">
      <header>
        <h2 className="text-3xl font-black text-neutral-900 uppercase tracking-tight">Proveedores</h2>
        <div className="h-1.5 w-12 bg-neutral-900 rounded-full mt-2" />
        <p className="text-sm text-neutral-500 font-bold mt-2">
          Recepción de mercancía y pagos a quienes te surten
        </p>
      </header>

      <div className="grid grid-cols-1 lg:grid-cols-5 gap-4">
        {/* Izquierda: mis proveedores (unos 6 visibles, scroll si hay más) */}
        <section className="lg:col-span-3 bg-white rounded-[2rem] border border-neutral-100 shadow-xl p-6">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-xs font-black text-neutral-900 uppercase tracking-widest">Mis proveedores</h3>
            <span className="rounded-xl bg-neutral-100 text-neutral-500 px-3 py-1.5 text-[10px] font-black">{proveedores.length}</span>
          </div>
          {proveedores.length === 0 ? (
            <div className="py-10 text-center border-2 border-dashed border-neutral-100 rounded-3xl">
              <p className="text-[10px] font-black text-neutral-300 uppercase tracking-widest">Sin proveedores todavía</p>
            </div>
          ) : (
            <div className="space-y-2 max-h-[460px] overflow-y-auto custom-scrollbar pr-1">
              {proveedores.map((p) => (
                <button
                  key={p.id}
                  onClick={() => abrirCompra(p)}
                  className="w-full text-left flex items-center gap-4 p-4 bg-neutral-50 rounded-2xl border border-neutral-100 hover:border-neutral-900 hover:shadow-md transition-all"
                >
                  <div className="w-12 h-12 bg-neutral-950 text-white rounded-2xl flex items-center justify-center shrink-0 text-lg font-black">
                    {p.nombre.charAt(0).toUpperCase()}
                  </div>
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-black text-neutral-900 uppercase truncate">{p.nombre}</p>
                    <p className="text-[10px] font-bold text-neutral-400 truncate">
                      {p.telefono ?? "sin teléfono"} · {p.total_compras} {p.total_compras === 1 ? "compra" : "compras"} · ${p.total_pagado.toFixed(2)}
                    </p>
                  </div>
                  <span className="text-neutral-300 shrink-0">›</span>
                </button>
              ))}
            </div>
          )}
          <p className="text-[10px] text-neutral-400 font-bold mt-4">Toca un proveedor para comprarle directo</p>
        </section>

        {/* Derecha: acciones apiladas */}
        <div className="lg:col-span-2 space-y-4">
          <button
            onClick={() => setAltaOpen(true)}
            className="w-full text-left bg-white rounded-[2rem] border border-neutral-100 shadow-xl p-6 hover:scale-[1.01] hover:border-neutral-900 active:scale-[0.99] transition-all"
          >
            <div className="w-14 h-14 bg-neutral-950 text-white rounded-2xl flex items-center justify-center text-2xl font-black shadow-lg mb-4">+</div>
            <p className="text-lg font-black text-neutral-900 uppercase tracking-tight">Agregar proveedor</p>
            <p className="text-[11px] font-bold text-neutral-400 mt-1">Alta en 30 segundos: nombre, teléfono y correo</p>
          </button>
          <button
            onClick={() => setCompraProv(null)}
            className="w-full text-left bg-neutral-950 text-white rounded-[2rem] shadow-xl shadow-neutral-300 p-6 hover:scale-[1.01] active:scale-[0.99] transition-all"
          >
            <div className="w-14 h-14 bg-white text-neutral-950 rounded-2xl flex items-center justify-center text-2xl font-black shadow-lg mb-4">$</div>
            <p className="text-lg font-black uppercase tracking-tight">Pagar al proveedor</p>
            <p className="text-[11px] font-bold text-neutral-400 mt-1">Directo a compra y pago, sin configurar nada</p>
          </button>
        </div>
      </div>

      <section className="bg-white rounded-[2.5rem] border border-neutral-100 shadow-xl p-6 sm:p-8">
        <div className="flex items-center justify-between mb-6">
          <div>
            <p className="text-[10px] font-black uppercase tracking-[0.35em] text-neutral-400">Facturas</p>
            <h3 className="text-xl font-black text-neutral-900 mt-1">Historial de pagos</h3>
          </div>
          {compras !== null && (
            <span className="rounded-xl bg-neutral-950 text-white px-3 py-1.5 text-[10px] font-black">{compras.length} pagos</span>
          )}
        </div>
        {compras === null ? (
          <div className="py-10 flex justify-center">
            <div className="w-8 h-8 rounded-full border-2 border-neutral-900 border-t-transparent animate-spin" />
          </div>
        ) : compras.length === 0 ? (
          <div className="py-12 text-center border-2 border-dashed border-neutral-100 rounded-3xl">
            <p className="text-[10px] font-black text-neutral-300 uppercase tracking-widest">Aún no hay pagos a proveedores</p>
          </div>
        ) : (
          <div className="grid gap-3 max-h-96 overflow-y-auto custom-scrollbar pr-1">
            {compras.map((c) => (
              <button
                key={c.id}
                onClick={() => setFacturaId(c.id)}
                className="w-full text-left flex items-center gap-4 p-4 bg-neutral-50 rounded-2xl border border-neutral-100 hover:border-neutral-900 hover:shadow-md transition-all"
              >
                <div className="w-12 h-12 bg-neutral-950 text-white rounded-2xl flex items-center justify-center shrink-0 text-[10px] font-black">
                  #{c.id}
                </div>
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-black text-neutral-900 truncate">
                    {c.proveedor} · ${c.pagado.toFixed(2)} {c.rectifica_a != null ? (
                      <span className="ml-1 text-[9px] font-black uppercase text-sky-600">↩ rectifica #{c.rectifica_a}</span>
                    ) : c.rectificada ? (
                      <span className="ml-1 text-[9px] font-black uppercase text-amber-600">! rectificada</span>
                    ) : c.movimiento_pendiente ? (
                      <span className="ml-1 text-[9px] font-black uppercase text-amber-600">! pendiente de corte</span>
                    ) : (
                      <span className="ml-1 text-[9px] font-black uppercase text-emerald-600">pagado</span>
                    )}
                  </p>
                  <p className="text-[10px] font-bold text-neutral-400 truncate">{c.fecha} · {c.items} renglones · {c.metodo_pago}</p>
                </div>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    editarDirecto(c.id);
                  }}
                  title="Editar (crea rectificativa)"
                  aria-label={`Editar factura #${c.id}`}
                  className="p-2.5 rounded-xl bg-neutral-950 text-white hover:scale-110 active:scale-95 transition-all shrink-0"
                >
                  <MorphIcon icon={ICONO_EDITAR} size={15} strokeWidth={2.4} spring="snappy" reducedMotion="user" />
                </button>
                <span className="text-neutral-300 shrink-0">›</span>
              </button>
            ))}
          </div>
        )}
      </section>

      {altaOpen && (
        <ModalAlta
          onCerrar={() => setAltaOpen(false)}
          onGuardado={() => {
            setAltaOpen(false);
            recargar();
          }}
          onComprar={(prov) => {
            setProveedores((prev) => [...prev, prov]);
            abrirCompra(prov);
          }}
        />
      )}
      {compraProv !== undefined && (
        <ModalCompra
          proveedor={compraProv}
          proveedores={proveedores}
          onProveedor={setCompraProv}
          onCerrar={() => {
            setCompraProv(undefined);
            setRectificando(null);
          }}
          onRegistrada={(id) => {
            setCompraProv(undefined);
            setRectificando(null);
            recargar();
            setFacturaId(id);
          }}
          modo={rectificando ? "rectificar" : "nueva"}
          compraOriginal={rectificando}
        />
      )}
      {facturaId !== null && (
        <ModalFactura
          compraId={facturaId}
          onCerrar={() => setFacturaId(null)}
          onEditar={(det) => iniciarRectificacion(det)}
          onVerFactura={(id) => setFacturaId(id)}
        />
      )}
    </div>
  );
};

export default Proveedores;
export { proveedoresNav };
