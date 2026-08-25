// Panel de administración de Y.A.R.V.I.S. — delega en el panel compartido.
import PanelYarvis from "./PanelYarvis";

interface AdminYarvisProps {
  active?: boolean;
}

const AdminYarvis = ({ active }: AdminYarvisProps) => <PanelYarvis rol="admin" active={active} />;

export default AdminYarvis;
