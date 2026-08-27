// Paso 2 del flujo de tickets: selección de la carpeta de tickets TXT.
import type { ArchivoTicket } from "./compartido";
import { formatSize } from "./compartido";

interface CarpetaProps {
  folderPath: string;
  ticketFiles: ArchivoTicket[];
  busy: boolean;
  catalogImported: boolean;
  onSelectFolder: () => void;
  onStartFlow: () => void;
}

const Carpeta = ({ folderPath, ticketFiles, busy, catalogImported, onSelectFolder, onStartFlow }: CarpetaProps) => (
  <section className="bg-white rounded-[2.5rem] border border-neutral-100 shadow-xl p-6 sm:p-10 space-y-6">
    <div className="flex items-start justify-between gap-4">
      <div><p className="text-[10px] font-black uppercase tracking-[0.35em] text-neutral-400">Paso 2 · Lote de tickets</p><h3 className="text-2xl font-black text-neutral-900 mt-2">Selecciona la carpeta completa</h3><p className="text-sm text-neutral-500 mt-2">Primero se analizarán 5 tickets al azar; después se procesará todo el lote.</p></div>
      {catalogImported ? (
        <span className="rounded-xl bg-emerald-50 text-emerald-700 px-3 py-2 text-[10px] font-black uppercase">Catálogo listo</span>
      ) : (
        <span className="rounded-xl bg-amber-50 text-amber-700 border border-amber-200 px-3 py-2 text-[10px] font-black uppercase">Sin catálogo — se poblará inventario</span>
      )}
    </div>
    <button onClick={onSelectFolder} className="w-full border-2 border-dashed border-neutral-200 rounded-3xl py-10 hover:border-neutral-900 hover:bg-neutral-50 transition-colors"><div className="text-3xl mb-3">▰</div><span className="text-[11px] font-black uppercase tracking-widest text-neutral-500">Seleccionar carpeta de tickets TXT</span>{folderPath && <p className="text-xs text-neutral-900 font-bold mt-3 break-all px-4">{folderPath}</p>}</button>
    {!!ticketFiles.length && <div className="rounded-2xl bg-neutral-50 p-5"><div className="flex justify-between items-center"><span className="text-[10px] font-black uppercase tracking-widest text-neutral-400">Carpeta preparada</span><span className="text-lg font-black text-neutral-900">{ticketFiles.length} tickets</span></div><p className="text-xs text-neutral-500 mt-2">{formatSize(ticketFiles.reduce((sum, file) => sum + file.tamano, 0))} en archivos TXT · se elegirán {Math.min(5, ticketFiles.length)} muestras automáticamente.</p><button disabled={busy} onClick={onStartFlow} className="w-full mt-5 rounded-2xl bg-neutral-950 text-neutral-50 py-5 text-[10px] font-black uppercase tracking-widest disabled:opacity-40">{busy ? "Preparando..." : "Analizar y parsear carpeta"}</button></div>}
  </section>
);

export default Carpeta;