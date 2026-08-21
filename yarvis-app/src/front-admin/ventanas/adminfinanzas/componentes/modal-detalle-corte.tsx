// ═══════════════════════════════════════════════════════════════════════════
// MODAL DETALLE CORTE — Vista de desglose de un corte de caja.
// Tarea única: mostrar en modal los montos clave de un corte (inicial,
// ventas, efectivo, diferencia con semáforo) y su lista de movimientos
// cargada desde get_movimientos_corte. Solo lectura.
// ═══════════════════════════════════════════════════════════════════════════

import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ModalShell, ICONO_CAJA } from "../../../../components/ui";
import type { CorteCaja, MovimientoCaja } from "../../../types";
import { moneda } from "../nucleo/utilidades";

export default function ModalDetalleCorte({ corte, onCerrar }: { corte: CorteCaja; onCerrar: () => void }) {
  const [movimientos, setMovimientos] = useState<MovimientoCaja[]>([]);

  useEffect(() => {
    invoke<MovimientoCaja[]>("get_movimientos_corte", { corteId: corte.id })
      .then(setMovimientos)
      .catch((e) => console.error("Error cargando movimientos:", e));
  }, [corte.id]);

  return (
    <ModalShell icono={ICONO_CAJA} titulo={`Corte ${corte.tipo_corte}`} subtitulo={corte.fecha_apertura} onClose={onCerrar}>
      <div className="space-y-4">
        <div className="grid grid-cols-2 gap-3">
          <div className="p-3 bg-neutral-50 rounded-2xl">
            <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest">Inicial</p>
            <p className="text-lg font-black text-neutral-900 mt-1">{moneda(corte.monto_inicial)}</p>
          </div>
          <div className="p-3 bg-neutral-50 rounded-2xl">
            <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest">Ventas</p>
            <p className="text-lg font-black text-neutral-900 mt-1">{moneda(corte.total_ventas)}</p>
          </div>
          <div className="p-3 bg-neutral-50 rounded-2xl">
            <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest">Efectivo</p>
            <p className="text-lg font-black text-neutral-900 mt-1">{moneda(corte.total_efectivo)}</p>
          </div>
          <div className="p-3 bg-neutral-50 rounded-2xl">
            <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest">Diferencia</p>
            <p className={`text-lg font-black mt-1 ${corte.diferencia === 0 ? "text-neutral-900" : Math.abs(corte.diferencia) > corte.total_ventas * 0.05 ? "text-red-500" : "text-amber-500"}`}>
              {moneda(corte.diferencia)}
            </p>
          </div>
        </div>
        {movimientos.length > 0 && (
          <div>
            <p className="text-[10px] font-black text-neutral-400 uppercase tracking-widest mb-2">Movimientos</p>
            <div className="space-y-2 max-h-40 overflow-y-auto custom-scrollbar">
              {movimientos.map((m) => (
                <div key={m.id} className="flex items-center justify-between p-2 bg-neutral-50 rounded-xl">
                  <div className="flex items-center gap-2">
                    <span className={`w-2 h-2 rounded-full ${m.tipo === "entrada" ? "bg-emerald-500" : "bg-red-500"}`} />
                    <span className="text-xs font-bold text-neutral-700">{m.concepto}</span>
                  </div>
                  <span className={`text-xs font-black ${m.tipo === "entrada" ? "text-emerald-600" : "text-red-500"}`}>
                    {m.tipo === "entrada" ? "+" : "-"}{moneda(m.monto)}
                  </span>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </ModalShell>
  );
}
