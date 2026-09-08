// Historial de parseos: tablas maestras y tickets ya procesados.
import { useEffect, useState } from "react";
import { reportarError } from "../../../../services/tauri";
import { obtenerTickets, obtenerTotalTickets, PAGE_SIZE_TICKETS, type TicketDb } from "../../../../services/tickets";
import { obtenerCatalogosImportados, type CatalogoImportado } from "../../../../services/inventario";

const PAGE_SIZE = PAGE_SIZE_TICKETS;

const Historial = () => {
  const [catalogos, setCatalogos] = useState<CatalogoImportado[]>([]);
  const [tickets, setTickets] = useState<TicketDb[]>([]);
  const [totalTickets, setTotalTickets] = useState(0);
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);

  useEffect(() => {
    const load = async () => {
      try {
        const [cats, tks, total] = await Promise.all([
          obtenerCatalogosImportados().catch(() => []),
          obtenerTickets(PAGE_SIZE, 0).catch(() => []),
          obtenerTotalTickets().catch(() => 0),
        ]);
        setCatalogos(cats || []);
        // Paginado: primera página de 100; el resto se carga con "Ver más".
        setTickets(tks || []);
        setTotalTickets(total || (tks || []).length);
      } catch (e) {
        reportarError("No se pudo cargar el historial", e);
      } finally {
        setLoading(false);
      }
    };
    load();
  }, []);

  const cargarMas = async () => {
    if (loadingMore || tickets.length >= totalTickets) return;
    setLoadingMore(true);
    try {
      const mas = await obtenerTickets(PAGE_SIZE, tickets.length).catch(() => []);
      setTickets((prev) => [...prev, ...(mas || [])]);
    } catch (e) {
      reportarError("No se pudieron cargar más tickets", e);
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
      {/* Tablas maestras */}
      <section className="bg-white rounded-[2.5rem] border border-neutral-100 shadow-xl p-6 sm:p-10">
        <div className="flex items-center justify-between mb-6">
          <div>
            <p className="text-[10px] font-black uppercase tracking-[0.35em] text-neutral-400">Tablas maestras</p>
            <h3 className="text-xl font-black text-neutral-900 mt-1">Catálogos ya parseados</h3>
          </div>
          <span className="rounded-xl bg-neutral-950 text-white px-3 py-1.5 text-[10px] font-black">{catalogos.length} tablas</span>
        </div>
        {catalogos.length === 0 ? (
          <p className="text-sm text-neutral-400 text-center py-10 border-2 border-dashed border-neutral-100 rounded-2xl">No se han parseado tablas maestras</p>
        ) : (
          <div className="grid gap-3 max-h-80 overflow-y-auto custom-scrollbar pr-1">
            {catalogos.map((cat) => (
              <div key={cat.id} className="flex gap-4 p-4 bg-neutral-50 rounded-2xl border border-neutral-100">
                <div className="w-10 h-10 bg-white border border-neutral-200 rounded-xl flex items-center justify-center shrink-0 text-neutral-900 font-black text-[10px]">
                  {cat.total_productos}
                </div>
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-black text-neutral-900 truncate">{cat.ruta_archivo.split("/").pop() || cat.ruta_archivo}</p>
                  <p className="text-[10px] text-neutral-400 truncate">{cat.ruta_archivo}</p>
                  <p className="text-[10px] text-neutral-400 mt-1">{new Date(cat.fecha_importacion).toLocaleDateString()} · {cat.total_productos} productos · {cat.hash.slice(0, 8)}</p>
                </div>
              </div>
            ))}
          </div>
        )}
      </section>

      {/* Tickets */}
      <section className="bg-white rounded-[2.5rem] border border-neutral-100 shadow-xl p-6 sm:p-10">
        <div className="flex items-center justify-between mb-6">
          <div>
            <p className="text-[10px] font-black uppercase tracking-[0.35em] text-neutral-400">Tickets</p>
            <h3 className="text-xl font-black text-neutral-900 mt-1">Tickets ya parseados</h3>
          </div>
          <span className="rounded-xl bg-neutral-950 text-white px-3 py-1.5 text-[10px] font-black">{totalTickets} tickets</span>
        </div>
        {totalTickets > tickets.length && (
          <p className="text-[10px] text-neutral-400 mb-4">Mostrando {tickets.length} de {totalTickets} (los más recientes).</p>
        )}
        {tickets.length === 0 ? (
          <p className="text-sm text-neutral-400 text-center py-10 border-2 border-dashed border-neutral-100 rounded-2xl">No se han parseado tickets</p>
        ) : (
          <div className="grid gap-3 max-h-80 overflow-y-auto custom-scrollbar pr-1">
            {tickets.map((t) => (
              <div key={t.id} className="flex gap-4 p-4 bg-neutral-50 rounded-2xl border border-neutral-100">
                <div className="w-10 h-10 bg-white border border-neutral-200 rounded-xl flex items-center justify-center shrink-0">
                  <span className="text-[10px] font-black text-neutral-900">#{t.id}</span>
                </div>
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-black text-neutral-900 truncate">{t.folio_ticket || `Ticket #${t.id}`} · ${t.total.toFixed(2)} · {t.metodo_pago}</p>
                  <p className="text-[10px] text-neutral-400">{new Date(t.fecha).toLocaleString()}</p>
                  <p className="text-[10px] text-neutral-500 mt-1 truncate">Preview: {t.folio_ticket || "sin folio"} — {(t as any).preview || ""}</p>
                </div>
              </div>
            ))}
            {tickets.length < totalTickets && (
              <button
                onClick={cargarMas}
                disabled={loadingMore}
                className="mt-2 w-full rounded-2xl border-2 border-dashed border-neutral-200 py-3 text-xs font-black text-neutral-500 hover:text-neutral-900 hover:border-neutral-900 transition disabled:opacity-50"
              >
                {loadingMore ? "Cargando..." : `Ver más (${totalTickets - tickets.length} restantes)`}
              </button>
            )}
          </div>
        )}
      </section>
    </div>
  );
};

export default Historial;
