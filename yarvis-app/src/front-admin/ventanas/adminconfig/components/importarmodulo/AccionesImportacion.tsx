// Renderiza la botonera del módulo de importación: permite seleccionar un archivo 
// (TXT/CSV), pedirle a la IA que lo analice, limpiar la vista y guardar los datos.
import React from "react";
import { useParserContext } from "../../../../../hooks/ParserContext";

interface ImportActionsProps {
  isAnalyzing: boolean;
  isSyncing: boolean;
  syncResult: string;
  onSyncEmbeddings: () => void;
  onTrainIA: () => void;
}

const ImportActions = React.memo(({
  isAnalyzing,
  isSyncing,
  syncResult,
  onSyncEmbeddings,
  onTrainIA,
}: ImportActionsProps) => {
  const { parserMode, selectedPath, parsedItems } = useParserContext();

  return (
    <>
      {parserMode === 'catalogo' && (
        <div className="pt-2">
          <button
            disabled={isSyncing}
            onClick={onSyncEmbeddings}
            className={`w-full py-4 rounded-2xl text-[11px] font-black uppercase tracking-[0.3em] transition-all shadow-2xl flex items-center justify-center gap-3 ${!isSyncing
                ? 'bg-blue-600 text-white hover:scale-[1.02] active:scale-95 shadow-blue-200'
                : 'bg-neutral-100 text-neutral-300 cursor-wait shadow-none'
              }`}
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"><path d="M21 12a9 9 0 1 1-9-9" /><path d="M21 3v6h-6" /></svg>
            {isSyncing ? 'Sincronizando...' : 'Sincronizar embeddings del catálogo'}
          </button>
          {syncResult && (
            <p className="text-[10px] font-bold text-neutral-600 text-center pt-3">{syncResult}</p>
          )}
        </div>
      )}

      {parserMode !== 'entrenar IA' && (
        <div className="flex justify-center pt-2">
          <button
            disabled={!selectedPath || parsedItems.length === 0 || isAnalyzing}
            onClick={onTrainIA}
            className={`w-full py-5 rounded-2xl text-[11px] font-black uppercase tracking-[0.3em] transition-all shadow-2xl flex items-center justify-center gap-3 ${selectedPath && parsedItems.length > 0 && !isAnalyzing
                ? 'bg-neutral-900 text-white hover:scale-[1.02] active:scale-95 shadow-neutral-200'
                : 'bg-neutral-100 text-neutral-300 cursor-not-allowed shadow-none'
              }`}
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"><path d="m12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275L12 3Z" /></svg>
            {parserMode === 'catalogo' ? 'Entrenar IA con Catálogo' : 'Insertar Carpeta'}
          </button>
        </div>
      )}
    </>
  );
});

ImportActions.displayName = "ImportActions";

export default ImportActions;