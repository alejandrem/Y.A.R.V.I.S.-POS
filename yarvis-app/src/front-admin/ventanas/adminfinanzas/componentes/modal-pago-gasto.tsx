// ═══════════════════════════════════════════════════════════════════════════
// MODAL PAGO GASTO — Registro de pagos de gastos recurrentes.
// Tarea única: formulario modal para registrar el pago de un gasto vía
// registrar_pago_gasto (monto, método de pago y notas opcionales).
// El monto se pre-llena con el proyectado del gasto pero es editable.
// ═══════════════════════════════════════════════════════════════════════════

import { useState } from "react";
import { ModalShell, Campo, inputCls, ICONO_BILLETE } from "../../../../components/ui";
import { reportarError } from "../../../../services/tauri";
import { notificarError, notificarExito } from "../../../../components/notificaciones";
import { invokeTauri } from "../../../../services/tauri";
import type { GastoRecurrente } from "../../../types";

export default function ModalPagoGasto({ gasto, onCerrar, onGuardado }: { gasto: GastoRecurrente; onCerrar: () => void; onGuardado: () => void }) {
  const [monto, setMonto] = useState(gasto.monto_proyectado);
  const [metodo, setMetodo] = useState("efectivo");
  const [notas, setNotas] = useState("");
  const [guardando, setGuardando] = useState(false);

  const guardar = async () => {
    if (monto <= 0) {
      notificarError("El monto debe ser mayor a cero");
      return;
    }
    setGuardando(true);
    try {
      await invokeTauri("registrar_pago_gasto", {
        pago: {
          gasto_id: gasto.id,
          fecha_pago: new Date().toISOString().slice(0, 19).replace("T", " "),
          monto_pagado: monto,
          metodo_pago: metodo,
          folio_comprobante: null,
          notas: notas || null,
        },
      });
      notificarExito("Pago registrado");
      onGuardado();
    } catch (e) {
      reportarError("No se pudo registrar el pago", e);
    } finally {
      setGuardando(false);
    }
  };

  return (
    <ModalShell icono={ICONO_BILLETE} titulo="Registrar Pago" subtitulo={gasto.nombre} onClose={onCerrar}>
      <div className="space-y-4">
        <Campo label="Monto">
          <input type="number" className={inputCls} value={monto} onChange={(e) => setMonto(+e.target.value)} />
        </Campo>
        <Campo label="Metodo de Pago">
          <select className={inputCls} value={metodo} onChange={(e) => setMetodo(e.target.value)}>
            <option value="efectivo">Efectivo</option>
            <option value="tarjeta">Tarjeta</option>
            <option value="transferencia">Transferencia</option>
          </select>
        </Campo>
        <Campo label="Notas">
          <input className={inputCls} value={notas} onChange={(e) => setNotas(e.target.value)} placeholder="Opcional" />
        </Campo>
      </div>
      <div className="flex gap-3 pt-2">
        <button onClick={onCerrar} className="flex-1 py-3 text-[10px] font-black text-neutral-400 uppercase tracking-widest hover:text-neutral-900 transition-colors">
          Cancelar
        </button>
        <button
          onClick={guardar}
          disabled={guardando || monto <= 0}
          className="flex-1 py-4 rounded-xl bg-emerald-500 text-white text-xs font-black uppercase tracking-[0.2em] hover:bg-emerald-600 transition-all shadow-xl shadow-emerald-200 active:scale-[0.98] disabled:opacity-30"
        >
          {guardando ? "Registrando..." : "Registrar Pago"}
        </button>
      </div>
    </ModalShell>
  );
}
