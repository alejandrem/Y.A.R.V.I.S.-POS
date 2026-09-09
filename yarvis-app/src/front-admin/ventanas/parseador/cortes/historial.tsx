// Historial de cortes importados: lista paginada + modal de detalle.
import { useEffect, useState } from "react";
import { reportarError } from "../../../../services/tauri";
import {
  obtenerCortesImportados, PAGE_SIZE_CORTES, type CorteImportadoRow,
} from "../../../../services/cortes";
import DetalleCorte from "./detalle";

const HistorialCortes = () => {
  const [cortes, setCortes] = useState<CorteImportadoRow[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const [hayMas, setHayMas] = useState(true);
  const [seleccionado, setSeleccionado] = useState<CorteImportadoRow | null>(null);

  useEffect(() => {
    const load = async () => {
      try {
        const rows = await obtenerCortesImportados(PAGE_SIZE_CORTES, 0).catch(() => []);
        setCortes(rows || []);
        setHayMas((rows || []).length >= PAGE_SIZE_CORTES);
      } catch (e) {
        reportarError("No se pudo cargar el historial de cortes", e);
      } finally {
        setLoading(false);
      }
    };
    load();
  }, []);

  const cargarMas = async () => {
    if (loadingMore || !hayMas) return;
    setLoadingMore(true);
    try {
      const mas = await obtenerCortesImportados(PAGE_SIZE_CORTES, cortes.length).catch(() => []);
      setCortes((prev) => [...prev, ...(mas || [])]);
      if ((mas || []).length < PAGE_SIZE_CORTES) setHayMas(false);
    } catch (e) {
      reportarError("No se pudieron cargar más cortes", e);
    } finally {
      setLoadingMore(false);
    }
  };

  if (loading) {
    return (
      <section className="bg-white rounded-[2.5rem] border border-neutral-100 shadow-xl p-10 text-center">
        <p className="text-sm font-bold text-neutral-400">Cargando historial...</p>
      </section>
    );
  }

  return (
    <div className="space-y-6">
      <section className="bg-white rounded-[2.5rem] border border-neutral-100 shadow-xl p-6 sm:p-10">
        <div className="flex items-center justify-between mb-6">
          <div>
            <p className="text-[10px] font-black uppercase tracking-[0.35em] text-neutral-400">Cortes</p>
            <h3 className="text-xl font-black text-neutral-900 mt-1">Cortes ya parseados</h3>
          </div>
          <span className="rounded-xl bg-neutral-950 text-white px-3 py-1.5 text-[10px] font-black">{cortes.length}{hayMas ? "+" : ""} cortes</span>
        </div>
        {cortes.length === 0 ? (
          <p className="text-sm text-neutral-400 text-center py-10 border-2 border-dashed border-neutral-100 rounded-2xl">No se han parseado cortes</p>
        ) : (
          <div className="grid gap-3 max-h-80 overflow-y-auto custom-scrollbar pr-1">
            {cortes.map((c) => (
              <button key={c.id} onClick={() => setSeleccionado(c)} className="w-full text-left flex gap-4 p-4 bg-neutral-50 rounded-2xl border border-neutral-100 hover:border-neutral-900 transition-all">
                <div className={`w-10 h-10 rounded-xl flex items-center justify-center shrink-0 text-[10px] font-black ${c.tipo === "Z" ? "bg-neutral-950 text-white" : "bg-white border border-neutral-200 text-neutral-900"}`}>
                  {c.tipo}
                </div>
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-black text-neutral-900 truncate">
                    Corte {c.tipo}{c.folio ? ` #${c.folio}` : ""} · ${c.total_caja.toFixed(2)}
                    <span className={`ml-2 text-[9px] font-black uppercase ${c.verificado ? "text-emerald-600" : "text-amber-600"}`}>
                      {c.verificado ? "✓ verificado" : "! revisar"}
                    </span>
                  </p>
                  <p className="text-[10px] text-neutral-400 truncate">
                    {[c.estacion, c.cajero, c.fecha].filter(Boolean).join(" · ")}
                    {c.clientes_atendidos > 0 && ` · ${c.clientes_atendidos} clientes`}
                  </p>
                </div>
                <span className="text-neutral-300 shrink-0 self-center">›</span>
              </button>
            ))}
            {hayMas && (
              <button
                onClick={cargarMas}
                disabled={loadingMore}
                className="mt-2 w-full rounded-2xl border-2 border-dashed border-neutral-200 py-3 text-xs font-black text-neutral-500 hover:text-neutral-900 hover:border-neutral-900 transition disabled:opacity-50"
              >
                {loadingMore ? "Cargando..." : "Ver más"}
              </button>
            )}
          </div>
        )}
      </section>
      {seleccionado && <DetalleCorte corte={seleccionado} onCerrar={() => setSeleccionado(null)} />}
    </div>
  );
};

export default HistorialCortes;
