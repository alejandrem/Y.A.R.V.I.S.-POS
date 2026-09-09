// Paso 1 del flujo de cortes: selección de la carpeta con cortes X/Z.
import type { ArchivoTicket } from "./compartido";
import { formatSize } from "./compartido";

interface CarpetaCortesProps {
  folderPath: string;
  corteFiles: ArchivoTicket[];
  busy: boolean;
  onSelectFolder: () => void;
  onStartFlow: () => void;
}

const CarpetaCortes = ({ folderPath, corteFiles, busy, onSelectFolder, onStartFlow }: CarpetaCortesProps) => (
  <section className="bg-white rounded-[2.5rem] border border-neutral-100 shadow-xl p-6 sm:p-10 space-y-6">
    <div>
      <div><p className="text-[10px] font-black uppercase tracking-[0.35em] text-neutral-400">Paso 1 · Lote de cortes</p><h3 className="text-2xl font-black text-neutral-900 mt-2">Selecciona la carpeta completa</h3><p className="text-sm text-neutral-500 mt-2">Cada archivo se clasifica solo (corte X, corte Z o no es corte y se omite); los números se verifican con matemática exacta en centavos. Re-importar es seguro: el hash omite duplicados.</p></div>
    </div>
    <button onClick={onSelectFolder} className="w-full border-2 border-dashed border-neutral-200 rounded-3xl py-10 hover:border-neutral-900 hover:bg-neutral-50 transition-colors"><div className="text-3xl mb-3">▰</div><span className="text-[11px] font-black uppercase tracking-widest text-neutral-500">Seleccionar carpeta de cortes TXT</span>{folderPath && <p className="text-xs text-neutral-900 font-bold mt-3 break-all px-4">{folderPath}</p>}</button>
    {!!corteFiles.length && <div className="rounded-2xl bg-neutral-50 p-5"><div className="flex justify-between items-center"><span className="text-[10px] font-black uppercase tracking-widest text-neutral-400">Carpeta preparada</span><span className="text-lg font-black text-neutral-900">{corteFiles.length} archivos</span></div><p className="text-xs text-neutral-500 mt-2">{formatSize(corteFiles.reduce((sum, file) => sum + file.tamano, 0))} en archivos TXT · los tickets sueltos se omitirán solos.</p><button disabled={busy} onClick={onStartFlow} className="w-full mt-5 rounded-2xl bg-neutral-950 text-neutral-50 py-5 text-[10px] font-black uppercase tracking-widest disabled:opacity-40">{busy ? "Clasificando..." : "Clasificar y parsear carpeta"}</button></div>}
  </section>
);

export default CarpetaCortes;
