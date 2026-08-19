// Encabezado de la página de Configuración.
// Muestra el título de la sección y el mensaje de éxito cuando se guardan cambios.
// Renderiza el título y la descripción de la página de configuración. 
// También se encarga de mostrar mensajes de éxito o error al guardar cambios.

interface ConfigHeaderProps {
  successMessage: string;
}

const ConfigHeader = ({ successMessage }: ConfigHeaderProps) => (
  <header className="mb-8 text-left relative">
    <h2 className="text-3xl font-black text-neutral-900 uppercase tracking-tight mb-2">Ajustes del Sistema</h2>
    <div className="h-1.5 w-12 bg-neutral-900 rounded-full"></div>

    {/* Toast de éxito con fade-out */}
    {successMessage && (
      <div className="absolute top-0 right-0 bg-green-50 border border-green-200 text-green-700 px-5 py-3 rounded-2xl text-[10px] font-black uppercase tracking-widest animate-in fade-in slide-in-from-top-2 duration-500 shadow-lg">
        {successMessage}
      </div>
    )}
  </header>
);

export default ConfigHeader;