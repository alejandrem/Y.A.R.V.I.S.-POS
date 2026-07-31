import ChatWidget from "../../../components/ChatWidget";

const AdminYarvis = () => (
  <div className="h-full animate-in fade-in duration-500 flex flex-col bg-gradient-to-br from-neutral-50 via-white to-neutral-100">
    <div className="flex-shrink-0 px-8 pt-8 pb-4">
      <header className="flex justify-between items-end mb-6">
        <div>
          <h2 className="text-4xl font-black text-neutral-900 uppercase tracking-tight mb-1">Y.A.R.V.I.S.</h2>
          <p className="text-[11px] font-black text-neutral-400 uppercase tracking-[0.3em]">Asistente Inteligente de Negocio</p>
        </div>
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-2.5 px-5 py-3 bg-white/80 backdrop-blur-sm border border-neutral-200 rounded-2xl shadow-sm">
            <div className="w-2.5 h-2.5 rounded-full bg-emerald-500 animate-pulse shadow-lg shadow-emerald-500/50"></div>
            <span className="text-[11px] font-black text-neutral-600 uppercase tracking-widest">En línea</span>
          </div>
        </div>
      </header>
    </div>

    <div className="flex-1 min-h-0 px-8 pb-8">
      <div className="h-full bg-white/70 backdrop-blur-md rounded-[3rem] border border-neutral-200/80 shadow-2xl shadow-neutral-300/30 overflow-hidden relative">
        <div className="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-transparent via-neutral-900/10 to-transparent"></div>
        <ChatWidget
          role="admin"
          userId="admin"
          suggestions={[
            "¿Hubo algo raro hoy?",
            "¿Cuánto gané libre hoy quitando el costo de los productos?",
            "¿Qué debería comprar para el fin de semana?",
            "¿Qué productos están por agotarse?",
            "Resumen de ventas de hoy",
            "¿Qué empleados tienen más reembolsos?",
          ]}
        />
      </div>
    </div>
  </div>
);

export default AdminYarvis;
