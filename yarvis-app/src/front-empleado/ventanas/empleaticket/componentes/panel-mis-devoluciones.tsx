import { useEffect, useState } from "react";
import { MorphIcon } from "morphicons/react";
import { ICONO_DOCUMENTO } from "../../../../components/ui";
import { reportarError } from "../../../../services/tauri";
import { obtenerMisDevoluciones, type DevolucionRegistrada } from "../../../../services/devoluciones";

const dinero = (monto: number) => new Intl.NumberFormat("es-MX", { style: "currency", currency: "MXN" }).format(monto);

const PanelMisDevoluciones = () => {
  const [devoluciones, setDevoluciones] = useState<DevolucionRegistrada[] | null>(null);

  useEffect(() => {
    let vivo = true;
    obtenerMisDevoluciones()
      .then((filas) => { if (vivo) setDevoluciones(filas); })
      .catch((error) => reportarError("No se pudieron cargar tus devoluciones", error));
    return () => { vivo = false; };
  }, []);

  return (
    <section className="bg-white rounded-[2.5rem] border border-neutral-200 p-4 sm:p-8 shadow-sm">
      <h3 className="text-xs font-black text-neutral-900 uppercase tracking-widest flex items-center gap-2 mb-6">
        <span className="w-1.5 h-4 bg-neutral-900 rounded-full" />
        Mis devoluciones
        {devoluciones !== null && <span className="px-2 py-1 bg-neutral-100 text-neutral-500 text-[9px] font-black rounded-lg">{devoluciones.length}</span>}
      </h3>
      <div className="h-64 flex flex-col border-2 border-dashed border-neutral-100 rounded-3xl bg-neutral-50/50 overflow-hidden">
        {devoluciones === null ? (
          <div className="flex-1 flex items-center justify-center"><div className="w-8 h-8 rounded-full border-2 border-neutral-900 border-t-transparent animate-spin" /></div>
        ) : devoluciones.length === 0 ? (
          <div className="flex-1 flex flex-col items-center justify-center px-4 text-center">
            <div className="w-12 h-12 bg-white rounded-2xl flex items-center justify-center shadow-sm mb-3 text-neutral-400"><MorphIcon icon={ICONO_DOCUMENTO} size={20} strokeWidth={2} spring="smooth" /></div>
            <p className="text-[10px] font-black text-neutral-400 uppercase tracking-widest">Aún no has devuelto tickets</p>
          </div>
        ) : (
          <div className="flex-1 px-3 py-3 space-y-2 overflow-y-auto custom-scrollbar">
            {devoluciones.map((d) => (
              <article key={d.id} className="p-3 bg-white rounded-xl border border-neutral-100 shadow-sm flex items-center justify-between gap-3">
                <div className="min-w-0">
                  <p className="text-[10px] font-black text-neutral-900 uppercase truncate">{d.folio_ticket}</p>
                  <p className="text-[8px] text-neutral-400 font-bold mt-1">Devuelto: {d.creado_en}</p>
                  <p className={`text-[8px] font-black uppercase mt-1 ${d.estado_movimiento === "pendiente" ? "text-amber-600" : "text-emerald-600"}`}>
                    {d.estado_movimiento === "pendiente" ? "Egreso pendiente de corte" : "Egreso registrado"}
                  </p>
                </div>
                <span className="text-xs font-black text-neutral-900 shrink-0">{dinero(d.monto)}</span>
              </article>
            ))}
          </div>
        )}
      </div>
      <p className="text-[10px] text-neutral-300 font-bold mt-4">Historial completo de tickets devueltos por ti</p>
    </section>
  );
};

export default PanelMisDevoluciones;
