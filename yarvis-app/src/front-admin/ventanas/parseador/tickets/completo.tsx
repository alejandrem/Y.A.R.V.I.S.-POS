// Pantalla final del flujo de tickets: resumen del parseo y opción de reiniciar.
import type { ArchivoTicket, BatchProgress, CalibrationResult } from "./compartido";

interface CompletoProps {
  batch: BatchProgress | null;
  ticketFiles: ArchivoTicket[];
  trainingResult: CalibrationResult | null;
  onReset: () => void;
}

const Completo = ({ batch, ticketFiles, trainingResult, onReset }: CompletoProps) => (
  <section className="bg-neutral-950 text-neutral-50 rounded-[2.5rem] shadow-xl p-8 sm:p-12 text-center">
    <div className="mx-auto w-16 h-16 rounded-full bg-emerald-400 text-neutral-900 flex items-center justify-center text-3xl font-black">✓</div>
    <p className="text-[10px] font-black uppercase tracking-[0.35em] text-neutral-400 mt-6">Proceso terminado</p>
    <h3 className="text-3xl font-black mt-2">Carpeta parseada correctamente</h3>
    <p className="text-neutral-400 text-sm mt-3">El patrón ganador fue aplicado a los {batch?.procesados ?? ticketFiles.length} tickets.</p>
    <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 mt-8 text-left">
      {[["Tickets", batch?.procesados ?? ticketFiles.length], ["Correctos", batch?.exitosos ?? 0], ["Ventas", batch?.ventas_creadas ?? 0], ["Items", batch?.items_insertados ?? 0]].map(([label, value]) => <div key={String(label)} className="rounded-2xl bg-white/10 p-4"><p className="text-[9px] font-black uppercase tracking-widest text-neutral-400">{label}</p><p className="text-2xl font-black mt-1">{value}</p></div>)}
    </div>
    {!!batch?.ventas_omitidas && <p className="text-[10px] text-amber-400 mt-4">{batch.ventas_omitidas} ticket(s) se omitieron porque ya estaban importados (folio repetido): no se duplicó nada.</p>}
    <button onClick={onReset} className="mt-8 rounded-2xl bg-neutral-100 text-neutral-950 px-8 py-4 text-[10px] font-black uppercase tracking-widest">Procesar otra carpeta</button>
    {trainingResult && <p className="text-[10px] text-neutral-500 mt-5">Mapeo elegido por {trainingResult.votos_ganadores} de {trainingResult.total_muestras} muestras válidas.</p>}
  </section>
);

export default Completo;