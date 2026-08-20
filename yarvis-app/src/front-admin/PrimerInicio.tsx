import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { MorphIcon } from "morphicons/react";
import type { IconInput } from "morphicons/react";
import { ICONO_USUARIO, ICONO_TIENDA, ICONO_CANDADO } from "../icons";

const OJO: IconInput =
  "M2.062 12.348a1 1 0 0 1 0-.696 10.75 10.75 0 0 1 19.876 0 1 1 0 0 1 0 .696 10.75 10.75 0 0 1-19.876 0 m15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0";
const OJO_OCULTO: IconInput =
  "M9.88 9.88a3 3 0 1 0 4.24 4.24M10.73 5.08A10.43 10.43 0 0 1 12 5c7 0 10 7 10 7a13.16 13.16 0 0 1-1.67 2.68M6.61 6.61A13.526 13.526 0 0 0 2 12s3 7 10 7a9.74 9.74 0 0 0 5.39-1.61M2 2l20 20";

// Versiones espejadas (invertidas) para el campo "Repetir"
const INVERTIR = "translate(24,0) scale(-1,1)";
const OJO_INVERTIDO: IconInput = [
  ["path", { d: "M2.062 12.348a1 1 0 0 1 0-.696 10.75 10.75 0 0 1 19.876 0 1 1 0 0 1 0 .696 10.75 10.75 0 0 1-19.876 0 m15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0", transform: INVERTIR }],
];
const OJO_OCULTO_INVERTIDO: IconInput = [
  ["path", { d: "M9.88 9.88a3 3 0 1 0 4.24 4.24M10.73 5.08A10.43 10.43 0 0 1 12 5c7 0 10 7 10 7a13.16 13.16 0 0 1-1.67 2.68M6.61 6.61A13.526 13.526 0 0 0 2 12s3 7 10 7a9.74 9.74 0 0 0 5.39-1.61M2 2l20 20", transform: INVERTIR }],
];

const SMILE: IconInput =
  "M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0z M9 10h.01M15 10h.01 M9.5 15a3.5 3.5 0 0 0 5 0";
const FANTASMA: IconInput =
  "M9 10h.01M15 10h.01M12 2a8 8 0 0 0-8 8v12l3-3 2.5 2.5L12 19l2.5 2.5L17 19l3 3V10a8 8 0 0 0-8-8z";

const LABEL_CLS = "text-[11px] font-bold text-neutral-500 uppercase tracking-wider ml-1";
const INPUT_CLS =
  "w-full pl-10 pr-4 py-2.5 rounded-xl bg-neutral-50 border border-neutral-200 text-sm focus:outline-none focus:ring-2 focus:ring-neutral-900/10 focus:border-neutral-900 transition-all";

interface PerfilGoogle {
  nombre: string;
  email: string;
  simulado: boolean;
}

const GoogleLogo = ({ size = 18 }: { size?: number }) => (
  <svg width={size} height={size} viewBox="0 0 48 48" aria-hidden="true">
    <path fill="#FFC107" d="M43.611 20.083H42V20H24v8h11.303c-1.649 4.657-6.08 8-11.303 8-6.627 0-12-5.373-12-12s5.373-12 12-12c3.059 0 5.842 1.154 7.961 3.039l5.657-5.657C34.046 6.053 29.268 4 24 4 12.955 4 4 12.955 4 24s8.955 20 20 20 20-8.955 20-20c0-1.341-.138-2.65-.389-3.917z" />
    <path fill="#FF3D00" d="M6.306 14.691l6.571 4.819C14.655 15.108 18.961 12 24 12c3.059 0 5.842 1.154 7.961 3.039l5.657-5.657C34.046 6.053 29.268 4 24 4 16.318 4 9.656 8.337 6.306 14.691z" />
    <path fill="#4CAF50" d="M24 44c5.166 0 9.86-1.977 13.409-5.192l-6.19-5.238C29.211 35.091 26.715 36 24 36c-5.202 0-9.619-3.317-11.283-7.946l-6.522 5.025C9.505 39.556 16.227 44 24 44z" />
    <path fill="#1976D2" d="M43.611 20.083H42V20H24v8h11.303c-.792 2.237-2.231 4.166-4.087 5.571l6.19 5.238C36.971 39.205 44 34 44 24c0-1.341-.138-2.65-.389-3.917z" />
  </svg>
);

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
  const [google, setGoogle] = useState<PerfilGoogle | null>(null);
  const [loadingGoogle, setLoadingGoogle] = useState(false);
  const [googleError, setGoogleError] = useState<string | null>(null);
  const [showConfirmPassword, setShowConfirmPassword] = useState(false);
  const [showNewEmpConfirmPass, setShowNewEmpConfirmPass] = useState(false);
  const [hoverEmpleado, setHoverEmpleado] = useState(false);

  const handleLoginGoogle = async () => {
    setLoadingGoogle(true);
    setGoogleError(null);
    try {
      const perfil = await invoke<PerfilGoogle>("login_con_google");
      setGoogle(perfil);
      if (perfil.nombre) setAdminName(perfil.nombre);
    } catch (e) {
      setGoogleError(String(e));
    } finally {
      setLoadingGoogle(false);
    }
  };

  if (showAddEmployeeForm) {
    return (
      <div className="space-y-4">
        <header className="mb-4 text-center">
          <p className="text-[10px] font-semibold tracking-[0.2em] text-neutral-400 uppercase mb-1">Primeros pasos</p>
          <h2 className="text-xl font-black text-neutral-900 uppercase tracking-tight">Nuevo Empleado</h2>
          <div className="h-0.5 w-8 bg-neutral-900 mx-auto mt-2 rounded-full" />
          <p className="text-[10px] font-bold text-neutral-400 uppercase tracking-widest mt-2">Perfil de Acceso</p>
        </header>

        <div className="space-y-4">
          <div className="space-y-1">
            <label className={LABEL_CLS}>Nombre del Empleado</label>
            <div className="relative">
              <MorphIcon icon={ICONO_USUARIO} size={15} strokeWidth={1.8} className="absolute left-3 top-1/2 -translate-y-1/2 text-neutral-400 pointer-events-none" />
              <input type="text" value={newEmployeeName} onChange={(e) => setNewEmployeeName(e.target.value)} placeholder="Ej. Peter Parker" className={INPUT_CLS} />
            </div>
          </div>
          <div className="space-y-1">
            <label className={LABEL_CLS}>Crear contraseña</label>
            <div className="relative">
              <MorphIcon icon={ICONO_CANDADO} size={15} strokeWidth={1.8} className="absolute left-3 top-1/2 -translate-y-1/2 text-neutral-400 pointer-events-none" />
              <input type={showNewEmpPass ? "text" : "password"} value={newEmployeePass} onChange={(e) => setNewEmployeePass(e.target.value)} className={`${INPUT_CLS} pr-10`} />
              <button type="button" onClick={() => setShowNewEmpPass(!showNewEmpPass)} aria-label={showNewEmpPass ? "Ocultar contraseña" : "Mostrar contraseña"} className="absolute right-2 top-1/2 -translate-y-1/2 text-neutral-400 hover:text-neutral-600 transition-colors">
                <MorphIcon icon={showNewEmpPass ? OJO_INVERTIDO : OJO_OCULTO_INVERTIDO} size={18} strokeWidth={1.8} />
              </button>
            </div>
          </div>
          <div className="space-y-1">
            <label className={LABEL_CLS}>Confirmar contraseña</label>
            <div className="relative">
              <MorphIcon icon={ICONO_CANDADO} size={15} strokeWidth={1.8} className="absolute left-3 top-1/2 -translate-y-1/2 text-neutral-400 pointer-events-none" />
              <input type={showNewEmpConfirmPass ? "text" : "password"} value={newEmployeeConfirmPass} onChange={(e) => setNewEmployeeConfirmPass(e.target.value)} className={`${INPUT_CLS} pr-10`} />
              <button type="button" onClick={() => setShowNewEmpConfirmPass(!showNewEmpConfirmPass)} aria-label={showNewEmpConfirmPass ? "Ocultar contraseña" : "Mostrar contraseña"} className="absolute right-2 top-1/2 -translate-y-1/2 text-neutral-400 hover:text-neutral-600 transition-colors">
                <MorphIcon icon={showNewEmpConfirmPass ? OJO_INVERTIDO : OJO_OCULTO_INVERTIDO} size={18} strokeWidth={1.8} />
              </button>
            </div>
          </div>

          <div className="pt-2 space-y-3">
            <button type="button" onClick={handleSaveEmployee} className="w-full py-3.5 rounded-xl bg-neutral-900 text-white text-xs font-black uppercase tracking-[0.2em] hover:bg-neutral-800 hover:shadow-lg transition-all shadow-md">Guardar Usuario</button>
            <button type="button" onClick={() => setShowAddEmployeeForm(false)} className="w-full py-2.5 rounded-xl border border-dashed border-neutral-300 text-[10px] font-bold text-neutral-500 hover:border-neutral-400 hover:text-neutral-700 hover:bg-neutral-50 transition-all uppercase tracking-widest">Cancelar</button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div>
<header className="mb-7 text-center">
        <p className="text-[10px] font-semibold tracking-[0.2em] text-neutral-400 uppercase mb-1">Primeros pasos</p>
        <h2 className="text-xl font-black text-neutral-900 uppercase tracking-tight">Configuración de Acceso</h2>
        <div className="h-0.5 w-8 bg-neutral-900 mx-auto mt-2 rounded-full" />
        <p className="text-[10px] font-bold text-neutral-400 uppercase tracking-widest mt-1">Introduce los datos iniciales</p>
      </header>

      {google?.simulado && (
        <div className="mb-4 px-3 py-2 rounded-lg bg-amber-50 border border-amber-200 text-[10px] font-semibold text-amber-700">
          Modo demo: define YARVIS_GOOGLE_CLIENT_ID para el login real con Google.
        </div>
      )}

      {google && (
        <div className="mb-5 flex items-center gap-3 px-3 py-2.5 rounded-xl bg-neutral-50 border border-neutral-200">
          <div className="w-9 h-9 rounded-full bg-neutral-900 text-white flex items-center justify-center text-sm font-black uppercase shrink-0">
            {adminName.trim().charAt(0) || "?"}
          </div>
          <div className="flex-1 min-w-0">
            <p className="text-xs font-bold text-neutral-900 truncate">{adminName.trim() || "Cuenta de Google"}</p>
            <p className="text-[10px] font-semibold text-neutral-400 truncate">
              {google.email || (google.simulado ? "correo no capturado (demo)" : "")}
            </p>
          </div>
          <button type="button" onClick={() => setGoogle(null)} aria-label="Desvincular cuenta" className="text-neutral-400 hover:text-neutral-600 transition-colors shrink-0">
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M18 6 6 18M6 6l12 12" /></svg>
          </button>
        </div>
      )}

      <div className="space-y-4">
        <div className="space-y-1">
          <label className={LABEL_CLS}>Administrador</label>
          <div className="relative">
            <MorphIcon icon={ICONO_USUARIO} size={15} strokeWidth={1.8} className="absolute left-3 top-1/2 -translate-y-1/2 text-neutral-400 pointer-events-none" />
            <input type="text" value={adminName} onChange={(e) => setAdminName(e.target.value)} placeholder="Nombre completo" className={INPUT_CLS} />
          </div>
        </div>

        <div className="space-y-1">
          <label className={LABEL_CLS}>Tienda</label>
          <div className="relative">
            <MorphIcon icon={ICONO_TIENDA} size={15} strokeWidth={1.8} className="absolute left-3 top-1/2 -translate-y-1/2 text-neutral-400 pointer-events-none" />
            <input type="text" value={storeName} onChange={(e) => setStoreName(e.target.value)} placeholder="Nombre del negocio" className={INPUT_CLS} />
          </div>
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div className="space-y-1">
            <label className={LABEL_CLS}>Contraseña</label>
            <div className="relative">
              <MorphIcon icon={ICONO_CANDADO} size={15} strokeWidth={1.8} className="absolute left-3 top-1/2 -translate-y-1/2 text-neutral-400 pointer-events-none" />
              <input type={showPassword ? "text" : "password"} value={password} onChange={(e) => setPassword(e.target.value)} className={`${INPUT_CLS} pr-10`} />
              <button type="button" onClick={() => setShowPassword(!showPassword)} aria-label={showPassword ? "Ocultar contraseña" : "Mostrar contraseña"} className="absolute right-2 top-1/2 -translate-y-1/2 text-neutral-400 hover:text-neutral-600 transition-colors">
                <MorphIcon icon={showPassword ? OJO_OCULTO : OJO} size={18} strokeWidth={1.8} />
              </button>
            </div>
          </div>
          <div className="space-y-1">
            <label className={LABEL_CLS}>Repetir</label>
            <div className="relative">
              <MorphIcon icon={ICONO_CANDADO} size={15} strokeWidth={1.8} className="absolute left-3 top-1/2 -translate-y-1/2 text-neutral-400 pointer-events-none" />
              <input type={showConfirmPassword ? "text" : "password"} value={confirmPassword} onChange={(e) => setConfirmPassword(e.target.value)} className={`${INPUT_CLS} pr-10`} />
              <button type="button" onClick={() => setShowConfirmPassword(!showConfirmPassword)} aria-label={showConfirmPassword ? "Ocultar contraseña" : "Mostrar contraseña"} className="absolute right-2 top-1/2 -translate-y-1/2 text-neutral-400 hover:text-neutral-600 transition-colors">
                <MorphIcon icon={showConfirmPassword ? OJO_OCULTO_INVERTIDO : OJO_INVERTIDO} size={18} strokeWidth={1.8} />
              </button>
            </div>
          </div>
        </div>

        <div className="flex items-center gap-3 pt-1">
          <div className="flex-1 h-px bg-neutral-200" />
          <span className="text-[10px] font-bold text-neutral-400 uppercase tracking-widest">o continúa con</span>
          <div className="flex-1 h-px bg-neutral-200" />
        </div>

        <button
          type="button"
          onClick={handleLoginGoogle}
          disabled={loadingGoogle}
          className="w-full flex items-center justify-center gap-3 py-2.5 px-4 rounded-xl border border-neutral-200 bg-white text-xs font-bold text-neutral-600 hover:bg-neutral-50 hover:shadow-md hover:text-neutral-800 transition-all shadow-sm disabled:opacity-60 uppercase tracking-widest"
        >
          <GoogleLogo size={16} />
          {loadingGoogle ? "Abriendo Google…" : "Continuar con Google"}
        </button>

        {googleError && <p className="text-xs font-semibold text-red-500">{googleError}</p>}

        <div className="pt-2 space-y-3">
          <button
            type="button"
            onClick={() => setShowAddEmployeeForm(true)}
            onMouseEnter={() => setHoverEmpleado(true)}
            onMouseLeave={() => setHoverEmpleado(false)}
            className="w-full flex items-center justify-center gap-2 py-3 px-4 rounded-xl border border-dashed border-neutral-300 text-xs font-bold text-neutral-500 hover:border-neutral-400 hover:text-neutral-700 hover:bg-neutral-50 transition-all uppercase tracking-widest"
          >
            <MorphIcon icon={hoverEmpleado ? FANTASMA : SMILE} size={16} strokeWidth={1.8} />
            AGREGAR EMPLEADO
          </button>
          <button
            type="button"
            onClick={handleSaveAdmin}
            className="w-full py-3.5 px-4 rounded-xl bg-neutral-900 text-white text-xs font-black uppercase tracking-widest hover:bg-neutral-800 hover:shadow-lg hover:-translate-y-0.5 transition-all shadow-md"
          >
            Iniciar Sesión →
          </button>
        </div>
      </div>
    </div>
  );
};

export default PrimerInicio;
