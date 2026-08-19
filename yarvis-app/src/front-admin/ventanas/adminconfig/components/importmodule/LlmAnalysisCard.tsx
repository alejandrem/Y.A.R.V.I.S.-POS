interface LlmAnalysisCardProps {
  analysis: any;
}

const LlmAnalysisCard = ({ analysis }: LlmAnalysisCardProps) => {
  return (
    <div className="space-y-3 animate-in fade-in duration-500">
      <span className="text-[9px] font-black text-neutral-400 uppercase tracking-widest px-2 flex items-center gap-2">
        <div className="w-1.5 h-1.5 rounded-full bg-blue-500"></div>
        Análisis del Motor de IA
      </span>
      <div className="bg-neutral-900 rounded-3xl p-6 text-white shadow-inner">
        <div className="grid grid-cols-2 md:grid-cols-4 gap-2 sm:gap-4 mb-4">
          <div className="bg-white/5 rounded-xl p-3">
            <p className="text-[8px] font-black text-neutral-500 uppercase tracking-widest">Formato</p>
            <p className="text-[11px] font-bold mt-1 truncate">{analysis.mapeo.formato_detectado}</p>
          </div>
          <div className="bg-white/5 rounded-xl p-3">
            <p className="text-[8px] font-black text-neutral-500 uppercase tracking-widest">Confianza</p>
            <p className={`text-[11px] font-bold mt-1 ${analysis.confianza >= 0.8 ? 'text-green-400' : 'text-yellow-400'}`}>
              {(analysis.confianza * 100).toFixed(0)}%
            </p>
          </div>
          <div className="bg-white/5 rounded-xl p-3">
            <p className="text-[8px] font-black text-neutral-500 uppercase tracking-widest">Columnas</p>
            <p className="text-[11px] font-bold mt-1">{analysis.mapeo.total_columnas}</p>
          </div>
          <div className="bg-white/5 rounded-xl p-3">
            <p className="text-[8px] font-black text-neutral-500 uppercase tracking-widest">Delimitador</p>
            <p className="text-[11px] font-bold mt-1">{analysis.mapeo.delimitador}</p>
          </div>
        </div>
        <div className="flex gap-3 mb-4">
          {analysis.mapeo.tiene_descuento && (
            <span className="text-[9px] font-bold px-2 py-1 rounded-lg bg-yellow-500/20 text-yellow-300">TIENE DESCUENTOS</span>
          )}
          {analysis.mapeo.tiene_iva && (
            <span className="text-[9px] font-bold px-2 py-1 rounded-lg bg-blue-500/20 text-blue-300">TIENE IVA</span>
          )}
        </div>
        <div className="bg-white/5 rounded-xl p-4">
          <p className="text-[8px] font-black text-neutral-500 uppercase tracking-widest mb-2">Notas del Análisis</p>
          <p className="text-[11px] text-neutral-300 leading-relaxed">{analysis.notas}</p>
        </div>
        <div className="mt-3 bg-white/5 rounded-xl p-3">
          <p className="text-[8px] font-black text-neutral-500 uppercase tracking-widest mb-2">Mapeo de Columnas</p>
          <div className="flex flex-wrap gap-2">
            {Object.entries(analysis.mapeo.columnas as Record<string, any>).map(([key, val]) => (
              <span key={key} className={`text-[9px] font-bold px-2 py-1 rounded-lg ${val !== null ? 'bg-white/10 text-white' : 'bg-white/5 text-neutral-600 line-through'}`}>
                {key}: {val !== null ? `col ${val}` : 'N/A'}
              </span>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};

export default LlmAnalysisCard;