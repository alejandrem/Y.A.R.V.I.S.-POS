import { useState, useEffect, useMemo, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  Line, BarChart, Bar, PieChart, Pie, Cell,
  XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Legend, Area, AreaChart, ComposedChart,
} from "recharts";
import { MorphIcon } from "morphicons/react";
import {
  BotonAnimado, IconoMorph, ModalShell, Campo, inputCls,
  ICONO_DOLAR, ICONO_TRENDING, ICONO_GRAFICA, ICONO_CALCULADORA,
  ICONO_CAJA, ICONO_ALERTA, ICONO_CAMPANA,
  ICONO_MAS, ICONO_MAS_CIRCULO, ICONO_CHECK,
  ICONO_EDITAR, ICONO_BORRAR,
  ICONO_REINICIAR, ICONO_CALENDARIO, ICONO_BILLETE,
  ICONO_TARGET, ICONO_OJO,
} from "../../../components/ui";
import type {
  ResumenPeriodo, DatoGraficaPL, DatoGraficaGastosCategoria, DatoGraficaCortesZ,
  PuntoEquilibrio, AlertaFinanciera, GastoRecurrente, CorteCaja, MetricasUtilidad,
  CrearGastoRequest, MovimientoCaja,
} from "../../types";

// ── HELPERS ────────────────────────────────────────────────────────────────

const moneda = (v: number) =>
  new Intl.NumberFormat("es-MX", { style: "currency", currency: "MXN" }).format(v);

const porcentaje = (v: number) => `${v >= 0 ? "+" : ""}${v.toFixed(1)}%`;

const fechaRelativa = (f: string) => {
  const d = new Date(f);
  const hoy = new Date();
  const diff = Math.floor((hoy.getTime() - d.getTime()) / 86400000);
  if (diff === 0) return "Hoy";
  if (diff === 1) return "Ayer";
  if (diff < 7) return `Hace ${diff} dias`;
  return d.toLocaleDateString("es-MX", { day: "2-digit", month: "short" });
};

const rangoPorDefecto = () => {
  const fin = new Date();
  const ini = new Date();
  ini.setMonth(ini.getMonth() - 6);
  return { inicio: ini.toISOString().slice(0, 10), fin: fin.toISOString().slice(0, 10) };
};

const COLORS_PL = { ingresos: "#0a0a0a", gastos: "#525252", utilidad: "#22c55e" };
const COLORS_PIE = ["#0a0a0a", "#3b82f6", "#22c55e", "#f59e0b", "#a855f7", "#ef4444", "#737373"];
const COLORS_PREDICCION = { prediccion: "#3b82f6", confianza: "#3b82f620" };

const inputFecha = `${inputCls} text-xs`;

// ── SECCIONES / TABS ──────────────────────────────────────────────────────

type Seccion = "resumen" | "gastos" | "cortes" | "alertas" | "metricas";

const TABS: { id: Seccion; label: string; icono: typeof ICONO_DOLAR }[] = [
  { id: "resumen", label: "Resumen", icono: ICONO_GRAFICA },
  { id: "gastos", label: "Gastos", icono: ICONO_CALCULADORA },
  { id: "cortes", label: "Cortes", icono: ICONO_CAJA },
  { id: "alertas", label: "Alertas", icono: ICONO_CAMPANA },
  { id: "metricas", label: "Metricicas", icono: ICONO_TRENDING },
];

// ═══════════════════════════════════════════════════════════════════════════
// MODAL: CREAR / EDITAR GASTO
// ═══════════════════════════════════════════════════════════════════════════

function ModalGasto({ gasto, onCerrar, onGuardado }: { gasto?: GastoRecurrente; onCerrar: () => void; onGuardado: () => void }) {
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

// ═══════════════════════════════════════════════════════════════════════════
// MODAL: DETALLE CORTE
// ═══════════════════════════════════════════════════════════════════════════

function ModalDetalleCorte({ corte, onCerrar }: { corte: CorteCaja; onCerrar: () => void }) {
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

// ═══════════════════════════════════════════════════════════════════════════
// MODAL: REGISTRAR PAGO GASTO
// ═══════════════════════════════════════════════════════════════════════════

function ModalPagoGasto({ gasto, onCerrar, onGuardado }: { gasto: GastoRecurrente; onCerrar: () => void; onGuardado: () => void }) {
  const [monto, setMonto] = useState(gasto.monto_proyectado);
  const [metodo, setMetodo] = useState("efectivo");
  const [notas, setNotas] = useState("");
  const [guardando, setGuardando] = useState(false);

  const guardar = async () => {
    if (monto <= 0) return;
    setGuardando(true);
    try {
      await invoke("registrar_pago_gasto", {
        pago: {
          gasto_id: gasto.id,
          fecha_pago: new Date().toISOString().slice(0, 19).replace("T", " "),
          monto_pagado: monto,
          metodo_pago: metodo,
          folio_comprobante: null,
          notas: notas || null,
        },
      });
      onGuardado();
    } catch (e) {
      console.error("Error registrando pago:", e);
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

// ═══════════════════════════════════════════════════════════════════════════
// COMPONENTE PRINCIPAL
// ═══════════════════════════════════════════════════════════════════════════

export default function AdminFinanzas() {
  const [seccion, setSeccion] = useState<Seccion>("resumen");
  const [rango, setRango] = useState(rangoPorDefecto());
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
                <MorphIcon
                  icon={t.icono}
                  size={14}
                  strokeWidth={2.5}
                  spring="snappy"
                  reducedMotion="user"
                />
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

        {/* ── RANGO FECHAS ───────────────────────────────────────────── */}
        <div className="flex flex-wrap items-center gap-3 bg-neutral-50 rounded-2xl p-3 border border-neutral-100">
          <div className="flex items-center gap-2">
            <MorphIcon icon={ICONO_CALENDARIO} size={14} strokeWidth={2} spring="smooth" className="text-neutral-400" />
            <input type="date" className={inputFecha} value={rango.inicio} onChange={(e) => setRango((p) => ({ ...p, inicio: e.target.value }))} />
          </div>
          <span className="text-[10px] font-black text-neutral-300">a</span>
          <input type="date" className={inputFecha} value={rango.fin} onChange={(e) => setRango((p) => ({ ...p, fin: e.target.value }))} />
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
          {/* ═══ RESUMEN ═══════════════════════════════════════════════════ */}
          {seccion === "resumen" && (
            <div className="space-y-10">
              {/* KPIs - TARJETAS GORDITAS CON NEGRO INTENSO */}
              <div className="grid grid-cols-2 lg:grid-cols-4 gap-4 sm:gap-5">
                <KPI
                  icono={ICONO_DOLAR}
                  label="Ventas Totales"
                  valor={moneda(resumen?.total_ventas ?? 0)}
                  color="neutral"
                />
                <KPI
                  icono={ICONO_TRENDING}
                  label="Utilidad Neta"
                  valor={moneda(resumen?.total_utilidad_neta ?? 0)}
                  color={(resumen?.total_utilidad_neta ?? 0) >= 0 ? "verde" : "rojo"}
                />
                <KPI
                  icono={ICONO_GRAFICA}
                  label="Margen Neto"
                  valor={porcentaje(resumen?.margen_promedio_pct ?? 0)}
                  color={(resumen?.margen_promedio_pct ?? 0) >= 0 ? "verde" : "rojo"}
                />
                <KPI
                  icono={ICONO_TARGET}
                  label="Punto Equilibrio"
                  valor={moneda(puntoEq?.ventas_necesarias ?? 0)}
                  color="neutral"
                />
              </div>

              {/* GRAFICA P&L */}
              <SeccionGrafica titulo="Perdidas y Ganancias" subtitulo="Ingresos, gastos y utilidad neta">
                {plData.length > 0 ? (
                  <ResponsiveContainer width="100%" height={320}>
                    <AreaChart data={plData}>
                      <CartesianGrid strokeDasharray="3 3" stroke="#f5f5f5" />
                      <XAxis dataKey="fecha" tick={{ fontSize: 10, fontWeight: 700 }} tickFormatter={(v) => v.slice(5)} />
                      <YAxis tick={{ fontSize: 10, fontWeight: 700 }} tickFormatter={(v) => `$${(v / 1000).toFixed(0)}k`} />
                      <Tooltip contentStyle={{ backgroundColor: "#0a0a0a", border: "1px solid #262626", borderRadius: "16px", color: "#fff", fontSize: 11 }} formatter={(v) => moneda(Number(v))} />
                      <Area type="monotone" dataKey="ingresos" stroke={COLORS_PL.ingresos} fill={COLORS_PL.ingresos} fillOpacity={0.08} strokeWidth={2.5} />
                      <Area type="monotone" dataKey="gastos" stroke={COLORS_PL.gastos} fill={COLORS_PL.gastos} fillOpacity={0.05} strokeWidth={2} strokeDasharray="5 5" />
                      <Area type="monotone" dataKey="utilidad_neta" stroke={COLORS_PL.utilidad} fill={COLORS_PL.utilidad} fillOpacity={0.1} strokeWidth={2} />
                    </AreaChart>
                  </ResponsiveContainer>
                ) : (
                  <EmptyGrafica mensaje="Sin datos de P&L en este periodo" />
                )}
              </SeccionGrafica>

              {/* FILA: GASTOS CATEGORIA + VENTAS VS GASTOS */}
              <div className="grid grid-cols-1 lg:grid-cols-2 gap-5">
                <SeccionGrafica titulo="Gastos por Categoria" subtitulo="Distribucion del periodo">
                  {gastosCat.length > 0 ? (
                    <ResponsiveContainer width="100%" height={280}>
                      <PieChart>
                        <Pie data={gastosCat} dataKey="monto" nameKey="categoria" cx="50%" cy="50%" outerRadius={90} innerRadius={50} paddingAngle={3}>
                          {gastosCat.map((_, i) => <Cell key={i} fill={COLORS_PIE[i % COLORS_PIE.length]} />)}
                        </Pie>
                        <Tooltip contentStyle={{ backgroundColor: "#0a0a0a", border: "1px solid #262626", borderRadius: "16px", color: "#fff", fontSize: 11 }} formatter={(v) => moneda(Number(v))} />
                        <Legend wrapperStyle={{ fontSize: 10, fontWeight: 700 }} />
                      </PieChart>
                    </ResponsiveContainer>
                  ) : (
                    <EmptyGrafica mensaje="Sin gastos registrados" />
                  )}
                </SeccionGrafica>

                <SeccionGrafica titulo="Ventas vs Gastos" subtitulo="Ultimos 6 meses">
                  {ventasGastos.length > 0 ? (
                    <ResponsiveContainer width="100%" height={280}>
                      <BarChart data={ventasGastos}>
                        <CartesianGrid strokeDasharray="3 3" stroke="#f5f5f5" />
                        <XAxis dataKey="fecha" tick={{ fontSize: 10, fontWeight: 700 }} />
                        <YAxis tick={{ fontSize: 10, fontWeight: 700 }} tickFormatter={(v) => `$${(v / 1000).toFixed(0)}k`} />
                        <Tooltip contentStyle={{ backgroundColor: "#0a0a0a", border: "1px solid #262626", borderRadius: "16px", color: "#fff", fontSize: 11 }} formatter={(v) => moneda(Number(v))} />
                        <Bar dataKey="ingresos" fill="#0a0a0a" radius={[8, 8, 0, 0]} />
                        <Bar dataKey="gastos" fill="#525252" radius={[8, 8, 0, 0]} />
                      </BarChart>
                    </ResponsiveContainer>
                  ) : (
                    <EmptyGrafica mensaje="Sin datos mensuales" />
                  )}
                </SeccionGrafica>
              </div>

              {/* FILA: TENDENCIA CORTES Z + PREDICCIONES */}
              <div className="grid grid-cols-1 lg:grid-cols-2 gap-5">
                <SeccionGrafica titulo="Tendencia Cortes Z" subtitulo="Ventas por turno de cierre">
                  {cortesZ.length > 0 ? (
                    <ResponsiveContainer width="100%" height={280}>
                      <ComposedChart data={cortesZ}>
                        <CartesianGrid strokeDasharray="3 3" stroke="#f5f5f5" />
                        <XAxis dataKey="fecha" tick={{ fontSize: 10, fontWeight: 700 }} tickFormatter={(v) => v.slice(5)} />
                        <YAxis tick={{ fontSize: 10, fontWeight: 700 }} tickFormatter={(v) => `$${(v / 1000).toFixed(0)}k`} />
                        <Tooltip contentStyle={{ backgroundColor: "#0a0a0a", border: "1px solid #262626", borderRadius: "16px", color: "#fff", fontSize: 11 }} formatter={(v) => moneda(Number(v))} />
                        <Bar dataKey="total_ventas" fill="#0a0a0a" radius={[8, 8, 0, 0]} />
                        <Line type="monotone" dataKey="diferencia" stroke="#ef4444" strokeWidth={2} dot={{ r: 3 }} />
                      </ComposedChart>
                    </ResponsiveContainer>
                  ) : (
                    <EmptyGrafica mensaje="Sin cortes Z en este periodo" />
                  )}
                </SeccionGrafica>

                <SeccionGrafica
                  titulo="Predicciones de Ventas"
                  subtitulo="Modelo Holt-Winters"
                  accion={
                    <div className="flex bg-neutral-100 p-0.5 rounded-xl">
                      {[15, 30, 90].map((d) => (
                        <button
                          key={d}
                          onClick={() => setDiasPrediccion(d)}
                          className={`px-3 py-1.5 text-[9px] font-black rounded-lg transition-all ${diasPrediccion === d ? "bg-neutral-950 text-white shadow-md" : "text-neutral-400 hover:text-neutral-700"}`}
                        >
                          {d}D
                        </button>
                      ))}
                    </div>
                  }
                >
                  {predicciones.length > 0 ? (
                    <ResponsiveContainer width="100%" height={280}>
                      <AreaChart data={predicciones}>
                        <CartesianGrid strokeDasharray="3 3" stroke="#f5f5f5" />
                        <XAxis dataKey="fecha" tick={{ fontSize: 10, fontWeight: 700 }} tickFormatter={(v) => v.slice(5)} />
                        <YAxis tick={{ fontSize: 10, fontWeight: 700 }} tickFormatter={(v) => `$${(v / 1000).toFixed(0)}k`} />
                        <Tooltip contentStyle={{ backgroundColor: "#0a0a0a", border: "1px solid #262626", borderRadius: "16px", color: "#fff", fontSize: 11 }} formatter={(v) => moneda(Number(v))} />
                        <Area type="monotone" dataKey="prediccion" stroke={COLORS_PREDICCION.prediccion} fill={COLORS_PREDICCION.confianza} strokeWidth={2.5} strokeDasharray="6 3" />
                        <Area type="monotone" dataKey="maximo" stroke="transparent" fill={COLORS_PREDICCION.prediccion} fillOpacity={0.06} />
                        <Area type="monotone" dataKey="minimo" stroke="transparent" fill="#fff" fillOpacity={0} />
                      </AreaChart>
                    </ResponsiveContainer>
                  ) : (
                    <EmptyGrafica mensaje="Generando predicciones..." />
                  )}
                </SeccionGrafica>
              </div>

              {/* PUNTO DE EQUILIBRIO - PANEL OSCURO INTENSO */}
              {puntoEq && (
                <div className="bg-neutral-950 rounded-[2.5rem] p-6 sm:p-10 shadow-2xl relative overflow-hidden">
                  <div className="absolute top-0 right-0 w-80 h-80 bg-white/[0.03] rounded-full blur-3xl -translate-y-1/3 translate-x-1/3" />
                  <div className="absolute bottom-0 left-0 w-48 h-48 bg-emerald-500/[0.04] rounded-full blur-3xl translate-y-1/2 -translate-x-1/2" />
                  <div className="relative">
                    <div className="flex items-center gap-3 mb-8">
                      <div className="w-12 h-12 rounded-2xl bg-white/10 flex items-center justify-center">
                        <MorphIcon icon={ICONO_TARGET} size={22} strokeWidth={2} spring="smooth" className="text-white" />
                      </div>
                      <div>
                        <h3 className="text-base font-black text-white uppercase tracking-tight">Punto de Equilibrio</h3>
                        <p className="text-[9px] font-black text-white/30 uppercase tracking-widest">Analisis break-even ultimos 30 dias</p>
                      </div>
                    </div>
                    <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
                      <div className="bg-white/5 rounded-2xl p-5 border border-white/5">
                        <p className="text-[9px] font-black text-white/30 uppercase tracking-widest">Gastos Fijos Mensuales</p>
                        <p className="text-2xl font-black text-white mt-2">{moneda(puntoEq.gastos_fijos_mensuales)}</p>
                      </div>
                      <div className="bg-white/5 rounded-2xl p-5 border border-white/5">
                        <p className="text-[9px] font-black text-white/30 uppercase tracking-widest">Margen de Contribucion</p>
                        <p className="text-2xl font-black text-emerald-400 mt-2">{porcentaje(puntoEq.margen_contribucion_pct)}</p>
                      </div>
                      <div className="bg-white/5 rounded-2xl p-5 border border-white/5">
                        <p className="text-[9px] font-black text-white/30 uppercase tracking-widest">Ventas Necesarias</p>
                        <p className="text-2xl font-black text-white mt-2">{moneda(puntoEq.ventas_necesarias)}</p>
                      </div>
                      <div className="bg-white/5 rounded-2xl p-5 border border-white/5">
                        <p className="text-[9px] font-black text-white/30 uppercase tracking-widest">Tickets Necesarios</p>
                        <p className="text-2xl font-black text-white mt-2">{puntoEq.tickets_necesarios}</p>
                      </div>
                    </div>
                  </div>
                </div>
              )}
            </div>
          )}

          {/* ═══ GASTOS ═══════════════════════════════════════════════════ */}
          {seccion === "gastos" && (
            <div className="space-y-6">
              <div className="flex justify-between items-center">
                <div className="flex items-center gap-3">
                  <div className="w-1.5 h-5 bg-neutral-950 rounded-full" />
                  <h3 className="text-base sm:text-xl font-black text-neutral-950 uppercase tracking-tight">Gastos Recurrentes</h3>
                  <span className="px-3 py-1 bg-neutral-950 text-white text-[9px] font-black rounded-lg">{gastos.length}</span>
                </div>
                <BotonAnimado
                  icono={ICONO_MAS}
                  iconoHover={ICONO_MAS_CIRCULO}
                  onClick={() => setModalNuevoGasto(true)}
                  className="bg-neutral-950 text-neutral-50 hover:bg-neutral-800 shadow-xl shadow-neutral-200"
                >
                  Nuevo Gasto
                </BotonAnimado>
              </div>

              {gastos.length > 0 ? (
                <div className="bg-white rounded-[2.5rem] border border-neutral-200 shadow-sm overflow-hidden">
                  <div className="max-h-[560px] overflow-y-auto custom-scrollbar">
                    <table className="w-full text-left border-collapse">
                      <thead className="sticky top-0 z-10">
                        <tr className="bg-neutral-950">
                          <th className="px-8 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Nombre</th>
                          <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Categoria</th>
                          <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Proyectado</th>
                          <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Real</th>
                          <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Estado</th>
                          <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Vence</th>
                          <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest text-right">Acciones</th>
                        </tr>
                      </thead>
                      <tbody className="divide-y divide-neutral-50">
                        {gastos.map((g) => (
                          <tr key={g.id} className="group hover:bg-neutral-50/50 transition-all">
                            <td className="px-8 py-4">
                              <span className="text-xs font-black text-neutral-950 uppercase">{g.nombre}</span>
                              <p className="text-[9px] text-neutral-400 font-bold">{g.tipo} / {g.frecuencia}</p>
                            </td>
                            <td className="px-6 py-4">
                              <span className="px-2.5 py-1 bg-neutral-950 text-white text-[9px] font-black rounded-lg uppercase">{g.categoria}</span>
                            </td>
                            <td className="px-6 py-4 text-xs font-black text-neutral-900">{moneda(g.monto_proyectado)}</td>
                            <td className="px-6 py-4 text-xs font-black text-neutral-900">{moneda(g.monto_real)}</td>
                            <td className="px-6 py-4">
                              <span className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-[9px] font-black uppercase ${g.estado_pago === "pagado" ? "bg-emerald-50 text-emerald-600" :
                                  g.estado_pago === "vencido" ? "bg-red-50 text-red-500" :
                                    g.estado_pago === "proximo_vencer" ? "bg-amber-50 text-amber-600" :
                                      "bg-neutral-100 text-neutral-500"
                                }`}>
                                <span className={`w-1.5 h-1.5 rounded-full ${g.estado_pago === "pagado" ? "bg-emerald-500" :
                                    g.estado_pago === "vencido" ? "bg-red-500" :
                                      g.estado_pago === "proximo_vencer" ? "bg-amber-400" :
                                        "bg-neutral-300"
                                  }`} />
                                {g.estado_pago}
                              </span>
                            </td>
                            <td className="px-6 py-4">
                              <span className={`text-[10px] font-black ${g.dias_para_vencer !== null && g.dias_para_vencer <= 3 ? "text-red-500" : "text-neutral-400"}`}>
                                {g.dias_para_vencer !== null ? `${g.dias_para_vencer}d` : "-"}
                              </span>
                            </td>
                            <td className="px-6 py-4">
                              <div className="flex justify-end gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                                <button onClick={() => setModalPagoGasto(g)} className="p-2 bg-emerald-50 text-emerald-500 rounded-xl hover:bg-emerald-100 transition-all" title="Registrar pago">
                                  <MorphIcon icon={ICONO_BILLETE} size={13} strokeWidth={2.5} spring="snappy" reducedMotion="user" />
                                </button>
                                <button onClick={() => setModalGasto(g)} className="p-2 bg-neutral-100 text-neutral-400 rounded-xl hover:text-neutral-900 hover:bg-neutral-200 transition-all" title="Editar">
                                  <MorphIcon icon={ICONO_EDITAR} size={13} strokeWidth={2.5} spring="snappy" reducedMotion="user" />
                                </button>
                                <button onClick={async () => { try { await invoke("eliminar_gasto", { id: g.id }); recargarGastos(); } catch (e) { console.error("Error eliminando gasto:", e); } }} className="p-2 bg-neutral-100 text-neutral-400 rounded-xl hover:text-red-500 hover:bg-red-50 transition-all" title="Eliminar">
                                  <MorphIcon icon={ICONO_BORRAR} size={13} strokeWidth={2.5} spring="snappy" reducedMotion="user" />
                                </button>
                              </div>
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </div>
              ) : (
                <EmptyLargo icono={ICONO_CALCULADORA} mensaje="No hay gastos recurrentes" sub="Crea tu primer gasto para empezar a rastrearlos" />
              )}
            </div>
          )}

          {/* ═══ CORTES ═══════════════════════════════════════════════════ */}
          {seccion === "cortes" && (
            <div className="space-y-6">
              <div className="flex items-center gap-3">
                <div className="w-1.5 h-5 bg-neutral-950 rounded-full" />
                <h3 className="text-base sm:text-xl font-black text-neutral-950 uppercase tracking-tight">Cortes de Caja</h3>
                <span className="px-3 py-1 bg-neutral-950 text-white text-[9px] font-black rounded-lg">{cortes.length}</span>
              </div>

              {cortes.length > 0 ? (
                <div className="bg-white rounded-[2.5rem] border border-neutral-200 shadow-sm overflow-hidden">
                  <div className="max-h-[560px] overflow-y-auto custom-scrollbar">
                    <table className="w-full text-left border-collapse">
                      <thead className="sticky top-0 z-10">
                        <tr className="bg-neutral-950">
                          <th className="px-8 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Tipo</th>
                          <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Apertura</th>
                          <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Cajero</th>
                          <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Ventas</th>
                          <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Diferencia</th>
                          <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Estado</th>
                          <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest text-right">Detalle</th>
                        </tr>
                      </thead>
                      <tbody className="divide-y divide-neutral-50">
                        {cortes.map((c) => (
                          <tr key={c.id} className="group hover:bg-neutral-50/50 transition-all">
                            <td className="px-8 py-4">
                              <span className={`inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-[9px] font-black uppercase ${c.tipo_corte === "Z" ? "bg-neutral-950 text-white" : "bg-neutral-200 text-neutral-700"
                                }`}>
                                {c.tipo_corte}
                              </span>
                            </td>
                            <td className="px-6 py-4 text-xs font-bold text-neutral-900">{fechaRelativa(c.fecha_apertura)}</td>
                            <td className="px-6 py-4 text-xs font-bold text-neutral-700">{c.usuario_nombre ?? "-"}</td>
                            <td className="px-6 py-4 text-xs font-black text-neutral-950">{moneda(c.total_ventas)}</td>
                            <td className="px-6 py-4">
                              <span className={`text-xs font-black ${c.diferencia === 0 ? "text-neutral-400" : Math.abs(c.diferencia) > c.total_ventas * 0.05 ? "text-red-500" : "text-amber-500"}`}>
                                {moneda(c.diferencia)}
                              </span>
                            </td>
                            <td className="px-6 py-4">
                              <span className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-[9px] font-black uppercase ${c.estado === "cerrado" ? "bg-emerald-50 text-emerald-600" : "bg-amber-50 text-amber-600"
                                }`}>
                                <span className={`w-1.5 h-1.5 rounded-full ${c.estado === "cerrado" ? "bg-emerald-500" : "bg-amber-400 animate-pulse"}`} />
                                {c.estado}
                              </span>
                            </td>
                            <td className="px-6 py-4 text-right">
                              <button
                                onClick={() => setModalDetalleCorte(c)}
                                className="inline-flex items-center gap-2 px-4 py-2 rounded-xl bg-neutral-950 text-white text-[9px] font-black uppercase tracking-widest hover:bg-neutral-800 transition-all active:scale-[0.97] opacity-0 group-hover:opacity-100"
                              >
                                <MorphIcon icon={ICONO_OJO} size={11} strokeWidth={2.5} spring="snappy" reducedMotion="user" />
                                Ver
                              </button>
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </div>
              ) : (
                <EmptyLargo icono={ICONO_CAJA} mensaje="Sin cortes de caja" sub="Los cortes se registran desde el punto de venta" />
              )}
            </div>
          )}

          {/* ═══ ALERTAS ═══════════════════════════════════════════════════ */}
          {seccion === "alertas" && (
            <div className="space-y-6">
              <div className="flex items-center gap-3">
                <div className="w-1.5 h-5 bg-neutral-950 rounded-full" />
                <h3 className="text-base sm:text-xl font-black text-neutral-950 uppercase tracking-tight">Alertas Financieras</h3>
                {alertasNoLeidas > 0 && (
                  <span className="px-3 py-1 bg-red-500 text-white text-[9px] font-black rounded-lg shadow-lg shadow-red-500/20">{alertasNoLeidas} SIN LEER</span>
                )}
              </div>

              {alertas.length > 0 ? (
                <div className="space-y-3">
                  {alertas.map((a) => (
                    <div
                      key={a.id}
                      className={`flex items-start gap-4 p-5 rounded-[2rem] border transition-all ${a.leida ? "bg-white border-neutral-100 opacity-50" :
                          a.severidad === "rojo" ? "bg-red-50/60 border-red-200 shadow-sm shadow-red-100" :
                            a.severidad === "amarillo" ? "bg-amber-50/60 border-amber-200 shadow-sm shadow-amber-100" :
                              "bg-emerald-50/60 border-emerald-200 shadow-sm shadow-emerald-100"
                        }`}
                    >
                      <div className={`w-11 h-11 rounded-2xl flex items-center justify-center shrink-0 ${a.severidad === "rojo" ? "bg-red-500 text-white shadow-lg shadow-red-500/20" :
                          a.severidad === "amarillo" ? "bg-amber-400 text-white shadow-lg shadow-amber-400/20" :
                            "bg-emerald-500 text-white shadow-lg shadow-emerald-500/20"
                        }`}>
                        <MorphIcon
                          icon={a.tipo === "gasto_vencimiento" ? ICONO_CALCULADORA :
                            a.tipo === "corte_pendiente" ? ICONO_CAJA :
                              a.tipo === "diferencia_caja" ? ICONO_ALERTA :
                                ICONO_TRENDING}
                          size={18} strokeWidth={2.2} spring="smooth"
                        />
                      </div>
                      <div className="flex-1 min-w-0">
                        <p className="text-xs font-black text-neutral-950 uppercase">{a.titulo}</p>
                        <p className="text-[11px] text-neutral-500 font-bold mt-1">{a.mensaje}</p>
                        <p className="text-[9px] text-neutral-400 font-bold mt-1.5">{fechaRelativa(a.creada_en)}</p>
                      </div>
                      {!a.leida && (
                        <button
                          onClick={() => marcarLeida(a.id)}
                          className="p-2.5 bg-neutral-100 text-neutral-400 rounded-xl hover:text-emerald-500 hover:bg-emerald-50 transition-all shrink-0"
                          title="Marcar como leida"
                        >
                          <MorphIcon icon={ICONO_CHECK} size={14} strokeWidth={2.5} spring="snappy" reducedMotion="user" />
                        </button>
                      )}
                    </div>
                  ))}
                </div>
              ) : (
                <EmptyLargo icono={ICONO_CAMPANA} mensaje="No hay alertas financieras" sub="Todo esta en orden" />
              )}
            </div>
          )}

          {/* ═══ METRICAS ═════════════════════════════════════════════════ */}
          {seccion === "metricas" && (
            <div className="space-y-6">
              <div className="flex items-center gap-3">
                <div className="w-1.5 h-5 bg-neutral-950 rounded-full" />
                <h3 className="text-base sm:text-xl font-black text-neutral-950 uppercase tracking-tight">Metricicas Diarias</h3>
                <span className="px-3 py-1 bg-neutral-950 text-white text-[9px] font-black rounded-lg">{metricas.length} dias</span>
              </div>

              {metricas.length > 0 ? (
                <>
                  <SeccionGrafica titulo="Utilidad Neta Diaria" subtitulo="Tendencia del periodo">
                    <ResponsiveContainer width="100%" height={280}>
                      <AreaChart data={metricas}>
                        <CartesianGrid strokeDasharray="3 3" stroke="#f5f5f5" />
                        <XAxis dataKey="fecha" tick={{ fontSize: 10, fontWeight: 700 }} tickFormatter={(v) => v.slice(5)} />
                        <YAxis tick={{ fontSize: 10, fontWeight: 700 }} tickFormatter={(v) => `$${(v / 1000).toFixed(0)}k`} />
                        <Tooltip contentStyle={{ backgroundColor: "#0a0a0a", border: "1px solid #262626", borderRadius: "16px", color: "#fff", fontSize: 11 }} formatter={(v) => moneda(Number(v))} />
                        <Area type="monotone" dataKey="utilidad_neta" stroke="#22c55e" fill="#22c55e" fillOpacity={0.1} strokeWidth={2.5} />
                        <Line type="monotone" dataKey="ventas_totales" stroke="#0a0a0a" strokeWidth={1.5} strokeDasharray="5 5" dot={false} />
                      </AreaChart>
                    </ResponsiveContainer>
                  </SeccionGrafica>

                  <div className="bg-white rounded-[2.5rem] border border-neutral-200 shadow-sm overflow-hidden">
                    <div className="max-h-[560px] overflow-y-auto custom-scrollbar">
                      <table className="w-full text-left border-collapse">
                        <thead className="sticky top-0 z-10">
                          <tr className="bg-neutral-950">
                            <th className="px-8 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Fecha</th>
                            <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Ventas</th>
                            <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">COGS</th>
                            <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Util. Bruta</th>
                            <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Gastos</th>
                            <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Util. Neta</th>
                            <th className="px-6 py-5 text-[10px] font-black text-white/60 uppercase tracking-widest">Margen</th>
                          </tr>
                        </thead>
                        <tbody className="divide-y divide-neutral-50">
                          {metricas.map((m) => (
                            <tr key={m.fecha} className="group hover:bg-neutral-50/50 transition-all">
                              <td className="px-8 py-4 text-xs font-bold text-neutral-900">{m.fecha}</td>
                              <td className="px-6 py-4 text-xs font-black text-neutral-950">{moneda(m.ventas_totales)}</td>
                              <td className="px-6 py-4 text-xs font-bold text-neutral-500">{moneda(m.costo_ventas)}</td>
                              <td className="px-6 py-4 text-xs font-black text-neutral-900">{moneda(m.utilidad_bruta)}</td>
                              <td className="px-6 py-4 text-xs font-black text-red-500">{moneda(m.gastos_operativos)}</td>
                              <td className="px-6 py-4">
                                <span className={`text-xs font-black ${m.utilidad_neta >= 0 ? "text-emerald-600" : "text-red-500"}`}>
                                  {moneda(m.utilidad_neta)}
                                </span>
                              </td>
                              <td className="px-6 py-4">
                                <span className={`text-[10px] font-black px-2.5 py-1 rounded-lg ${m.margen_neto_pct >= 0 ? "bg-emerald-50 text-emerald-600" : "bg-red-50 text-red-500"
                                  }`}>
                                  {porcentaje(m.margen_neto_pct)}
                                </span>
                              </td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                  </div>
                </>
              ) : (
                <EmptyLargo icono={ICONO_TRENDING} mensaje="Sin metricicas" sub="Selecciona un rango con actividad" />
              )}
            </div>
          )}
        </>
      )}

      {/* MODALES */}
      {(modalNuevoGasto || modalGasto) && (
        <ModalGasto gasto={modalGasto} onCerrar={() => { setModalNuevoGasto(false); setModalGasto(undefined); }} onGuardado={() => { setModalNuevoGasto(false); setModalGasto(undefined); recargarGastos(); }} />
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

// ── SUB-COMPONENTES ──────────────────────────────────────────────────────

function KPI({ icono, label, valor, color }: { icono: typeof ICONO_DOLAR; label: string; valor: string; color: "neutral" | "verde" | "rojo" }) {
  const [hover, setHover] = useState(false);
  const bg = color === "verde" ? "bg-emerald-500" : color === "rojo" ? "bg-red-500" : "bg-neutral-950";
  const texto = color === "verde" ? "text-emerald-600" : color === "rojo" ? "text-red-500" : "text-neutral-950";

  return (
    <div
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      className="rounded-[2rem] p-5 sm:p-6 border bg-white border-neutral-200 hover:shadow-lg transition-all group"
    >
      <div className={`w-12 h-12 rounded-2xl flex items-center justify-center ${bg} shadow-lg`}>
        <IconoMorph
          icono={icono}
          iconoHover={ICONO_CHECK}
          size={18}
          strokeWidth={2}
          hover={hover}
          className="text-white"
        />
      </div>
      <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest mt-4">{label}</p>
      <p className={`text-2xl font-black mt-1.5 ${texto}`}>{valor}</p>
    </div>
  );
}

function SeccionGrafica({ titulo, subtitulo, accion, children }: { titulo: string; subtitulo: string; accion?: React.ReactNode; children: React.ReactNode }) {
  return (
    <div className="bg-white rounded-[2.5rem] border border-neutral-200 shadow-sm overflow-hidden">
      <div className="flex items-center justify-between px-8 pt-6 pb-2">
        <div>
          <h4 className="text-sm font-black text-neutral-950 uppercase tracking-tight">{titulo}</h4>
          <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest mt-0.5">{subtitulo}</p>
        </div>
        {accion}
      </div>
      <div className="px-6 pb-6 pt-2">
        {children}
      </div>
    </div>
  );
}

function EmptyGrafica({ mensaje }: { mensaje: string }) {
  return (
    <div className="py-14 text-center">
      <div className="w-12 h-12 mx-auto bg-neutral-100 rounded-2xl flex items-center justify-center mb-3">
        <MorphIcon icon={ICONO_GRAFICA} size={20} strokeWidth={1.8} spring="smooth" className="text-neutral-300" />
      </div>
      <p className="text-[10px] font-black text-neutral-300 uppercase tracking-widest">{mensaje}</p>
    </div>
  );
}

function EmptyLargo({ icono, mensaje, sub }: { icono: typeof ICONO_DOLAR; mensaje: string; sub: string }) {
  return (
    <div className="bg-white rounded-[2.5rem] border border-neutral-200 shadow-sm py-20 text-center">
      <div className="mx-auto w-16 h-16 bg-neutral-950 rounded-2xl flex items-center justify-center shadow-lg">
        <MorphIcon icon={icono} size={28} strokeWidth={1.8} spring="smooth" className="text-white" />
      </div>
      <p className="text-[10px] font-black text-neutral-300 uppercase tracking-[0.2em] mt-5">{mensaje}</p>
      <p className="text-[9px] font-bold text-neutral-400 mt-1.5">{sub}</p>
    </div>
  );
}
