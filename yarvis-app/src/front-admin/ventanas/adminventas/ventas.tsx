// Vista Ventas: el pulso de la tienda. Contenedor a 1200px como el resto
// de módulos; cada gráfica vive en su propio archivo de graficas/.
import Kpis from "./graficas/kpis";
import Pronostico from "./graficas/pronostico";
import TopProductos from "./graficas/topProductos";
import MetodosPago from "./graficas/metodosPago";
import GananciaNeta from "./graficas/gananciaNeta";
import Nomina from "./graficas/nomina";
import ProximaSemana from "./graficas/proximaSemana";

const AdminVentas = () => (
  <div className="w-full max-w-[1200px] mx-auto space-y-6 animate-in fade-in slide-in-from-bottom-2 duration-500">
    <header>
      <h2 className="text-3xl font-black text-neutral-900 uppercase tracking-tight">Ventas</h2>
      <div className="h-1.5 w-12 bg-neutral-900 rounded-full mt-2"></div>
      <p className="text-sm text-neutral-500 font-bold mt-2">
        El pulso de tu tienda, aprendido de tus tickets
      </p>
    </header>

    <Kpis />
    <Pronostico />

    <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <TopProductos />
      <MetodosPago />
    </div>

    <GananciaNeta />
    <Nomina />
    <ProximaSemana />

    <p className="text-center text-[11px] text-neutral-400 font-bold pb-4">
      Mínimo 4 días de historia para pronosticar · modelo Holt-Winters local, sin nube
    </p>
  </div>
);

export default AdminVentas;
