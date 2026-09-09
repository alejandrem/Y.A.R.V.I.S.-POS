// Modal de detalle de un corte importado: totales, verificación e items.
import { useEffect, useState } from "react";
import { reportarError } from "../../../../services/tauri";
import {
  obtenerCorteImportadoDetalle, type CorteImportadoDetalle, type CorteImportadoRow,
  type ItemImportado,
} from "../../../../services/cortes";

interface DetalleCorteProps {
  corte: CorteImportadoRow;
  onCerrar: () => void;
}

const KIND_LABEL: Record<string, string> = {
  ARTICULO: "Artículos",
  TICKET: "Tickets",
  INGRESO: "Ingresos",
  EGRESO: "Egresos",
};

const DetalleCorte = ({ corte, onCerrar }: DetalleCorteProps) => {
  const [detalle, setDetalle] = useState<CorteImportadoDetalle | null>(null);

  useEffect(() => {
    let viva = true;
    obtenerCorteImportadoDetalle(corte.id)
      .then((d) => {
        if (viva) setDetalle(d);
      })
      .catch((e) => reportarError("No se pudo abrir el detalle del corte", e));
    return () => {
      viva = false;
    };
  }, [corte.id]);

  useEffect(() => {
    const tecla = (e: KeyboardEvent) => {
      if (e.key === "Escape") onCerrar();
    };
    document.addEventListener("keydown", tecla);
    return () => document.removeEventListener("keydown", tecla);
  }, [onCerrar]);

  const grupos = (detalle?.items ?? []).reduce<Record<string, ItemImportado[]>>((acc, it) => {
    (acc[it.kind] = acc[it.kind] || []).push(it);
    return acc;
  }, {});

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4 backdrop-blur-sm"
      onClick={onCerrar}
    >
      <div
        className="w-full max-w-2xl bg-white rounded-[2rem] border border-neutral-200 shadow-2xl overflow-hidden animate-in fade-in zoom-in-95 duration-200"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between px-6 py-5 border-b border-neutral-100">
          <div>
            <h3 className="text-lg font-black text-neutral-900 uppercase tracking-tight">
              Corte {corte.tipo}{corte.folio ? ` #${corte.folio}` : ""}
            </h3>
            <p className="text-[10px] font-bold text-neutral-400 uppercase tracking-widest mt-1">
              {[corte.estacion, corte.cajero, corte.fecha].filter(Boolean).join(" · ")}
            </p>
          </div>
          <button
            onClick={onCerrar}
            className="w-9 h-9 flex items-center justify-center rounded-xl bg-neutral-100 text-neutral-500 hover:bg-neutral-900 hover:text-white transition-all text-xl"
            aria-label="Cerrar detalle"
          >
            ×
          </button>
        </div>

        <div className="px-6 py-5 max-h-[60vh] overflow-y-auto custom-scrollbar">
          {detalle === null ? (
            <div className="py-10 flex justify-center">
              <div className="w-8 h-8 rounded-full border-2 border-neutral-900 border-t-transparent animate-spin" />
            </div>
          ) : (
            <div className="space-y-5">
              <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
                {[
                  ["Ingresos", detalle.total_ingresos],
                  ["Egresos", detalle.total_egresos],
                  ["En caja", detalle.total_caja],
                  ["Ventas", detalle.total_ventas],
                ].map(([label, value]) => (
                  <div key={String(label)} className="rounded-2xl bg-neutral-50 border border-neutral-100 p-4">
                    <p className="text-[9px] font-black uppercase tracking-widest text-neutral-400">{label}</p>
                    <p className="text-lg font-black text-neutral-900 mt-1">${Number(value).toFixed(2)}</p>
                  </div>
                ))}
              </div>
              <div className="flex gap-2">
                <span className={`px-3 py-1.5 rounded-xl text-[10px] font-black uppercase tracking-widest ${detalle.caja_ok ? "bg-emerald-50 text-emerald-700" : "bg-amber-50 text-amber-700 border border-amber-200"}`}>
                  {detalle.caja_ok ? "✓ caja cuadra" : "! caja descuadrada"}
                </span>
                <span className={`px-3 py-1.5 rounded-xl text-[10px] font-black uppercase tracking-widest ${detalle.ventas_ok ? "bg-emerald-50 text-emerald-700" : "bg-amber-50 text-amber-700 border border-amber-200"}`}>
                  {detalle.ventas_ok ? "✓ ventas cuadran" : "! ventas descuadradas"}
                </span>
              </div>
              {Object.entries(grupos).map(([kind, items]) => (
                <div key={kind}>
                  <p className="text-[10px] font-black uppercase tracking-widest text-neutral-400 mb-2">
                    {KIND_LABEL[kind] ?? kind} ({items.length})
                  </p>
                  <div className="space-y-2 max-h-48 overflow-y-auto custom-scrollbar pr-1">
                    {items.map((it, i) => (
                      <div key={i} className="flex items-center justify-between gap-3 p-3 bg-neutral-50 rounded-xl border border-neutral-100">
                        <div className="min-w-0">
                          <p className="text-[11px] font-black text-neutral-900 uppercase truncate">{it.nombre}</p>
                          <p className="text-[10px] font-bold text-neutral-400">
                            {it.cantidad !== null && it.cantidad !== undefined ? `${it.cantidad} × $${it.precio_unitario.toFixed(2)}` : `$${it.precio_unitario.toFixed(2)}`}
                          </p>
                        </div>
                        <p className="text-xs font-black text-neutral-900 shrink-0">${it.subtotal.toFixed(2)}</p>
                      </div>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="px-6 py-4 border-t border-neutral-100 bg-neutral-50/60">
          <button
            onClick={onCerrar}
            className="w-full py-3 rounded-xl bg-neutral-900 text-white text-[10px] font-black uppercase tracking-widest hover:bg-neutral-700 transition-all"
          >
            Cerrar
          </button>
        </div>
      </div>
    </div>
  );
};

export default DetalleCorte;
