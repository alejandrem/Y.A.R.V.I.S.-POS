// ═══════════════════════════════════════════════════════════════════════════
// TICKETS Y CORTES DEL EMPLEADO — El pulso de MIS ventas (solo yo).
// Esqueleto calcado de adminventas/ventas.tsx: contenedor 1200px, header
// con subrayado, KPIs, gráfica y grid de 2 columnas. Cada bloque vive en
// su propio archivo de ./componentes (1 archivo = 1 tarea).
// Cortes: placeholder PROXIMAMENTE (solo lectura cuando exista comando).
// ═══════════════════════════════════════════════════════════════════════════

import { useState } from "react";
import KpisMisTickets from "./componentes/kpis-mis-tickets";
import GraficaDiaSueldo from "./componentes/grafica-dia-sueldo";
import ListaMisTickets from "./componentes/lista-mis-tickets";
import PanelCortesProx from "./componentes/panel-cortes-prox";
import ModalDetalleTicket from "./componentes/modal-detalle-ticket";
import type { MiTicket } from "../../../services/misventas";

const ticketsNav = {
  id: "tickets",
  label: "TICKETS Y CORTES",
  icon: (
    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M2 9a3 3 0 0 1 0 6v2a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-2a3 3 0 0 1 0-6V7a2 2 0 0 0-2-2H4a2 2 0 0 0-2 2Z" />
      <path d="M13 5v2" />
      <path d="M13 17v2" />
      <path d="M13 11v2" />
    </svg>
  ),
};

interface TicketsProps {
  activeTab: string;
  operatorName?: string;
}

const Tickets = ({ activeTab, operatorName = "" }: TicketsProps) => {
  // Cada bloque trae su propio rango y solo se afecta a sí mismo
  // (igual que adminventas): aquí solo vive el modal.
  const [seleccionado, setSeleccionado] = useState<MiTicket | null>(null);

  if (activeTab !== "tickets") return null;

  return (
    <div className="w-full max-w-[1200px] mx-auto space-y-6 animate-in fade-in slide-in-from-bottom-2 duration-500">
      <header>
        <h2 className="text-3xl font-black text-neutral-900 uppercase tracking-tight">Tickets y cortes</h2>
        <div className="h-1.5 w-12 bg-neutral-900 rounded-full mt-2"></div>
        <p className="text-sm text-neutral-500 font-bold mt-2">
          Mis ventas y mis tickets, aprendido de mis tickets
        </p>
      </header>

      <KpisMisTickets />

      <GraficaDiaSueldo operatorName={operatorName} />

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <ListaMisTickets onVer={setSeleccionado} />
        <PanelCortesProx />
      </div>

      {seleccionado && (
        <ModalDetalleTicket ticket={seleccionado} onCerrar={() => setSeleccionado(null)} />
      )}
    </div>
  );
};

export default Tickets;
export { ticketsNav };
