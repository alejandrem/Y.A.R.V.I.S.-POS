interface SetupWizardProps {
  adminName: string;
  setAdminName: (name: string) => void;
  storeName: string;
  setStoreName: (name: string) => void;
  password: string;
  setPassword: (pass: string) => void;
  confirmPassword: string;
  setConfirmPassword: (pass: string) => void;
  showPassword: boolean;
  setShowPassword: (show: boolean) => void;
  handleSaveEmployee: () => void;
  handleSaveAdmin: () => void;
  setShowAddEmployeeForm: (show: boolean) => void;
  showAddEmployeeForm: boolean;
  newEmployeeName: string;
  setNewEmployeeName: (name: string) => void;
  newEmployeePass: string;
  setNewEmployeePass: (pass: string) => void;
  newEmployeeConfirmPass: string;
  setNewEmployeeConfirmPass: (pass: string) => void;
  showNewEmpPass: boolean;
  setShowNewEmpPass: (show: boolean) => void;
}

const PrimerInicio = ({
  adminName,
  setAdminName,
  storeName,
  setStoreName,
  password,
  setPassword,
  confirmPassword,
  setConfirmPassword,
  showPassword,
  setShowPassword,
  handleSaveEmployee,
  handleSaveAdmin,
  setShowAddEmployeeForm,
  showAddEmployeeForm,
  newEmployeeName,
  setNewEmployeeName,
  newEmployeePass,
  setNewEmployeePass,
  newEmployeeConfirmPass,
  setNewEmployeeConfirmPass,
  showNewEmpPass,
  setShowNewEmpPass,
}: SetupWizardProps) => {
  return (
    <div className="animate-in fade-in duration-500">
      <header className="mb-8">
        <h2 className="text-xl font-bold text-neutral-900 uppercase tracking-tight">Configuración de Acceso</h2>
        <p className="text-[10px] font-bold text-neutral-400 uppercase tracking-widest mt-1">Introduce los datos iniciales</p>
      </header>

      <div className="space-y-4">
        {!showAddEmployeeForm ? (
          <>
            <div className="space-y-1">
              <label className="text-[11px] font-bold text-neutral-500 uppercase tracking-wider ml-1">Administrador</label>
              <input type="text" value={adminName} onChange={(e) => setAdminName(e.target.value)} placeholder="Nombre completo" className="w-full px-3 py-2 rounded-lg bg-neutral-50 border border-neutral-200 text-sm focus:outline-none focus:border-neutral-400" />
            </div>
            <div className="space-y-1">
              <label className="text-[11px] font-bold text-neutral-500 uppercase tracking-wider ml-1">Tienda</label>
              <input type="text" value={storeName} onChange={(e) => setStoreName(e.target.value)} placeholder="Nombre del negocio" className="w-full px-3 py-2 rounded-lg bg-neutral-50 border border-neutral-200 text-sm focus:outline-none focus:border-neutral-400" />
            </div>
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
              <div className="space-y-1">
                <label className="text-[11px] font-bold text-neutral-500 uppercase tracking-wider ml-1">Contraseña</label>
                <div className="relative">
                  <input type={showPassword ? "text" : "password"} value={password} onChange={(e) => setPassword(e.target.value)} className="w-full px-3 py-2 pr-10 rounded-lg bg-neutral-50 border border-neutral-200 text-sm" />
                  <button onClick={() => setShowPassword(!showPassword)} className="absolute right-2 top-1/2 -translate-y-1/2 text-neutral-400 hover:text-neutral-600 transition-colors">
                    {showPassword ? "👁️" : "🙈"}
                  </button>
                </div>
              </div>
              <div className="space-y-1">
                <label className="text-[11px] font-bold text-neutral-500 uppercase tracking-wider ml-1">Repetir</label>
                <input type="password" value={confirmPassword} onChange={(e) => setConfirmPassword(e.target.value)} className="w-full px-3 py-2 rounded-lg bg-neutral-50 border border-neutral-200 text-sm" />
              </div>
            </div>

            <div className="pt-4 space-y-3">
              <button onClick={() => setShowAddEmployeeForm(true)} className="w-full py-3 px-4 rounded-lg border border-neutral-200 text-xs font-bold text-neutral-500 hover:bg-neutral-50 transition-all uppercase tracking-widest">+ AGREGAR EMPLEADO</button>
              <button onClick={handleSaveAdmin} className="w-full py-3 px-4 rounded-lg bg-neutral-900 text-white text-xs font-bold hover:bg-neutral-800 transition-all shadow-md uppercase tracking-widest">Iniciar Sesión →</button>
            </div>
          </>
        ) : (
          /* FORMULARIO AGREGAR EMPLEADO */
          <div className="animate-in zoom-in-95 duration-300 space-y-4">
            <header className="mb-4 text-center">
              <h2 className="text-xl font-black text-neutral-900 uppercase">Nuevo Registro</h2>
              <div className="h-0.5 w-8 bg-neutral-900 mx-auto mt-2 rounded-full"></div>
              <p className="text-[10px] font-bold text-neutral-400 uppercase tracking-widest mt-2">Perfil de Acceso</p>
            </header>

            <div className="space-y-4">
              <div className="space-y-1">
                <label className="text-[11px] font-bold text-neutral-500 uppercase tracking-wider ml-1">Nombre del Empleado</label>
                <input type="text" value={newEmployeeName} onChange={(e) => setNewEmployeeName(e.target.value)} placeholder="Ej. Peter Parker" className="w-full px-3 py-2 rounded-lg bg-neutral-50 border border-neutral-200 text-sm focus:outline-none focus:border-neutral-900" />
              </div>
              <div className="space-y-1">
                <label className="text-[11px] font-bold text-neutral-500 uppercase tracking-wider ml-1">Crear contraseña</label>
                <div className="relative">
                  <input type={showNewEmpPass ? "text" : "password"} value={newEmployeePass} onChange={(e) => setNewEmployeePass(e.target.value)} className="w-full px-3 py-2 pr-10 rounded-lg bg-neutral-50 border border-neutral-200 text-sm focus:outline-none focus:border-neutral-900" />
                  <button onClick={() => setShowNewEmpPass(!showNewEmpPass)} className="absolute right-2 top-1/2 -translate-y-1/2 text-neutral-400 hover:text-neutral-600 transition-colors">
                    {showNewEmpPass ? "👁️" : "🙈"}
                  </button>
                </div>
              </div>
              <div className="space-y-1">
                <label className="text-[11px] font-bold text-neutral-500 uppercase tracking-wider ml-1">Confirmar contraseña</label>
                <input type="password" value={newEmployeeConfirmPass} onChange={(e) => setNewEmployeeConfirmPass(e.target.value)} className="w-full px-3 py-2 rounded-lg bg-neutral-50 border border-neutral-200 text-sm focus:outline-none focus:border-neutral-900" />
              </div>
              <div className="pt-2">
                <button onClick={handleSaveEmployee} className="w-full py-4 rounded-xl bg-neutral-900 text-white text-xs font-black uppercase tracking-[0.2em] hover:bg-neutral-800 transition-all shadow-xl shadow-neutral-200">Guardar Usuario</button>
                <button onClick={() => setShowAddEmployeeForm(false)} className="w-full py-3 mt-2 text-[10px] font-bold text-neutral-400 uppercase tracking-widest">Cancelar</button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default PrimerInicio;
