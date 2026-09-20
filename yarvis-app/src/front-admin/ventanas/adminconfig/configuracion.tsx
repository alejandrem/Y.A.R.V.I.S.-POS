// Ventana de Configuración del Administrador.
// Coordina la identidad de la tienda (FormularioIdentidad), la seguridad del
// admin (FormularioSeguridad) y la apariencia/tema (FormularioApariencia).
// El parseador de tickets vive en su propia ventana del panel administrativo.
import { lazy, Suspense } from "react";
import { useDatosAdmin } from "./hooks/useDatosAdmin";
import EncabezadoConfiguracion from "./components/EncabezadoConfiguracion";
import FormularioIdentidad from "./components/FormularioIdentidad";
import FormularioSeguridad from "./components/FormularioSeguridad";
import FormularioApariencia from "./components/FormularioApariencia";

// Libro de Datos Inutiles compartido (mismo patron lazy que empleaajustes/ajustes.tsx:
// no infla el bundle inicial, solo carga al abrir Ajustes).
const LibroAdmin = lazy(() => import("./libro"));

interface ConfiguracionProps {
  adminName: string;
  storeName: string;
  adminPass: string;
  initialLocation?: string;
  initialCp?: string;
}

const Configuracion = ({
  adminName,
  storeName,
  initialLocation = "",
  initialCp = "",
}: ConfiguracionProps) => {
  const {
    currentAdminName,
    setCurrentAdminName,
    currentStoreName,
    setCurrentStoreName,
    currentPass,
    setCurrentPass,
    passwordChanged,
    setPasswordChanged,
    location,
    setLocation,
    cp,
    setCp,
    successMessage,
    handleUpdate,
    handleSaveIdentity,
  } = useDatosAdmin(adminName, storeName, initialLocation, initialCp);

  return (
    <div className="w-full max-w-[1200px] mx-auto space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500">
      <EncabezadoConfiguracion successMessage={successMessage} />

      <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
        <FormularioIdentidad
          currentAdminName={currentAdminName}
          setCurrentAdminName={setCurrentAdminName}
          currentStoreName={currentStoreName}
          setCurrentStoreName={setCurrentStoreName}
          location={location}
          setLocation={setLocation}
          cp={cp}
          setCp={setCp}
          onSave={handleSaveIdentity}
        />

        <div className="space-y-6">
          <FormularioSeguridad
            currentPass={currentPass}
            setCurrentPass={setCurrentPass}
            passwordChanged={passwordChanged}
            setPasswordChanged={setPasswordChanged}
            onSave={handleUpdate}
          />

          <FormularioApariencia />
        </div>
      </div>

      {/* LIBRO - DATOS INUTILES (lazy: mismo manual que ve el empleado) */}
      <Suspense
        fallback={
          <div className="w-full max-w-[1200px] h-[620px] bg-white border-2 border-neutral-200 rounded-[1.8rem] animate-pulse flex items-center justify-center">
            <span className="font-mono text-[11px] font-black tracking-widest text-neutral-400">
              CARGANDO MANUAL...
            </span>
          </div>
        }
      >
        <LibroAdmin />
      </Suspense>
    </div>
  );
};

export default Configuracion;
