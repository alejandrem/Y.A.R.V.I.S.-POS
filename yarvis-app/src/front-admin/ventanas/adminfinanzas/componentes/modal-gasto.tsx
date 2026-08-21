// ═══════════════════════════════════════════════════════════════════════════
// MODAL GASTO — Alta y edición de gastos recurrentes.
// Tarea única: formulario modal para crear (crear_gasto) o actualizar
// (actualizar_gasto) un gasto recurrente vía comando Tauri. Si recibe un
// `gasto` pre-cargado opera en modo edición; si no, en modo creación.
// ═══════════════════════════════════════════════════════════════════════════

import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ModalShell, Campo, inputCls, ICONO_CALCULADORA } from "../../../../components/ui";
import type { GastoRecurrente, CrearGastoRequest } from "../../../types";
import { inputFecha } from "../nucleo/constantes";

export default function ModalGasto({ gasto, onCerrar, onGuardado }: { gasto?: GastoRecurrente; onCerrar: () => void; onGuardado: () => void }) {
  const [form, setForm] = useState<CrearGastoRequest>({
    nombre: gasto?.nombre ?? "",
    tipo: gasto?.tipo ?? "fijo",
    categoria: gasto?.categoria ?? "operativo",
    monto_proyectado: gasto?.monto_proyectado ?? 0,
    frecuencia: gasto?.frecuencia ?? "mensual",
    dia_pago: gasto?.dia_pago ?? 1,
    intervalo_dias: gasto?.intervalo_dias ?? null,
    fecha_inicio: gasto?.fecha_inicio ?? new Date().toISOString().slice(0, 10),
    fecha_fin: gasto?.fecha_fin ?? null,
    folio_comprobante: gasto?.folio_comprobante ?? null,
    notas: gasto?.notas ?? null,
  });
  const [guardando, setGuardando] = useState(false);

  const set = <K extends keyof CrearGastoRequest>(k: K, v: CrearGastoRequest[K]) =>
    setForm((p) => ({ ...p, [k]: v }));

  const guardar = async () => {
    if (!form.nombre || form.monto_proyectado <= 0) return;
    setGuardando(true);
    try {
      if (gasto) {
        await invoke("actualizar_gasto", { id: gasto.id, gasto: form });
      } else {
        await invoke("crear_gasto", { gasto: form });
      }
      onGuardado();
    } catch (e) {
      console.error("Error guardando gasto:", e);
    } finally {
      setGuardando(false);
    }
  };

  return (
    <ModalShell
      icono={ICONO_CALCULADORA}
      titulo={gasto ? "Editar Gasto" : "Nuevo Gasto"}
      subtitulo="Gasto recurrente"
      onClose={onCerrar}
    >
      <div className="space-y-4">
        <Campo label="Nombre">
          <input className={inputCls} value={form.nombre} onChange={(e) => set("nombre", e.target.value)} placeholder="Ej: Renta" />
        </Campo>
        <div className="grid grid-cols-2 gap-4">
          <Campo label="Tipo">
            <select className={inputCls} value={form.tipo} onChange={(e) => set("tipo", e.target.value)}>
              <option value="fijo">Fijo</option>
              <option value="variable">Variable</option>
              <option value="extraordinario">Extraordinario</option>
            </select>
          </Campo>
          <Campo label="Categoria">
            <select className={inputCls} value={form.categoria} onChange={(e) => set("categoria", e.target.value)}>
              <option value="operativo">Operativo</option>
              <option value="administrativo">Administrativo</option>
              <option value="marketing">Marketing</option>
              <option value="servicios">Servicios</option>
              <option value="impuestos">Impuestos</option>
              <option value="otro">Otro</option>
            </select>
          </Campo>
        </div>
        <div className="grid grid-cols-2 gap-4">
          <Campo label="Monto Proyectado">
            <input type="number" className={inputCls} value={form.monto_proyectado} onChange={(e) => set("monto_proyectado", +e.target.value)} />
          </Campo>
          <Campo label="Frecuencia">
            <select className={inputCls} value={form.frecuencia} onChange={(e) => set("frecuencia", e.target.value)}>
              <option value="semanal">Semanal</option>
              <option value="quincenal">Quincenal</option>
              <option value="mensual">Mensual</option>
              <option value="trimestral">Trimestral</option>
              <option value="personalizado">Personalizado</option>
            </select>
          </Campo>
        </div>
        <div className="grid grid-cols-2 gap-4">
          <Campo label="Dia de Pago">
            <input type="number" className={inputCls} min={1} max={31} value={form.dia_pago ?? 1} onChange={(e) => set("dia_pago", +e.target.value)} />
          </Campo>
          <Campo label="Fecha Inicio">
            <input type="date" className={inputFecha} value={form.fecha_inicio} onChange={(e) => set("fecha_inicio", e.target.value)} />
          </Campo>
        </div>
        <Campo label="Notas">
          <input className={inputCls} value={form.notas ?? ""} onChange={(e) => set("notas", e.target.value || null)} placeholder="Opcional" />
        </Campo>
      </div>
      <div className="flex gap-3 pt-2">
        <button onClick={onCerrar} className="flex-1 py-3 text-[10px] font-black text-neutral-400 uppercase tracking-widest hover:text-neutral-900 transition-colors">
          Cancelar
        </button>
        <button
          onClick={guardar}
          disabled={guardando || !form.nombre}
          className="flex-1 py-4 rounded-xl bg-neutral-950 text-neutral-50 text-xs font-black uppercase tracking-[0.2em] hover:bg-neutral-800 transition-all shadow-xl shadow-neutral-200 active:scale-[0.98] disabled:opacity-30"
        >
          {guardando ? "Guardando..." : gasto ? "Actualizar" : "Crear"}
        </button>
      </div>
    </ModalShell>
  );
}
