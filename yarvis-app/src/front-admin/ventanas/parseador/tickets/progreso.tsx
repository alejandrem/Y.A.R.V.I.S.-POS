// Pantallas de progreso del flujo de tickets: calibrando y parseando la carpeta.
import type { BatchProgress, TrainingProgress } from "./compartido";
import { ProgressCard } from "./compartido";

interface ProgresoProps {
  phase: "calibrando" | "procesando";
  training: TrainingProgress[];
  trainingDone: number;
  trainingTotal: number;
  trainingPercent: number;
  batch: BatchProgress | null;
  batchTotal: number;
  batchPercent: number;
}

const Progreso = ({ phase, training, trainingDone, trainingTotal, trainingPercent, batch, batchTotal, batchPercent }: ProgresoProps) =>
  phase === "calibrando" ? (
    <ProgressCard title="Calibrando el analizador" subtitle="La IA está comparando 5 tickets aleatorios para encontrar el patrón más estable." current={trainingDone} total={trainingTotal} percent={trainingPercent}>
      <div className="space-y-2">{training.map((ticket, index) => <div key={`${ticket.archivo}-${index}`} className="flex items-center gap-3 rounded-xl bg-neutral-50 px-4 py-3"><span className={`w-2 h-2 rounded-full ${ticket.estado === "ok" ? "bg-emerald-500" : "bg-amber-500"}`}></span><span className="text-xs font-bold text-neutral-700 truncate">{ticket.mensaje}</span><span className="ml-auto text-[10px] text-neutral-400 truncate max-w-[35%]">{ticket.archivo}</span></div>)}</div>
    </ProgressCard>
  ) : (
    <ProgressCard title="Parseando la carpeta completa" subtitle="Cada ticket se está convirtiendo e insertando en el inventario y el historial." current={batch?.procesados ?? 0} total={batchTotal} percent={batchPercent}>
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">{[["Procesados", batch?.procesados ?? 0], ["Correctos", batch?.exitosos ?? 0], ["Errores", batch?.errores ?? 0], ["Productos nuevos", batch?.productos_nuevos ?? 0]].map(([label, value]) => <div key={String(label)} className="rounded-2xl bg-neutral-50 p-4"><p className="text-[9px] font-black uppercase tracking-widest text-neutral-400">{label}</p><p className="text-2xl font-black text-neutral-900 mt-1">{value}</p></div>)}</div>
    </ProgressCard>
  );

export default Progreso;