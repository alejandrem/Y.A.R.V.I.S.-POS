import { useState, useCallback, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";

interface BatchProcessorProps {
  onVolver: () => void;
  initialFolder?: string;
}

interface ArchivoCarpeta {
  nombre: string;
  ruta: string;
  tamano: number;
  preview: string;
}

interface ProgressData {
  type: "progress" | "complete";
  procesados: number;
  total: number;
  exitosos: number;
  errores: number;
  ventas_creadas?: number;
  items_insertados?: number;
  productos_nuevos?: number;
  productos_existentes?: number;
  duplicados_detectados?: number;
  productos_nuevos_lista?: string[];
  resumen_ventas?: any[];
}

interface VinculacionResult {
  vinculados: any[];
  sin_vincular: any[];
  estadisticas: {
    total_parseados: number;
    exactos: number;
    por_embedding: number;
    sin_vincular: number;
  };
}

const BatchProcessor = ({ onVolver, initialFolder }: BatchProcessorProps) => {
  const [selectedFolder, setSelectedFolder] = useState(initialFolder || "");
  const [archivos, setArchivos] = useState<ArchivoCarpeta[]>([]);
  const [isLoadingFiles, setIsLoadingFiles] = useState(false);
  const [selectedFilePreview, setSelectedFilePreview] = useState<ArchivoCarpeta | null>(null);
  const [isProcessing, setIsProcessing] = useState(false);
  const [progress, setProgress] = useState<ProgressData | null>(null);
  const [isComplete, setIsComplete] = useState(false);
  const [error, setError] = useState("");
  const [vinculacionResult, setVinculacionResult] = useState<VinculacionResult | null>(null);
  const [isVinculando, setIsVinculando] = useState(false);
  const [dbPath, setDbPath] = useState("");
  const unlistenRef = useRef<(() => void) | null>(null);

  useEffect(() => {
    if (initialFolder) {
      setSelectedFolder(initialFolder);
    }
  }, [initialFolder]);

  useEffect(() => {
    invoke<string>("get_db_path").then(setDbPath).catch(() => {});
    return () => {
      if (unlistenRef.current) {
        unlistenRef.current();
      }
    };
  }, []);

  useEffect(() => {
    if (!selectedFolder) {
      setArchivos([]);
      setSelectedFilePreview(null);
      return;
    }

    setIsLoadingFiles(true);
    setError("");
    invoke<ArchivoCarpeta[]>("listar_archivos_carpeta", { carpeta: selectedFolder })
      .then((files) => {
        setArchivos(files);
        setSelectedFilePreview(files.length > 0 ? files[0] : null);
      })
      .catch((err) => {
        setError(err || "Error al listar archivos");
        setArchivos([]);
      })
      .finally(() => setIsLoadingFiles(false));
  }, [selectedFolder]);

  const handleSelectFolder = async () => {
    try {
      const selected = await open({ directory: true, multiple: false });
      if (selected) {
        setSelectedFolder(selected as string);
        setProgress(null);
        setIsComplete(false);
        setError("");
      }
    } catch (err) {
      console.error("Error selecting folder:", err);
    }
  };

  const handleStartProcessing = useCallback(async () => {
    if (!selectedFolder) return;

    setIsProcessing(true);
    setProgress(null);
    setIsComplete(false);
    setError("");

    try {
      const unlisten = await listen<ProgressData>("batch-progress", (event) => {
        const data = event.payload;
        setProgress(data);

        if (data.type === "complete") {
          setIsComplete(true);
          setIsProcessing(false);
        }
      });
      unlistenRef.current = unlisten;

      await invoke("parsear_carpeta_stream", {
        carpeta: selectedFolder,
        mapeo: { cantidad: 0, producto: [1, -4], precio_unitario: -3, total: -1 },
        dbPath: dbPath,
      });

      unlisten();
      unlistenRef.current = null;
    } catch (err: any) {
      setError(err.message || err || "Error desconocido");
      setIsProcessing(false);
      if (unlistenRef.current) {
        unlistenRef.current();
        unlistenRef.current = null;
      }
    }
  }, [selectedFolder, dbPath]);

  const porcentaje = progress ? Math.round((progress.procesados / progress.total) * 100) : 0;

  const handleVincular = async () => {
    if (!progress?.productos_nuevos_lista || progress.productos_nuevos_lista.length === 0) return;

    setIsVinculando(true);
    try {
      const productos = progress.productos_nuevos_lista.map((nombre) => ({
        producto: nombre,
        precio_unitario: 0,
      }));

      const result = await invoke("vincular_inventario", {
        productos,
        dbPath: dbPath,
        umbral: 0.85,
      });

      setVinculacionResult(result as VinculacionResult);
    } catch (err: any) {
      console.error("Error vinculando:", err);
    } finally {
      setIsVinculando(false);
    }
  };

  const formatTamano = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  return (
    <div className="space-y-6 animate-in fade-in duration-500">
      <div className="flex items-center gap-4 mb-6">
        <button
          onClick={onVolver}
          className="w-10 h-10 rounded-xl bg-neutral-100 flex items-center justify-center hover:bg-neutral-200 transition-colors"
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round">
            <path d="m15 18-6-6 6-6" />
          </svg>
        </button>
        <div>
          <h3 className="text-sm font-black text-neutral-900 uppercase tracking-tighter">Procesamiento Masivo de Tickets</h3>
          <p className="text-[9px] font-bold text-neutral-400 uppercase tracking-widest">Importación por Lote con SSE en Tiempo Real</p>
        </div>
      </div>

      {!isProcessing && !isComplete && (
        <div className="bg-neutral-50 p-8 rounded-[2.5rem] border border-neutral-100 space-y-6">
          <h4 className="text-[10px] font-black text-neutral-400 uppercase tracking-[0.4em]">Seleccionar Carpeta de Tickets</h4>
          
          <div className="flex gap-4">
            <button
              onClick={handleSelectFolder}
              className="flex-1 bg-white border-2 border-dashed border-neutral-200 py-8 rounded-2xl text-center hover:border-neutral-400 transition-colors group"
            >
              <svg className="mx-auto mb-3 text-neutral-300 group-hover:text-neutral-500 transition-colors" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z" />
              </svg>
              <p className="text-[10px] font-black text-neutral-400 uppercase tracking-widest">
                {selectedFolder || "Click para seleccionar carpeta"}
              </p>
            </button>
          </div>

          {isLoadingFiles && (
            <div className="flex items-center justify-center py-8">
              <div className="w-6 h-6 border-2 border-neutral-300 border-t-neutral-900 rounded-full animate-spin" />
              <span className="ml-3 text-[10px] font-black text-neutral-400 uppercase tracking-widest">Escaneando archivos...</span>
            </div>
          )}

          {!isLoadingFiles && archivos.length > 0 && (
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <h4 className="text-[10px] font-black text-neutral-400 uppercase tracking-[0.4em]">
                  {archivos.length} archivo{archivos.length !== 1 ? "s" : ""} .txt encontrado{archivos.length !== 1 ? "s" : ""}
                </h4>
                <span className="text-[9px] font-bold text-neutral-400">
                  {formatTamano(archivos.reduce((acc, a) => acc + a.tamano, 0))} total
                </span>
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="bg-white rounded-2xl border border-neutral-100 max-h-72 overflow-y-auto">
                  {archivos.map((archivo, i) => (
                    <button
                      key={i}
                      onClick={() => setSelectedFilePreview(archivo)}
                      className={`w-full text-left px-4 py-3 border-b border-neutral-50 hover:bg-neutral-50 transition-colors ${
                        selectedFilePreview?.ruta === archivo.ruta ? "bg-neutral-100 border-l-2 border-l-neutral-900" : ""
                      }`}
                    >
                      <div className="flex items-center justify-between">
                        <span className="text-[10px] font-black text-neutral-900 truncate">{archivo.nombre}</span>
                        <span className="text-[8px] font-bold text-neutral-400 ml-2 shrink-0">{formatTamano(archivo.tamano)}</span>
                      </div>
                    </button>
                  ))}
                </div>

                <div className="bg-white rounded-2xl border border-neutral-100 p-4">
                  {selectedFilePreview ? (
                    <div className="space-y-3">
                      <div className="flex items-center justify-between border-b border-neutral-100 pb-2">
                        <span className="text-[10px] font-black text-neutral-900">{selectedFilePreview.nombre}</span>
                        <span className="text-[8px] font-bold text-neutral-400">{formatTamano(selectedFilePreview.tamano)}</span>
                      </div>
                      <pre className="text-[9px] font-mono text-neutral-700 whitespace-pre-wrap leading-relaxed bg-neutral-50 rounded-xl p-3 max-h-52 overflow-y-auto">
                        {selectedFilePreview.preview || "Sin preview disponible"}
                      </pre>
                    </div>
                  ) : (
                    <div className="flex items-center justify-center h-32">
                      <span className="text-[9px] font-bold text-neutral-300 uppercase">Selecciona un archivo para previsualizar</span>
                    </div>
                  )}
                </div>
              </div>
            </div>
          )}

          {!isLoadingFiles && selectedFolder && archivos.length === 0 && !error && (
            <div className="text-center py-6">
              <p className="text-[10px] font-black text-neutral-400 uppercase tracking-widest">No se encontraron archivos .txt en esta carpeta</p>
            </div>
          )}

          {selectedFolder && archivos.length > 0 && (
            <button
              onClick={handleStartProcessing}
              className="w-full bg-neutral-900 text-white py-5 rounded-2xl text-[11px] font-black uppercase tracking-[0.3em] hover:scale-[1.02] active:scale-95 transition-all shadow-xl"
            >
              Iniciar Procesamiento ({archivos.length} archivos)
            </button>
          )}
        </div>
      )}

      {(isProcessing || isComplete) && progress && (
        <div className="bg-white rounded-[2.5rem] border border-neutral-100 shadow-xl overflow-hidden border-t-4 border-t-neutral-900">
          <div className="p-8 border-b border-neutral-50 bg-neutral-50/30">
            <div className="flex items-center justify-between mb-4">
              <h4 className="text-sm font-black text-neutral-900 uppercase tracking-tighter">
                {isComplete ? "Procesamiento Completado" : "Procesando Tickets..."}
              </h4>
              <span className={`text-[10px] font-black px-3 py-1 rounded-full ${isComplete ? 'bg-green-100 text-green-700' : 'bg-neutral-100 text-neutral-600'}`}>
                {porcentaje}%
              </span>
            </div>

            <div className="relative h-4 bg-neutral-100 rounded-full overflow-hidden">
              <div
                className={`absolute left-0 top-0 h-full rounded-full transition-all duration-300 ${isComplete ? 'bg-green-500' : 'bg-neutral-900'}`}
                style={{ width: `${porcentaje}%` }}
              />
              <div className="absolute inset-0 flex items-center justify-center">
                <span className="text-[8px] font-black text-white drop-shadow-sm">
                  {progress.procesados} / {progress.total}
                </span>
              </div>
            </div>
          </div>

          <div className="p-8 grid grid-cols-2 md:grid-cols-4 gap-4">
            <StatCard label="Procesados" value={progress.procesados} total={progress.total} icon="📁" color="neutral" />
            <StatCard label="Exitosos" value={progress.exitosos} icon="✅" color="green" />
            <StatCard label="Errores" value={progress.errores} icon="⚠️" color={progress.errores > 0 ? "red" : "neutral"} />
            <StatCard label="Ventas Creadas" value={progress.ventas_creadas || 0} icon="💰" color="blue" />
          </div>

          <div className="px-8 pb-8 grid grid-cols-2 md:grid-cols-4 gap-4">
            <StatCard label="Items Insertados" value={progress.items_insertados || 0} icon="📦" color="purple" />
            <StatCard label="Productos Nuevos" value={progress.productos_nuevos || 0} icon="🆕" color="amber" />
            <StatCard label="Productos Existentes" value={progress.productos_existentes || 0} icon="🔄" color="neutral" />
            <StatCard label="Duplicados" value={progress.duplicados_detectados || 0} icon="👯" color={progress.duplicados_detectados ? "yellow" : "neutral"} />
          </div>

          {isComplete && progress.productos_nuevos_lista && progress.productos_nuevos_lista.length > 0 && (
            <div className="px-8 pb-8">
              <div className="bg-amber-50 rounded-2xl p-6 border border-amber-100">
                <h5 className="text-[10px] font-black text-amber-700 uppercase tracking-[0.3em] mb-4">
                  Productos Nuevos Detectados ({progress.productos_nuevos_lista.length})
                </h5>
                <div className="flex flex-wrap gap-2 max-h-32 overflow-y-auto">
                  {progress.productos_nuevos_lista.slice(0, 50).map((nombre, i) => (
                    <span key={i} className="text-[9px] font-bold px-3 py-1.5 bg-white rounded-lg border border-amber-200 text-amber-800">
                      {nombre}
                    </span>
                  ))}
                  {progress.productos_nuevos_lista.length > 50 && (
                    <span className="text-[9px] font-bold px-3 py-1.5 text-amber-600">
                      +{progress.productos_nuevos_lista.length - 50} mas...
                    </span>
                  )}
                </div>
                <p className="text-[9px] text-amber-600 mt-3 font-bold">
                  Estos productos fueron insertados en la tabla de ventas. Puedes vincularlos al inventario desde la seccion de Inventario.
                </p>

                {!vinculacionResult && (
                  <button
                    onClick={handleVincular}
                    disabled={isVinculando}
                    className="mt-4 w-full bg-amber-500 text-white py-3 rounded-xl text-[10px] font-black uppercase tracking-widest hover:bg-amber-600 transition-colors disabled:opacity-50"
                  >
                    {isVinculando ? "Vinculando con inventario..." : "Vincular con Inventario Existente"}
                  </button>
                )}
              </div>
            </div>
          )}

          {vinculacionResult && (
            <div className="px-4 sm:px-8 pb-4 sm:pb-8 space-y-3 sm:space-y-4">
              <div className="bg-blue-50 rounded-xl sm:rounded-2xl p-4 sm:p-6 border border-blue-100">
                <h5 className="text-[9px] sm:text-[10px] font-black text-blue-700 uppercase tracking-[0.3em] mb-3 sm:mb-4">
                  Resultado de Vinculacion
                </h5>
                <div className="grid grid-cols-1 sm:grid-cols-3 gap-4 mb-4">
                  <div className="text-center">
                    <p className="text-2xl font-black text-green-600">{vinculacionResult.estadisticas.exactos}</p>
                    <p className="text-[8px] font-black text-green-700 uppercase">Exactos</p>
                  </div>
                  <div className="text-center">
                    <p className="text-2xl font-black text-blue-600">{vinculacionResult.estadisticas.por_embedding}</p>
                    <p className="text-[8px] font-black text-blue-700 uppercase">Por Similitud</p>
                  </div>
                  <div className="text-center">
                    <p className="text-2xl font-black text-amber-600">{vinculacionResult.estadisticas.sin_vincular}</p>
                    <p className="text-[8px] font-black text-amber-700 uppercase">Sin Vincular</p>
                  </div>
                </div>

                {vinculacionResult.sin_vincular.length > 0 && (
                  <div className="mt-4">
                    <p className="text-[9px] font-black text-amber-600 uppercase tracking-widest mb-2">
                      Requieren revision manual ({vinculacionResult.sin_vincular.length})
                    </p>
                    <div className="flex flex-wrap gap-2 max-h-24 overflow-y-auto">
                      {vinculacionResult.sin_vincular.slice(0, 30).map((item, i) => (
                        <span key={i} className="text-[9px] font-bold px-3 py-1.5 bg-white rounded-lg border border-amber-200 text-amber-800">
                          {item.producto_parseado.producto}
                        </span>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            </div>
          )}

          {isComplete && (
            <div className="px-8 pb-8">
              <button
                onClick={() => {
                  setIsComplete(false);
                  setProgress(null);
                  setSelectedFolder("");
                  setArchivos([]);
                  setSelectedFilePreview(null);
                }}
                className="w-full bg-neutral-900 text-white py-4 rounded-2xl text-[10px] font-black uppercase tracking-[0.3em] hover:scale-[1.02] active:scale-95 transition-all"
              >
                Procesar Otra Carpeta
              </button>
            </div>
          )}
        </div>
      )}

      {error && (
        <div className="bg-red-50 p-6 rounded-2xl border border-red-100">
          <p className="text-[10px] font-black text-red-600 uppercase tracking-widest">Error</p>
          <p className="text-[11px] text-red-700 mt-2">{error}</p>
        </div>
      )}
    </div>
  );
};

const StatCard = ({
  label,
  value,
  total,
  icon,
  color,
}: {
  label: string;
  value: number;
  total?: number;
  icon: string;
  color: string;
}) => {
  const colorClasses: Record<string, string> = {
    neutral: "bg-neutral-50 text-neutral-700",
    green: "bg-green-50 text-green-700",
    red: "bg-red-50 text-red-700",
    blue: "bg-blue-50 text-blue-700",
    purple: "bg-purple-50 text-purple-700",
    amber: "bg-amber-50 text-amber-700",
    yellow: "bg-yellow-50 text-yellow-700",
  };

  return (
    <div className={`p-4 rounded-2xl ${colorClasses[color] || colorClasses.neutral}`}>
      <div className="flex items-center gap-2 mb-2">
        <span className="text-base">{icon}</span>
        <span className="text-[8px] font-black uppercase tracking-widest opacity-60">{label}</span>
      </div>
      <p className="text-xl font-black">
        {value.toLocaleString()}
        {total !== undefined && (
          <span className="text-xs font-bold opacity-40"> / {total.toLocaleString()}</span>
        )}
      </p>
    </div>
  );
};

export default BatchProcessor;
