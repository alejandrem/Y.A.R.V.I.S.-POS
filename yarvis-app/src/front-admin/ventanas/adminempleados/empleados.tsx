import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import ModalEmpleados from "./modalEmpleados";
import ModalTurnos from "./modalTurnos";
import ModalMetas from "./modalMetas";

interface EmpleadoProfile {
  id: number;
  nombre: string;
  estado: string;
  turno: string;
  horario_inicio: string;
  horario_fin: string;
  salario_semanal: number;
  salario_diario: number;
  dias_semana: number;
  meta_mensual: number;
  bono: number;
  registrado_en: string | null;
  ultimo_login: string | null;
}

interface EmpleadoVentas {
  empleado_id: number;
  nombre: string;
  total_ventas: number;
  ventas_canceladas: number;
  total_canceladas_count: number;
  ventas_con_descuento: number;
  ticket_count: number;
}

interface EmpleadoResumen {
  empleados_activos: number;
  ventas_totales: number;
  costo_nomina: number;
  roi_neto: number;
}

interface CorteEmpleado {
  id: number;
  fecha_apertura: string | null;
  fecha_cierre: string | null;
  monto_inicial: number;
  total_ventas: number;
  estado: string;
}

interface AdminEmpleadosProps {
  activeTab: string;
}

const AdminEmpleados = ({ activeTab }: AdminEmpleadosProps) => {
  const [empleados, setEmpleados] = useState<EmpleadoProfile[]>([]);
  const [resumen, setResumen] = useState<EmpleadoResumen>({
    empleados_activos: 0,
    ventas_totales: 0,
    costo_nomina: 0,
    roi_neto: 0,
  });
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [ventasDetalle, setVentasDetalle] = useState<EmpleadoVentas | null>(null);
  const [cortes, setCortes] = useState<CorteEmpleado[]>([]);
  const [showModal, setShowModal] = useState(false);
  const [showTurnosModal, setShowTurnosModal] = useState(false);
  const [showModalMetas, setShowModalMetas] = useState(false);

  useEffect(() => {
    if (activeTab === "empleados") loadData();
  }, [activeTab]);

  const loadData = async () => {
    try {
      const [emp, res] = await Promise.all([
        invoke<EmpleadoProfile[]>("get_empleados"),
        invoke<EmpleadoResumen>("get_resumen_empleados"),
      ]);
      setEmpleados(emp);
      setResumen(res);
    } catch (error) {
      console.error("Error al cargar empleados:", error);
    }
  };

  const loadDetalle = async (id: number) => {
    try {
      const [ventas, cortesData] = await Promise.all([
        invoke<EmpleadoVentas>("get_empleado_ventas", { empleadoId: id }),
        invoke<CorteEmpleado[]>("get_cortes_empleado", { empleadoId: id }),
      ]);
      setVentasDetalle(ventas);
      setCortes(cortesData);
      setSelectedId(id);
    } catch (error) {
      console.error("Error al cargar detalle:", error);
    }
  };

  const detectTurno = (horarioInicio: string) => {
    if (!horarioInicio || horarioInicio === "00:00") return "";
    const h = parseInt(horarioInicio.split(":")[0], 10);
    if (h >= 5 && h < 12) return "Matutino";
    if (h >= 12) return "Vespertino";
    return "Nocturno";
  };

  const isInShift = (emp: EmpleadoProfile) => {
    if (!emp.horario_inicio || !emp.horario_fin || emp.horario_inicio === "00:00") return false;
    const now = new Date();
    const currentMins = now.getHours() * 60 + now.getMinutes();
    const [sh, sm] = emp.horario_inicio.split(":").map(Number);
    const [eh, em] = emp.horario_fin.split(":").map(Number);
    const startMins = sh * 60 + sm;
    const endMins = eh * 60 + em;
    if (startMins <= endMins) {
      return currentMins >= startMins && currentMins <= endMins;
    }
    return currentMins >= startMins || currentMins <= endMins;
  };

  const estadoDot = (emp: EmpleadoProfile) => {
    if (isInShift(emp)) return "bg-green-500";
    if (emp.estado === "descanso") return "bg-yellow-500";
    return "bg-neutral-300";
  };

  const estadoLabel = (emp: EmpleadoProfile) => {
    if (isInShift(emp)) return "En Turno";
    if (emp.estado === "descanso") return "Descanso";
    return "Fuera de Turno";
  };

  const formatMoney = (v: number) =>
    `$${v.toLocaleString("es-MX", { minimumFractionDigits: 2 })}`;

  const formatTime12 = (t: string) => {
    if (!t || t === "00:00") return "";
    const [h, m] = t.split(":").map(Number);
    const ampm = h >= 12 ? "PM" : "AM";
    const h12 = h % 12 || 12;
    return `${String(h12).padStart(2, "0")}:${String(m).padStart(2, "0")}${ampm}`;
  };

  const formatEntrada = (emp: EmpleadoProfile) => {
    const hasHorario = emp.horario_inicio && emp.horario_fin && emp.horario_inicio !== "00:00";
    const horario = hasHorario ? `${formatTime12(emp.horario_inicio)}-${formatTime12(emp.horario_fin)}` : "";
    const login = emp.ultimo_login ? formatShortDate(emp.ultimo_login) : "";
    if (hasHorario && login) return `${horario} / ${login}`;
    if (hasHorario) return horario;
    return "Sin horario";
  };

  const formatShortDate = (d: string | null) => {
    if (!d) return "";
    const date = new Date(d);
    const hours = date.getHours();
    const ampm = hours >= 12 ? "PM" : "AM";
    const h12 = hours % 12 || 12;
    const mins = String(date.getMinutes()).padStart(2, "0");
    return `${String(h12).padStart(2, "0")}:${mins} ${ampm}`;
  };

  const turnoDetectado = (emp: EmpleadoProfile) => {
    const detected = detectTurno(emp.horario_inicio);
    return detected || "Sin turno";
  };

  const formatDate = (d: string | null) => {
    if (!d) return "—";
    const date = new Date(d);
    const day = String(date.getDate()).padStart(2, "0");
    const month = String(date.getMonth() + 1).padStart(2, "0");
    const hours = date.getHours();
    const ampm = hours >= 12 ? "PM" : "AM";
    const h12 = hours % 12 || 12;
    const mins = String(date.getMinutes()).padStart(2, "0");
    return `${day}/${month} - ${String(h12).padStart(2, "0")}:${mins} ${ampm}`;
  };

  const selectedEmp = empleados.find((e) => e.id === selectedId);

  return (
    <div className="animate-in fade-in slide-in-from-bottom-2 duration-500 space-y-6 w-full">

      {/* ── HEADER ── */}
      <header className="flex items-center gap-2 border-b border-neutral-100 pb-6">
        <h2 className="text-3xl font-black text-neutral-900 uppercase tracking-tight">
          Empleados
        </h2>
        <svg
          xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24"
          fill="none" stroke="currentColor" strokeWidth="3" className="text-neutral-300"
        >
          <path d="m9 18 6-6-6-6" />
        </svg>
        <p className="text-[10px] font-black text-neutral-400 uppercase tracking-[0.3em]">
          Panel de Gestión y Rendimiento
        </p>
      </header>

      {/* ── RESUMEN GLOBAL ── */}
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 sm:gap-6">
        {[
          { label: "Emp. Activos", value: `${resumen.empleados_activos}`, info: "Personal activo", color: "bg-neutral-900 text-white shadow-2xl shadow-neutral-200" },
          { label: "Ventas Totales", value: formatMoney(resumen.ventas_totales), info: "Acumulado", color: "bg-green-50 border-green-100" },
          { label: "Costo de Nómina", value: formatMoney(resumen.costo_nomina), info: "Gasto fijo", color: "bg-red-50 border-red-100" },
          { label: "ROI Neto", value: `${resumen.roi_neto >= 0 ? '+' : ''}${formatMoney(resumen.roi_neto)}`, info: resumen.roi_neto >= 0 ? "Rentable" : "En pérdida", color: resumen.roi_neto >= 0 ? "bg-neutral-50" : "bg-red-50 border-red-100" },
        ].map((stat, i) => (
          <div key={i} className={`${stat.color} p-3 sm:p-6 rounded-2xl sm:rounded-[2rem] border border-neutral-100 relative overflow-hidden group hover:scale-[1.02] transition-all duration-300`}>
            <p className={`text-[8px] sm:text-[10px] font-black uppercase tracking-widest mb-1 sm:mb-2 ${stat.color.includes('white') || stat.color.includes('red-50') ? 'text-neutral-400' : 'text-neutral-400'}`}>{stat.label}</p>
            <h3 className={`text-lg sm:text-2xl font-black ${stat.color.includes('text-white') ? 'text-white' : stat.label === 'ROI Neto' && resumen.roi_neto < 0 ? 'text-red-600' : 'text-neutral-900'}`}>{stat.value}</h3>
            <span className={`absolute top-3 sm:top-6 right-3 sm:right-6 text-[7px] sm:text-[8px] font-black px-1.5 sm:px-2 py-0.5 sm:py-1 rounded-lg uppercase ${stat.color.includes('white') ? 'bg-white/10 text-neutral-400' : 'bg-neutral-100 text-neutral-400'}`}>
              {stat.info}
            </span>
          </div>
        ))}
      </div>

      {/* ── LISTA DE PERSONAL ── */}
      <div className="space-y-3">
        <p className="text-[10px] font-black text-neutral-400 uppercase tracking-widest px-2">
          Lista de Personal (Activos & Recientes)
        </p>
        {empleados.length === 0 ? (
          <div className="bg-neutral-50 rounded-[2rem] border border-neutral-100 p-14 text-center">
            <p className="text-[10px] font-black text-neutral-300 uppercase tracking-widest">
              Sin empleados registrados
            </p>
          </div>
        ) : (
          <div className="space-y-4">
            {empleados.map((emp) => (
              <div
                key={emp.id}
                onClick={() => loadDetalle(emp.id)}
                className={`grid grid-cols-2 sm:grid-cols-3 md:grid-cols-5 gap-3 cursor-pointer transition-all duration-200 ${
                  selectedId === emp.id
                    ? "ring-2 ring-neutral-900/10 rounded-[2rem]"
                    : ""
                }`}
              >
                {/* NOMBRE */}
                <div className={`rounded-xl sm:rounded-[1.5rem] p-3 sm:p-6 text-center transition-all hover:scale-[1.02] ${
                  selectedId === emp.id
                    ? "bg-neutral-900 text-white shadow-xl"
                    : "bg-white border border-neutral-200 shadow-sm"
                }`}>
                  <p className={`text-[9px] font-black uppercase tracking-widest mb-2 ${selectedId === emp.id ? 'text-neutral-400' : 'text-neutral-400'}`}>
                    Empleado
                  </p>
                  <p className={`text-lg font-black uppercase ${selectedId === emp.id ? 'text-white' : 'text-neutral-900'}`}>
                    {emp.nombre}
                  </p>
                </div>

                {/* ESTADO */}
                <div className="bg-white rounded-xl sm:rounded-[1.5rem] border border-neutral-200 p-3 sm:p-6 text-center shadow-sm hover:scale-[1.02] transition-all">
                  <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest mb-2">
                    Estado
                  </p>
                  <div className="flex items-center justify-center gap-2 mt-1">
                    <span className={`w-3 h-3 rounded-full ${estadoDot(emp)}`}></span>
                    <span className="text-sm font-black text-neutral-700 uppercase">
                      {estadoLabel(emp)}
                    </span>
                  </div>
                </div>

                {/* ENTRADA / REGISTRO */}
                <div className="bg-white rounded-xl sm:rounded-[1.5rem] border border-neutral-200 p-3 sm:p-6 text-center shadow-sm hover:scale-[1.02] transition-all">
                  <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest mb-2">
                    Entrada / Registro
                  </p>
                  <p className="text-sm font-black text-neutral-700 mt-1">
                    {formatEntrada(emp)}
                  </p>
                </div>

                {/* TURNO */}
                <div className="bg-white rounded-xl sm:rounded-[1.5rem] border border-neutral-200 p-3 sm:p-6 text-center shadow-sm hover:scale-[1.02] transition-all">
                  <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest mb-2">
                    Turno
                  </p>
                  <span className={`inline-block text-xs font-black uppercase px-4 py-1.5 rounded-xl mt-1 ${
                    turnoDetectado(emp) === "Sin turno"
                      ? "text-neutral-300 bg-neutral-50"
                      : "text-neutral-600 bg-neutral-100"
                  }`}>
                    {turnoDetectado(emp)}
                  </span>
                </div>

                {/* CAJA INICIAL */}
                <div className="bg-white rounded-xl sm:rounded-[1.5rem] border border-neutral-200 p-3 sm:p-6 text-center shadow-sm hover:scale-[1.02] transition-all">
                  <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest mb-2">
                    Caja Inicial
                  </p>
                  <p className="text-lg font-black text-neutral-900 mt-1">
                    {formatMoney(emp.salario_semanal)}
                  </p>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* ── BOTTOM: ACCIONES + DETALLE ── */}
      <div className="flex gap-6">

        {/* IZQ: ACCIONES DE GESTIÓN */}
        <div className="w-[280px] shrink-0">
          <div className="bg-white rounded-[2rem] border border-neutral-200 p-5 space-y-3">
            <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest mb-3">
              Acciones de Gestión
            </p>

            <button
              onClick={() => setShowModal(true)}
              className="w-full p-4 rounded-2xl border-2 border-neutral-900 bg-neutral-900 text-white transition-all text-left hover:shadow-xl"
            >
              <div className="flex items-center gap-3">
                <div className="w-9 h-9 bg-white/10 text-white rounded-xl flex items-center justify-center">
                  <svg
                    xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24"
                    fill="none" stroke="currentColor" strokeWidth="3"
                  >
                    <path d="M5 12h14" />
                    <path d="M12 5v14" />
                  </svg>
                </div>
                <p className="text-[10px] font-black uppercase tracking-widest">
                  Registrar Nuevo Empleado
                </p>
              </div>
            </button>

            <button
              onClick={() => setShowTurnosModal(true)}
              className="w-full p-4 rounded-2xl border-2 border-neutral-900 bg-neutral-900 text-white transition-all text-left hover:shadow-xl"
            >
              <div className="flex items-center gap-3">
                <div className="w-9 h-9 bg-white/10 text-white rounded-xl flex items-center justify-center text-sm">
                  🕒
                </div>
                <p className="text-[10px] font-black uppercase tracking-widest">
                  Configurar Turnos / Horarios
                </p>
              </div>
            </button>

            <button
              onClick={() => setShowModalMetas(true)}
              className="w-full p-4 rounded-2xl border border-neutral-100 text-left hover:bg-neutral-50 transition-all cursor-pointer"
            >
              <div className="flex items-center gap-3">
                <div className="w-9 h-9 bg-neutral-100 text-neutral-700 rounded-xl flex items-center justify-center text-sm">
                  🎯
                </div>
                <p className="text-[10px] font-black text-neutral-700 uppercase tracking-widest">
                  Definir Metas y Salarios
                </p>
              </div>
            </button>

            <button className="w-full p-4 rounded-2xl border border-neutral-100 text-left opacity-50 cursor-not-allowed transition-all">
              <div className="flex items-center gap-3">
                <div className="w-9 h-9 bg-neutral-50 text-neutral-300 rounded-xl flex items-center justify-center text-sm">
                  📊
                </div>
                <p className="text-[10px] font-black text-neutral-400 uppercase tracking-widest">
                  Reportes de Nómina & ROI
                </p>
              </div>
            </button>
          </div>
        </div>

        {/* DER: VISTA DETALLADA */}
        <div className="flex-1 min-w-0">
          {selectedId && selectedEmp && ventasDetalle ? (
            <div className="bg-white rounded-[2rem] border border-neutral-200 p-6 space-y-5 animate-in slide-in-from-right-2 duration-300">
              <div className="flex items-center justify-between border-b border-neutral-100 pb-4">
                <div>
                  <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest">
                    Vista Detallada: {selectedEmp.nombre}
                  </p>
                </div>
                <button
                  onClick={() => {
                    setSelectedId(null);
                    setVentasDetalle(null);
                    setCortes([]);
                  }}
                  className="p-2 text-neutral-300 hover:text-neutral-600 transition-colors"
                >
                  <svg
                    xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24"
                    fill="none" stroke="currentColor" strokeWidth="3"
                  >
                    <path d="M18 6 6 18" />
                    <path d="m6 6 12 12" />
                  </svg>
                </button>
              </div>

              {/* DESGLOSE DE VENTAS */}
              <div className="space-y-3">
                <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest">
                  📈 Desglose de Ventas y Operaciones
                </p>
                <div className="pl-2 space-y-2">
                  <div className="flex items-center gap-2">
                    <span className="text-neutral-300 text-[10px]">├─</span>
                    <span className="text-[10px] font-bold text-neutral-500 w-44">
                      Total Vendido:
                    </span>
                    <span className="text-[11px] font-black text-green-600">
                      {formatMoney(ventasDetalle.total_ventas)}
                    </span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="text-neutral-300 text-[10px]">├─</span>
                    <span className="text-[10px] font-bold text-neutral-500 w-44">
                      Ventas Canceladas:
                    </span>
                    <span className="text-[11px] font-black text-red-500">
                      {formatMoney(ventasDetalle.ventas_canceladas)} (
                      {ventasDetalle.total_canceladas_count} tks)
                    </span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="text-neutral-300 text-[10px]">└─</span>
                    <span className="text-[10px] font-bold text-neutral-500 w-44">
                      Ventas con Descuento:
                    </span>
                    <span className="text-[11px] font-black text-yellow-600">
                      {formatMoney(ventasDetalle.ventas_con_descuento)}
                    </span>
                  </div>
                </div>
              </div>

              {/* METAS Y BONOS */}
              <div className="space-y-3">
                <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest">
                  🎯 Metas y Acceso a Bonos
                </p>
                <div className="pl-2 space-y-2">
                  <div className="flex items-center gap-2">
                    <span className="text-neutral-300 text-[10px]">├─</span>
                    <span className="text-[10px] font-bold text-neutral-500 w-44">
                      Meta Mensual:
                    </span>
                    <span className="text-[11px] font-black text-neutral-900">
                      {formatMoney(selectedEmp.meta_mensual)}
                    </span>
                  </div>
                  {selectedEmp.meta_mensual > 0 && (
                    <div className="flex items-center gap-2">
                      <span className="text-neutral-300 text-[10px]">├─</span>
                      <span className="text-[10px] font-bold text-neutral-500 w-44">
                        Progreso:
                      </span>
                      <div className="flex items-center gap-2 flex-1">
                        <div className="flex-1 h-1.5 bg-neutral-100 rounded-full overflow-hidden max-w-[200px]">
                          <div
                            className={`h-full rounded-full ${
                              ventasDetalle.total_ventas >= selectedEmp.meta_mensual
                                ? "bg-green-500"
                                : "bg-neutral-900"
                            }`}
                            style={{
                              width: `${Math.min(
                                100,
                                (ventasDetalle.total_ventas /
                                  selectedEmp.meta_mensual) *
                                  100
                              )}%`,
                            }}
                          ></div>
                        </div>
                        <span className="text-[10px] font-black text-neutral-600">
                          {Math.min(
                            100,
                            Math.round(
                              (ventasDetalle.total_ventas /
                                selectedEmp.meta_mensual) *
                                100
                            )
                          )}
                          %
                        </span>
                      </div>
                    </div>
                  )}
                  <div className="flex items-center gap-2">
                    <span className="text-neutral-300 text-[10px]">└─</span>
                    <span className="text-[10px] font-bold text-neutral-500 w-44">
                      Bono Prometido:
                    </span>
                    <span className="text-[11px] font-black text-blue-600">
                      {formatMoney(selectedEmp.bono)}
                    </span>
                  </div>
                </div>
              </div>

              {/* ROI */}
              {(() => {
                const neto = ventasDetalle.total_ventas - selectedEmp.salario_semanal;
                const isNegative = neto < 0;
                return (
              <div className={`rounded-2xl p-5 space-y-3 transition-all ${isNegative ? 'bg-red-950 border-2 border-red-800' : 'bg-neutral-900'}`}>
                <p className={`text-[9px] font-black uppercase tracking-widest ${isNegative ? 'text-red-400' : 'text-neutral-500'}`}>
                  💰 Relación Costo / Beneficio (ROI)
                  {isNegative && <span className="ml-2 text-red-500">⚠ PÉRDIDA DETECTADA</span>}
                </p>
                <div className="pl-2 space-y-2">
                  <div className="flex items-center gap-2">
                    <span className="text-neutral-600 text-[10px]">├─</span>
                    <span className="text-[10px] font-bold text-neutral-400 w-44">Salario Semanal Fijo:</span>
                    <span className="text-[11px] font-black text-white">{formatMoney(selectedEmp.salario_semanal)}</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="text-neutral-600 text-[10px]">├─</span>
                    <span className="text-[10px] font-bold text-neutral-400 w-44">Costo Total Empleado:</span>
                    <span className="text-[11px] font-black text-white">{formatMoney(selectedEmp.salario_semanal)}</span>
                  </div>
                  <div className={`h-px my-1 ${isNegative ? 'bg-red-800' : 'bg-white/10'}`}></div>
                  <div className="flex items-center gap-2">
                    <span className={`text-[10px] ${isNegative ? 'text-red-600' : 'text-neutral-600'}`}>└─</span>
                    <span className={`text-[10px] font-bold w-44 ${isNegative ? 'text-red-300' : 'text-neutral-400'}`}>RENDIMIENTO NETO:</span>
                    <span className={`text-[11px] font-black ${isNegative ? 'text-red-400 bg-red-500/20 px-2 py-0.5 rounded-lg border border-red-700' : 'text-green-400'}`}>
                      {isNegative ? '' : '+'}{formatMoney(neto)}
                    </span>
                  </div>
                </div>
              </div>
              );
              })()}

              {/* CORTES DE CAJA */}
              {cortes.length > 0 && (
                <div className="space-y-3">
                  <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest">
                    🧾 Cortes de Caja Recientes
                  </p>
                  <div className="max-h-32 overflow-y-auto custom-scrollbar space-y-2 pl-2">
                    {cortes.map((c) => (
                      <div
                        key={c.id}
                        className="flex items-center justify-between p-3 bg-neutral-50 rounded-xl border border-neutral-100"
                      >
                        <div>
                          <p className="text-[9px] font-bold text-neutral-600">
                            {formatDate(c.fecha_apertura)}
                          </p>
                          <p
                            className={`text-[8px] font-black uppercase ${
                              c.estado === "cerrado"
                                ? "text-green-500"
                                : "text-yellow-500"
                            }`}
                          >
                            {c.estado}
                          </p>
                        </div>
                        <div className="text-right">
                          <p className="text-[10px] font-black text-neutral-900">
                            {formatMoney(c.total_ventas)}
                          </p>
                          <p className="text-[8px] text-neutral-400 font-bold">
                            Inicial: {formatMoney(c.monto_inicial)}
                          </p>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          ) : (
            <div className="bg-neutral-50 rounded-[2rem] border border-neutral-100 p-10 h-full flex items-center justify-center min-h-[300px]">
              <div className="text-center">
                <p className="text-[10px] font-black text-neutral-300 uppercase tracking-widest">
                  Selecciona un empleado para ver su detalle
                </p>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* MODAL AGREGAR EMPLEADO */}
      {showModal && (
        <ModalEmpleados onClose={() => setShowModal(false)} onSaved={loadData} />
      )}

      {/* MODAL CONFIGURAR TURNOS */}
      {showTurnosModal && (
        <ModalTurnos
          empleados={empleados}
          onClose={() => setShowTurnosModal(false)}
          onSaved={loadData}
        />
      )}

      {/* MODAL DEFINIR METAS Y SALARIOS */}
      {showModalMetas && (
        <ModalMetas
          empleados={empleados}
          onClose={() => setShowModalMetas(false)}
          onSaved={loadData}
        />
      )}
    </div>
  );
};

export default AdminEmpleados;
