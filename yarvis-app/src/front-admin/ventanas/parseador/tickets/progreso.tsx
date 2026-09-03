// Pantalla de progreso del flujo de tickets: parseo de la carpeta.
import type { BatchProgress } from "./compartido";
import { ProgressCard } from "./compartido";

interface ProgresoProps {
  batch: BatchProgress | null;
  batchTotal: number;
  batchPercent: number;
}

const Progreso = ({ batch, batchTotal, batchPercent }: ProgresoProps) => (
  <ProgressCard title="Parseando la carpeta completa" subtitle="Cada ticket se está convirtiendo e insertando en el inventario y el historial." current={batch?.procesados ?? 0} total={batchTotal} percent={batchPercent}>
    <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">{[["Procesados", batch?.procesados ?? 0], ["Correctos", batch?.exitosos ?? 0], ["Errores", batch?.errores ?? 0], ["Productos nuevos", batch?.productos_nuevos ?? 0]].map(([label, value]) => <div key={String(label)} className="rounded-2xl bg-neutral-50 p-4"><p className="text-[9px] font-black uppercase tracking-widest text-neutral-400">{label}</p><p className="text-2xl font-black text-neutral-900 mt-1">{value}</p></div>)}</div>
  </ProgressCard>
);

export default Progreso;
