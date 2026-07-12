import { nuevaVentaNav } from "./ventanas/emplea_new_venta/nueva_venta";
import { inventarioNav } from "./ventanas/empleainventario/inventario";
import { ticketsNav } from "./ventanas/empleaticket/ticket";
import { clientesNav } from "./ventanas/empleaclientes/clientes";
import { perfilNav } from "./ventanas/empleaperfil/perfil";
import { yarvisNav } from "./ventanas/empleayarvis/yarvis";
import { ajustesNav } from "./ventanas/empleaajustes/ajustes";

import Inventario from "./ventanas/empleainventario/inventario";

interface EmployeeDashboardProps {
  activeTab: string;
  setActiveTab: (tab: string) => void;
  setStep: (step: number) => void;
  setSelectedRole: (role: string | null) => void;
  shiftStart?: string;
  shiftEnd?: string;
  shiftProgress?: number;
  operatorName?: string;
}

const EmployeeDashboard = ({
  activeTab,
  setActiveTab,
  setStep,
  setSelectedRole,
  shiftStart = "0:00",
  shiftEnd = "0:00",
  shiftProgress = 0,
  operatorName = "",
}: EmployeeDashboardProps) => {
  const cart: any[] = [];
  const iaSuggestion = "";

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
      case "nueva_venta":
        return (
          <div className="flex-1 flex flex-col gap-4 animate-in fade-in duration-500 max-w-5xl mx-auto w-full">
            <div className="relative group">
              <div className="absolute inset-y-0 left-4 flex items-center pointer-events-none">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" className="text-neutral-400 group-hover:text-neutral-900 transition-colors"><circle cx="11" cy="11" r="8" /><path d="m21 21-4.3-4.3" /></svg>
              </div>
              <input
                type="text"
                placeholder="Escanea o busca un producto con IA..."
                className="w-full pl-11 pr-4 py-3 bg-white border border-neutral-200 rounded-xl shadow-sm text-xs focus:outline-none focus:ring-4 focus:ring-neutral-900/5 focus:border-neutral-400 focus:shadow-[0_0_20px_rgba(0,0,0,0.03)] transition-all duration-300"
              />
              <div className="absolute right-4 top-1/2 -translate-y-1/2">
                <span className="px-1.5 py-0.5 bg-neutral-50 text-neutral-400 text-[8px] font-black rounded border border-neutral-200 uppercase tracking-tighter group-focus-within:border-neutral-900 group-focus-within:text-neutral-900 transition-all">Buscador Inteligente</span>
              </div>
            </div>

            <div className="flex-1 bg-white rounded-2xl border border-neutral-200 shadow-sm overflow-hidden flex flex-col">
              <div className="px-6 py-4 border-b border-neutral-50 bg-neutral-50/30 flex justify-between items-center">
                <h3 className="text-[10px] font-black text-neutral-900 uppercase tracking-widest flex items-center gap-2">
                  <div className="w-1 h-4 bg-black rounded-full"></div>
                  Detalle de Venta
                </h3>
                <span className="text-[9px] font-bold text-neutral-400 uppercase tracking-tighter">{cart.length} ARTÍCULOS</span>
              </div>
              <div className="flex-1 overflow-y-auto px-6">
                <table className="w-full text-left border-collapse">
                  <thead className="sticky top-0 bg-white z-10">
                    <tr className="text-[9px] font-black text-neutral-400 uppercase tracking-widest border-b border-neutral-50">
                      <th className="py-3 px-2">Cant.</th>
                      <th className="py-3 px-2">Producto</th>
                      <th className="py-3 px-2">Total</th>
                      <th className="py-3 px-2">Desc.</th>
                      <th className="py-3 px-2 text-right">Subtotal</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-neutral-50">
                    {cart.length === 0 ? (
                      <tr>
                        <td colSpan={5} className="py-24 text-center">
                          <div className="flex flex-col items-center gap-3 opacity-10">
                            <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"><circle cx="8" cy="21" r="1" /><circle cx="19" cy="21" r="1" /><path d="M2.05 2.05h2l2.66 12.42a2 2 0 0 0 2 1.58h9.78a2 2 0 0 0 1.95-1.57l1.56-7.43H5.12" /></svg>
                            <p className="text-[10px] font-black uppercase tracking-[0.3em]">Esperando productos...</p>
                          </div>
                        </td>
                      </tr>
                    ) : (
                      cart.map((item: any, idx: number) => (
                        <tr key={idx} className="group hover:bg-neutral-50/50 transition-colors">
                          <td className="py-3 px-2 font-black text-neutral-900 text-xs">
                            <input type="number" value={item.cant} className="w-12 bg-transparent focus:outline-none" />
                          </td>
                          <td className="py-3 px-2 font-bold text-neutral-700 text-xs">{item.producto}</td>
                          <td className="py-3 px-2 font-medium text-neutral-300 text-xs line-through">${item.total.toFixed(2)}</td>
                          <td className="py-3 px-2 font-bold text-neutral-400 text-xs">-${item.descuento.toFixed(2)}</td>
                          <td className="py-3 px-2 text-right font-black text-neutral-900 text-sm">${item.final.toFixed(2)}</td>
                        </tr>
                      ))
                    )}
                  </tbody>
                </table>
              </div>

              <div className="p-4 bg-neutral-900 text-white flex items-center gap-4">
                <div className="flex-1 flex items-center gap-3 bg-white/5 p-3 rounded-xl border border-white/10">
                  <div className="w-8 h-8 bg-white/10 rounded-lg flex items-center justify-center text-sm">✨</div>
                  <div>
                    <p className="text-[8px] font-black text-neutral-400 uppercase tracking-widest mb-0.5">Sugerencia IA</p>
                    <p className="text-[11px] font-medium text-neutral-200 leading-tight">
                      {iaSuggestion || <span className="opacity-30 italic">Sin recomendaciones...</span>}
                    </p>
                  </div>
                </div>
                <div className="flex flex-col items-end gap-1">
                  <p className="text-[8px] font-black text-neutral-500 uppercase tracking-widest leading-none">Total a cobrar</p>
                  <button className="px-8 py-3.5 bg-white hover:bg-neutral-50 text-black rounded-xl font-black text-base shadow-lg hover:shadow-white/5 transition-all hover:scale-[1.05] active:scale-95 flex items-center gap-2 leading-none border border-transparent hover:border-white/20">
                    ${(cart.reduce((acc: number, item: any) => acc + item.final, 0)).toFixed(2)}
                    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"><polyline points="20 6 9 17 4 12" /></svg>
                  </button>
                </div>
              </div>
            </div>
          </div>
        );
      default:
        return (
          <div className="flex-1 flex items-center justify-center bg-white rounded-2xl border border-dashed border-neutral-200">
            <div className="text-center">
              <div className="w-14 h-14 bg-neutral-50 rounded-2xl flex items-center justify-center mx-auto mb-4 text-xl">⚙️</div>
              <h3 className="text-sm font-black text-neutral-900 uppercase mb-1">{employeeMenuItems.find(i => i.id === activeTab)?.label}</h3>
              <p className="text-neutral-400 text-[10px] font-medium max-w-[180px] mx-auto leading-relaxed italic">Boceto pendiente de implementación</p>
            </div>
          </div>
        );
    }
  };

  return (
    <main className="h-screen w-full flex bg-white font-sans text-neutral-800 animate-in fade-in duration-500 overflow-hidden">
      <aside className="w-64 bg-white border-r border-neutral-100 flex flex-col p-6">
        <div className="mb-10 px-2 flex items-center gap-3">
          <div className="w-8 h-8 bg-black rounded-lg flex items-center justify-center text-white font-black text-xl">Y</div>
          <div>
            <h1 className="text-sm font-black tracking-tighter leading-none">Y.A.R.V.I.S.</h1>
            <p className="text-[9px] font-bold text-neutral-400 tracking-widest uppercase">POS System</p>
          </div>
        </div>

        <nav className="flex-1 space-y-1">
          {employeeMenuItems.map((item) => (
            <button key={item.id} onClick={() => setActiveTab(item.id)} className={`w-full flex items-center gap-3 px-4 py-3 rounded-xl text-xs font-bold transition-all duration-300 ${activeTab === item.id ? "bg-neutral-900 text-white shadow-xl shadow-neutral-300 scale-[1.05]" : "text-neutral-400 hover:bg-neutral-100 hover:text-neutral-900 hover:scale-[1.02]"}`}>
              <span className={activeTab === item.id ? "text-white" : "text-neutral-300 transition-colors group-hover:text-neutral-900"}>{item.icon}</span>
              {item.label}
            </button>
          ))}
        </nav>

        <div className="mt-auto pt-6 border-t border-neutral-50">
          <button onClick={() => { setStep(1); setSelectedRole(null); }} className="w-full flex items-center gap-3 px-4 py-3 rounded-xl text-xs font-bold text-neutral-400 hover:bg-neutral-50 hover:text-neutral-500 transition-all uppercase tracking-wider">
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" /><polyline points="16 17 21 12 16 7" /><line x1="21" y1="12" x2="9" y2="12" /></svg>
            Cerrar Turno
          </button>
        </div>
      </aside>

      <div className="flex-1 flex flex-col bg-neutral-50/50">
        <header className="h-20 bg-white border-b border-neutral-100 px-6 flex flex-col justify-center">
          <div className="flex items-center justify-between mb-2">
            <div className="flex gap-2">
              {["F5 COBRAR", "F6 CAJA", "F7 BUSCAR", "F8 ATAJOS"].map((txt, i) => (
                <button key={i} className="px-2 py-1 bg-white rounded border border-neutral-200 text-[9px] font-bold text-neutral-500 hover:border-neutral-900 hover:text-neutral-900 hover:scale-105 hover:shadow-sm transition-all duration-200 uppercase">
                  <span className="text-neutral-900 mr-1">[{txt.split(' ')[0]}]</span> {txt.split(' ')[1]}
                </button>
              ))}
            </div>
            <div className="flex items-center gap-2">
              <div className="text-right">
                <p className="text-[8px] font-black text-neutral-400 uppercase tracking-widest leading-none mb-0.5">Operador</p>
                <p className="text-[10px] font-bold text-neutral-900 leading-none">{operatorName}</p>
              </div>
              <div className="w-7 h-7 bg-neutral-100 rounded-full flex items-center justify-center text-neutral-600 text-[10px] font-bold border border-neutral-200 uppercase">
                {operatorName.charAt(0)}
              </div>
            </div>
          </div>
          <div className="flex items-center gap-3">
            <span className="text-[8px] font-black text-neutral-900 uppercase tracking-widest whitespace-nowrap">Turno: {shiftStart}</span>
            <div className="flex-1 h-1.5 bg-neutral-100 rounded-full overflow-hidden flex border border-neutral-200/50">
              <div
                className="bg-neutral-900 h-full rounded-full transition-all duration-1000 ease-in-out"
                style={{ width: `${shiftProgress}%` }}
              />
            </div>
            <span className="text-[8px] font-black text-neutral-900 uppercase tracking-widest whitespace-nowrap">{shiftEnd}</span>
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
