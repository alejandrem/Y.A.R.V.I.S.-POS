// ═══════════════════════════════════════════════════════════════════════════
// EMPLOYEE DASHBOARD — Shell del punto de venta del operador.
// Tarea única: sidebar de navegación + topbar (atajos F5-F8, progreso del
// turno y operador) + enrutado del contenido por pestaña activa.
// ═══════════════════════════════════════════════════════════════════════════

import { useCallback, useEffect, useState } from "react";
import { MorphIcon } from "morphicons/react";
import {
  geometriaBarra, fmtHM, etiquetaEntrada, type MiTurno,
} from "../components/turno-extra";
import { invokeTauri, reportarError } from "../services/tauri";
import { notificarExito } from "../components/notificaciones";
import { abrirCajonPredeterminado } from "../services/impresora";
import { obtenerMiTurno } from "../services/turno";
import { nuevaVentaNav } from "./ventanas/emplea_new_venta/nueva_venta";
import { inventarioNav } from "./ventanas/empleainventario/inventario";
import { ticketsNav } from "./ventanas/empleaticket/ticket";
import { perfilNav } from "./ventanas/empleaperfil/perfil";
import { yarvisNav } from "./ventanas/empleayarvis/yarvis";
import { ajustesNav } from "./ventanas/empleaajustes/ajustes";
import {
  ICONO_CAJA, ICONO_AYUDA,
  ICONO_RELOJ, ICONO_USUARIO, ICONO_CERRAR,
} from "../components/ui";
import { TABLA_ATAJOS, type AccionAtajoMenu } from "./atajos/tabla-atajos";

import NuevaVenta from "./ventanas/emplea_new_venta/nueva_venta";
import ModalCorte from "./ventanas/empleacortes/modal-corte";
import ModalReimprimir from "./ventanas/empleaticket/componentes/modal-reimprimir";
import ModalAtajos from "./atajos/modal-atajos";
import { useAtajosEmpleados } from "./atajos/useAtajos";
import { solicitarAccion } from "./atajos/atajos";
import { CartProvider } from "./ventanas/emplea_new_venta/CartProvider";
import { ChatProvider } from "../front-admin/ventanas/adminyarvis/ChatProvider";
import Inventario from "./ventanas/empleainventario/inventario";
import Proveedores, { proveedoresNav } from "./ventanas/empleaproveedores/proveedores";
import Tickets from "./ventanas/empleaticket/ticket";
import Perfil from "./ventanas/empleaperfil/perfil";
import Ajustes from "./ventanas/empleaajustes/ajustes";
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

// Los atajos salen de TABLA_ATAJOS (atajos/tabla-atajos.ts): topbar, menú
// F8 y ejecución comparten la fuente. Aquí solo se filtran los de topbar.

// Aviso de contraseña débil predeterminada (empleados creados solos desde
// tickets: pass = nombre+123). Se pregunta al backend en cada login y NO se
// puede ocultar: persiste hasta que el admin la cambie en Empleados (el flag
// password_defecto se apaga en editar_empleado). Es intencional: con login
// solo-password, una clave predecible la adivina cualquiera que sepa el nombre.
export const AvisoPasswordDefecto = () => {
  const [avisar, setAvisar] = useState(false);

  useEffect(() => {
    invokeTauri<boolean>("aviso_password_defecto")
      .then(setAvisar)
      .catch(() => setAvisar(false));
  }, []);

  if (!avisar) return null;
  return (
    <div className="mb-4 rounded-2xl border border-amber-300 bg-amber-50 px-5 py-4 flex items-start gap-3">
      <div className="flex-1">
        <p className="text-[11px] font-black uppercase tracking-widest text-amber-700">
          Contraseña predeterminada — cámbiala con tu administrador
        </p>
        <p className="text-xs font-bold text-amber-800 mt-1">
          Tu contraseña es la que el sistema te asignó al detectarte en los tickets (débil).
          Pídele a tu administrador que la cambie en Empleados. Este aviso no se puede ocultar.
        </p>
      </div>
    </div>
  );
};

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
    proveedoresNav,
    perfilNav,
    yarvisNav,
    ajustesNav,
  ];

  // Turno real (misma fuente que el perfil): asistencia de hoy + horarios.
  const [turno, setTurno] = useState<MiTurno | null>(null);
  const [ahora, setAhora] = useState(() => new Date());

  // Modales de shell: F3 corte, F2 reimprimir y F8 menú funcionan desde
  // cualquier pestaña. Con alguno abierto no entra otro atajo.
  const [showModalCorte, setShowModalCorte] = useState(false);
  const [showModalReimprimir, setShowModalReimprimir] = useState(false);
  const [showModalAtajos, setShowModalAtajos] = useState(false);
  const abrirCorte = useCallback(() => setShowModalCorte(true), []);
  const abrirReimprimir = useCallback(() => setShowModalReimprimir(true), []);
  const abrirAyuda = useCallback(() => setShowModalAtajos(true), []);
  const shellOcupado = showModalCorte || showModalReimprimir || showModalAtajos;
  // F5/F4: cambian a la pestaña destino y dejan la acción pendiente; la
  // pestaña la consume al montar (o al momento si ya estaba montada).
  // Con el corte abierto no se hace nada (no encimar modales).
  const irACobrar = useCallback(() => {
    if (showModalCorte || showModalReimprimir) return;
    setActiveTab("nueva_venta");
    solicitarAccion("cobrar");
  }, [showModalCorte, showModalReimprimir]);
  const irAPagar = useCallback(() => {
    if (showModalCorte || showModalReimprimir) return;
    setActiveTab("proveedores");
    solicitarAccion("pagar-proveedor");
  }, [showModalCorte, showModalReimprimir]);
  // F7: a la barra de búsqueda de nueva venta, lista para escribir.
  const irABuscar = useCallback(() => {
    if (showModalCorte || showModalReimprimir) return;
    setActiveTab("nueva_venta");
    solicitarAccion("buscar");
  }, [showModalCorte, showModalReimprimir]);
  // F6: abre el cajón con la impresora predeterminada del spooler.
  // Misma lógica que "Cobrar sin ticket": pulso sin imprimir nada.
  const abrirCajonF6 = useCallback(() => {
    if (shellOcupado) return;
    abrirCajonPredeterminado()
      .then((msg) => notificarExito(msg))
      .catch((e) => reportarError("No se pudo abrir el cajón", e));
  }, [shellOcupado]);
  useAtajosEmpleados({ onCorte: abrirCorte, onCobrar: irACobrar, onPagar: irAPagar, onReimprimir: abrirReimprimir, onAyuda: abrirAyuda, onBuscar: irABuscar, onCajon: abrirCajonF6, deshabilitado: shellOcupado });

  // El menú F8 ejecuta y se cierra solo.
  const ejecutarAtajoMenu = useCallback((a: AccionAtajoMenu) => {
    setShowModalAtajos(false);
    if (a === "corte") abrirCorte();
    else if (a === "cobrar") irACobrar();
    else if (a === "pagar") irAPagar();
    else if (a === "reimprimir") abrirReimprimir();
    else if (a === "buscar") irABuscar();
    else if (a === "cajon") abrirCajonF6();
  }, [abrirCorte, irACobrar, irAPagar, abrirReimprimir, irABuscar, abrirCajonF6]);

  useEffect(() => {
    obtenerMiTurno().then(setTurno).catch((e) => reportarError("No se pudo cargar la información de tu turno", e));
    const t = window.setInterval(() => setAhora(new Date()), 30000);
    return () => window.clearInterval(t);
  }, []);

  // Solo la pestaña activa está en el DOM (sin árboles ocultos). Lo que debe
  // sobrevivir al cambio de tab vive en providers: CartProvider (carrito de
  // la venta en curso) y ChatProvider (respuesta en streaming en curso).
  const renderContent = () => {
    switch (activeTab) {
      case "inventario":
        return <Inventario activeTab={activeTab} />;
      case "proveedores":
        return <Proveedores activeTab={activeTab} />;
      case "tickets":
        return <Tickets activeTab={activeTab} operatorName={operatorName} />;
      case "perfil":
        return <Perfil activeTab={activeTab} operatorName={operatorName} />;
      case "ajustes":
        return <Ajustes operatorName={operatorName} />;
      case "nueva_venta":
        return <NuevaVenta activeTab={activeTab} onAbrirCorte={abrirCorte} />;
      case "yarvis":
        return <YarvisEmpleado active={activeTab === "yarvis"} />;
      default:
        return (
          <div className="flex-1 flex items-center justify-center bg-white rounded-[2.5rem] border border-dashed border-neutral-200">
            <div className="text-center py-16">
              <div className="w-16 h-16 bg-neutral-950 rounded-3xl flex items-center justify-center mx-auto mb-5 shadow-lg">
                <MorphIcon icon={employeeMenuItems.find((i) => i.id === activeTab)?.id === "yarvis" ? ICONO_AYUDA : ICONO_CAJA} size={24} strokeWidth={2.2} spring="smooth" className="text-white" />
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
          {/* ATAJOS GORDITOS (misma tabla que el menú F8) */}
          <div className="flex gap-2">
            {TABLA_ATAJOS.filter((a) => a.enTopbar).map((a) => (
              <button
                key={a.tecla}
                title={a.listo ? `Atajo ${a.tecla}: ${a.descripcion}` : `${a.tecla}: ${a.descripcion}`}
                onClick={() => {
                  if (!a.listo) return;
                  if (a.accion === "corte") abrirCorte();
                  else if (a.accion === "cobrar") irACobrar();
                  else if (a.accion === "reimprimir") abrirReimprimir();
                  else if (a.accion === "buscar") irABuscar();
                  else if (a.accion === "menu") abrirAyuda();
                  else if (a.accion === "cajon") abrirCajonF6();
                }}
                className="group flex items-center gap-2 pl-1.5 pr-3.5 py-1.5 bg-neutral-50 rounded-2xl border border-transparent hover:border-neutral-950 hover:bg-white hover:shadow-lg hover:shadow-neutral-200 transition-all duration-200 active:scale-95"
              >
                <span className="px-1.5 py-0.5 bg-neutral-950 text-white text-[8px] font-black rounded-lg">{a.tecla}</span>
                <MorphIcon icon={a.icono} size={13} strokeWidth={2.4} spring="snappy" reducedMotion="user" className="text-neutral-400 group-hover:text-neutral-950 transition-colors" />
                <span className="text-[10px] font-black uppercase tracking-widest text-neutral-400 group-hover:text-neutral-950 transition-colors">{a.labelTopbar}</span>
              </button>
            ))}
          </div>

          {/* TURNO — misma geometría que "Mi Turno", versión resumida */}
          {(() => {
            const barra = geometriaBarra(turno, ahora);
            if (!barra) {
              return (
                <div className="flex-1 flex items-center gap-3 min-w-0">
                  <MorphIcon icon={ICONO_RELOJ} size={14} strokeWidth={2.2} spring="smooth" className="text-neutral-300 shrink-0" />
                  <span className="text-[9px] font-black text-neutral-400 uppercase tracking-widest whitespace-nowrap">{shiftStart}</span>
                  <div className="flex-1 h-2.5 bg-neutral-100 rounded-full overflow-hidden">
                    <div className="bg-neutral-950 h-full rounded-full transition-all duration-1000 ease-in-out" style={{ width: `${shiftProgress}%` }} />
                  </div>
                  <span className="text-[9px] font-black text-neutral-400 uppercase tracking-widest whitespace-nowrap">{shiftEnd}</span>
                </div>
              );
            }
            return (
              <div className="flex-1 flex items-center gap-3 min-w-0" title={`Entrada oficial ${fmtHM(barra.inicio)} · Salida ${fmtHM(barra.fin)}`}>
                <MorphIcon icon={ICONO_RELOJ} size={14} strokeWidth={2.2} spring="smooth" className={barra.enExtra ? "text-emerald-500 shrink-0" : "text-neutral-300 shrink-0"} />
                {/* Hora de entrada real si llegó extra-temprano; si no, la oficial */}
                <span className="text-[9px] font-black uppercase tracking-widest whitespace-nowrap text-neutral-500">
                  {etiquetaEntrada(barra, turno?.primer_login ?? null)}
                </span>
                <div className="relative flex-1 h-2.5 bg-neutral-100 rounded-full overflow-visible min-w-[80px]">
                  {/* Extra tempranero (verde claro) */}
                  {barra.preExtraActivo && (
                    <div
                      className="absolute inset-y-0 bg-emerald-400"
                      style={{ left: `${barra.loginPct}%`, width: `${Math.max(0, barra.preExtraPct)}%`, borderRadius: "999px 0 0 999px" }}
                    />
                  )}
                  {/* Trabajo normal (negro) */}
                  <div
                    className="absolute inset-y-0 bg-neutral-950 rounded-full transition-all duration-700 ease-out"
                    style={{ left: `${barra.inicioPct}%`, width: `${Math.max(0, barra.trabajoPct)}%` }}
                  />
                  {/* Extra post-turno (verde) */}
                  {barra.enExtraPost && (
                    <div
                      className="absolute inset-y-0 bg-emerald-500"
                      style={{ left: `${barra.finPct}%`, width: `${Math.max(0, barra.postExtraPct)}%`, borderRadius: "0 999px 999px 0" }}
                    />
                  )}
                  {/* Bolita: primer login */}
                  {barra.loginPct !== null && (
                    <div
                      className="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-2 h-2 bg-white border-2 border-neutral-900 rounded-full shadow-sm z-10"
                      style={{ left: `${barra.loginPct}%` }}
                      title={`Primer login: ${turno?.primer_login ?? ""}`}
                    />
                  )}
                  {/* Bolita: frontera entrada oficial con extra tempranero */}
                  {barra.preExtraActivo && (
                    <div
                      className="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-2 h-2 bg-white border-2 border-neutral-900 rounded-full shadow-sm z-10"
                      style={{ left: `${barra.inicioPct}%` }}
                    />
                  )}
                  {/* Bolita: frontera salida con extra post */}
                  {barra.enExtraPost && (
                    <div
                      className="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-2 h-2 bg-white border-2 border-emerald-500 rounded-full shadow-sm z-10"
                      style={{ left: `${barra.finPct}%` }}
                    />
                  )}
                </div>
                <span className={`text-[9px] font-black uppercase tracking-widest whitespace-nowrap ${barra.enExtra ? "text-emerald-600" : "text-neutral-400"}`}>
                  {fmtHM(barra.fin)}
                </span>
                {barra.enExtra && (
                  <span className="px-2 py-0.5 bg-emerald-100 text-emerald-600 rounded-lg text-[8px] font-black uppercase tracking-widest whitespace-nowrap animate-pulse">
                    +{Math.floor(barra.extraMinutos / 60)}h {barra.extraMinutos % 60}m
                  </span>
                )}
              </div>
            );
          })()}

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
          <AvisoPasswordDefecto />
          <CartProvider>
          <ChatProvider role="empleado" userId="empleado">
          {renderContent()}
          </ChatProvider>
          </CartProvider>
        </section>
      </div>

      {showModalCorte && (
        <ModalCorte onClose={() => setShowModalCorte(false)} cajero={operatorName || "GENERAL"} />
      )}

      {showModalReimprimir && (
        <ModalReimprimir onClose={() => setShowModalReimprimir(false)} />
      )}

      {showModalAtajos && (
        <ModalAtajos onClose={() => setShowModalAtajos(false)} onAccion={ejecutarAtajoMenu} />
      )}
    </main>
  );
};

export default EmployeeDashboard;
