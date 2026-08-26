// ═══════════════════════════════════════════════════════════════════════════
// DATOS DE SESIÓN — Tarjeta de solo lectura con el contexto de quien usa
// el POS: tienda, empleado en sesión y ubicación. Nada editable aquí.
// ═══════════════════════════════════════════════════════════════════════════

import { MorphIcon } from "morphicons/react";
import { ICONO_USUARIO, ICONO_CAJA, ICONO_RELOJ } from "../../../../components/ui";

interface DatosSesionProps {
  nombreEmpleado: string;
  tienda: string | null;
  ubicacion: string | null;
  cp: string | null;
}

export default function DatosSesion({ nombreEmpleado, tienda, ubicacion, cp }: DatosSesionProps) {
  const filas = [
    { icono: ICONO_CAJA, label: "Tienda", valor: tienda || "—" },
    { icono: ICONO_USUARIO, label: "Empleado", valor: nombreEmpleado || "—" },
    {
      icono: ICONO_RELOJ,
      label: "Ubicación",
      valor: [ubicacion, cp].filter(Boolean).join(" · ") || "—",
    },
  ];

  return (
    <div className="bg-white rounded-[2.5rem] border border-neutral-200 shadow-sm p-6 sm:p-8">
      <p className="text-[10px] font-black text-neutral-400 uppercase tracking-widest mb-5">
        Sesión actual
      </p>
      <div className="grid grid-cols-1 gap-4">
        {filas.map((fila) => (
          <div key={fila.label} className="flex items-center gap-3 p-4 bg-neutral-50 rounded-2xl">
            <div className="w-10 h-10 bg-neutral-950 text-neutral-50 rounded-xl flex items-center justify-center shrink-0">
              <MorphIcon icon={fila.icono} size={16} strokeWidth={2.2} spring="smooth" />
            </div>
            <div className="min-w-0">
              <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest">{fila.label}</p>
              <p className="text-sm font-black text-neutral-900 truncate">{fila.valor}</p>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
