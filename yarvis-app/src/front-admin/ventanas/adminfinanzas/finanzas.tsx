// ═══════════════════════════════════════════════════════════════════════════
// FINANZAS — Orquestador principal del módulo financiero (AdminFinanzas).
// Tarea única: poseer el estado global del panel (sección activa, rango de
// fechas, datos cargados y modales), ejecutar la carga de los 30+ comandos
// Tauri con Promise.allSettled y enrutar el render a las secciones:
//   seccion-resumen · seccion-gastos · seccion-cortes · seccion-alertas ·
//   seccion-metricas (+ modales de gasto, pago y detalle de corte).
// La presentación vive en cada archivo de sección; aquí solo hay estado,
// carga de datos y navegación.
// ═══════════════════════════════════════════════════════════════════════════

import { useState, useEffect, useMemo, useCallback } from "react";
import { MorphIcon } from "morphicons/react";
import { BotonAnimado, ICONO_REINICIAR, ICONO_CHECK, ICONO_GRAFICA } from "../../../components/ui";
import { reportarError } from "../../../services/tauri";
import { notificarExito } from "../../../components/notificaciones";
import {
  obtenerResumenPeriodo, obtenerPuntoEquilibrio, obtenerDatosGraficaPL,
  obtenerGastosPorCategoria, obtenerVentasVsGastos, obtenerTendenciaCortesZ,
  obtenerPrediccionesFinancieras, obtenerGastos, obtenerCortes, obtenerAlertas,
  obtenerMetricasDiarias, marcarAlertaLeida, exportarGastosCsv, exportarBalancePdf,
} from "../../../services/finanzas";
import type {
  ResumenPeriodo, DatoGraficaPL, DatoGraficaGastosCategoria, DatoGraficaCortesZ,
  PuntoEquilibrio, AlertaFinanciera, GastoRecurrente, CorteCaja, MetricasUtilidad,
} from "../../types";
import { TABS, type Seccion } from "./nucleo/constantes";
import { rangoDeDias, type RangoFechas } from "./nucleo/utilidades";
import ModalGasto from "./componentes/modal-gasto";
import ModalDetalleCorte from "./componentes/modal-detalle-corte";
import ModalPagoGasto from "./componentes/modal-pago-gasto";
import SeccionResumen from "./componentes/seccion-resumen";
import SeccionGastos from "./componentes/seccion-gastos";
import SeccionCortes from "./componentes/seccion-cortes";
import SeccionAlertas from "./componentes/seccion-alertas";
import SeccionMetricas from "./componentes/seccion-metricas";

export default function AdminFinanzas({ active = true }: { active?: boolean }) {
  const [seccion, setSeccion] = useState<Seccion>("resumen");
  const [rango, setRango] = useState<RangoFechas>(() => rangoDeDias(180));
  const [cargando, setCargando] = useState(true);

  // Data
  const [resumen, setResumen] = useState<ResumenPeriodo | null>(null);
  const [puntoEq, setPuntoEq] = useState<PuntoEquilibrio | null>(null);
  const [plData, setPlData] = useState<DatoGraficaPL[]>([]);
  const [gastosCat, setGastosCat] = useState<DatoGraficaGastosCategoria[]>([]);
  const [ventasGastos, setVentasGastos] = useState<DatoGraficaPL[]>([]);
  const [cortesZ, setCortesZ] = useState<DatoGraficaCortesZ[]>([]);
  const [gastos, setGastos] = useState<GastoRecurrente[]>([]);
  const [cortes, setCortes] = useState<CorteCaja[]>([]);
  const [alertas, setAlertas] = useState<AlertaFinanciera[]>([]);
  const [metricas, setMetricas] = useState<MetricasUtilidad[]>([]);

  // UI state
  const [modalGasto, setModalGasto] = useState<GastoRecurrente | undefined>();
  const [modalNuevoGasto, setModalNuevoGasto] = useState(false);
  const [modalPagoGasto, setModalPagoGasto] = useState<GastoRecurrente | undefined>();
  const [modalDetalleCorte, setModalDetalleCorte] = useState<CorteCaja | undefined>();
  const [predicciones, setPredicciones] = useState<any[]>([]);
  const [diasPrediccion, setDiasPrediccion] = useState(30);

  // ── Carga de datos ──────────────────────────────────────────────────────

  const cargarResumen = useCallback(async () => {
    try {
      setResumen(await obtenerResumenPeriodo(rango));
    } catch (e) {
      reportarError("No se pudo cargar el resumen financiero", e);
    }
  }, [rango]);

  const cargarPuntoEq = useCallback(async () => {
    try {
      setPuntoEq(await obtenerPuntoEquilibrio());
    } catch (e) {
      reportarError("No se pudo cargar el punto de equilibrio", e);
    }
  }, []);

  const cargarGraficas = useCallback(async () => {
    try {
      setPlData(await obtenerDatosGraficaPL(rango));
    } catch (e) {
      reportarError("No se pudo cargar la gráfica de pérdidas y ganancias", e);
    }
    try {
      setGastosCat(await obtenerGastosPorCategoria(rango));
    } catch (e) {
      reportarError("No se pudo cargar el desglose de gastos por categoría", e);
    }
    try {
      setVentasGastos(await obtenerVentasVsGastos(6));
    } catch (e) {
      reportarError("No se pudo cargar la comparativa de ventas vs gastos", e);
    }
    try {
      setCortesZ(await obtenerTendenciaCortesZ(rango));
    } catch (e) {
      reportarError("No se pudo cargar la tendencia de cortes Z", e);
    }
  }, [rango]);

  const cargarPredicciones = useCallback(async () => {
    try {
      setPredicciones(await obtenerPrediccionesFinancieras(diasPrediccion));
    } catch (e) {
      reportarError("No se pudieron cargar las predicciones financieras", e);
    }
  }, [diasPrediccion]);

  const cargarGastos = useCallback(async () => {
    try {
      setGastos(await obtenerGastos());
    } catch (e) {
      reportarError("No se pudo cargar la lista de gastos", e);
    }
  }, []);

  const cargarCortes = useCallback(async () => {
    try {
      setCortes(await obtenerCortes(rango));
    } catch (e) {
      reportarError("No se pudo cargar el historial de cortes de caja", e);
    }
  }, [rango]);

  const cargarAlertas = useCallback(async () => {
    try {
      setAlertas(await obtenerAlertas());
    } catch (e) {
      reportarError("No se pudieron cargar las alertas financieras", e);
    }
  }, []);

  const cargarMetricas = useCallback(async () => {
    try {
      setMetricas(await obtenerMetricasDiarias(rango));
    } catch (e) {
      reportarError("No se pudieron cargar las métricas diarias", e);
    }
  }, [rango]);

  const cargarTodo = useCallback(async () => {
    setCargando(true);
    await Promise.allSettled([
      cargarResumen(),
      cargarPuntoEq(),
      cargarGraficas(),
      cargarPredicciones(),
      cargarGastos(),
      cargarCortes(),
      cargarAlertas(),
      cargarMetricas(),
    ]);
    setCargando(false);
  }, [cargarResumen, cargarPuntoEq, cargarGraficas, cargarPredicciones, cargarGastos, cargarCortes, cargarAlertas, cargarMetricas]);

  // Con keep-alive el módulo ya no se desmonta al cambiar de pestaña:
  // se recarga al volver (datos frescos) y no hace nada oculto.
  useEffect(() => { if (active) cargarTodo(); }, [cargarTodo, active]);

  const recargarGastos = () => { cargarGastos(); cargarResumen(); cargarPuntoEq(); cargarGraficas(); };
  const marcarLeida = async (id: number) => {
    try {
      await marcarAlertaLeida(id);
      cargarAlertas();
    } catch (e) {
      reportarError("No se pudo marcar la alerta como leída", e);
    }
  };

  const manejarExportCsv = async () => {
    try {
      await exportarGastosCsv(rango);
      notificarExito("Gastos exportados a CSV");
    } catch (e) {
      reportarError("No se pudo exportar el CSV de gastos", e);
    }
  };

  const manejarExportPdf = async () => {
    try {
      await exportarBalancePdf(rango);
      notificarExito("Balance exportado a PDF");
    } catch (e) {
      reportarError("No se pudo exportar el balance a PDF", e);
    }
  };

  const alertasNoLeidas = useMemo(() => alertas.filter((a) => !a.leida).length, [alertas]);
  const cerrarModalesGasto = () => { setModalNuevoGasto(false); setModalGasto(undefined); };

  // ── RENDER ──────────────────────────────────────────────────────────────

  return (
    <div className="max-w-[1200px] mx-auto space-y-10 animate-in fade-in slide-in-from-bottom-2 duration-500">

      {/* ═══ HEADER ═══════════════════════════════════════════════════════ */}
      <header className="space-y-6">
        <div className="flex flex-col sm:flex-row justify-between items-start sm:items-end gap-4">
          <div>
            <h2 className="text-3xl font-black text-neutral-950 uppercase tracking-tight">Finanzas</h2>
            <p className="text-[10px] font-black text-neutral-400 uppercase tracking-[0.3em]">Panel financiero completo</p>
          </div>
          <div className="flex flex-wrap gap-2">
            <BotonAnimado
              icono={ICONO_REINICIAR}
              iconoHover={ICONO_CHECK}
              onClick={manejarExportCsv}
              className="bg-white text-neutral-950 border border-neutral-200 hover:bg-neutral-50 shadow-xl shadow-neutral-200"
            >
              CSV gastos
            </BotonAnimado>
            <BotonAnimado
              icono={ICONO_GRAFICA}
              iconoHover={ICONO_CHECK}
              onClick={manejarExportPdf}
              className="bg-white text-neutral-950 border border-neutral-200 hover:bg-neutral-50 shadow-xl shadow-neutral-200"
            >
              Balance PDF
            </BotonAnimado>
            <BotonAnimado
              icono={ICONO_REINICIAR}
              iconoHover={ICONO_CHECK}
              onClick={cargarTodo}
              className="bg-neutral-950 text-neutral-50 hover:bg-neutral-800 shadow-xl shadow-neutral-200"
            >
              Actualizar
            </BotonAnimado>
          </div>
        </div>

        {/* ── TABS GORDITOS ──────────────────────────────────────────── */}
        <div className="flex bg-neutral-950 p-1.5 rounded-2xl shadow-2xl">
          {TABS.map((t) => {
            const activo = seccion === t.id;
            return (
              <button
                key={t.id}
                onClick={() => setSeccion(t.id)}
                className={`relative flex-1 flex items-center justify-center gap-2 px-4 py-3 text-[10px] font-black rounded-xl transition-all duration-200 ${activo
                    ? "bg-white text-neutral-950 shadow-lg scale-[1.02]"
                    : "text-neutral-500 hover:text-neutral-300 hover:bg-white/5"
                  }`}
              >
                <MorphIcon icon={t.icono} size={14} strokeWidth={2.5} spring="snappy" reducedMotion="user" />
                <span>{t.label}</span>
                {t.id === "alertas" && alertasNoLeidas > 0 && (
                  <span className="absolute -top-1.5 -right-0.5 min-w-[18px] h-[18px] bg-red-500 text-white text-[8px] font-black rounded-full flex items-center justify-center px-1 shadow-lg shadow-red-500/30">
                    {alertasNoLeidas}
                  </span>
                )}
              </button>
            );
          })}
        </div>
      </header>

      {/* ═══ CONTENIDO ═════════════════════════════════════════════════ */}
      {cargando ? (
        <div className="py-20 text-center space-y-3">
          <div className="w-12 h-12 mx-auto bg-neutral-950 rounded-2xl flex items-center justify-center animate-pulse">
            <MorphIcon icon={ICONO_GRAFICA} size={20} strokeWidth={2} spring="smooth" className="text-white" />
          </div>
          <p className="text-[11px] font-black text-neutral-300 uppercase tracking-widest italic">
            Cargando datos financieros...
          </p>
        </div>
      ) : (
        <>
          {seccion === "resumen" && (
            <SeccionResumen
              resumen={resumen}
              puntoEq={puntoEq}
              plData={plData}
              gastosCat={gastosCat}
              ventasGastos={ventasGastos}
              cortesZ={cortesZ}
              predicciones={predicciones}
              diasPrediccion={diasPrediccion}
              rango={rango}
              onRango={setRango}
              onDiasPrediccion={setDiasPrediccion}
            />
          )}

          {seccion === "gastos" && (
            <SeccionGastos
              gastos={gastos}
              onNuevo={() => setModalNuevoGasto(true)}
              onEditar={setModalGasto}
              onPago={setModalPagoGasto}
              onRecargar={recargarGastos}
            />
          )}

          {seccion === "cortes" && (
            <SeccionCortes cortes={cortes} rango={rango} onRango={setRango} onVerDetalle={setModalDetalleCorte} />
          )}

          {seccion === "alertas" && (
            <SeccionAlertas alertas={alertas} alertasNoLeidas={alertasNoLeidas} onMarcarLeida={marcarLeida} />
          )}

          {seccion === "metricas" && (
            <SeccionMetricas metricas={metricas} rango={rango} onRango={setRango} />
          )}
        </>
      )}

      {/* MODALES */}
      {(modalNuevoGasto || modalGasto) && (
        <ModalGasto gasto={modalGasto} onCerrar={cerrarModalesGasto} onGuardado={() => { cerrarModalesGasto(); recargarGastos(); }} />
      )}
      {modalPagoGasto && (
        <ModalPagoGasto gasto={modalPagoGasto} onCerrar={() => setModalPagoGasto(undefined)} onGuardado={() => { setModalPagoGasto(undefined); recargarGastos(); }} />
      )}
      {modalDetalleCorte && (
        <ModalDetalleCorte
          corte={modalDetalleCorte}
          onCerrar={() => { setModalDetalleCorte(undefined); cargarCortes(); }}
          onActualizado={cargarCortes}
        />
      )}
    </div>
  );
}
