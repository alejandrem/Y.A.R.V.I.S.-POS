// ═══════════════════════════════════════════════════════════════════════════
// EMPLEACORTES · MODAL CORTE — Flujo aprobado en 3 pasos:
//   1. ELEGIR  → pastillas X / Z (misma modal, cambia el mensaje).
//   2. PREVIA  → previsualización del ticket 80mm + impresora + IMPRIMIR.
// Cancelar en el paso 1 no guarda nada. El corte se guarda al continuar
// (paso 2) y la impresión va directa a la térmica por backend.
// Z cierra el turno automáticamente (ver #26). Escape cierra.
// ═══════════════════════════════════════════════════════════════════════════

import { useState, useEffect } from "react";
import { MorphIcon } from "morphicons/react";
import { pedirCorteX, pedirCorteZ, imprimirCorte } from "../../../services/empleacortes";
import { listarImpresoras, type DestinoPrint, type ImpresoraInfo } from "../../../services/impresora";
import { obtenerTiendaInfo } from "../../../services/turno";
import { ICONO_CAJA, ICONO_EQUIS, ICONO_CHECK, ICONO_IMPRESORA } from "../../../components/ui";
import type { CorteXReporte, CorteZReporte, TipoCorte } from "./tipos";
import { textoCorteX, textoCorteZ } from "./ticket-texto";

interface ModalCorteProps {
  onClose: () => void;
  cajero: string;
}

type Paso = "elegir" | "previa";

const MENSAJE_X = "El corte X es una foto de tu turno: cuánto llevas vendido y en qué tickets, desde que iniciaste hasta ahora. Puedes hacerlo las veces que quieras, no afecta en nada a tus ventas.";

const MENSAJE_Z = "El corte Z es el cierre DEFINITIVO de tu turno: suma todo lo vendido, cierra tu turno y reinicia el contador a $0.00. Úsalo cuando ya te vayas a casa.";

export default function ModalCorte({ onClose, cajero }: ModalCorteProps) {
  const [tipo, setTipo] = useState<TipoCorte>("X");
  const [paso, setPaso] = useState<Paso>("elegir");
  const [procesando, setProcesando] = useState(false);
  const [error, setError] = useState("");
  const [tienda, setTienda] = useState("MI TIENDA");
  const [reporteX, setReporteX] = useState<CorteXReporte | null>(null);
  const [reporteZ, setReporteZ] = useState<CorteZReporte | null>(null);
  const [impresoras, setImpresoras] = useState<ImpresoraInfo[]>([]);
  const [impresoraSel, setImpresoraSel] = useState("");
  const [anchoMm, setAnchoMm] = useState<80 | 58>(80);
  const [imprimiendo, setImprimiendo] = useState(false);
  const [msgImpresion, setMsgImpresion] = useState("");

  useEffect(() => {
    obtenerTiendaInfo()
      .then((t) => {
        if (t.nombre) setTienda(t.nombre);
      })
      .catch(() => {});
  }, []);

  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, [onClose]);

  // Las impresoras se cargan al llegar a la previa (no antes: más rápido abrir).
  useEffect(() => {
    if (paso !== "previa") return;
    listarImpresoras()
      .then((lista) => {
        setImpresoras(lista);
        setImpresoraSel(lista.find((i) => i.predeterminada)?.nombre ?? lista[0]?.nombre ?? "");
      })
      .catch((err) => setError(`No se pudieron listar impresoras: ${String(err)}`));
  }, [paso]);

  const handleContinuar = async () => {
    setProcesando(true);
    setError("");
    try {
      if (tipo === "X") {
        setReporteX(await pedirCorteX());
      } else {
        setReporteZ(await pedirCorteZ());
      }
      setPaso("previa");
    } catch (err) {
      setError(String(err));
    } finally {
      setProcesando(false);
    }
  };

  const handleImprimir = async () => {
    const corteId = tipo === "X" ? reporteX?.corte_id : reporteZ?.corte_id;
    if (!corteId) return;
    if (!impresoraSel) {
      setError("Elige una impresora instalada.");
      return;
    }
    setImprimiendo(true);
    setError("");
    setMsgImpresion("");
    try {
      const destino: DestinoPrint = { Spooler: { nombre: impresoraSel } };
      const msg = await imprimirCorte(destino, corteId, anchoMm);
      setMsgImpresion(msg);
    } catch (err) {
      setError(String(err));
    } finally {
      setImprimiendo(false);
    }
  };

  const textoPrevia =
    paso === "previa"
      ? tipo === "X" && reporteX
        ? textoCorteX(reporteX, tienda, cajero)
        : reporteZ
          ? textoCorteZ(reporteZ, tienda, cajero)
          : ""
      : "";

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4" onClick={onClose}>
      <div
        className="bg-white rounded-[2.5rem] shadow-2xl w-full max-w-md overflow-hidden animate-in zoom-in-95 fade-in duration-200 max-h-[90vh] overflow-y-auto custom-scrollbar"
        onClick={(e) => e.stopPropagation()}
      >
        {/* HEADER OSCURO */}
        <div className="bg-neutral-950 px-8 pt-7 pb-6 text-center relative overflow-hidden">
          <div className="absolute -top-10 -right-10 w-40 h-40 bg-white/[0.04] rounded-full blur-2xl" />
          <button
            onClick={onClose}
            className="absolute top-5 right-5 p-2 rounded-xl hover:bg-white/10 text-neutral-500 hover:text-white transition-all"
            title="Cerrar (Esc)"
          >
            <MorphIcon icon={ICONO_EQUIS} size={16} strokeWidth={2.5} spring="snappy" reducedMotion="user" />
          </button>
          <div className="w-12 h-12 mx-auto bg-white/10 rounded-2xl flex items-center justify-center mb-3">
            <MorphIcon icon={ICONO_CAJA} size={20} strokeWidth={2.2} spring="smooth" className="text-white" />
          </div>
          <h2 className="text-lg font-black text-white uppercase tracking-tight">
            {paso === "elegir" ? "Corte de caja" : `Previsualización — Corte ${tipo} #${tipo === "X" ? reporteX?.corte_id : reporteZ?.corte_id}`}
          </h2>
          <p className="text-[9px] font-black text-neutral-500 uppercase tracking-[0.25em] mt-1">
            {paso === "elegir" ? "Elige el tipo de corte" : "Así saldrá en la térmica"}
          </p>
        </div>

        <div className="p-7 space-y-4">
          {paso === "elegir" && (
            <>
              {/* PASTILLAS X / Z */}
              <div className="grid grid-cols-2 gap-3">
                {(["X", "Z"] as TipoCorte[]).map((t) => (
                  <button
                    key={t}
                    onClick={() => setTipo(t)}
                    className={`py-4 rounded-3xl text-sm font-black uppercase tracking-[0.2em] border-2 transition-all active:scale-[0.97] ${
                      tipo === t
                        ? "bg-neutral-950 text-white border-neutral-950 shadow-xl shadow-neutral-300"
                        : "bg-neutral-100 text-neutral-400 border-transparent hover:bg-neutral-200"
                    }`}
                  >
                    {t}
                  </button>
                ))}
              </div>

              {/* MENSAJE SEGÚN PASTILLA */}
              <div className="bg-neutral-50 rounded-3xl p-5 border border-neutral-100">
                <p className="text-xs font-bold text-neutral-600 leading-relaxed">
                  {tipo === "X" ? MENSAJE_X : MENSAJE_Z}
                </p>
                {tipo === "Z" && (
                  <p className="text-sm font-black text-neutral-900 mt-3">¿Quieres continuar?</p>
                )}
              </div>

              {error && (
                <p className="text-[11px] font-black text-red-500 text-center uppercase tracking-widest bg-red-50 rounded-2xl py-3 px-4">
                  {error}
                </p>
              )}

              <div className="pt-1 space-y-2.5">
                <button
                  onClick={handleContinuar}
                  disabled={procesando}
                  className="w-full py-5 rounded-3xl bg-neutral-950 text-white text-sm font-black uppercase tracking-[0.2em] hover:bg-neutral-800 transition-all shadow-xl shadow-neutral-300 active:scale-[0.98] disabled:opacity-30 disabled:cursor-not-allowed flex items-center justify-center gap-3"
                >
                  <MorphIcon icon={ICONO_CHECK} size={18} strokeWidth={3} spring="snappy" reducedMotion="user" />
                  {procesando ? "Generando..." : "Continuar"}
                </button>
                <button
                  onClick={onClose}
                  className="w-full py-3.5 text-[10px] font-black text-neutral-400 uppercase tracking-widest hover:text-neutral-950 transition-colors"
                >
                  Cancelar
                </button>
              </div>
            </>
          )}

          {paso === "previa" && (
            <>
              {/* TICKET MONOESPACIADO */}
              <pre className="bg-neutral-950 text-emerald-300 rounded-3xl p-5 text-[10px] leading-relaxed font-mono whitespace-pre-wrap break-words overflow-x-auto custom-scrollbar">
                {textoPrevia}
              </pre>

              {/* SELECTOR DE IMPRESORA + ANCHO */}
              <div className="space-y-2">
                <div className="flex items-center justify-between">
                  <label className="text-[10px] font-black uppercase tracking-widest text-neutral-400">
                    Impresora térmica
                  </label>
                  <div className="flex bg-neutral-100 rounded-lg p-0.5">
                    {([80, 58] as const).map((mm) => (
                      <button
                        key={mm}
                        onClick={() => setAnchoMm(mm)}
                        className={`px-2.5 py-1 text-[8px] font-black rounded-md transition-all ${
                          anchoMm === mm ? "bg-neutral-950 text-white" : "text-neutral-400"
                        }`}
                      >
                        {mm}mm
                      </button>
                    ))}
                  </div>
                </div>
                <select
                  value={impresoraSel}
                  onChange={(e) => setImpresoraSel(e.target.value)}
                  className="w-full py-4 px-5 rounded-2xl bg-neutral-50 border-2 border-transparent focus:border-neutral-900 focus:bg-white text-sm font-black text-neutral-900 focus:outline-none transition-all"
                >
                  {impresoras.length === 0 && <option value="">Sin impresoras...</option>}
                  {impresoras.map((i) => (
                    <option key={i.nombre} value={i.nombre}>
                      {i.nombre}{i.predeterminada ? " (predeterminada)" : ""}
                    </option>
                  ))}
                </select>
              </div>

              {msgImpresion && (
                <p className="text-[11px] font-black text-emerald-600 text-center uppercase tracking-widest bg-emerald-50 rounded-2xl py-3 px-4">
                  {msgImpresion}
                </p>
              )}

              {error && (
                <p className="text-[11px] font-black text-red-500 text-center uppercase tracking-widest bg-red-50 rounded-2xl py-3 px-4">
                  {error}
                </p>
              )}

              <div className="pt-1 space-y-2.5">
                <button
                  onClick={handleImprimir}
                  disabled={imprimiendo || !impresoraSel}
                  className="w-full py-5 rounded-3xl bg-neutral-950 text-white text-sm font-black uppercase tracking-[0.2em] hover:bg-neutral-800 transition-all shadow-xl shadow-neutral-300 active:scale-[0.98] disabled:opacity-30 disabled:cursor-not-allowed flex items-center justify-center gap-3"
                >
                  <MorphIcon icon={ICONO_IMPRESORA} size={18} strokeWidth={2.5} spring="snappy" reducedMotion="user" />
                  {imprimiendo ? "Imprimiendo..." : "Imprimir en térmica"}
                </button>
                <button
                  onClick={onClose}
                  className="w-full py-3.5 text-[10px] font-black text-neutral-400 uppercase tracking-widest hover:text-neutral-950 transition-colors"
                >
                  Cerrar
                </button>
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
