import { useState, useMemo, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { LLMAnalysis, ColumnMapping } from "../../types";

interface ColumnMapperProps {
  onGuardarTicket: (items: any[], analysis: LLMAnalysis) => void;
  onPreviewUpdate: (items: any[]) => void;
  fileContent?: string;
  selectedPath?: string;
}

const COLUMN_OPTIONS = [
  { value: -5, label: "-5 (5ta desde derecha)" },
  { value: -4, label: "-4 (4ta desde derecha)" },
  { value: -3, label: "-3 (3ra desde derecha)" },
  { value: -2, label: "-2 (Penúltima)" },
  { value: -1, label: "-1 (Última)" },
  { value: 0, label: "0 (1ra desde izquierda)" },
  { value: 1, label: "1 (2da desde izquierda)" },
  { value: 2, label: "2 (3ra desde izquierda)" },
  { value: 3, label: "3 (4ta desde izquierda)" },
  { value: 4, label: "4 (5ta desde izquierda)" },
  { value: 5, label: "5 (6ta desde izquierda)" },
  { value: 6, label: "6 (7ma desde izquierda)" },
  { value: 7, label: "7 (8va desde izquierda)" },
  { value: 8, label: "8 (9na desde izquierda)" },
  { value: 9, label: "9 (10ma desde izquierda)" },
];

const ColumnMapper = ({ onGuardarTicket, onPreviewUpdate, fileContent: initialFileContent }: ColumnMapperProps) => {
  const [fileContent, setFileContent] = useState(initialFileContent || "");
  const [analysis, setAnalysis] = useState<LLMAnalysis | null>(null);
  const [isAnalyzing, setIsAnalyzing] = useState(false);

  const [columnas, setColumnas] = useState<ColumnMapping>({
    cantidad: null,
    producto: null,
    precio_unitario: null,
    total: null,
    descuento: null,
  });

  useEffect(() => {
    if (initialFileContent) setFileContent(initialFileContent);
  }, [initialFileContent]);

  const handleAnalizar = async () => {
    if (!fileContent) return;
    setIsAnalyzing(true);
    try {
      const resultado = await invoke("analizar_ticket_con_ia", {
        texto: fileContent,
      }) as LLMAnalysis;
      setAnalysis(resultado);
      setColumnas({
        cantidad: resultado.mapeo.columnas.cantidad,
        producto: Array.isArray(resultado.mapeo.columnas.producto)
          ? resultado.mapeo.columnas.producto
          : resultado.mapeo.columnas.producto !== null
            ? [resultado.mapeo.columnas.producto]
            : null,
        precio_unitario: resultado.mapeo.columnas.precio_unitario,
        total: resultado.mapeo.columnas.total,
        descuento: resultado.mapeo.columnas.descuento,
      });
    } catch (err) {
      console.error("Error en análisis IA:", err);
      alert("Error al analizar: " + err);
    } finally {
      setIsAnalyzing(false);
    }
  };

  const handleReanalizar = () => {
    setAnalysis(null);
    handleAnalizar();
  };

  const handleAceptar = () => {
    if (!analysis) return;
    const analysisCorregido = {
      ...analysis,
      mapeo: {
        ...analysis.mapeo,
        columnas: { ...columnas },
      },
    };
    onGuardarTicket(previewItems, analysisCorregido);
  };

  const previewItems = useMemo(() => {
    if (!analysis?.ejemplo_parseado) return [];
    return analysis.ejemplo_parseado;
  }, [analysis]);

  useEffect(() => {
    onPreviewUpdate(previewItems);
  }, [previewItems]);

  const productoRangeOptions = COLUMN_OPTIONS.flatMap((opt) =>
    COLUMN_OPTIONS.filter((opt2) => opt2.value >= opt.value).map((opt2) => ({
      value: `${opt.value}:${opt2.value}`,
      label: `${opt.value} → ${opt2.value}`,
    }))
  );

  return (
    <div className="space-y-4 animate-in fade-in duration-500">
      {/* BOTÓN ANALIZAR (si hay contenido pero no hay análisis) */}
      {fileContent && !analysis && (
        <div className="flex justify-center">
          <button
            onClick={handleAnalizar}
            disabled={isAnalyzing}
            className={`px-8 py-4 rounded-2xl text-[11px] font-black uppercase tracking-[0.3em] transition-all shadow-xl flex items-center gap-3 ${
              isAnalyzing
                ? "bg-neutral-400 text-white cursor-not-allowed"
                : "bg-neutral-900 text-white hover:scale-[1.02] active:scale-95"
            }`}
          >
            {isAnalyzing ? (
              <>
                <svg className="animate-spin h-4 w-4" viewBox="0 0 24 24">
                  <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" fill="none" />
                  <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
                </svg>
                Analizando con IA...
              </>
            ) : (
              <>
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"><path d="m12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275L12 3Z"/></svg>
                Analizar con IA
              </>
            )}
          </button>
        </div>
      )}

      {/* MAPEO DETECTADO */}
      {analysis && (
        <div className="bg-white rounded-3xl border border-neutral-100 p-6 shadow-sm space-y-5 animate-in fade-in duration-500">
          <div className="flex items-center justify-between">
            <span className="text-[9px] font-black text-neutral-400 uppercase tracking-widest flex items-center gap-2">
              <div className="w-1.5 h-1.5 rounded-full bg-blue-500"></div>
              Mapeo Detectado por IA
            </span>
            {analysis.reintentado_con && (
              <span className="text-[8px] font-bold px-2 py-1 rounded-lg bg-purple-100 text-purple-700 uppercase">
                Reintentado con 1.7B
              </span>
            )}
          </div>

          {/* BARRA DE CONFIANZA */}
          <div className="bg-neutral-50 rounded-2xl p-4">
            <div className="flex items-center justify-between mb-2">
              <span className="text-[9px] font-black text-neutral-500 uppercase tracking-widest">Confianza</span>
              <span className={`text-sm font-black ${analysis.confianza >= 0.8 ? "text-green-600" : "text-yellow-500"}`}>
                {(analysis.confianza * 100).toFixed(0)}%
              </span>
            </div>
            <div className="w-full h-2 bg-neutral-200 rounded-full overflow-hidden">
              <div
                className={`h-full rounded-full transition-all duration-700 ${
                  analysis.confianza >= 0.8 ? "bg-green-500" : "bg-yellow-400"
                }`}
                style={{ width: `${analysis.confianza * 100}%` }}
              />
            </div>
          </div>

          {/* INFO RÁPIDA */}
          <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
            <div className="bg-neutral-50 rounded-xl p-3">
              <p className="text-[8px] font-black text-neutral-400 uppercase tracking-widest">Formato</p>
              <p className="text-[10px] font-bold mt-1 text-neutral-900">{analysis.mapeo.formato_detectado}</p>
            </div>
            <div className="bg-neutral-50 rounded-xl p-3">
              <p className="text-[8px] font-black text-neutral-400 uppercase tracking-widest">Delimitador</p>
              <p className="text-[10px] font-bold mt-1 text-neutral-900">{analysis.mapeo.delimitador}</p>
            </div>
            <div className="bg-neutral-50 rounded-xl p-3">
              <p className="text-[8px] font-black text-neutral-400 uppercase tracking-widest">Moneda</p>
              <p className="text-[10px] font-bold mt-1 text-neutral-900">{analysis.mapeo.moneda}</p>
            </div>
            <div className="bg-neutral-50 rounded-xl p-3">
              <p className="text-[8px] font-black text-neutral-400 uppercase tracking-widest">Total Columnas</p>
              <p className="text-[10px] font-bold mt-1 text-neutral-900">{analysis.mapeo.total_columnas}</p>
            </div>
          </div>

          {/* SELECTORES DE COLUMNAS */}
          <div className="bg-neutral-50 rounded-2xl p-5 space-y-4">
            <p className="text-[9px] font-black text-neutral-500 uppercase tracking-widest">
              Ajustar Mapeo de Columnas
            </p>

            <div className="flex items-center gap-4">
              <label className="w-32 text-[10px] font-black text-neutral-700 uppercase">Cantidad</label>
              <select
                value={columnas.cantidad ?? ""}
                onChange={(e) => setColumnas({ ...columnas, cantidad: e.target.value ? Number(e.target.value) : null })}
                className="flex-1 bg-white border border-neutral-200 rounded-xl px-4 py-2.5 text-[10px] font-bold focus:outline-none focus:ring-2 focus:ring-neutral-900/10"
              >
                <option value="">No detectada</option>
                {COLUMN_OPTIONS.map((opt) => (
                  <option key={opt.value} value={opt.value}>{opt.label}</option>
                ))}
              </select>
            </div>

            <div className="flex items-center gap-4">
              <label className="w-32 text-[10px] font-black text-neutral-700 uppercase">Producto</label>
              <select
                value={columnas.producto ? `${columnas.producto[0]}:${columnas.producto[columnas.producto.length - 1]}` : ""}
                onChange={(e) => {
                  if (e.target.value) {
                    const [ini, fin] = e.target.value.split(":").map(Number);
                    const range = [];
                    for (let i = ini; i <= fin; i++) range.push(i);
                    setColumnas({ ...columnas, producto: range });
                  } else {
                    setColumnas({ ...columnas, producto: null });
                  }
                }}
                className="flex-1 bg-white border border-neutral-200 rounded-xl px-4 py-2.5 text-[10px] font-bold focus:outline-none focus:ring-2 focus:ring-neutral-900/10"
              >
                <option value="">No detectada</option>
                {productoRangeOptions.map((opt) => (
                  <option key={opt.value} value={opt.value}>{opt.label}</option>
                ))}
              </select>
            </div>

            <div className="flex items-center gap-4">
              <label className="w-32 text-[10px] font-black text-neutral-700 uppercase">Precio Unit.</label>
              <select
                value={columnas.precio_unitario ?? ""}
                onChange={(e) => setColumnas({ ...columnas, precio_unitario: e.target.value ? Number(e.target.value) : null })}
                className="flex-1 bg-white border border-neutral-200 rounded-xl px-4 py-2.5 text-[10px] font-bold focus:outline-none focus:ring-2 focus:ring-neutral-900/10"
              >
                <option value="">No detectada</option>
                {COLUMN_OPTIONS.map((opt) => (
                  <option key={opt.value} value={opt.value}>{opt.label}</option>
                ))}
              </select>
            </div>

            <div className="flex items-center gap-4">
              <label className="w-32 text-[10px] font-black text-neutral-700 uppercase">Total</label>
              <select
                value={columnas.total ?? ""}
                onChange={(e) => setColumnas({ ...columnas, total: e.target.value ? Number(e.target.value) : null })}
                className="flex-1 bg-white border border-neutral-200 rounded-xl px-4 py-2.5 text-[10px] font-bold focus:outline-none focus:ring-2 focus:ring-neutral-900/10"
              >
                <option value="">No detectada</option>
                {COLUMN_OPTIONS.map((opt) => (
                  <option key={opt.value} value={opt.value}>{opt.label}</option>
                ))}
              </select>
            </div>

            <div className="flex items-center gap-4">
              <label className="w-32 text-[10px] font-black text-neutral-700 uppercase">Descuento</label>
              <select
                value={columnas.descuento ?? ""}
                onChange={(e) => setColumnas({ ...columnas, descuento: e.target.value ? Number(e.target.value) : null })}
                className="flex-1 bg-white border border-neutral-200 rounded-xl px-4 py-2.5 text-[10px] font-bold focus:outline-none focus:ring-2 focus:ring-neutral-900/10"
              >
                <option value="">No detectada</option>
                {COLUMN_OPTIONS.map((opt) => (
                  <option key={opt.value} value={opt.value}>{opt.label}</option>
                ))}
              </select>
            </div>
          </div>

          {/* NOTAS */}
          {analysis.notas && (
            <div className="bg-blue-50 rounded-2xl p-4">
              <p className="text-[9px] font-black text-blue-500 uppercase tracking-widest mb-1">Notas IA</p>
              <p className="text-[10px] text-blue-700">{analysis.notas}</p>
            </div>
          )}

          {/* BOTONES DE ACCIÓN */}
          <div className="flex gap-4">
            <button
              onClick={handleReanalizar}
              disabled={isAnalyzing}
              className="flex-1 py-3 rounded-2xl text-[10px] font-black uppercase tracking-[0.2em] border-2 border-neutral-200 text-neutral-500 hover:border-neutral-900 hover:text-neutral-900 transition-all"
            >
              Reanalizar
            </button>
            <button
              onClick={handleAceptar}
              disabled={previewItems.length === 0}
              className={`flex-1 py-3 rounded-2xl text-[10px] font-black uppercase tracking-[0.2em] transition-all shadow-xl ${
                previewItems.length > 0
                  ? "bg-neutral-900 text-white hover:scale-[1.02] active:scale-95 shadow-neutral-200"
                  : "bg-neutral-200 text-neutral-400 cursor-not-allowed shadow-none"
              }`}
            >
              Guardar Ticket ({previewItems.length} items)
            </button>
          </div>
        </div>
      )}
    </div>
  );
};

export default ColumnMapper;
