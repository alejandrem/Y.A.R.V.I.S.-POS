// Ventana de Configuración del Administrador.
// Coordina la identidad de la tienda (FormularioIdentidad), la seguridad del
// admin (FormularioSeguridad), la apariencia/tema (FormularioApariencia) y el
// Módulo de Importación Inteligente de tickets (ModuloImportacion).
// Usa useDatosAdmin para cargar/guardar los datos y useAccionesParser para el parseo.
import { useDatosAdmin } from "./hooks/useDatosAdmin";
import { useAccionesParser } from "./hooks/useAccionesParser";
import EncabezadoConfiguracion from "./components/EncabezadoConfiguracion";
import FormularioIdentidad from "./components/FormularioIdentidad";
import FormularioSeguridad from "./components/FormularioSeguridad";
import FormularioApariencia from "./components/FormularioApariencia";
import ModuloImportacion from "./components/importarmodulo/ModuloImportacion";

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

  const {
    isAnalyzing,
    showBatchProcessor,
    setShowBatchProcessor,
    isSyncing,
    syncResult,
    handleFileSelect,
    handleGuardarTicket,
    handleTrainIA,
    handleSyncEmbeddings,
    handleChangeMode,
  } = useAccionesParser();

  return (
    <div className="flex-1 space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500 max-w-5xl mx-auto w-full">
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

      <ModuloImportacion
        isAnalyzing={isAnalyzing}
        isSyncing={isSyncing}
        syncResult={syncResult}
        showBatchProcessor={showBatchProcessor}
        setShowBatchProcessor={setShowBatchProcessor}
        onFileSelect={handleFileSelect}
        onGuardarTicket={handleGuardarTicket}
        onTrainIA={handleTrainIA}
        onSyncEmbeddings={handleSyncEmbeddings}
        onChangeMode={handleChangeMode}
      />
    </div>
  );
};

export default Configuracion;