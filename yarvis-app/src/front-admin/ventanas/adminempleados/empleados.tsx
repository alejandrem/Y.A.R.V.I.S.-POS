// Panel de administración de empleados.
// Piel reconstruida con el ancho de inventario general (max-w-6xl),
// botones gorditos, minimalista blanco/negro con rojo/verde discretos y
// morphicons animados. La lógica (estado, cargas, handlers) se conserva.
import { useState, useEffect, type ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { MorphIcon, type IconInput } from "morphicons/react";
import ModalEmpleados from "./modalEmpleados";
import ModalTurnos from "./modalTurnos";
import ModalMetas from "./modalMetas";
import {
  ICONO_USUARIOS,
  ICONO_USUARIO,
  ICONO_MAS,
  ICONO_RELOJ,
  ICONO_TARGET,
  ICONO_DOLAR,
  ICONO_TRENDING,
  ICONO_CALENDARIO,
  ICONO_PREMIO,
  ICONO_FLECHA,
  ICONO_CERRAR,
  ICONO_CHECK,
  ICONO_ALERTA,
  BotonAnimado,
  IconoMorph,
} from "../../../components/ui";

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
  if (isInShift(emp)) return "En turno";
  if (emp.estado === "descanso") return "Descanso";
  return "Fuera de turno";
};

const estadoVisual: Record<string, { dot: string; texto: string; fondo: string }> = {
  "En turno": { dot: "bg-emerald-500", texto: "text-emerald-600", fondo: "bg-emerald-50" },
  Descanso: { dot: "bg-amber-400", texto: "text-amber-600", fondo: "bg-amber-50" },
  "Fuera de turno": { dot: "bg-neutral-300", texto: "text-neutral-400", fondo: "bg-neutral-50" },
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

const formatShortDate = (d: string | null) => {
  if (!d) return "";
  const date = new Date(d);
  const hours = date.getHours();
  const ampm = hours >= 12 ? "PM" : "AM";
  const h12 = hours % 12 || 12;
  const mins = String(date.getMinutes()).padStart(2, "0");
  return `${String(h12).padStart(2, "0")}:${mins} ${ampm}`;
};

const formatEntrada = (emp: EmpleadoProfile) => {
  const hasHorario = emp.horario_inicio && emp.horario_fin && emp.horario_inicio !== "00:00";
  const horario = hasHorario ? `${formatTime12(emp.horario_inicio)}-${formatTime12(emp.horario_fin)}` : "";
  const login = emp.ultimo_login ? formatShortDate(emp.ultimo_login) : "";
  if (hasHorario && login) return `${horario} / ${login}`;
  if (hasHorario) return horario;
  return "Sin horario";
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

// Tarjeta de resumen: el hover de TODO el bloque activa el loop del morphicon.
interface TarjetaResumenProps {
  icono: IconInput;
  iconoHover: IconInput;
  label: string;
  valor: string;
  oscura?: boolean;
  children?: ReactNode;
}

const TarjetaResumen = ({ icono, iconoHover, label, valor, oscura = false, children }: TarjetaResumenProps) => {
  const [hover, setHover] = useState(false);
  return (
    <div
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      className={`rounded-[2rem] p-5 sm:p-6 border transition-colors duration-200 ${
        oscura ? "bg-neutral-950 text-neutral-50 border-neutral-950" : "bg-white text-neutral-900 border-neutral-200"
      }`}
    >
      <div
        className={`w-10 h-10 rounded-xl flex items-center justify-center mb-3 transition-colors duration-200 ${
          oscura ? "bg-white/10 text-neutral-50" : "bg-neutral-950 text-neutral-50"
        }`}
      >
        <IconoMorph icono={icono} iconoHover={iconoHover} size={16} strokeWidth={2.2} hover={hover} />
      </div>
      <p className={`text-[9px] font-black uppercase tracking-widest ${oscura ? "opacity-70" : "text-neutral-400"}`}>
        {label}
      </p>
      <p className="text-2xl font-black mt-1">{valor}</p>
      {children}
    </div>
  );
};

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
  const [loading, setLoading] = useState(false);
  const [recargado, setRecargado] = useState(false);

  useEffect(() => {
    if (activeTab === "empleados") loadData();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [activeTab]);

  const loadData = async () => {
    setLoading(true);
    try {
      const [emp, res] = await Promise.all([
        invoke<EmpleadoProfile[]>("get_empleados"),
        invoke<EmpleadoResumen>("get_resumen_empleados"),
      ]);
      setEmpleados(emp);
      setResumen(res);
    } catch (error) {
      console.error("Error al cargar empleados:", error);
    } finally {
      setLoading(false);
    }
  };

  const recargar = async () => {
    setRecargado(false);
    await loadData();
    setRecargado(true);
    window.setTimeout(() => setRecargado(false), 1600);
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

  const selectedEmp = empleados.find((e) => e.id === selectedId) || null;

  const cerrarDetalle = () => {
    setSelectedId(null);
    setVentasDetalle(null);
    setCortes([]);
  };

  return (
    <div className="max-w-6xl mx-auto space-y-12">
      {/* HEADER */}
      <header className="flex justify-between items-end flex-wrap gap-6">
        <div>
          <div className="flex items-center gap-2">
            <MorphIcon icon={ICONO_USUARIOS} size={22} strokeWidth={2} spring="smooth" className="text-neutral-950" />
            <h2 className="text-3xl font-black text-neutral-900 uppercase tracking-tight">Empleados</h2>
          </div>
          <p className="text-[10px] font-black text-neutral-400 uppercase tracking-[0.3em] mt-1">Gestión de Personal y Rendimiento</p>
        </div>
        <div className="flex gap-3 flex-wrap">
          <BotonAnimado
            icono={ICONO_MAS}
            iconoHover={ICONO_USUARIOS}
            onClick={() => setShowModal(true)}
            className="bg-neutral-950 text-neutral-50 hover:bg-neutral-800 shadow-xl shadow-neutral-200"
          >
            Registrar
          </BotonAnimado>
          <BotonAnimado
            icono={ICONO_RELOJ}
            iconoHover={ICONO_TARGET}
            onClick={() => setShowTurnosModal(true)}
            className="bg-white text-neutral-900 border-2 border-neutral-950 hover:bg-neutral-950 hover:text-neutral-50"
          >
            Turnos
          </BotonAnimado>
          <BotonAnimado
            icono={ICONO_TARGET}
            iconoHover={ICONO_PREMIO}
            onClick={() => setShowModalMetas(true)}
            className="bg-white text-neutral-900 border-2 border-neutral-950 hover:bg-neutral-950 hover:text-neutral-50"
          >
            Metas y sueldos
          </BotonAnimado>
        </div>
      </header>

      {/* RESUMEN GLOBAL */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4 sm:gap-6">
        <TarjetaResumen icono={ICONO_USUARIOS} iconoHover={ICONO_CHECK} label="Activos" valor={String(resumen.empleados_activos)} />
        <TarjetaResumen icono={ICONO_TRENDING} iconoHover={ICONO_DOLAR} label="Ventas totales" valor={formatMoney(resumen.ventas_totales)} />
        <TarjetaResumen icono={ICONO_DOLAR} iconoHover={ICONO_PREMIO} label="Costo nómina" valor={formatMoney(resumen.costo_nomina)} />
        <TarjetaResumen
          icono={ICONO_PREMIO}
          iconoHover={resumen.roi_neto < 0 ? ICONO_ALERTA : ICONO_TRENDING}
          label="ROI neto"
          valor={formatMoney(resumen.roi_neto)}
          oscura
        >
          {resumen.roi_neto < 0 && (
            <p className="text-[8px] font-black text-red-200 uppercase tracking-widest mt-1 inline-flex items-center gap-1">
              <MorphIcon icon={ICONO_ALERTA} size={11} strokeWidth={2.5} spring="snappy" className="text-red-300" />
              Pérdida
            </p>
          )}
        </TarjetaResumen>
      </div>

      {/* LISTA DE PERSONAL */}
      <div className="bg-white rounded-[2.5rem] border border-neutral-200 shadow-sm overflow-hidden">
        <div className="flex items-center justify-between px-6 sm:px-10 pt-6 sm:pt-8 pb-2">
          <div>
            <h3 className="text-base sm:text-xl font-black text-neutral-900 uppercase tracking-tight">Personal</h3>
            <p className="text-[9px] text-neutral-400 uppercase font-black tracking-widest">Estado en vivo por horario</p>
          </div>
          <div className="flex items-center gap-3">
            <span className="px-3 py-1 bg-neutral-950 text-neutral-50 text-[9px] font-black rounded-lg">
              {empleados.length} REGISTROS
            </span>
            <button
              onClick={recargar}
              className="inline-flex items-center gap-2 px-3 py-1.5 text-[9px] font-black uppercase tracking-widest text-neutral-400 hover:text-neutral-950 transition-colors"
            >
              <MorphIcon icon={recargado ? ICONO_CHECK : ICONO_FLECHA} size={13} strokeWidth={2.5} spring="snappy" reducedMotion="user" />
              Recargar datos
            </button>
          </div>
        </div>

        <div className="overflow-x-auto custom-scrollbar">
          <table className="w-full text-left border-collapse min-w-[720px]">
            <thead className="sticky top-0 z-10">
              <tr className="bg-neutral-50/95 backdrop-blur-sm border-y border-neutral-100">
                <th className="px-6 sm:px-10 py-4 text-[10px] font-black text-neutral-400 uppercase tracking-widest">Empleado</th>
                <th className="px-6 py-4 text-[10px] font-black text-neutral-400 uppercase tracking-widest">Estado</th>
                <th className="px-6 py-4 text-[10px] font-black text-neutral-400 uppercase tracking-widest">Turno</th>
                <th className="px-6 py-4 text-[10px] font-black text-neutral-400 uppercase tracking-widest">Entrada / Último acceso</th>
                <th className="px-6 py-4 text-[10px] font-black text-neutral-400 uppercase tracking-widest text-right">Detalle</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-neutral-50">
              {empleados.map((emp) => {
                const estado = estadoDot(emp);
                const visual = estadoVisual[estado];
                const turno = detectTurno(emp.horario_inicio);
                return (
                  <tr key={emp.id} className="hover:bg-neutral-50/40 transition-colors">
                    <td className="px-6 sm:px-10 py-4">
                      <div className="flex items-center gap-3">
                        <div className="w-9 h-9 bg-neutral-950 text-neutral-50 rounded-xl flex items-center justify-center">
                          <MorphIcon icon={ICONO_USUARIO} size={15} strokeWidth={2.2} spring="smooth" />
                        </div>
                        <span className="text-xs font-black text-neutral-900 uppercase">{emp.nombre}</span>
                      </div>
                    </td>
                    <td className="px-6 py-4">
                      <span className={`inline-flex items-center gap-2 px-3 py-1.5 rounded-full text-[9px] font-black uppercase tracking-widest ${visual.fondo} ${visual.texto}`}>
                        <span className={`w-1.5 h-1.5 rounded-full ${visual.dot} ${estado === "En turno" ? "animate-pulse" : ""}`} />
                        {estado}
                      </span>
                    </td>
                    <td className="px-6 py-4">
                      <span className="text-[10px] font-bold text-neutral-900 uppercase bg-neutral-100 px-3 py-1.5 rounded-lg">
                        {turno || "Sin turno"}
                      </span>
                    </td>
                    <td className="px-6 py-4">
                      <span className="text-[10px] font-bold text-neutral-500">{formatEntrada(emp)}</span>
                    </td>
                    <td className="px-6 py-4 text-right">
                      <button
                        onClick={() => (selectedId === emp.id ? cerrarDetalle() : loadDetalle(emp.id))}
                        className="inline-flex items-center gap-2 px-5 py-2.5 rounded-xl border-2 border-neutral-950 text-neutral-900 text-[9px] font-black uppercase tracking-widest hover:bg-neutral-950 hover:text-neutral-50 transition-all active:scale-[0.97]"
                      >
                        <MorphIcon icon={selectedId === emp.id ? ICONO_CERRAR : ICONO_FLECHA} size={13} strokeWidth={2.5} spring="snappy" reducedMotion="user" />
                        {selectedId === emp.id ? "Cerrar" : "Ver"}
                      </button>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>

        {!empleados.length && (
          <p className="py-14 text-center text-[11px] font-black text-neutral-300 uppercase tracking-widest italic">
            {loading ? "Cargando personal..." : "Aún no hay empleados registrados"}
          </p>
        )}
      </div>

      {/* DETALLE DE EMPLEADO */}
      {selectedEmp && ventasDetalle && (
        <div className="animate-in fade-in slide-in-from-bottom-2 duration-500 space-y-8">
          <div className="flex items-center justify-between flex-wrap gap-4">
            <div>
              <h3 className="text-xl font-black text-neutral-900 uppercase tracking-tight">{selectedEmp.nombre}</h3>
              <p className="text-[9px] text-neutral-400 uppercase font-black tracking-widest mt-1">Resumen de rendimiento individual</p>
            </div>
            <button
              onClick={cerrarDetalle}
              className="inline-flex items-center gap-2 px-4 py-2.5 text-[9px] font-black uppercase tracking-widest text-neutral-400 hover:text-neutral-950 transition-colors"
            >
              <MorphIcon icon={ICONO_CERRAR} size={14} strokeWidth={2.5} spring="snappy" />
              Cerrar
            </button>
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-3 gap-4 sm:gap-8">
            {/* VENTAS */}
            <div className="bg-white rounded-[2.5rem] border border-neutral-200 p-6 sm:p-8 space-y-5">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 bg-neutral-950 text-neutral-50 rounded-xl flex items-center justify-center">
                  <MorphIcon icon={ICONO_TRENDING} size={16} strokeWidth={2.2} spring="smooth" />
                </div>
                <div>
                  <h4 className="text-xs font-black text-neutral-900 uppercase tracking-widest">Ventas</h4>
                  <p className="text-[9px] text-neutral-400 font-bold uppercase tracking-wider">Desglose del cajero</p>
                </div>
              </div>
              <div className="space-y-3">
                <div className="p-4 bg-neutral-50 rounded-2xl">
                  <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest">Vendido</p>
                  <p className="text-xl font-black text-neutral-900 mt-1">{formatMoney(ventasDetalle.total_ventas)}</p>
                  <p className="text-[9px] font-bold text-neutral-400 mt-0.5">{ventasDetalle.ticket_count} tickets</p>
                </div>
                <div className="p-4 bg-neutral-50 rounded-2xl">
                  <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest">Canceladas</p>
                  <p className="text-xl font-black text-neutral-900 mt-1">{formatMoney(ventasDetalle.ventas_canceladas)}</p>
                  <p className="text-[9px] font-bold text-neutral-400 mt-0.5">{ventasDetalle.total_canceladas_count} canceladas</p>
                </div>
                <div className="p-4 bg-neutral-50 rounded-2xl">
                  <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest">Con descuento</p>
                  <p className="text-xl font-black text-neutral-900 mt-1">{formatMoney(ventasDetalle.ventas_con_descuento)}</p>
                </div>
              </div>
            </div>

            {/* METAS Y BONOS */}
            <div className="bg-white rounded-[2.5rem] border border-neutral-200 p-6 sm:p-8 space-y-6">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 bg-neutral-950 text-neutral-50 rounded-xl flex items-center justify-center">
                  <MorphIcon icon={ICONO_TARGET} size={16} strokeWidth={2.2} spring="smooth" />
                </div>
                <div>
                  <h4 className="text-xs font-black text-neutral-900 uppercase tracking-widest">Metas y bonos</h4>
                  <p className="text-[9px] text-neutral-400 font-bold uppercase tracking-wider">Avance contra meta mensual</p>
                </div>
              </div>

              <div className="p-4 bg-neutral-50 rounded-2xl">
                <div className="flex justify-between items-center mb-3">
                  <p className="text-[9px] font-black text-neutral-400 uppercase tracking-widest">Meta mensual</p>
                  <p className="text-[10px] font-black text-neutral-900">{formatMoney(selectedEmp.meta_mensual)}</p>
                </div>
                <div className="h-3 rounded-full bg-neutral-200 overflow-hidden">
                  <div
                    className={`h-full rounded-full transition-all duration-700 ${
                      ventasDetalle.total_ventas >= selectedEmp.meta_mensual && selectedEmp.meta_mensual > 0
                        ? "bg-emerald-500"
                        : "bg-neutral-950"
                    }`}
                    style={{
                      width: `${Math.min(100, selectedEmp.meta_mensual > 0 ? (ventasDetalle.total_ventas / selectedEmp.meta_mensual) * 100 : 0)}%`,
                    }}
                  />
                </div>
                <p className="text-[9px] font-bold text-neutral-400 mt-2 uppercase tracking-widest">
                  {selectedEmp.meta_mensual > 0
                    ? `${Math.round((ventasDetalle.total_ventas / selectedEmp.meta_mensual) * 100)}% de la meta`
                    : "Sin meta definida"}
                </p>
              </div>

              <div className={`p-4 rounded-2xl flex items-center justify-between ${ventasDetalle.total_ventas >= selectedEmp.meta_mensual && selectedEmp.meta_mensual > 0 ? "bg-emerald-50" : "bg-neutral-50"}`}>
                <span className="text-[9px] font-black text-neutral-500 uppercase tracking-widest">Bono actual</span>
                <span className={`text-sm font-black ${selectedEmp.meta_mensual > 0 && ventasDetalle.total_ventas >= selectedEmp.meta_mensual ? "text-emerald-600" : "text-neutral-900"}`}>
                  {formatMoney(selectedEmp.bono)}
                </span>
              </div>

              {/* ROI individual */}
              <div className={`p-4 rounded-2xl border ${ventasDetalle.total_ventas - selectedEmp.salario_semanal < 0 ? "bg-red-50 border-red-200" : "bg-neutral-50 border-neutral-100"}`}>
                <p className="text-[9px] font-black uppercase tracking-widest text-neutral-400">ROI individual</p>
                <p className="text-[11px] font-black uppercase tracking-widest mt-2 flex items-center gap-1.5">
                  <span className={`flex items-center gap-1.5 ${ventasDetalle.total_ventas - selectedEmp.salario_semanal < 0 ? "text-red-600" : "text-emerald-600"}`}>
                    <MorphIcon
                      icon={ventasDetalle.total_ventas - selectedEmp.salario_semanal < 0 ? ICONO_ALERTA : ICONO_CHECK}
                      size={13}
                      strokeWidth={2.5}
                      spring="snappy"
                      reducedMotion="user"
                    />
                    {ventasDetalle.total_ventas - selectedEmp.salario_semanal < 0 ? "Pérdida detectada" : "Renta positivo"}
                  </span>
                  <span className="text-neutral-900 ml-auto text-sm">{formatMoney(ventasDetalle.total_ventas - selectedEmp.salario_semanal)}</span>
                </p>
              </div>
            </div>

            {/* CORTES DE CAJA */}
            <div className="bg-white rounded-[2.5rem] border border-neutral-200 p-6 sm:p-8 space-y-5">
              <div className="flex items-center gap-3">
                <div className="w-10 h-10 bg-neutral-950 text-neutral-50 rounded-xl flex items-center justify-center">
                  <MorphIcon icon={ICONO_CALENDARIO} size={16} strokeWidth={2.2} spring="smooth" />
                </div>
                <div>
                  <h4 className="text-xs font-black text-neutral-900 uppercase tracking-widest">Cortes de caja</h4>
                  <p className="text-[9px] text-neutral-400 font-bold uppercase tracking-wider">Últimos {cortes.length} cortes</p>
                </div>
              </div>

              <div className="space-y-2.5 max-h-72 overflow-y-auto custom-scrollbar pr-1">
                {cortes.length > 0 ? (
                  cortes.map((c, idx) => (
                    <div key={idx} className="p-3.5 bg-neutral-50 rounded-2xl flex items-center justify-between">
                      <div>
                        <p className="text-[10px] font-black text-neutral-900 uppercase">{formatDate(c.fecha_apertura)}</p>
                        <p className="text-[9px] font-bold text-neutral-400 mt-0.5">Inicial: {formatMoney(c.monto_inicial)}</p>
                      </div>
                      <div className="text-right">
                        <p className="text-xs font-black text-neutral-900">{formatMoney(c.total_ventas)}</p>
                        <span
                          className={`inline-block mt-1 px-2 py-0.5 text-[8px] font-black uppercase rounded-md ${
                            c.estado === "abierto" ? "bg-emerald-50 text-emerald-600" : "bg-neutral-200 text-neutral-500"
                          }`}
                        >
                          {c.estado}
                        </span>
                      </div>
                    </div>
                  ))
                ) : (
                  <p className="py-10 text-center text-[10px] font-black text-neutral-300 uppercase tracking-widest italic">
                    Sin cortes registrados
                  </p>
                )}
              </div>
            </div>
          </div>
        </div>
      )}

      {showModal && (
        <ModalEmpleados onClose={() => setShowModal(false)} onSaved={loadData} />
      )}
      {showTurnosModal && (
        <ModalTurnos
          empleados={empleados}
          onClose={() => setShowTurnosModal(false)}
          onSaved={loadData}
        />
      )}
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