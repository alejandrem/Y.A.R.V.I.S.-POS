import { useParserContext } from "../../../../../hooks/ParserContext";

interface ImportHeaderProps {
  onChangeMode: (m: "catalogo" | "entrenar IA" | "insertar") => void;
}

const ImportHeader = ({ onChangeMode }: ImportHeaderProps) => {
  const { parserMode } = useParserContext();

  return (
    <div className="p-8 border-b border-neutral-50 flex justify-between items-center bg-neutral-50/30">
      <div>
        <h3 className="text-sm font-black text-neutral-900 uppercase tracking-tighter">Módulo de Importación Inteligente</h3>
        <p className="text-[9px] font-bold text-neutral-400 uppercase tracking-widest">Parseador de Datos Raw & Catálogos</p>
      </div>
      <div className="flex bg-neutral-100 p-1 rounded-xl">
        {(['entrenar IA', 'catalogo', 'insertar'] as const).map((m) => (
          <button
            key={m}
            onClick={() => onChangeMode(m)}
            className={`px-4 py-2 rounded-lg text-[9px] font-black uppercase tracking-widest transition-all ${parserMode === m ? 'bg-white text-neutral-900 shadow-sm' : 'text-neutral-400 hover:text-neutral-600'}`}
          >
            {m}
          </button>
        ))}
      </div>
    </div>
  );
};

export default ImportHeader;