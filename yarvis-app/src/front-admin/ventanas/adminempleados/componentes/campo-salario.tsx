// ═══════════════════════════════════════════════════════════════════════════
// CAMPO SALARIO — Sección de pago semanal del modal de empleados con la
// proyección en vivo (hora/día/semana/mes) y el resumen de horas/días.
// Presentacional: recibe el salario, los totales derivados y el callback.
// ═══════════════════════════════════════════════════════════════════════════

import { MorphIcon } from "morphicons/react";
import { Campo, inputCls, ICONO_DOLAR } from "../../../../components/ui";

interface CampoSalarioProps {
  salarioSemanal: number;
  onChange: (valor: number) => void;
  diasSemana: number;
  horasTotales: number;
  totalBloques: number;
}

const CampoSalario = ({ salarioSemanal, onChange, diasSemana, horasTotales, totalBloques }: CampoSalarioProps) => {
  const salarioDiario = diasSemana > 0 ? salarioSemanal / diasSemana : 0;
  const horasPorDia = diasSemana > 0 ? horasTotales / diasSemana : 0;
  const salarioHora = horasTotales > 0 ? salarioSemanal / horasTotales : 0;
  const salarioMensual = salarioSemanal * 4.33;

  const proyeccion = [
    { label: "× Hora", valor: salarioHora },
    { label: "× Día", valor: salarioDiario },
    { label: "× Semana", valor: salarioSemanal },
    { label: "× Mes", valor: salarioMensual },
  ];

  return (
    <div className="bg-neutral-50 rounded-2xl p-4 space-y-3">
      <p className="flex items-center gap-2 text-[10px] font-black text-neutral-500 uppercase tracking-widest">
        <MorphIcon icon={ICONO_DOLAR} size={14} strokeWidth={2.4} spring="smooth" />
        Pago semanal
      </p>
      <Campo label="Pago por semana ($)">
        <input
          type="number"
          min={0}
          step={50}
          value={salarioSemanal || ""}
          onChange={(e) => onChange(Math.max(0, Number(e.target.value)))}
          placeholder="0"
          className={inputCls}
        />
      </Campo>
      <div className="grid grid-cols-4 gap-2">
        {proyeccion.map((p) => (
          <div key={p.label} className="bg-white rounded-xl p-2.5 text-center border border-neutral-100">
            <p className="text-[8px] font-black text-neutral-400 uppercase tracking-widest">{p.label}</p>
            <p className="text-[13px] font-black text-neutral-900 mt-0.5">${p.valor.toFixed(2)}</p>
          </div>
        ))}
      </div>
      <p className="text-[8px] font-bold text-neutral-400 text-center">
        Basado en {horasPorDia.toFixed(1)}h/día · {diasSemana} días/semana · {totalBloques} {totalBloques === 1 ? "horario" : "horarios"}
      </p>
    </div>
  );
};

export default CampoSalario;
