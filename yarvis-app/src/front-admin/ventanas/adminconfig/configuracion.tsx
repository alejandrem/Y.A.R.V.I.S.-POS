import { useAdminData } from "./hooks/useAdminData";
import { useParserActions } from "./hooks/useParserActions";
import ConfigHeader from "./components/ConfigHeader";
import IdentityForm from "./components/IdentityForm";
import SecurityForm from "./components/SecurityForm";
import AppearanceForm from "./components/AppearanceForm";
import ImportModule from "./components/importmodule/ImportModule";

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
  } = useAdminData(adminName, storeName, initialLocation, initialCp);

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
  } = useParserActions();

  return (
    <div className="flex-1 space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500 max-w-5xl mx-auto w-full">
      <ConfigHeader successMessage={successMessage} />

      <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
        <IdentityForm
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
          <SecurityForm
            currentPass={currentPass}
            setCurrentPass={setCurrentPass}
            passwordChanged={passwordChanged}
            setPasswordChanged={setPasswordChanged}
            onSave={handleUpdate}
          />

          <AppearanceForm />
        </div>
      </div>

      <ImportModule
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