import Inventario from "./ventanas/Inventario";
import Configuracion from "./ventanas/Configuracion";
import Tickets from "./ventanas/Tickets";

interface AdminDashboardProps {
  activeTab: string;
  setActiveTab: (tab: string) => void;
  setStep: (step: number) => void;
  setSelectedRole: (role: string | null) => void;
  setLoginPass: (pass: string) => void;
  adminName: string;
  storeName: string;
  adminPass: string;
  initialLocation?: string;
  initialCp?: string;
}

const AdminDashboard = ({
  activeTab,
  setActiveTab,
  setStep,
  setSelectedRole,
  setLoginPass,
  adminName,
  storeName,
  adminPass,
  initialLocation = "",
  initialCp = "",
}: AdminDashboardProps) => {
  const menuItems = [
    {
      id: "ventas", label: "VENTAS", icon: (
        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="23 6 13.5 15.5 8.5 10.5 1 18" /><polyline points="17 6 23 6 23 12" /></svg>
      )
    },
    {
      id: "inventario", label: "INVENTARIO", icon: (
        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M21 8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16Z" /><path d="m3.3 7 8.7 5 8.7-5" /><path d="M12 22V12" /></svg>
      )
    },
    {
      id: "finanzas", label: "FINANZAS", icon: (
        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><line x1="12" y1="1" x2="12" y2="23" /><path d="M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6" /></svg>
      )
    },
    {
      id: "clientes", label: "CLIENTES", icon: (
        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" /><circle cx="9" cy="7" r="4" /><path d="M22 21v-2a4 4 0 0 0-3-3.87" /><path d="M16 3.13a4 4 0 0 1 0 7.75" /></svg>
      )
    },
    {
      id: "tickets", label: "TICKETS", icon: (
        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M2 9a3 3 0 0 1 0 6v2a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-2a3 3 0 0 1 0-6V7a2 2 0 0 0-2-2H4a2 2 0 0 0-2 2Z" /><path d="M13 5v2" /><path d="M13 17v2" /><path d="M13 11v2" /></svg>
      )
    },
    {
      id: "empleados", label: "EMPLEADOS", icon: (
        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect width="18" height="18" x="3" y="4" rx="2" ry="2" /><line x1="16" x2="8" y1="2" y2="2" /><line x1="7" x2="7" y1="8" y2="8" /><line x1="12" x2="12" y1="8" y2="8" /><line x1="17" x2="17" y1="8" y2="8" /><line x1="7" x2="7" y1="12" y2="12" /><line x1="12" x2="12" y1="12" y2="12" /><line x1="17" x2="17" y1="12" y2="12" /><line x1="7" x2="7" y1="16" y2="16" /><line x1="12" x2="12" y1="16" y2="16" /><line x1="17" x2="17" y1="16" y2="16" /></svg>
      )
    },
    {
      id: "yarvis", label: "Y.A.R.V.I.S.", icon: (
        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M12 8V4H8" /><rect width="16" height="12" x="4" y="8" rx="2" /><path d="M2 14h2" /><path d="M20 14h2" /><path d="M15 13v2" /><path d="M9 13v2" /></svg>
      )
    },
    {
      id: "ajustes", label: "AJUSTES", icon: (
        <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" /><circle cx="12" cy="12" r="3" /></svg>
      )
    },
  ];

  const content: Record<string, string> = {
    ventas: "ventas <- El sistema debe aprender de tus ventas y decirte cuanto vas a vender el proximo fin de semana, mes etc. con un margen de error, el sistema presente las predicciones con Intervalos de Confianza seran graficas complejas pero tambien",
    finanzas: "finainzas <- aqui si veremos las graficas complejas y completas de cada seccion controlaremos todos los Cortes de caja (X y Z) visualizando todos los que se han hecho tambien las graficas de Utilidad bruta, Utilidad Operativa, Utilidad Neta, Utilidad Marginal.",
    clientes: "clientes <- aqui mostraremos la gestión de los clientes y pedidos como Quién me compra, cuánto me compra, cuanto me genera y cuándo fue su última visita, ademas de ver los pedidos encargados por clientes generados por el empleado, visualizando el formato de pago, los productos, si se aplico algun descuento, los tipos de ganancia etc. con la opcion de crear facturas reportes",
    tickets: "tickets y facturacion <- todo lo que ha pasado por la impresora y todo lo que se ha facturado. aqui podremos visualizar todos los tickets que se han impreso a lo largo del tiempo el promedio de venta por ticket el promedio de tickets por dia, cosas asi ademas de la visualizacion de los cortes de caja de igual manera mostrar todos los cortes de caja generados el promedio de estos al igual que graficas tanto para tickets como para cortes de caja y la importantisima facturacion electronica.",
    empleados: "empleados <- desde aqui podremos gestionar quienes son los empleados nombre de los empleados, cuanto venden, ventas canceladas, con descuentos ventas, metas que les puedes poner para generar bonos, definir turnos, horarios, definir su salario semanal, para que puedas visualizar cuanto te estan generando y cuanto te esta costando mantenerlo, visualizar cuando hicieron cortes de caja x y z recordemos que el corte de caja Z es el cierre de turno, visualizar cuando iniciaron turno",
    yarvis: "Y.A.R.V.I.S. <- AQUI SOLO PONDRAS LA INTERFAZ DEL CHAT BOT NO HARAS QUE FUNCIONE POR QUE FALTA CONECTAR EL MOTOR.",
  };

  return (
    <main className="h-screen w-full flex bg-white font-sans text-neutral-800 animate-in fade-in duration-500 overflow-hidden">
      {/* SIDEBAR */}
      <aside className="w-64 bg-neutral-50 border-r border-neutral-100 flex flex-col p-6 shadow-sm">
        <div className="mb-10 px-2 flex items-center gap-3">
          <div className="w-8 h-8 bg-neutral-900 rounded-lg flex items-center justify-center text-white font-black text-xl">Y</div>
          <div>
            <h1 className="text-sm font-black tracking-tighter leading-none">Y.A.R.V.I.S.</h1>
            <p className="text-[9px] font-bold text-neutral-400 tracking-widest uppercase">Admin Panel</p>
          </div>
        </div>

        <nav className="flex-1 space-y-1">
          {menuItems.map((item) => (
            <button key={item.id} onClick={() => setActiveTab(item.id)} className={`w-full flex items-center gap-3 px-4 py-3 rounded-xl text-xs font-bold transition-all ${activeTab === item.id ? "bg-neutral-900 text-white shadow-lg shadow-neutral-200 scale-[1.02]" : "text-neutral-400 hover:bg-neutral-100 hover:text-neutral-600"}`}>
              <span className={activeTab === item.id ? "text-white" : "text-neutral-300"}>{item.icon}</span>
              {item.label}
            </button>
          ))}
        </nav>

        <div className="mt-auto pt-6 border-t border-neutral-100">
          <button onClick={() => { setStep(1); setSelectedRole(null); setLoginPass(""); }} className="w-full flex items-center gap-3 px-4 py-3 rounded-xl text-xs font-bold text-neutral-400 hover:bg-neutral-50 transition-all">
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1-2-2h4" /><polyline points="16 17 21 12 16 7" /><line x1="21" y1="12" x2="9" y2="12" /></svg>
            CERRAR SESIÓN
          </button>
        </div>
      </aside>

      {/* CONTENIDO CENTRAL */}
      <section className="flex-1 p-12 bg-white overflow-y-auto custom-scrollbar">
        {activeTab === "inventario" ? (
          <Inventario activeTab={activeTab} />
        ) : activeTab === "ajustes" ? (
          <Configuracion 
            adminName={adminName}
            storeName={storeName}
            adminPass={adminPass}
            initialLocation={initialLocation}
            initialCp={initialCp}
          />
        ) : activeTab === "tickets" ? (
          <Tickets />
        ) : (
          <div className="max-w-3xl animate-in fade-in slide-in-from-bottom-2 duration-500 mx-auto">
            <header className="mb-10 text-center">
              <h2 className="text-3xl font-black text-neutral-900 uppercase tracking-tight mb-2">{menuItems.find(i => i.id === activeTab)?.label}</h2>
              <div className="h-1.5 w-12 bg-neutral-900 rounded-full mx-auto"></div>
            </header>

            <div className="bg-neutral-50 p-10 rounded-[2.5rem] border border-neutral-100 shadow-inner">
              <p className="text-xl text-neutral-600 font-light leading-relaxed first-letter:text-4xl first-letter:font-black first-letter:text-neutral-900">{content[activeTab]}</p>
            </div>
          </div>
        )}
      </section>
    </main>
  );
};

export default AdminDashboard;
