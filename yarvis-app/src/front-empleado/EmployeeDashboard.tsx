// ═══════════════════════════════════════════════════════════════════════════
// EMPLOYEE DASHBOARD — Shell del punto de venta del operador.
// Tarea única: sidebar de navegación + topbar (atajos F5-F8, progreso del
// turno y operador) + enrutado del contenido por pestaña activa.
// ═══════════════════════════════════════════════════════════════════════════

import { MorphIcon } from "morphicons/react";
import { nuevaVentaNav } from "./ventanas/emplea_new_venta/nueva_venta";
import { inventarioNav } from "./ventanas/empleainventario/inventario";
import { ticketsNav } from "./ventanas/empleaticket/ticket";
import { clientesNav } from "./ventanas/empleaclientes/clientes";
import { perfilNav } from "./ventanas/empleaperfil/perfil";
import { yarvisNav } from "./ventanas/empleayarvis/yarvis";
import { ajustesNav } from "./ventanas/empleaajustes/ajustes";
import {
  ICONO_BILLETE, ICONO_CAJA, ICONO_BUSCAR, ICONO_AYUDA,
  ICONO_RELOJ, ICONO_USUARIO, ICONO_CERRAR,
} from "../components/ui";

import NuevaVenta from "./ventanas/emplea_new_venta/nueva_venta";
import Inventario from "./ventanas/empleainventario/inventario";
import Perfil from "./ventanas/empleaperfil/perfil";
import YarvisEmpleado from "./ventanas/empleayarvis/yarvis";

interface EmployeeDashboardProps {
  activeTab: string;
  setActiveTab: (tab: string) => void;
  onLogout: () => void;
  shiftStart?: string;
  shiftEnd?: string;
  shiftProgress?: number;
  operatorName?: string;
}

// Atajos de teclado de la topbar (F5 cobra, el resto en camino).
const ATAJOS = [
  { tecla: "F5", label: "Cobrar", icono: ICONO_BILLETE },
  { tecla: "F6", label: "Caja", icono: ICONO_CAJA },
  { tecla: "F7", label: "Buscar", icono: ICONO_BUSCAR },
  { tecla: "F8", label: "Atajos", icono: ICONO_AYUDA },
];

const EmployeeDashboard = ({
  activeTab,
  setActiveTab,
  onLogout,
  shiftStart = "0:00",
  shiftEnd = "0:00",
  shiftProgress = 0,
  operatorName = "",
}: EmployeeDashboardProps) => {
  const employeeMenuItems = [
    nuevaVentaNav,
    inventarioNav,
    ticketsNav,
    clientesNav,
    perfilNav,
    yarvisNav,
    ajustesNav,
  ];

  const renderContent = () => {
    switch (activeTab) {
      case "inventario":
        return <Inventario activeTab={activeTab} />;
      case "perfil":
        return <Perfil activeTab={activeTab} operatorName={operatorName} />;
      case "nueva_venta":
        return <NuevaVenta activeTab={activeTab} />;
      case "yarvis":
        return <YarvisEmpleado active={true} />;
      default:
        return (
          <div className="flex-1 flex items-center justify-center bg-white rounded-[2.5rem] border border-dashed border-neutral-200">
            <div className="text-center py-16">
              <div className="w-16 h-16 bg-neutral-950 rounded-3xl flex items-center justify-center mx-auto mb-5 shadow-lg">
                <MorphIcon icon={employeeMenuItems.find((i) => i.id === activeTab)?.id === "yarvis" ? ICONO_AYUDA : ICONO_CAJA} size={24} strokeWidth={2} spring="smooth" className="text-white" />
              </div>
              <h3 className="text-base font-black text-neutral-900 uppercase tracking-tight">{employeeMenuItems.find(i => i.id === activeTab)?.label}</h3>
              <p className="text-[10px] font-black uppercase tracking-widest text-neutral-300 mt-2">Boceto pendiente de implementación</p>
            </div>
          </div>
        );
    }
  };

  return (
    <main className="h-screen w-full flex bg-white font-sans text-neutral-800 animate-in fade-in duration-500 overflow-hidden">
      {/* ═══ SIDEBAR ═════════════════════════════════════════════════ */}
      <aside className="w-64 bg-white border-r border-neutral-100 flex flex-col p-5">
        <div className="mb-10 px-2 flex items-center gap-3">
          <div className="w-10 h-10 bg-neutral-950 rounded-2xl flex items-center justify-center text-white font-black text-lg shadow-lg">Y</div>
          <div>
            <h1 className="text-sm font-black tracking-tighter leading-none">Y.A.R.V.I.S.</h1>
            <p className="text-[9px] font-black text-neutral-400 tracking-[0.25em] uppercase mt-0.5">POS System</p>
          </div>
        </div>

        <nav className="flex-1 space-y-1.5">
          {employeeMenuItems.map((item) => (
            <button
              key={item.id}
              onClick={() => setActiveTab(item.id)}
              className={`w-full flex items-center gap-3 px-4 py-3.5 rounded-2xl text-[11px] font-black uppercase tracking-wider transition-all duration-300 ${
                activeTab === item.id
                  ? "bg-neutral-950 text-white shadow-xl shadow-neutral-300 scale-[1.03]"
                  : "text-neutral-400 hover:bg-neutral-50 hover:text-neutral-950"
              }`}
            >
              <span className={activeTab === item.id ? "text-white" : ""}>{item.icon}</span>
              {item.label}
            </button>
          ))}
        </nav>

        <div className="mt-auto pt-5 border-t border-neutral-100">
          <button
            onClick={onLogout}
            className="w-full flex items-center gap-3 px-4 py-3 rounded-2xl text-[11px] font-black text-neutral-400 hover:bg-red-50 hover:text-red-500 transition-all uppercase tracking-widest"
          >
            <MorphIcon icon={ICONO_CERRAR} size={15} strokeWidth={2.2} spring="snappy" reducedMotion="user" />
            Cerrar Turno
          </button>
        </div>
      </aside>

      {/* ═══ CONTENIDO ═══════════════════════════════════════════════ */}
      <div className="flex-1 flex flex-col bg-neutral-50/50 overflow-hidden">
        {/* ── TOPBAR ──────────────────────────────────────────────── */}
        <header className="bg-white border-b border-neutral-100 px-6 py-3.5 flex items-center gap-5">
          {/* ATAJOS GORDITOS */}
          <div className="flex gap-2">
            {ATAJOS.map((a) => (
              <button
                key={a.tecla}
                title={`Atajo ${a.tecla}`}
                className="group flex items-center gap-2 pl-1.5 pr-3.5 py-1.5 bg-neutral-50 rounded-2xl border border-transparent hover:border-neutral-950 hover:bg-white hover:shadow-lg hover:shadow-neutral-200 transition-all duration-200 active:scale-95"
              >
                <span className="px-1.5 py-0.5 bg-neutral-950 text-white text-[8px] font-black rounded-lg">{a.tecla}</span>
                <MorphIcon icon={a.icono} size={13} strokeWidth={2.4} spring="snappy" reducedMotion="user" className="text-neutral-400 group-hover:text-neutral-950 transition-colors" />
                <span className="text-[10px] font-black uppercase tracking-widest text-neutral-400 group-hover:text-neutral-950 transition-colors">{a.label}</span>
              </button>
            ))}
          </div>

          {/* TURNO */}
          <div className="flex-1 flex items-center gap-3 min-w-0">
            <MorphIcon icon={ICONO_RELOJ} size={14} strokeWidth={2.2} spring="smooth" className="text-neutral-300 shrink-0" />
            <span className="text-[9px] font-black text-neutral-400 uppercase tracking-widest whitespace-nowrap">{shiftStart}</span>
            <div className="flex-1 h-2.5 bg-neutral-100 rounded-full overflow-hidden">
              <div
                className="bg-neutral-950 h-full rounded-full transition-all duration-1000 ease-in-out"
                style={{ width: `${shiftProgress}%` }}
              />
            </div>
            <span className="text-[9px] font-black text-neutral-400 uppercase tracking-widest whitespace-nowrap">{shiftEnd}</span>
          </div>

          {/* OPERADOR */}
          <div className="flex items-center gap-3 bg-neutral-50 rounded-2xl pl-2 pr-4 py-1.5 border border-neutral-100">
            <div className="w-9 h-9 bg-neutral-950 rounded-xl flex items-center justify-center shadow-md">
              <MorphIcon icon={ICONO_USUARIO} size={15} strokeWidth={2.2} spring="smooth" className="text-white" />
            </div>
            <div>
              <p className="text-[8px] font-black text-neutral-400 uppercase tracking-[0.2em] leading-none mb-0.5">Operador</p>
              <p className="text-[11px] font-black text-neutral-900 leading-none truncate max-w-[140px]">{operatorName}</p>
            </div>
          </div>
        </header>

        <section className="flex-1 flex flex-col p-6 overflow-y-auto custom-scrollbar">
          {renderContent()}
        </section>
      </div>
    </main>
  );
};

export default EmployeeDashboard;
