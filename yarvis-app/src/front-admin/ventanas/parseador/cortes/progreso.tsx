// Pantalla de progreso del flujo de cortes: importación en curso.
import { ProgressCard } from "./compartido";

interface ProgresoCortesProps {
  totalArchivos: number;
}

const ProgresoCortes = ({ totalArchivos }: ProgresoCortesProps) => (
  <ProgressCard title="Parseando los cortes" subtitle="Cada archivo se clasifica (X, Z o no es corte), se verifica su matemática y se guarda con idempotencia por hash." current={0} total={totalArchivos} percent={0}>
    <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">{[["Archivos", totalArchivos], ["Cortes X", "…"], ["Cortes Z", "…"]].map(([label, value]) => <div key={String(label)} className="rounded-2xl bg-neutral-50 p-4"><p className="text-[9px] font-black uppercase tracking-widest text-neutral-400">{label}</p><p className="text-2xl font-black text-neutral-900 mt-1">{value}</p></div>)}</div>
  </ProgressCard>
);

export default ProgresoCortes;
