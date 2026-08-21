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
import { invoke } from "@tauri-apps/api/core";
import { MorphIcon } from "morphicons/react";
import { BotonAnimado, ICONO_REINICIAR, ICONO_CHECK, ICONO_GRAFICA } from "../../../components/ui";
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

export default function AdminFinanzas() {
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
      const r = await invoke<ResumenPeriodo>("get_resumen_periodo", { fechaInicio: rango.inicio, fechaFin: rango.fin });
      setResumen(r);
    } catch (e) {
      console.error("[FINANZAS] Error en get_resumen_periodo:", e);
    }
  }, [rango]);

  const cargarPuntoEq = useCallback(async () => {
    try {
      const pe = await invoke<PuntoEquilibrio>("get_punto_equilibrio");
      setPuntoEq(pe);
    } catch (e) {
      console.error("[FINANZAS] Error en get_punto_equilibrio:", e);
    }
  }, []);

  const cargarGraficas = useCallback(async () => {
    try {
      const pl = await invoke<DatoGraficaPL[]>("get_datos_grafica_pl", { fechaInicio: rango.inicio, fechaFin: rango.fin, granularidad: "dia" });
      setPlData(pl);
    } catch (e) {
      console.error("[FINANZAS] Error en get_datos_grafica_pl:", e);
    }
    try {
      const gc = await invoke<DatoGraficaGastosCategoria[]>("get_gastos_por_categoria", { fechaInicio: rango.inicio, fechaFin: rango.fin });
      setGastosCat(gc);
    } catch (e) {
      console.error("[FINANZAS] Error en get_gastos_por_categoria:", e);
    }
    try {
      const vz = await invoke<DatoGraficaPL[]>("get_ventas_vs_gastos_mensual", { meses: 6 });
      setVentasGastos(vz);
    } catch (e) {
      console.error("[FINANZAS] Error en get_ventas_vs_gastos_mensual:", e);
    }
    try {
      const cz = await invoke<DatoGraficaCortesZ[]>("get_tendencia_cortes_z", { fechaInicio: rango.inicio, fechaFin: rango.fin });
      setCortesZ(cz);
    } catch (e) {
      console.error("[FINANZAS] Error en get_tendencia_cortes_z:", e);
    }
  }, [rango]);

  const cargarPredicciones = useCallback(async () => {
    try {
      const res = await invoke<{ data: any[] }>("get_predicciones_financieras", { days: diasPrediccion });
      setPredicciones(res.data ?? []);
    } catch (e) {
      console.error("[FINANZAS] Error en cargarPredicciones:", e);
    }
  }, [diasPrediccion]);

  const cargarGastos = useCallback(async () => {
    try {
      const g = await invoke<GastoRecurrente[]>("get_gastos_recurrentes");
      setGastos(g);
    } catch (e) {
      console.error("[FINANZAS] Error en cargarGastos:", e);
    }
  }, []);

  const cargarCortes = useCallback(async () => {
    try {
      const c = await invoke<CorteCaja[]>("get_cortes_caja", {
        filtros: { cajero_id: null, fecha_inicio: rango.inicio, fecha_fin: rango.fin, turno: null, tipo_corte: null, estado: null },
      });
      setCortes(c);
    } catch (e) {
      console.error("[FINANZAS] Error en cargarCortes:", e);
    }
  }, [rango]);

  const cargarAlertas = useCallback(async () => {
    try {
      await invoke("generar_alertas_automaticas");
      const a = await invoke<AlertaFinanciera[]>("get_alertas", { soloNoLeidas: false });
      setAlertas(a);
    } catch (e) {
      console.error("[FINANZAS] Error en cargarAlertas:", e);
    }
  }, []);

  const cargarMetricas = useCallback(async () => {
    try {
      const m = await invoke<MetricasUtilidad[]>("get_metricas_diarias", { fechaInicio: rango.inicio, fechaFin: rango.fin });
      setMetricas(m);
    } catch (e) {
      console.error("[FINANZAS] Error en cargarMetricas:", e);
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

  useEffect(() => { cargarTodo(); }, [cargarTodo]);

  const recargarGastos = () => { cargarGastos(); cargarResumen(); cargarPuntoEq(); cargarGraficas(); };
  const marcarLeida = async (id: number) => {
    try {
      await invoke("marcar_alerta_leida", { id });
      cargarAlertas();
    } catch (e) {
      console.error("[FINANZAS] Error marcando alerta:", e);
    }
  };

  const alertasNoLeidas = useMemo(() => alertas.filter((a) => !a.leida).length, [alertas]);
  const cerrarModalesGasto = () => { setModalNuevoGasto(false); setModalGasto(undefined); };

  // ── RENDER ──────────────────────────────────────────────────────────────

  return (
    <div className="max-w-6xl mx-auto space-y-10 animate-in fade-in slide-in-from-bottom-2 duration-500">

      {/* ═══ HEADER ═══════════════════════════════════════════════════════ */}
      <header className="space-y-6">
        <div className="flex flex-col sm:flex-row justify-between items-start sm:items-end gap-4">
          <div>
            <h2 className="text-3xl font-black text-neutral-950 uppercase tracking-tight">Finanzas</h2>
            <p className="text-[10px] font-black text-neutral-400 uppercase tracking-[0.3em]">Panel financiero completo</p>
          </div>
          <BotonAnimado
            icono={ICONO_REINICIAR}
            iconoHover={ICONO_CHECK}
            onClick={cargarTodo}
            className="bg-neutral-950 text-neutral-50 hover:bg-neutral-800 shadow-xl shadow-neutral-200"
          >
            Actualizar
          </BotonAnimado>
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
        <ModalDetalleCorte corte={modalDetalleCorte} onCerrar={() => setModalDetalleCorte(undefined)} />
      )}
    </div>
  );
}
