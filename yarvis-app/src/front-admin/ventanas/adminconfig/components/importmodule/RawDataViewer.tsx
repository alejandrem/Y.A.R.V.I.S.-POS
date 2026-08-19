import { useParserContext } from "../../../../../hooks/ParserContext";

interface RawDataViewerProps {
  isAnalyzing: boolean;
  showBatchProcessor: boolean;
  onFileSelect: () => void;
}

const RawDataViewer = ({ isAnalyzing, showBatchProcessor, onFileSelect }: RawDataViewerProps) => {
  const { selectedPath, fileContent, parserMode } = useParserContext();

  return (
    <div className="space-y-3">
      <div className="flex justify-between items-center px-2">
        <span className="text-[9px] font-black text-neutral-400 uppercase tracking-widest flex items-center gap-2">
          <div className="w-1.5 h-1.5 rounded-full bg-neutral-900"></div>
          {showBatchProcessor && parserMode === 'insertar' ? 'Procesamiento Masivo Activo' : 'Visualizador de Datos Raw'}
        </span>
        {!showBatchProcessor && (
          <button
            onClick={onFileSelect}
            disabled={isAnalyzing}
            className={`px-5 py-2 rounded-xl text-[9px] font-black uppercase tracking-widest transition-all shadow-lg shadow-neutral-200 ${
              isAnalyzing
                ? 'bg-neutral-400 text-white cursor-not-allowed'
                : 'bg-neutral-900 text-white hover:scale-105 active:scale-95'
            }`}
          >
            {isAnalyzing ? 'Analizando con IA...' : parserMode === 'insertar' ? 'Seleccionar Carpeta' : 'Cargar Archivo (.txt, .csv, .xlsx)'}
          </button>
        )}
      </div>
      {!showBatchProcessor && (
        <div className="w-full h-48 bg-neutral-900 rounded-3xl p-6 font-mono text-[11px] text-neutral-400 overflow-auto border border-neutral-800 shadow-inner custom-scrollbar">
          {selectedPath ? (
            <pre className="animate-in fade-in duration-700">{fileContent || "// Archivo vacío o sin datos legibles"}</pre>
          ) : (
            <div className="h-full flex flex-col items-center justify-center opacity-30 gap-3">
              <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/><polyline points="14 2 14 8 20 8"/></svg>
              <p className="uppercase tracking-[0.3em] text-[8px] font-black">Esperando entrada de datos...</p>
            </div>
          )}
        </div>
      )}
    </div>
  );
};

export default RawDataViewer;