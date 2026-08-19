// Contenedor principal del módulo de importación inteligente. 
// Agrupa los botones de acción, el estado, las tablas de previsualización y el análisis de la IA.

import { useParserContext } from "../../../../../hooks/ParserContext";
import ColumnMapper from "../../../parseadodetickets/ColumnMapper";
import BatchProcessor from "../../../parseadodetickets/BatchProcessor";
import CatalogosParseados from "../../../parseadodetickets/CatalogosParseados";
import EncabezadoImportacion from "./EncabezadoImportacion";
import EstadoImportacion from "./EstadoImportacion";
import VistaDatosCrudos from "./VistaDatosCrudos";
import TablaPrevisualizacion from "./TablaPrevisualizacion";
import TarjetaAnalisisLlm from "./TarjetaAnalisisLlm";
import AccionesImportacion from "./AccionesImportacion";

interface ModuloImportacionProps {
  isAnalyzing: boolean;
  isSyncing: boolean;
  syncResult: string;
  showBatchProcessor: boolean;
  setShowBatchProcessor: (v: boolean) => void;
  onFileSelect: () => void;
  onGuardarTicket: (items: any[], analysis: any) => void;
  onTrainIA: () => void;
  onSyncEmbeddings: () => void;
  onChangeMode: (m: "catalogo" | "entrenar IA" | "insertar") => void;
}

const ModuloImportacion = ({
  isAnalyzing,
  isSyncing,
  syncResult,
  showBatchProcessor,
  setShowBatchProcessor,
  onFileSelect,
  onGuardarTicket,
  onTrainIA,
  onSyncEmbeddings,
  onChangeMode,
}: ModuloImportacionProps) => {
  const {
    parserMode,
    selectedPath,
    fileContent,
    showColumnMapper,
    llmAnalysis,
    setParsedItems,
  } = useParserContext();

  return (
    <div className="bg-white rounded-[2.5rem] border border-neutral-100 shadow-xl overflow-hidden mt-8 border-t-4 border-t-neutral-900">
      <EncabezadoImportacion onChangeMode={onChangeMode} />

      <EstadoImportacion />

      {showBatchProcessor && parserMode === 'insertar' && (
        <div className="px-8 pb-8">
          <BatchProcessor onVolver={() => setShowBatchProcessor(false)} initialFolder={selectedPath} />
        </div>
      )}

      {!showBatchProcessor && (
        <div className="p-5 sm:p-10 grid grid-cols-1 lg:grid-cols-12 gap-5 sm:gap-10">
          <div className="lg:col-span-12 space-y-8">
            <VistaDatosCrudos onFileSelect={onFileSelect} isAnalyzing={isAnalyzing} showBatchProcessor={showBatchProcessor} />

            <TablaPrevisualizacion isAnalyzing={isAnalyzing} />

            {parserMode === 'entrenar IA' && showColumnMapper && fileContent && (
              <ColumnMapper
                onGuardarTicket={onGuardarTicket}
                onPreviewUpdate={setParsedItems}
                fileContent={fileContent}
                selectedPath={selectedPath}
              />
            )}

            {parserMode === 'entrenar IA' && llmAnalysis && (
              <TarjetaAnalisisLlm analysis={llmAnalysis} />
            )}

            {parserMode === 'catalogo' && (
              <CatalogosParseados />
            )}

            <AccionesImportacion
              isAnalyzing={isAnalyzing}
              isSyncing={isSyncing}
              syncResult={syncResult}
              onSyncEmbeddings={onSyncEmbeddings}
              onTrainIA={onTrainIA}
            />
          </div>
        </div>
      )}
    </div>
  );
};

export default ModuloImportacion;