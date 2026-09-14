// ═══════════════════════════════════════════════════════════════════════════
// TARJETA AMARILLA — Sugerencia con confirmación humana (issue #11).
// Muestra ean + nombre crudo + mejor candidato + score con [SI ES ESTE]
// [NO ES]. NO abre el top-5 (ELEGIR o NINGUNO → rojo). Todo con morphicons.
// ═══════════════════════════════════════════════════════════════════════════

import { useState } from "react";
import { MorphIcon } from "morphicons/react";
import {
  ICONO_ALERTA_CIRCULO,
  ICONO_CHECK,
  ICONO_CERRAR,
  ICONO_CODIGO_BARRAS,
  ICONO_EQUIS,
} from "../../../icons";
import { notificarExito } from "../../../components/notificaciones";
import { reportarError } from "../../../services/tauri";
import {
  confirmarAmarillo,
  ningunoAmarillo,
  rechazarAmarillo,
  sugerirAmarillo,
  type PendienteCodigo,
  type SugerenciaTop,
} from "../../../services/semaforo";

interface TarjetaAmarillaProps {
  pendiente: PendienteCodigo;
  candidatoNombre: string | null;
  seleccionada: boolean;
  onToggleSeleccion: () => void;
  onResuelto: () => void;
}

const TarjetaAmarilla = ({
  pendiente,
  candidatoNombre,
  seleccionada,
  onToggleSeleccion,
  onResuelto,
}: TarjetaAmarillaProps) => {
  const [top, setTop] = useState<SugerenciaTop | null>(null);
  const [cargandoTop, setCargandoTop] = useState(false);
  const [resolviendo, setResolviendo] = useState(false);

  const mejorId = pendiente.mejor_candidato_id;
  const score = pendiente.mejor_score ?? 0;
  const nombreCandidato = candidatoNombre ?? `#${mejorId}`;

  const confirmar = async (productoId: number) => {
    setResolviendo(true);
    try {
      await confirmarAmarillo(pendiente.id, productoId, pendiente.ean ?? undefined);
      notificarExito("Código confirmado y aprendido");
      onResuelto();
    } catch (e) {
      reportarError("No se pudo confirmar la sugerencia", e);
    } finally {
      setResolviendo(false);
    }
  };

  const abrirTop = async () => {
    if (!mejorId) return;
    setCargandoTop(true);
    try {
      await rechazarAmarillo(pendiente.id, mejorId, score);
      const s = await sugerirAmarillo(pendiente.nombre_crudo, undefined, pendiente.ean ?? undefined);
      setTop(s);
    } catch (e) {
      reportarError("No se pudo cargar el top-5", e);
    } finally {
      setCargandoTop(false);
    }
  };

  const ninguno = async () => {
    setResolviendo(true);
    try {
      await ningunoAmarillo(pendiente.id);
      notificarExito("Pasó a rojo para captura manual");
      onResuelto();
    } catch (e) {
      reportarError("No se pudo pasar a rojo", e);
    } finally {
      setResolviendo(false);
    }
  };

  const resto = (top?.candidatos ?? []).filter((c) => c.producto_id !== mejorId);

  return (
    <div className="overflow-hidden rounded-2xl border border-amber-200 bg-white shadow-sm">
      <div className="flex items-stretch gap-0">
        <div className="w-1.5 shrink-0 bg-amber-400" />
        <div className="flex-1 p-4">
          <div className="flex items-start justify-between gap-3">
            <div className="flex items-start gap-3">
              <input
                type="checkbox"
                aria-label="Seleccionar para lote"
                checked={seleccionada}
                disabled={!mejorId}
                onChange={onToggleSeleccion}
                className="mt-1 h-4 w-4 accent-amber-500"
              />
              <div>
                <p className="flex items-center gap-1.5 font-mono text-xs font-black text-neutral-900">
                  <MorphIcon icon={ICONO_CODIGO_BARRAS} size={14} strokeWidth={2.5} className="text-amber-500" />
                  {pendiente.ean ?? "SIN EAN"}
                </p>
                <p className="mt-1 text-[11px] font-bold uppercase text-neutral-500">
                  Ticket: <span className="text-neutral-900">‘{pendiente.nombre_crudo}’</span>
                </p>
              </div>
            </div>
            <span className="flex items-center gap-1 rounded-lg bg-amber-50 px-2 py-1 text-[9px] font-black uppercase tracking-widest text-amber-700">
              <MorphIcon icon={ICONO_ALERTA_CIRCULO} size={12} strokeWidth={2.5} />
              {pendiente.veces_visto}× visto
            </span>
          </div>

          {mejorId ? (
            <div className="mt-3 rounded-xl bg-amber-50/60 border border-amber-100 p-3">
              <p className="text-[9px] font-black uppercase tracking-widest text-amber-600">Mejor candidato</p>
              <div className="mt-1 flex items-center justify-between gap-2">
                <p className="text-xs font-black uppercase text-neutral-900">{nombreCandidato}</p>
                <span className="rounded-md bg-amber-400 px-2 py-0.5 text-[10px] font-black text-white">
                  {(score * 100).toFixed(0)}%
                </span>
              </div>
              <div className="mt-3 flex gap-2">
                <button
                  onClick={() => confirmar(mejorId)}
                  disabled={resolviendo}
                  className="flex flex-1 items-center justify-center gap-1.5 rounded-xl bg-emerald-500 px-3 py-2.5 text-[10px] font-black uppercase tracking-widest text-white hover:bg-emerald-600 disabled:opacity-50"
                >
                  <MorphIcon icon={ICONO_CHECK} size={14} strokeWidth={3} /> Sí es este
                </button>
                <button
                  onClick={abrirTop}
                  disabled={cargandoTop || resolviendo}
                  className="flex flex-1 items-center justify-center gap-1.5 rounded-xl bg-neutral-900 px-3 py-2.5 text-[10px] font-black uppercase tracking-widest text-white hover:bg-neutral-700 disabled:opacity-50"
                >
                  <MorphIcon icon={ICONO_EQUIS} size={14} strokeWidth={3} /> {cargandoTop ? "Buscando…" : "No es"}
                </button>
              </div>
            </div>
          ) : (
            <p className="mt-3 rounded-xl bg-neutral-50 px-3 py-2 text-[10px] font-bold uppercase text-neutral-400">
              Sin candidato calculado todavía
            </p>
          )}

          {top && (
            <div className="mt-3 space-y-2 rounded-xl border border-neutral-100 bg-neutral-50/60 p-3">
              <p className="text-[9px] font-black uppercase tracking-widest text-neutral-400">
                Otros candidatos ({resto.length})
              </p>
              {resto.length === 0 && (
                <p className="text-[10px] font-bold uppercase text-neutral-400">No hay más parecidos</p>
              )}
              {resto.map((c) => (
                <div key={c.producto_id} className="flex items-center justify-between gap-2 rounded-lg bg-white border border-neutral-100 px-3 py-2">
                  <div className="min-w-0">
                    <p className="truncate text-[11px] font-black uppercase text-neutral-900">{c.nombre}</p>
                    <p className="text-[9px] font-bold text-neutral-400">{(c.score * 100).toFixed(0)}% · {c.origen}</p>
                  </div>
                  <button
                    onClick={() => confirmar(c.producto_id)}
                    disabled={resolviendo}
                    className="shrink-0 rounded-lg bg-neutral-900 px-3 py-1.5 text-[9px] font-black uppercase tracking-widest text-white hover:bg-neutral-700 disabled:opacity-50"
                  >
                    Elegir
                  </button>
                </div>
              ))}
              <button
                onClick={ninguno}
                disabled={resolviendo}
                className="flex w-full items-center justify-center gap-1.5 rounded-xl border border-red-200 bg-red-50 px-3 py-2 text-[10px] font-black uppercase tracking-widest text-red-600 hover:bg-red-100 disabled:opacity-50"
              >
                <MorphIcon icon={ICONO_CERRAR} size={13} strokeWidth={2.5} /> Ninguno → a rojo
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default TarjetaAmarilla;
