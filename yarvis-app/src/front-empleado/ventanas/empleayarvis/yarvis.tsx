// ═══════════════════════════════════════════════════════════════════════════
// Y.A.R.V.I.S. EMPLEADO — mismo módulo que el del administrador, con la regla
// de protección de API keys del administrador. Delega en el panel compartido.
// ═══════════════════════════════════════════════════════════════════════════

import PanelYarvis from "../../../front-admin/ventanas/adminyarvis/PanelYarvis";

interface YarvisEmpleadoProps {
  active?: boolean;
}

const YarvisEmpleado = ({ active }: YarvisEmpleadoProps) => <PanelYarvis rol="empleado" active={active} />;

const yarvisNav = {
  id: "yarvis",
  label: "Y.A.R.V.I.S.",
  icon: (
    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M12 8V4H8" />
      <rect width="16" height="12" x="4" y="8" rx="2" />
      <path d="M2 14h2" />
      <path d="M20 14h2" />
      <path d="M15 13v2" />
      <path d="M9 13v2" />
    </svg>
  ),
};

export default YarvisEmpleado;
export { yarvisNav };
