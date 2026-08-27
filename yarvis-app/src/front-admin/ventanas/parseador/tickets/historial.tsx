// Historial de parseos: tablas maestras y tickets ya procesados.
import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface CatalogoImportado {
  id: number;
  hash: string;
  ruta_archivo: string;
  fecha_importacion: string;
  total_productos: number;
}

interface TicketDb {
  id: number;
  folio_ticket: string | null;
  fecha: string;
  total: number;
  metodo_pago: string;
}

const Historial = () => {
  const [catalogos, setCatalogos] = useState<CatalogoImportado[]>([]);
  const [tickets, setTickets] = useState<TicketDb[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const load = async () => {
      try {
        const [cats, tks] = await Promise.all([
          invoke<CatalogoImportado[]>("get_catalogos_importados").catch(() => []),
          invoke<TicketDb[]>("get_tickets").catch(() => []),
        ]);
        setCatalogos(cats || []);
        setTickets((tks || []).slice(0, 20));
      } finally {
        setLoading(false);
      }
    };
    load();
  }, []);

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
          <span className="rounded-xl bg-neutral-950 text-white px-3 py-1.5 text-[10px] font-black">{tickets.length} tickets</span>
        </div>
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
          </div>
        )}
      </section>
    </div>
  );
};

export default Historial;
