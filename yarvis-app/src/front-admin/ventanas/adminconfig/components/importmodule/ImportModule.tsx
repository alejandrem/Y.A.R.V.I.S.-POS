import { useParserContext } from "../../../../../hooks/ParserContext";
import ColumnMapper from "../../../parseadodetickets/ColumnMapper";
import BatchProcessor from "../../../parseadodetickets/BatchProcessor";
import CatalogosParseados from "../../../parseadodetickets/CatalogosParseados";
import ImportHeader from "./ImportHeader";
import ImportStatus from "./ImportStatus";
import RawDataViewer from "./RawDataViewer";
import PreviewTable from "./PreviewTable";
import LlmAnalysisCard from "./LlmAnalysisCard";
import ImportActions from "./ImportActions";

interface ImportModuleProps {
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

const ImportModule = ({
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
}: ImportModuleProps) => {
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
      <ImportHeader onChangeMode={onChangeMode} />

      <ImportStatus />

      {showBatchProcessor && parserMode === 'insertar' && (
        <div className="px-8 pb-8">
          <BatchProcessor onVolver={() => setShowBatchProcessor(false)} initialFolder={selectedPath} />
        </div>
      )}

      {!showBatchProcessor && (
        <div className="p-5 sm:p-10 grid grid-cols-1 lg:grid-cols-12 gap-5 sm:gap-10">
          <div className="lg:col-span-12 space-y-8">
            <RawDataViewer onFileSelect={onFileSelect} isAnalyzing={isAnalyzing} showBatchProcessor={showBatchProcessor} />

            <PreviewTable isAnalyzing={isAnalyzing} />

            {parserMode === 'entrenar IA' && showColumnMapper && fileContent && (
              <ColumnMapper
                onGuardarTicket={onGuardarTicket}
                onPreviewUpdate={setParsedItems}
                fileContent={fileContent}
                selectedPath={selectedPath}
              />
            )}

            {parserMode === 'entrenar IA' && llmAnalysis && (
              <LlmAnalysisCard analysis={llmAnalysis} />
            )}

            {parserMode === 'catalogo' && (
              <CatalogosParseados />
            )}

            <ImportActions
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

export default ImportModule;