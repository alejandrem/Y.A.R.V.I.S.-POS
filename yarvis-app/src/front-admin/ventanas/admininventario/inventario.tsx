import PanelInventario from "./PanelInventario";

interface InventarioProps {
  activeTab: string;
}

const Inventario = ({ activeTab }: InventarioProps) => (
  <PanelInventario rol="admin" activeTab={activeTab} />
);

export default Inventario;
