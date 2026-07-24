import { useState } from 'react';
import { FinanzasDashboard } from './FinanzasDashboard';
import GastosManager from './GastosManager';
import CortesManager from './CortesManager';
import GraficasPanel from './GraficasPanel';
import AlertasPanel from './AlertasPanel';

type TabFinanzas = 'dashboard' | 'gastos' | 'cortes' | 'graficas' | 'alertas';

const TABS: { id: TabFinanzas; label: string; icon: string }[] = [
  { id: 'dashboard', label: 'Dashboard', icon: '📊' },
  { id: 'gastos',    label: 'Gastos',    icon: '💸' },
  { id: 'cortes',    label: 'Cortes X/Z', icon: '🧾' },
  { id: 'graficas',  label: 'Gráficas',  icon: '📈' },
  { id: 'alertas',   label: 'Alertas',   icon: '🔔' },
];

const AdminFinanzas = () => {
  const [activeTab, setActiveTab] = useState<TabFinanzas>('dashboard');

  return (
    <div className="max-w-6xl mx-auto space-y-10 animate-in fade-in duration-500">

      <header className="flex justify-between items-end border-b border-neutral-100 pb-8">
        <div>
          <h2 className="text-3xl font-black text-neutral-900 uppercase tracking-tight mb-1">Finanzas</h2>
          <p className="text-[10px] font-black text-neutral-400 uppercase tracking-[0.3em]">Control Financiero y Análisis de Rentabilidad</p>
        </div>
        <div className="flex gap-4">
          <div className="flex bg-neutral-100 p-1 rounded-xl flex-wrap gap-1">
            {TABS.map(tab => (
              <button
                key={tab.id}
                id={`tab-finanzas-${tab.id}`}
                onClick={() => setActiveTab(tab.id)}
                className={`px-3 py-2 text-[8px] font-black rounded-lg transition-all flex items-center gap-2 ${
                  activeTab === tab.id
                    ? 'bg-white shadow-sm text-neutral-900'
                    : 'text-neutral-400 hover:text-neutral-600'
                }`}
              >
                <span className="text-sm leading-none">{tab.icon}</span>
                {tab.label}
              </button>
            ))}
          </div>
        </div>
      </header>

      <main>
        {activeTab === 'dashboard' && <FinanzasDashboard />}
        {activeTab === 'gastos'    && <GastosManager />}
        {activeTab === 'cortes'    && <CortesManager />}
        {activeTab === 'graficas'  && <GraficasPanel />}
        {activeTab === 'alertas'   && <AlertasPanel />}
      </main>
    </div>
  );
};

export default AdminFinanzas;
