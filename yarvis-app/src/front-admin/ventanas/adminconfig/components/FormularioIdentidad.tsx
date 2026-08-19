// Formulario para consultar y modificar los datos básicos de la tienda y del administrador
//  (nombre del negocio, nombre del admin, ubicación, código postal).

interface IdentityFormProps {
  currentAdminName: string;
  setCurrentAdminName: (v: string) => void;
  currentStoreName: string;
  setCurrentStoreName: (v: string) => void;
  location: string;
  setLocation: (v: string) => void;
  cp: string;
  setCp: (v: string) => void;
  onSave: () => void;
}

const IdentityForm = ({
  currentAdminName,
  setCurrentAdminName,
  currentStoreName,
  setCurrentStoreName,
  location,
  setLocation,
  cp,
  setCp,
  onSave,
}: IdentityFormProps) => {
  return (
    <div className="bg-neutral-50 p-8 rounded-[2.5rem] border border-neutral-100 space-y-6 shadow-sm">
      <h3 className="text-[10px] font-black text-neutral-400 uppercase tracking-[0.4em] mb-4">Datos de Identidad</h3>

      <div className="space-y-4">
        <div className="group">
          <label className="text-[9px] font-black text-neutral-400 uppercase ml-2 mb-1 block group-focus-within:text-neutral-900 transition-colors">Nombre del Administrador</label>
          <input
            type="text"
            value={currentAdminName}
            onChange={(e) => setCurrentAdminName(e.target.value)}
            className="w-full bg-white border border-neutral-100 px-6 py-4 rounded-2xl text-xs font-bold focus:outline-none focus:ring-4 focus:ring-neutral-900/5 focus:border-neutral-900 transition-all"
            placeholder="Ej. Alejandro"
          />
        </div>

        <div className="group">
          <label className="text-[9px] font-black text-neutral-400 uppercase ml-2 mb-1 block group-focus-within:text-neutral-900 transition-colors">Nombre de la Tienda</label>
          <input
            type="text"
            value={currentStoreName}
            onChange={(e) => setCurrentStoreName(e.target.value)}
            className="w-full bg-white border border-neutral-100 px-6 py-4 rounded-2xl text-xs font-bold focus:outline-none focus:ring-4 focus:ring-neutral-900/5 focus:border-neutral-900 transition-all"
            placeholder="Ej. Tienda Y.A.R.V.I.S."
          />
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div className="group">
            <label className="text-[9px] font-black text-neutral-400 uppercase ml-2 mb-1 block group-focus-within:text-neutral-900 transition-colors">Ubicación</label>
            <input
              type="text"
              value={location}
              onChange={(e) => setLocation(e.target.value)}
              className="w-full bg-white border border-neutral-100 px-6 py-4 rounded-2xl text-xs font-bold focus:outline-none focus:ring-4 focus:ring-neutral-900/5 focus:border-neutral-900 transition-all"
            />
          </div>
          <div className="group">
            <label className="text-[9px] font-black text-neutral-400 uppercase ml-2 mb-1 block group-focus-within:text-neutral-900 transition-colors">Código Postal</label>
            <input
              type="text"
              value={cp}
              onChange={(e) => setCp(e.target.value)}
              className="w-full bg-white border border-neutral-100 px-6 py-4 rounded-2xl text-xs font-bold focus:outline-none focus:ring-4 focus:ring-neutral-900/5 focus:border-neutral-900 transition-all"
            />
          </div>
        </div>

        {/* Botón Guardar Cambios exclusivo para Datos de Identidad */}
        <button
          onClick={onSave}
          className="w-full bg-neutral-900 text-white py-4 rounded-2xl text-[10px] font-black uppercase tracking-widest hover:scale-[1.02] active:scale-95 transition-all shadow-lg"
        >
          Guardar Cambios
        </button>
      </div>
    </div>
  );
};

export default IdentityForm;