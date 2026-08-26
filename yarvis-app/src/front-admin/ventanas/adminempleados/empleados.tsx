// Panel de administración de empleados.
// Orquestador: estado, cargas (loadData/loadDetalle/recargar), header
// con botones y composición de los componentes del detalle.
import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { MorphIcon } from "morphicons/react";
import type { EmpleadoProfile } from "../../../services/empleado";
import type { MiTurno, DiaExtra } from "../../../components/turno";
import ModalEmpleados from "./modalEmpleados";
import ModalMetas from "./modalMetas";
import {
  ICONO_USUARIOS,
  ICONO_MAS,
  ICONO_TARGET,
  ICONO_DOLAR,
  ICONO_TRENDING,
  ICONO_PREMIO,
  ICONO_ALERTA,
  ICONO_CERRAR,
  ICONO_CHECK,
  BotonAnimado,
} from "../../../components/ui";
import { TarjetaResumen } from "./componentes/tarjeta-resumen";
import { TablaPersonal } from "./componentes/tabla-personal";
import { DetalleAsistencia } from "./componentes/detalle-asistencia";
import { DetalleVentas } from "./componentes/detalle-ventas";
import { DetalleMetas } from "./componentes/detalle-metas";
import { DetalleCortes } from "./componentes/detalle-cortes";
import { HorasExtra } from "./componentes/horas-extra";
import {
  formatMoney,
  type EmpleadoResumen,
  type EmpleadoVentas,
  type CorteEmpleado,
} from "./utilidades/helpers";

interface AdminEmpleadosProps {
  activeTab: string;
}

const AdminEmpleados = ({ activeTab }: AdminEmpleadosProps) => {
  const [empleados, setEmpleados] = useState<EmpleadoProfile[]>([]);
  const [resumen, setResumen] = useState<EmpleadoResumen>({
    empleados_activos: 0,
    ventas_totales: 0,
    tickets_totales: 0,
    costo_nomina: 0,
    roi_neto: 0,
  });
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [ventasDetalle, setVentasDetalle] = useState<EmpleadoVentas | null>(null);
  const [cortes, setCortes] = useState<CorteEmpleado[]>([]);
  const [asistenciaDetalle, setAsistenciaDetalle] = useState<MiTurno | null>(null);
  const [extrasDetalle, setExtrasDetalle] = useState<DiaExtra[] | null>(null);
  const [expandidasAdmin, setExpandidasAdmin] = useState<Set<string>>(new Set());
  const [ahora, setAhora] = useState(() => new Date());

  // Reloj vivo para la barra de asistencia del detalle.
  useEffect(() => {
    const t = window.setInterval(() => setAhora(new Date()), 30000);
    return () => window.clearInterval(t);
  }, []);
  const [showModal, setShowModal] = useState(false);
  const [empleadoEditando, setEmpleadoEditando] = useState<EmpleadoProfile | null>(null);
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

  const recargarTimer = useRef<number | null>(null);
  useEffect(() => () => { if (recargarTimer.current) window.clearTimeout(recargarTimer.current); }, []);

  const recargar = async () => {
    setRecargado(false);
    await loadData();
    setRecargado(true);
    if (recargarTimer.current) window.clearTimeout(recargarTimer.current);
    recargarTimer.current = window.setTimeout(() => setRecargado(false), 1600);
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
    // Asistencia de hoy (independiente: si falla no rompe el resto)
    invoke<MiTurno>("get_asistencia_empleado", { empleadoId: id })
      .then(setAsistenciaDetalle)
      .catch(() => setAsistenciaDetalle(null));
    invoke<DiaExtra[]>("get_horas_extra_empleado", { empleadoId: id })
      .then(setExtrasDetalle)
      .catch(() => setExtrasDetalle(null));
  };

  const toggleExtraAdmin = (fecha: string) => {
    setExpandidasAdmin((prev) => {
      const next = new Set(prev);
      if (next.has(fecha)) next.delete(fecha); else next.add(fecha);
      return next;
    });
  };

  const selectedEmp = empleados.find((e) => e.id === selectedId) || null;

  const cerrarDetalle = () => {
    setSelectedId(null);
    setVentasDetalle(null);
    setCortes([]);
    setAsistenciaDetalle(null);
    setExtrasDetalle(null);
    setExpandidasAdmin(new Set());
  };

  return (
    <div className="w-full max-w-[1200px] mx-auto space-y-12">
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
        <TarjetaResumen icono={ICONO_TRENDING} iconoHover={ICONO_DOLAR} label="Ventas acumuladas" valor={formatMoney(resumen.ventas_totales)}>
          <p className="text-[8px] font-black text-neutral-400 uppercase tracking-widest mt-1">
            {resumen.tickets_totales.toLocaleString("es-MX")} tickets completados
          </p>
        </TarjetaResumen>
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
      <TablaPersonal
        empleados={empleados}
        loading={loading}
        selectedId={selectedId}
        recargado={recargado}
        onEditar={setEmpleadoEditando}
        onToggleDetalle={(id) => (selectedId === id ? cerrarDetalle() : loadDetalle(id))}
        onRecargar={recargar}
      />

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

          {/* ASISTENCIA DE HOY — misma barra que ve el empleado */}
          <DetalleAsistencia asistenciaDetalle={asistenciaDetalle} ahora={ahora} />

          <div className="grid grid-cols-1 lg:grid-cols-3 gap-4 sm:gap-8">
            {/* VENTAS */}
            <DetalleVentas ventasDetalle={ventasDetalle} />

            {/* METAS Y BONOS */}
            <DetalleMetas empleado={selectedEmp} ventasDetalle={ventasDetalle} />

            {/* CORTES DE CAJA */}
            <DetalleCortes cortes={cortes} />
          </div>

          {/* HORAS EXTRAS INDEFINIDAS — historial completo del empleado */}
          {extrasDetalle && extrasDetalle.length > 0 && (
            <HorasExtra
              extrasDetalle={extrasDetalle}
              expandidasAdmin={expandidasAdmin}
              onToggle={toggleExtraAdmin}
            />
          )}
        </div>
      )}

      {showModal && (
        <ModalEmpleados onClose={() => setShowModal(false)} onSaved={loadData} />
      )}
      {empleadoEditando && (
        <ModalEmpleados
          empleado={empleadoEditando}
          onClose={() => setEmpleadoEditando(null)}
          onSaved={() => { setEmpleadoEditando(null); loadData(); }}
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
