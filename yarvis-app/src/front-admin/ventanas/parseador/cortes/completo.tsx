// Pantalla final del flujo de cortes: resumen de la importación.
import type { ResumenCortes } from "../../../../services/cortes";

interface CompletoCortesProps {
  resumen: ResumenCortes | null;
  totalArchivos: number;
  onReset: () => void;
}

const CompletoCortes = ({ resumen, totalArchivos, onReset }: CompletoCortesProps) => (
  <section className="bg-neutral-950 text-neutral-50 rounded-[2.5rem] shadow-xl p-8 sm:p-12 text-center">
    <div className="mx-auto w-16 h-16 rounded-full bg-emerald-400 text-neutral-900 flex items-center justify-center text-3xl font-black">✓</div>
    <p className="text-[10px] font-black uppercase tracking-[0.35em] text-neutral-400 mt-6">Proceso terminado</p>
    <h3 className="text-3xl font-black mt-2">Carpeta parseada correctamente</h3>
    <p className="text-neutral-400 text-sm mt-3">Se clasificaron {resumen?.archivos ?? totalArchivos} archivos: cortes X y Z verificados con matemática exacta.</p>
    <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 mt-8 text-left">
      {[["Cortes X", resumen?.cortes_x ?? 0], ["Cortes Z", resumen?.cortes_z ?? 0], ["Vinculados", resumen?.productos_vinculados ?? 0], ["Nuevos", resumen?.productos_nuevos ?? 0]].map(([label, value]) => <div key={String(label)} className="rounded-2xl bg-white/10 p-4"><p className="text-[9px] font-black uppercase tracking-widest text-neutral-400">{label}</p><p className="text-2xl font-black mt-1">{value}</p></div>)}
    </div>
    {!!resumen?.omitidos_duplicados && <p className="text-[10px] text-amber-400 mt-4">{resumen.omitidos_duplicados} archivo(s) se omitieron porque ya estaban importados (mismo contenido): no se duplicó nada.</p>}
    {!!resumen?.omitidos_no_corte && <p className="text-[10px] text-sky-400 mt-2">{resumen.omitidos_no_corte} archivo(s) no eran cortes de caja y se omitieron solos (tickets sueltos, por ejemplo).</p>}
    {!!resumen?.errores.length && (
      <div className="mt-4 rounded-2xl bg-red-500/10 border border-red-500/30 p-4 text-left">
        <p className="text-[10px] font-black uppercase tracking-widest text-red-400">{resumen.errores.length} archivo(s) con error</p>
        <ul className="mt-2 space-y-1 max-h-40 overflow-y-auto custom-scrollbar">
          {resumen.errores.map((e, i) => <li key={i} className="text-xs font-bold text-neutral-100">{e}</li>)}
        </ul>
      </div>
    )}
    <button onClick={onReset} className="mt-8 rounded-2xl bg-neutral-100 text-neutral-950 px-8 py-4 text-[10px] font-black uppercase tracking-widest">Procesar otra carpeta</button>
  </section>
);

export default CompletoCortes;
