// Formulario encargado de gestionar la configuración visual de la aplicación 
// como el cambio entre modo claro y modo oscuro.

import { useThemeContext } from "../../../../hooks/ThemeContext";

const AppearanceForm = () => {
  const { theme, setTheme } = useThemeContext();

  return (
    <div className="bg-neutral-50 p-8 rounded-[2.5rem] border border-neutral-100 shadow-sm">
      <h3 className="text-[10px] font-black text-neutral-400 uppercase tracking-[0.4em] mb-6">Apariencia del Sistema</h3>
      <div className="flex gap-4">
        {(['claro', 'oscuro', 'sistema'] as const).map((t) => (
          <button
            key={t}
            onClick={() => setTheme(t)}
            className={`flex-1 py-3 rounded-xl text-[9px] font-black uppercase tracking-widest transition-all ${theme === t ? 'bg-neutral-900 text-white shadow-lg' : 'bg-white text-neutral-400 border border-neutral-100 hover:bg-neutral-100'}`}
          >
            {t}
          </button>
        ))}
      </div>
    </div>
  );
};

export default AppearanceForm;