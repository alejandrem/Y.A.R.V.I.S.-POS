// ═══════════════════════════════════════════════════════════════════════════
// MODAL DETALLE CORTE — Vista de desglose de un corte de caja.
// Muestra montos clave y movimientos (get_movimientos_corte). Para cortes
// ABIERTOS ofrece el cierre: entradas/retiros manuales + botón que invoca
// cerrar_corte. El backend RECALCULA los totales desde ventas — los valores
// del cliente se ignoran; lo mostrado tras cerrar es la cifra real.
// ═══════════════════════════════════════════════════════════════════════════

import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ModalShell, ICONO_CAJA } from "../../../../components/ui";
import { notificarError } from "../../../../components/notificaciones";
import type { CorteCaja, CierreCorte, MovimientoCaja } from "../../../types";
import { moneda } from "../nucleo/utilidades";

interface Props {
  corte: CorteCaja;
  onCerrar: () => void;
  onActualizado?: () => void;
}

export default function ModalDetalleCorte({ corte, onCerrar, onActualizado }: Props) {
  const [movimientos, setMovimientos] = useState<MovimientoCaja[]>([]);
  const [entradas, setEntradas] = useState("0");
  const [retiros, setRetiros] = useState("0");
  const [cerrando, setCerrando] = useState(false);
  const [error, setError] = useState("");
  const [resumen, setResumen] = useState<CierreCorte | null>(null);

  useEffect(() => {
    invoke<MovimientoCaja[]>("get_movimientos_corte", { corteId: corte.id })
      .then(setMovimientos)
      .catch((e) => { console.error("Error cargando movimientos:", e); notificarError("No se pudieron cargar los movimientos del corte", e); });
  }, [corte.id]);

  const abierto = corte.estado === "abierto";

  const cerrarCorte = async () => {
    setCerrando(true);
    setError("");
    try {
      // Los totales de venta/métodos van solo por compatibilidad del
      // contrato: el servidor recalcula TODO desde la tabla de ventas.
      const r = await invoke<CierreCorte>("cerrar_corte", {
        corteId: corte.id,
        totalVentas: corte.total_ventas,
        totalEfectivo: corte.total_efectivo,
        totalTarjeta: corte.total_tarjeta,
        totalTransferencia: corte.total_transferencia,
        entradasManuales: parseFloat(entradas) || 0,
        retirosManuales: parseFloat(retiros) || 0,
      });
      setResumen(r);
      onActualizado?.();
    } catch (e) {
      console.error("[FINANZAS] error al cerrar corte:", e);
      setError(String(e));
    } finally {
      setCerrando(false);
    }
  };

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
            <p className="text-lg font-black text-neutral-900 mt-1">{moneda(resumen?.total_ventas ?? corte.total_ventas)}</p>
          </div>
          <div className="p-3 bg-neutral-50 rounded-2xl">
            <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest">Efectivo</p>
            <p className="text-lg font-black text-neutral-900 mt-1">{moneda(resumen?.total_efectivo ?? corte.total_efectivo)}</p>
          </div>
          <div className="p-3 bg-neutral-50 rounded-2xl">
            <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest">Diferencia</p>
            <p className={`text-lg font-black mt-1 ${(resumen?.diferencia ?? corte.diferencia) === 0 ? "text-neutral-900" : Math.abs(resumen?.diferencia ?? corte.diferencia) > (resumen?.total_ventas ?? corte.total_ventas) * 0.05 ? "text-red-500" : "text-amber-500"}`}>
              {moneda(resumen?.diferencia ?? corte.diferencia)}
            </p>
          </div>
        </div>

        {abierto && !resumen && (
          <div className="p-4 border border-dashed border-neutral-300 rounded-3xl space-y-3">
            <p className="text-[10px] font-black text-neutral-500 uppercase tracking-widest">
              Cerrar corte · captura entradas y retiros manuales
            </p>
            <p className="text-[10px] font-bold text-neutral-400 leading-relaxed">
              Las ventas del periodo se recalculan en el servidor desde la base de datos:
              esta diferencia no puede manipularse desde la interfaz.
            </p>
            <div className="grid grid-cols-2 gap-3">
              <label className="block">
                <span className="text-[9px] font-black text-neutral-400 uppercase tracking-widest">Entradas manuales ($)</span>
                <input
                  type="number"
                  min="0"
                  step="0.01"
                  value={entradas}
                  onChange={(e) => setEntradas(e.target.value)}
                  className="w-full mt-1 px-3 py-2 border border-neutral-200 rounded-xl text-sm font-bold focus:outline-none focus:border-neutral-900"
                />
              </label>
              <label className="block">
                <span className="text-[9px] font-black text-neutral-400 uppercase tracking-widest">Retiros manuales ($)</span>
                <input
                  type="number"
                  min="0"
                  step="0.01"
                  value={retiros}
                  onChange={(e) => setRetiros(e.target.value)}
                  className="w-full mt-1 px-3 py-2 border border-neutral-200 rounded-xl text-sm font-bold focus:outline-none focus:border-neutral-900"
                />
              </label>
            </div>
            {error && <p className="text-xs font-black text-red-500">{error}</p>}
            <button
              onClick={cerrarCorte}
              disabled={cerrando}
              className="w-full py-3 rounded-2xl bg-neutral-950 text-white text-[10px] font-black uppercase tracking-widest hover:bg-neutral-800 transition-all active:scale-[0.98] disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {cerrando ? "Cerrando…" : "Cerrar corte y calcular diferencia"}
            </button>
          </div>
        )}

        {resumen && (
          <div className="p-4 bg-emerald-50 border border-emerald-200 rounded-3xl space-y-2">
            <p className="flex items-center gap-2 text-[10px] font-black text-emerald-700 uppercase tracking-widest">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3"><polyline points="20 6 9 17 4 12"/></svg>
              Corte cerrado · cifras recalculadas por el servidor
            </p>
            <div className="grid grid-cols-3 gap-2 text-center">
              <div><p className="text-[9px] font-black text-emerald-600/70 uppercase">Ventas</p><p className="text-sm font-black text-emerald-800">{moneda(resumen.total_ventas)}</p></div>
              <div><p className="text-[9px] font-black text-emerald-600/70 uppercase">Contado</p><p className="text-sm font-black text-emerald-800">{moneda(resumen.total_efectivo + resumen.total_tarjeta + resumen.total_transferencia)}</p></div>
              <div><p className="text-[9px] font-black text-emerald-600/70 uppercase">Diferencia</p><p className={`text-sm font-black ${resumen.diferencia === 0 ? "text-emerald-800" : Math.abs(resumen.diferencia) > resumen.total_ventas * 0.05 ? "text-red-600" : "text-amber-600"}`}>{moneda(resumen.diferencia)}</p></div>
            </div>
          </div>
        )}

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
