import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import AdminDashboard from "./front-admin/AdminDashboard";
import PrimerInicio from "./front-admin/PrimerInicio";
import EmployeeDashboard from "./front-empleado/EmployeeDashboard";
import "./App.css";

function App() {
  // 0 = Registro Inicial, 1 = Login, 2 = Dashboard Administrador, 3 = Dashboard Empleado
  const [step, setStep] = useState<number | null>(null);
  const [setupFinished, setSetupFinished] = useState<boolean>(false);
  const [selectedRole, setSelectedRole] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState("ventas");
  const [currentOperator, setCurrentOperator] = useState("");
  const [adminLocation, setAdminLocation] = useState("");
  const [adminCp, setAdminCp] = useState("");

  // Estados del formulario inicial (Admin y Tienda)
  const [adminName, setAdminName] = useState("");
  const [storeName, setStoreName] = useState("");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);

  // Formulario de empleado (se guarda directo a DB, sin estado local redundante)
  const [showAddEmployeeForm, setShowAddEmployeeForm] = useState(false);
  const [newEmployeeName, setNewEmployeeName] = useState("");
  const [newEmployeePass, setNewEmployeePass] = useState("");
  const [newEmployeeConfirmPass, setNewEmployeeConfirmPass] = useState("");
  const [showNewEmpPass, setShowNewEmpPass] = useState(false);

  // Login Passwords
  const [loginPass, setLoginPass] = useState("");
  const [employeeLoginPass, setEmployeeLoginPass] = useState("");
  const [showLoginPass, setShowLoginPass] = useState(false);
  const [showEmployeeLoginPass, setShowEmployeeLoginPass] = useState(false);

  // Al cargar la app, checar si ya se hizo el registro inicial
  useEffect(() => {
    const checkSetup = async () => {
      try {
        const setupDone = await invoke<boolean>("check_setup_done");
        setSetupFinished(setupDone);
        setStep(setupDone ? 1 : 0);
      } catch (error) {
        console.error("Error checking setup:", error);
        setStep(0);
      }
    };
    checkSetup();
  }, []);

  const isPasswordValid = (pass: string) => {
    return pass.length >= 6 && /[A-Za-z]/.test(pass) && /[0-9]/.test(pass);
  };

  const handleSaveAdmin = async () => {
    if (adminName && storeName && password && password === confirmPassword) {
      if (!isPasswordValid(password)) {
        alert("La contraseña debe tener al menos 6 caracteres, incluyendo letras y números.");
        return;
      }

      try {
        await invoke("guardar_admin", {
          data: {
            name: adminName,
            store: storeName,
            pass: password,
          }
        });
        setSetupFinished(true);
        setStep(1);
      } catch (error) {
        console.error("Error al guardar admin:", error);
        alert("Error al guardar en la base de datos");
      }
    } else {
      alert("Por favor rellena todos los campos correctamente");
    }
  };

  const handleLoginAdmin = async () => {
    try {
      const isValid = await invoke<boolean>("validar_login_admin", { pass: loginPass });
      if (isValid) {
        // Cargar datos completos del admin
        const profile = await invoke<any>("get_admin_data");
        if (profile) {
          setAdminName(profile.nombre);
          setStoreName(profile.tienda);
          setPassword(profile.password);
          setAdminLocation(profile.ubicacion || "");
          setAdminCp(profile.cp || "");
          setCurrentOperator(profile.nombre);
        }
        setStep(2);
      } else {
        alert("Contraseña incorrecta. Inténtalo de nuevo.");
      }
    } catch (error) {
      console.error("Error en login:", error);
      alert("Error al conectar con la base de datos");
    }
  };

  const handleSaveEmployee = async () => {
    if (!isPasswordValid(newEmployeePass)) {
      alert("La contraseña del empleado debe tener letras y números.");
      return;
    }

    if (newEmployeeName && newEmployeePass === newEmployeeConfirmPass) {
      try {
        await invoke("guardar_empleado", { name: newEmployeeName, pass: newEmployeePass });
        setNewEmployeeName("");
        setNewEmployeePass("");
        setNewEmployeeConfirmPass("");
        setShowAddEmployeeForm(false);
      } catch (error) {
        console.error("Error al guardar empleado:", error);
        alert("Error al guardar empleado en la DB");
      }
    }
  };

  const handleLoginEmployee = async () => {
    try {
      const empName = await invoke<string | null>("validar_login_empleado", { pass: employeeLoginPass });
      if (empName) {
        setCurrentOperator(empName);
        setStep(3);
        setActiveTab("nueva_venta");
        setEmployeeLoginPass("");
      } else {
        alert("Contraseña de empleado incorrecta.");
      }
    } catch (error) {
      console.error("Error en login empleado:", error);
      alert("Error al conectar con la base de datos");
    }
  };

  if (step === null) return null;

  // --- RENDERING LOGIC ---

  if (step === 2) {
    return (
      <AdminDashboard
        activeTab={activeTab}
        setActiveTab={setActiveTab}
        setStep={setStep}
        setSelectedRole={setSelectedRole}
        setLoginPass={setLoginPass}
        adminName={adminName}
        storeName={storeName}
        adminPass={password}
        initialLocation={adminLocation}
        initialCp={adminCp}
      />
    );
  }

  if (step === 3) {
    return (
      <EmployeeDashboard
        activeTab={activeTab}
        setActiveTab={setActiveTab}
        setStep={setStep}
        setSelectedRole={setSelectedRole}
        operatorName={currentOperator}
      />
    );
  }

  return (
    <main className="h-screen w-full flex bg-white font-sans text-neutral-800 overflow-hidden">
      {/* SECCIÓN IZQUIERDA: BIENVENIDA */}
      <section className="w-1/2 bg-neutral-50 flex flex-col justify-center p-12 border-r border-neutral-100">
        <div className="max-w-md mx-auto">
          <header className="mb-6">
            <p className="text-[10px] font-semibold tracking-[0.2em] text-neutral-400 uppercase mb-1">Gracias por elegirnos</p>
            <h1 className="text-5xl font-black text-neutral-900 leading-tight">Y.A.R.V.I.S. <span className="text-neutral-300">POS</span></h1>
          </header>

          <div className="w-20 h-20 bg-white rounded-3xl flex items-center justify-center shadow-xl shadow-neutral-200 border border-neutral-100 mb-8">
            <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="text-neutral-400"><path d="m3 9 9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" /><polyline points="9 22 9 12 15 12 15 22" /></svg>
          </div>

          <p className="text-neutral-500 font-medium text-lg leading-relaxed mb-6">Nos emociona acompañarte en el crecimiento de tu negocio.</p>
          <p className="text-neutral-400 text-sm leading-relaxed max-w-sm">Sistema diseñado para mejorar tus ganancias y predecir ventas con exactitud.</p>
        </div>
      </section>

      {/* SECCIÓN DERECHA: SETUP O LOGIN */}
      <section className="w-1/2 flex items-center justify-center p-12 bg-white relative">
        <div className="w-full max-w-sm">
          {step === 0 ? (
            <PrimerInicio
              adminName={adminName} setAdminName={setAdminName}
              storeName={storeName} setStoreName={setStoreName}
              password={password} setPassword={setPassword}
              confirmPassword={confirmPassword} setConfirmPassword={setConfirmPassword}
              showPassword={showPassword} setShowPassword={setShowPassword}
              handleSaveEmployee={handleSaveEmployee}
              handleSaveAdmin={handleSaveAdmin}
              showAddEmployeeForm={showAddEmployeeForm}
              setShowAddEmployeeForm={setShowAddEmployeeForm}
              newEmployeeName={newEmployeeName} setNewEmployeeName={setNewEmployeeName}
              newEmployeePass={newEmployeePass} setNewEmployeePass={setNewEmployeePass}
              newEmployeeConfirmPass={newEmployeeConfirmPass} setNewEmployeeConfirmPass={setNewEmployeeConfirmPass}
              showNewEmpPass={showNewEmpPass} setShowNewEmpPass={setShowNewEmpPass}
            />
          ) : (
            /* PASO 1: LOGIN */
            <div className="animate-in fade-in slide-in-from-right-4 duration-500">
              <header className="mb-10 text-center">
                <h2 className="text-xl font-black text-neutral-900 uppercase tracking-tight">Inicio de Sesión</h2>
                <p className="text-[11px] font-bold text-neutral-400 uppercase tracking-widest mt-1">Acceso al Sistema</p>
              </header>

              <div className="space-y-4">
                {/* BLOQUE ADMINISTRADOR */}
                <div className="space-y-3">
                  <button onClick={() => setSelectedRole(selectedRole === 'admin' ? null : 'admin')} className={`w-full group p-5 rounded-2xl border-2 transition-all text-left relative overflow-hidden ${selectedRole === 'admin' ? 'border-neutral-900 bg-neutral-900 shadow-xl' : 'border-neutral-50 bg-neutral-50'}`}>
                    <div className="relative z-10 flex items-center justify-between">
                      <div>
                        <p className={`text-[9px] font-black tracking-widest uppercase mb-1 ${selectedRole === 'admin' ? 'text-neutral-500' : 'text-neutral-400'}`}>Perfil Maestro</p>
                        <p className={`text-base font-bold ${selectedRole === 'admin' ? 'text-white' : 'text-neutral-900'}`}>ADMINISTRADOR</p>
                      </div>
                      <div className={`${selectedRole === 'admin' ? 'text-white' : 'text-neutral-300'}`}><svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10" /></svg></div>
                    </div>
                  </button>

                  {selectedRole === 'admin' && (
                    <div className="animate-in slide-in-from-top-2 duration-300 space-y-4 p-5 bg-white rounded-2xl border border-neutral-200 shadow-inner">
                      <div className="space-y-1">
                        <label className="text-[10px] font-black text-neutral-400 uppercase tracking-widest ml-1">Contraseña</label>
                        <div className="relative">
                          <input type={showLoginPass ? "text" : "password"} value={loginPass} onChange={(e) => setLoginPass(e.target.value)} placeholder="••••••••" className="w-full px-4 py-2 pr-10 rounded-xl bg-neutral-50 border border-neutral-100 text-sm focus:outline-none focus:border-neutral-900" />
                          <button onClick={() => setShowLoginPass(!showLoginPass)} className="absolute right-3 top-1/2 -translate-y-1/2 text-neutral-300 hover:text-neutral-500 transition-colors">
                            {showLoginPass ? "👁️" : "🙈"}
                          </button>
                        </div>
                      </div>
                      <button onClick={handleLoginAdmin} className="w-full py-3 rounded-xl bg-neutral-900 text-white text-[10px] font-black tracking-[0.2em] hover:bg-neutral-800 transition-all uppercase shadow-lg shadow-neutral-200">ENTRAR AL POS →</button>
                    </div>
                  )}
                </div>

                {/* BLOQUE EMPLEADO */}
                <div className="space-y-3">
                  <button onClick={() => setSelectedRole(selectedRole === 'employee' ? null : 'employee')} className={`w-full group p-5 rounded-2xl border-2 transition-all text-left flex items-center justify-between ${selectedRole === 'employee' ? 'border-neutral-900 bg-neutral-900 shadow-xl' : 'border-neutral-50 bg-neutral-50'}`}>
                    <div>
                      <p className={`text-[9px] font-black tracking-widest uppercase mb-1 ${selectedRole === 'employee' ? 'text-neutral-500' : 'text-neutral-400'}`}>Acceso Operativo</p>
                      <p className={`text-base font-bold ${selectedRole === 'employee' ? 'text-white' : 'text-neutral-900'}`}>EMPLEADO</p>
                    </div>
                    <div className={selectedRole === 'employee' ? 'text-white' : 'text-neutral-300'}><svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" /><circle cx="12" cy="7" r="4" /></svg></div>
                  </button>

                  {selectedRole === 'employee' && (
                    <div className="animate-in slide-in-from-top-2 duration-300 p-5 bg-white rounded-2xl border border-neutral-200 shadow-inner space-y-4">
                      <div className="space-y-1">
                        <label className="text-[10px] font-black text-neutral-400 uppercase tracking-widest ml-1">Contraseña de Operador</label>
                        <div className="relative">
                          <input
                            type={showEmployeeLoginPass ? "text" : "password"}
                            value={employeeLoginPass}
                            onChange={(e) => setEmployeeLoginPass(e.target.value)}
                            placeholder="••••"
                            className="w-full px-4 py-2 pr-10 rounded-xl bg-neutral-50 border border-neutral-100 text-sm focus:outline-none focus:border-neutral-900"
                          />
                          <button onClick={() => setShowEmployeeLoginPass(!showEmployeeLoginPass)} className="absolute right-3 top-1/2 -translate-y-1/2 text-neutral-300 hover:text-neutral-500 transition-colors">
                            {showEmployeeLoginPass ? "👁️" : "🙈"}
                          </button>
                        </div>
                      </div>
                      <button onClick={handleLoginEmployee} className="w-full py-3 rounded-xl bg-neutral-900 text-white text-[10px] font-black tracking-[0.2em] hover:bg-neutral-800 transition-all uppercase shadow-lg shadow-neutral-200">ENTRAR AL POS →</button>
                    </div>
                  )}
                </div>

                {!selectedRole && !setupFinished && (
                  <button onClick={() => setStep(0)} className="w-full py-4 text-[9px] font-black text-neutral-300 hover:text-neutral-600 transition-colors tracking-[0.3em] uppercase">← Regresar</button>
                )}
              </div>
            </div>
          )}
        </div>
      </section>
    </main>
  );
}

export default App;
