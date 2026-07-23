import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useThemeContext } from "../../../hooks/ThemeContext";
import { useParserContext } from "../../../hooks/ParserContext";
import ColumnMapper from "../parseadodetickets/ColumnMapper";
import BatchProcessor from "../parseadodetickets/BatchProcessor";
import CatalogosParseados from "../parseadodetickets/CatalogosParseados";

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
  const [currentAdminName, setCurrentAdminName] = useState(adminName);
  const [currentStoreName, setCurrentStoreName] = useState(storeName);
  const [currentPass, setCurrentPass] = useState("");
  const [location, setLocation] = useState(initialLocation);
  const [cp, setCp] = useState(initialCp);
  const [successMessage, setSuccessMessage] = useState("");
  const { theme, setTheme } = useThemeContext();
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [showBatchProcessor, setShowBatchProcessor] = useState(false);

  const {
    parsedItems, setParsedItems,
    fileContent, setFileContent,
    selectedPath, setSelectedPath,
    parserMode, setParserMode,
    showColumnMapper, setShowColumnMapper,
    llmAnalysis, setLlmAnalysis,
    lastCatalogPath, setLastCatalogPath,
    lastCatalogItems, setLastCatalogItems,
    catalogParsed, setCatalogParsed,
    iaTrained, setIaTrained,
    ticketsParsed, setTicketsParsed,
    ticketsCount, setTicketsCount,
    ticketsGuardados, setTicketsGuardados,
  } = useParserContext();

  const handleUpdate = async () => {
    try {
      await invoke("update_admin_data", {
        nombre: currentAdminName,
        tienda: currentStoreName,
        pass: currentPass,
        ubicacion: location,
        cp: cp
      });
      alert("¡Datos actualizados con éxito, patrón!");
    } catch (error) {
      console.error("Error al actualizar:", error);
      alert("Hubo una falla al guardar los datos.");
    }
  };

  // Botón independiente para guardar solo Datos de Identidad (nombre, tienda, ubicación, CP)
  const handleSaveIdentity = async () => {
    try {
      await invoke("update_admin_data", {
        nombre: currentAdminName,
        tienda: currentStoreName,
        pass: "",  // vacío → Rust no re-hashea
        ubicacion: location,
        cp: cp
      });
      setSuccessMessage("Cambios guardados exitosamente");
      setTimeout(() => setSuccessMessage(""), 3000);
    } catch (error) {
      console.error("Error al guardar datos de identidad:", error);
      alert("Hubo una falla al guardar los datos.");
    }
  };

  const handleFileSelect = async () => {
    try {
      const selected = await open({
        multiple: false,
        directory: parserMode === 'insertar',
        filters: parserMode !== 'insertar' ? [{
          name: 'Archivos soportados',
          extensions: ['txt', 'csv', 'xlsx']
        }] : []
      });

      if (selected) {
        const path = Array.isArray(selected) ? selected[0] : selected;
        setSelectedPath(path);
        setLlmAnalysis(null);

        if (parserMode !== 'insertar') {
          const ext = path.split('.').pop()?.toLowerCase() || '';
          
          if (ext === 'xlsx' || ext === 'xls') {
            const bytes = await invoke("leer_archivo_bytes", { path }) as number[];
            const result = await invoke("parsear_excel", { archivo: bytes }) as any;
            if (result.productos && result.productos.length > 0) {
              setParsedItems(result.productos);
              setFileContent(`Excel cargado: ${result.total} productos encontrados`);
            } else {
              setFileContent('No se encontraron productos en el Excel');
            }
          } else {
            const raw = await invoke("leer_archivo_raw", { path });
            setFileContent(raw as string);

            if (parserMode === 'entrenar IA') {
              setShowColumnMapper(true);
            } else if (parserMode === 'catalogo') {
              const result = await invoke("parsear_catalogo_visual", { path }) as any;
              if (result.productos && result.productos.length > 0) {
                setParsedItems(result.productos);
                setLastCatalogPath(path);
                setLastCatalogItems(result.productos);
              } else {
                const items = await invoke("parsear_catalogo_csv", { path });
                setParsedItems(items as any[]);
                setLastCatalogPath(path);
                setLastCatalogItems(items as any[]);
              }
            } else {
              setParsedItems([]);
            }
          }
        } else {
          setShowBatchProcessor(true);
        }
      }
    } catch (error) {
      console.error("Error en la selección:", error);
      setIsAnalyzing(false);
    }
  };

  const handleGuardarTicket = async (items: any[], _analysis: any) => {
    if (!items || items.length === 0) return;

    try {
      if (parserMode === 'entrenar IA') {
        const total = items.reduce((acc: number, item: any) => acc + (item.total || 0), 0);
        await invoke("guardar_ticket_parseado", { 
          items, 
          total,
          fecha: _analysis?.fecha_ticket || null,
          hora: _analysis?.hora_ticket || null
        });
        setIaTrained(true);
        setTicketsParsed(true);
        setTicketsCount((c) => c + items.length);
        setTicketsGuardados((c) => c + 1);
        alert("¡Ticket guardado en el historial histórico!");
      }
      setParsedItems([]);
      setSelectedPath("");
      setFileContent("");
      setLlmAnalysis(null);
      setShowColumnMapper(false);
    } catch (error) {
      console.error("Error al guardar ticket:", error);
      alert("Fallo al guardar el ticket.");
    }
  };

  const handleTrainIA = async () => {
    if (!parsedItems || parsedItems.length === 0) return;

    try {
      if (parserMode === 'entrenar IA') {
        const total = parsedItems.reduce((acc, item) => acc + (item.total || 0), 0);
        await invoke("guardar_ticket_parseado", { 
          items: parsedItems, 
          total,
          fecha: llmAnalysis?.fecha_ticket || null,
          hora: llmAnalysis?.hora_ticket || null
        });
        setIaTrained(true);
        setTicketsParsed(true);
        setTicketsCount((c) => c + parsedItems.length);
        setTicketsGuardados((c) => c + 1);
        alert("¡IA Entrenada! Ticket guardado en el historial histórico.");
      } else if (parserMode === 'catalogo') {
        const items = parsedItems.map((item: any) => ({
          id: null,
          nombre: item.nombre || item.producto || "",
          descripcion: null,
          precio_costo: item.precio_costo || 0,
          precio_venta: item.precio_venta || item.precio_venta || 0,
          stock: item.stock || 0,
          vendido: 0,
          stock_minimo: 5,
          codigo_barras: null,
          categoria: item.categoria || null,
        }));

        const result = await invoke("importar_catalogo", { 
          items,
          rutaArchivo: selectedPath,
          contenidoArchivo: fileContent
        }) as string;
        
        setCatalogParsed(true);
        setLastCatalogPath(selectedPath);
        setLastCatalogItems(parsedItems);
        alert(result || "¡Catálogo importado con éxito al inventario!");
      } else {
        alert("Modo no soportado aún, patrón.");
      }
      setParsedItems([]);
      setSelectedPath("");
      setFileContent("");
      setLlmAnalysis(null);
    } catch (error) {
      console.error("Error al entrenar IA:", error);
      const errorMsg = error instanceof Error ? error.message : String(error);
      if (errorMsg.includes("ya fue importado") || errorMsg.includes("duplicados")) {
        alert(errorMsg);
      } else {
        alert("Fallo al importar los datos a la base de datos.");
      }
    }
  };

  return (
    <div className="flex-1 space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500 max-w-5xl mx-auto w-full">
      <header className="mb-8 text-left relative">
        <h2 className="text-3xl font-black text-neutral-900 uppercase tracking-tight mb-2">Ajustes del Sistema</h2>
        <div className="h-1.5 w-12 bg-neutral-900 rounded-full"></div>

        {/* Toast de éxito con fade-out */}
        {successMessage && (
          <div className="absolute top-0 right-0 bg-green-50 border border-green-200 text-green-700 px-5 py-3 rounded-2xl text-[10px] font-black uppercase tracking-widest animate-in fade-in slide-in-from-top-2 duration-500 shadow-lg">
            {successMessage}
          </div>
        )}
      </header>

      {(
        <>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
        <div className="bg-neutral-50 p-8 rounded-[2.5rem] border border-neutral-100 space-y-6 shadow-sm">
          <h3 className="text-[10px] font-black text-neutral-400 uppercase tracking-[0.4em] mb-4">Datos de Identidad</h3>

          <div className="space-y-4">
            <div className="group">
              <label className="text-[9px] font-black text-neutral-400 uppercase ml-2 mb-1 block group-focus-within:text-neutral-900 transition-colors">Nombre del Administrador</label>
              <input
                type="text"
                value={currentAdminName}
                onChange={(e) => setCurrentAdminName(e.target.value)}
                className="w-full bg-white border border-neutral-100 px-6 py-4 rounded-2xl text-xs font-bold focus:outline-none focus:ring-4 focus:ring-neutral-900/5 focus:border-neutral-900 transition-all"
                placeholder="Ej. Alejandro"
              />
            </div>

            <div className="group">
              <label className="text-[9px] font-black text-neutral-400 uppercase ml-2 mb-1 block group-focus-within:text-neutral-900 transition-colors">Nombre de la Tienda</label>
              <input
                type="text"
                value={currentStoreName}
                onChange={(e) => setCurrentStoreName(e.target.value)}
                className="w-full bg-white border border-neutral-100 px-6 py-4 rounded-2xl text-xs font-bold focus:outline-none focus:ring-4 focus:ring-neutral-900/5 focus:border-neutral-900 transition-all"
                placeholder="Ej. Tienda Y.A.R.V.I.S."
              />
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div className="group">
                <label className="text-[9px] font-black text-neutral-400 uppercase ml-2 mb-1 block group-focus-within:text-neutral-900 transition-colors">Ubicación</label>
                <input
                  type="text"
                  value={location}
                  onChange={(e) => setLocation(e.target.value)}
                  className="w-full bg-white border border-neutral-100 px-6 py-4 rounded-2xl text-xs font-bold focus:outline-none focus:ring-4 focus:ring-neutral-900/5 focus:border-neutral-900 transition-all"
                />
              </div>
              <div className="group">
                <label className="text-[9px] font-black text-neutral-400 uppercase ml-2 mb-1 block group-focus-within:text-neutral-900 transition-colors">Código Postal</label>
                <input
                  type="text"
                  value={cp}
                  onChange={(e) => setCp(e.target.value)}
                  className="w-full bg-white border border-neutral-100 px-6 py-4 rounded-2xl text-xs font-bold focus:outline-none focus:ring-4 focus:ring-neutral-900/5 focus:border-neutral-900 transition-all"
                />
              </div>
            </div>

            {/* Botón Guardar Cambios exclusivo para Datos de Identidad */}
            <button
              onClick={handleSaveIdentity}
              className="w-full bg-neutral-900 text-white py-4 rounded-2xl text-[10px] font-black uppercase tracking-widest hover:scale-[1.02] active:scale-95 transition-all shadow-lg"
            >
              Guardar Cambios
            </button>
          </div>
        </div>

        <div className="space-y-6">
          <div className="bg-neutral-900 p-8 rounded-[2.5rem] shadow-2xl text-white space-y-6 relative overflow-hidden group">
            <div className="absolute top-0 right-0 w-32 h-32 bg-white/5 rounded-full -translate-y-16 translate-x-16 blur-3xl group-hover:bg-white/10 transition-all"></div>
            <h3 className="text-[10px] font-black text-neutral-500 uppercase tracking-[0.4em]">Seguridad & Acceso</h3>
            <div className="space-y-4 relative z-10">
              <div className="group/input">
                <label className="text-[9px] font-black text-neutral-500 uppercase ml-2 mb-1 block group-focus-within/input:text-white transition-colors">Contraseña Maestra</label>
                <input
                  type="password"
                  value={currentPass}
                  onChange={(e) => setCurrentPass(e.target.value)}
                  className="w-full bg-white/5 border border-white/10 px-6 py-4 rounded-2xl text-xs font-bold focus:outline-none focus:ring-4 focus:ring-white/5 focus:border-white/20 transition-all text-white placeholder:text-white/20"
                />
              </div>
              <button
                onClick={handleUpdate}
                className="w-full bg-white text-neutral-900 py-4 rounded-2xl text-[10px] font-black uppercase tracking-widest hover:scale-[1.02] active:scale-95 transition-all shadow-xl shadow-white/5"
              >
                Guardar Cambios
              </button>
            </div>
          </div>

          <div className="bg-neutral-50 p-8 rounded-[2.5rem] border border-neutral-100 shadow-sm">
            <h3 className="text-[10px] font-black text-neutral-400 uppercase tracking-[0.4em] mb-6">Apariencia del Sistema</h3>
            <div className="flex gap-4">
              {(['claro', 'oscuro', 'sistema'] as const).map((t) => (
                <button
                  key={t}
                  onClick={() => setTheme(t)}
                  className={`flex-1 py-3 rounded-xl text-[9px] font-black uppercase tracking-widest transition-all ${theme === t ? 'bg-neutral-900 text-white shadow-lg' : 'bg-white text-neutral-400 border border-neutral-100 hover:bg-neutral-100'}`}
                >
                  {t}
                </button>
              ))}
            </div>
          </div>
        </div>
      </div>

      <div className="bg-white rounded-[2.5rem] border border-neutral-100 shadow-xl overflow-hidden mt-8 border-t-4 border-t-neutral-900">
        <div className="p-8 border-b border-neutral-50 flex justify-between items-center bg-neutral-50/30">
          <div>
            <h3 className="text-sm font-black text-neutral-900 uppercase tracking-tighter">Módulo de Importación Inteligente</h3>
            <p className="text-[9px] font-bold text-neutral-400 uppercase tracking-widest">Parseador de Datos Raw & Catálogos</p>
          </div>
          <div className="flex bg-neutral-100 p-1 rounded-xl">
            {(['entrenar IA', 'catalogo', 'insertar'] as const).map((m) => (
              <button
                key={m}
                onClick={() => {
                  if (parserMode === 'catalogo' && parsedItems.length > 0) {
                    setLastCatalogPath(selectedPath);
                    setLastCatalogItems(parsedItems);
                  }
                  setParserMode(m);
                  setSelectedPath("");
                  setFileContent("");
                  setParsedItems([]);
                  setLlmAnalysis(null);
                  setShowColumnMapper(false);
                  if (m === 'catalogo' && lastCatalogItems.length > 0) {
                    setSelectedPath(lastCatalogPath);
                    setParsedItems(lastCatalogItems);
                  }
                }}
                className={`px-4 py-2 rounded-lg text-[9px] font-black uppercase tracking-widest transition-all ${parserMode === m ? 'bg-white text-neutral-900 shadow-sm' : 'text-neutral-400 hover:text-neutral-600'}`}
              >
                {m}
              </button>
            ))}
          </div>
        </div>

        {(() => {
          let label: string;
          let colorClass: string;
          let dotClass: string;
          if (!catalogParsed && !iaTrained && !ticketsParsed) {
            label = "Esperando datos";
            colorClass = "bg-neutral-100 text-neutral-400";
            dotClass = "bg-neutral-300";
          } else if (catalogParsed && !iaTrained) {
            label = "Esperando entrenamiento de IA";
            colorClass = "bg-orange-50 text-orange-500";
            dotClass = "bg-orange-400";
          } else if (iaTrained && !ticketsParsed) {
            label = "Esperando parseamiento de tickets";
            colorClass = "bg-yellow-50 text-yellow-600";
            dotClass = "bg-yellow-400";
          } else {
            const ticketLabel = ticketsGuardados === 1 ? "1 ticket" : `${ticketsGuardados} tickets`;
            const productoLabel = ticketsCount === 1 ? "1 producto" : `${ticketsCount} productos`;
            label = `${ticketLabel} · ${productoLabel} parseados`;
            colorClass = "bg-green-50 text-green-600";
            dotClass = "bg-green-500";
          }
          return (
            <div className={`mx-8 mt-6 px-5 py-3 rounded-2xl flex items-center gap-3 ${colorClass}`}>
              <div className={`w-2 h-2 rounded-full ${dotClass}`}></div>
              <span className="text-[9px] font-black uppercase tracking-widest">{label}</span>
            </div>
          );
        })()}

        {showBatchProcessor && parserMode === 'insertar' && (
          <div className="px-8 pb-8">
            <BatchProcessor onVolver={() => setShowBatchProcessor(false)} initialFolder={selectedPath} />
          </div>
        )}

        {!showBatchProcessor && (
        <div className="p-5 sm:p-10 grid grid-cols-1 lg:grid-cols-12 gap-5 sm:gap-10">
          <div className="lg:col-span-12 space-y-8">
                <div className="space-y-3">
                  <div className="flex justify-between items-center px-2">
                    <span className="text-[9px] font-black text-neutral-400 uppercase tracking-widest flex items-center gap-2">
                      <div className="w-1.5 h-1.5 rounded-full bg-neutral-900"></div>
                      {showBatchProcessor && parserMode === 'insertar' ? 'Procesamiento Masivo Activo' : 'Visualizador de Datos Raw'}
                    </span>
                    {!showBatchProcessor && (
                    <button
                      onClick={handleFileSelect}
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

                {!(showBatchProcessor && parserMode === 'insertar') && (
                <div className="space-y-3">
                  <span className="text-[9px] font-black text-neutral-400 uppercase tracking-widest px-2 flex items-center gap-2">
                    <div className="w-1.5 h-1.5 rounded-full bg-green-500"></div>
                    Previsualización de Datos Estructurados
                  </span>
                  <div className="bg-neutral-50 rounded-3xl border border-neutral-100 overflow-hidden shadow-sm h-64 overflow-y-auto custom-scrollbar">
                    <table className="w-full text-left border-collapse text-[11px]">
                      <thead className="sticky top-0 bg-neutral-50 shadow-sm z-10">
                        <tr className="border-b border-neutral-100">
                          {parserMode === 'entrenar IA' ? (
                            <>
                              <th className="px-4 py-4 font-black text-neutral-400 uppercase tracking-widest">Producto</th>
                              <th className="px-4 py-4 font-black text-neutral-400 uppercase tracking-widest text-center">Cant</th>
                              <th className="px-4 py-4 font-black text-neutral-400 uppercase tracking-widest text-right">Precio</th>
                              <th className="px-4 py-4 font-black text-neutral-400 uppercase tracking-widest text-right">Descuento</th>
                              <th className="px-4 py-4 font-black text-neutral-400 uppercase tracking-widest text-right">Total</th>
                            </>
                          ) : parserMode === 'catalogo' ? (
                            <>
                              <th className="px-6 py-4 font-black text-neutral-400 uppercase tracking-widest">Nombre</th>
                              <th className="px-6 py-4 font-black text-neutral-400 uppercase tracking-widest text-right">Costo</th>
                              <th className="px-6 py-4 font-black text-neutral-400 uppercase tracking-widest text-right">Venta</th>
                              <th className="px-6 py-4 font-black text-neutral-400 uppercase tracking-widest text-center">Cantidad</th>
                            </>
                          ) : (
                            <>
                              <th className="px-6 py-4 font-black text-neutral-400 uppercase tracking-widest">Archivo</th>
                              <th className="px-6 py-4 font-black text-neutral-400 uppercase tracking-widest text-center">Estado</th>
                              <th className="px-6 py-4 font-black text-neutral-400 uppercase tracking-widest text-right">Tamaño</th>
                            </>
                          )}
                        </tr>
                      </thead>
                      <tbody className="divide-y divide-neutral-100">
                        {parsedItems.length > 0 ? (
                          parserMode === 'catalogo' ? (
                            parsedItems.map((item, i) => (
                              <tr key={i} className="hover:bg-white transition-colors group">
                                <td className="px-6 py-3 font-bold text-neutral-900">{item.nombre}</td>
                                <td className="px-6 py-3 text-right font-bold">${item.precio_costo.toFixed(2)}</td>
                                <td className="px-6 py-3 text-right font-bold text-neutral-900 group-hover:text-green-600 transition-colors">${item.precio_venta.toFixed(2)}</td>
                                <td className="px-6 py-3 text-center font-black text-neutral-400 text-[9px]">{item.stock || 0}</td>
                              </tr>
                            ))
                          ) : parserMode === 'entrenar IA' ? (
                            parsedItems.map((item, i) => (
                              <tr key={i} className="hover:bg-white transition-colors group">
                                <td className="px-4 py-3 font-bold text-neutral-900">{item.producto}</td>
                                <td className="px-4 py-3 text-center font-black text-neutral-400">{item.cantidad}</td>
                                <td className="px-4 py-3 text-right font-bold">${(item.precio_unitario ?? item.precio ?? 0).toFixed(2)}</td>
                                <td className="px-4 py-3 text-right text-[10px] font-bold text-red-500">{item.descuento ? `-$${item.descuento.toFixed(2)}` : '-'}</td>
                                <td className="px-4 py-3 text-right font-black text-neutral-900 group-hover:text-green-600 transition-colors">${item.total.toFixed(2)}</td>
                              </tr>
                            ))
                          ) : (
                            <tr>
                              <td colSpan={3} className="px-6 py-20 text-center opacity-20 font-black uppercase tracking-widest text-[9px]">
                                Selecciona los archivos del directorio
                              </td>
                            </tr>
                          )
                        ) : (
                          <tr>
                            <td colSpan={parserMode === 'insertar' ? 3 : 5} className="px-6 py-20 text-center opacity-20 font-black uppercase tracking-widest text-[9px]">
                              {isAnalyzing ? 'Analizando ticket con IA...' : 'Sin datos para previsualizar'}
                            </td>
                          </tr>
                        )}
                      </tbody>
                    </table>
                  </div>
                </div>
                )}

                {parserMode === 'entrenar IA' && showColumnMapper && fileContent && (
                  <ColumnMapper
                    onGuardarTicket={handleGuardarTicket}
                    onPreviewUpdate={setParsedItems}
                    fileContent={fileContent}
                    selectedPath={selectedPath}
                  />
                )}

                {parserMode === 'entrenar IA' && llmAnalysis && (
                  <div className="space-y-3 animate-in fade-in duration-500">
                    <span className="text-[9px] font-black text-neutral-400 uppercase tracking-widest px-2 flex items-center gap-2">
                      <div className="w-1.5 h-1.5 rounded-full bg-blue-500"></div>
                      Análisis del Motor de IA
                    </span>
                    <div className="bg-neutral-900 rounded-3xl p-6 text-white shadow-inner">
                      <div className="grid grid-cols-2 md:grid-cols-4 gap-2 sm:gap-4 mb-4">
                        <div className="bg-white/5 rounded-xl p-3">
                          <p className="text-[8px] font-black text-neutral-500 uppercase tracking-widest">Formato</p>
                          <p className="text-[11px] font-bold mt-1 truncate">{llmAnalysis.mapeo.formato_detectado}</p>
                        </div>
                        <div className="bg-white/5 rounded-xl p-3">
                          <p className="text-[8px] font-black text-neutral-500 uppercase tracking-widest">Confianza</p>
                          <p className={`text-[11px] font-bold mt-1 ${llmAnalysis.confianza >= 0.8 ? 'text-green-400' : 'text-yellow-400'}`}>
                            {(llmAnalysis.confianza * 100).toFixed(0)}%
                          </p>
                        </div>
                        <div className="bg-white/5 rounded-xl p-3">
                          <p className="text-[8px] font-black text-neutral-500 uppercase tracking-widest">Columnas</p>
                          <p className="text-[11px] font-bold mt-1">{llmAnalysis.mapeo.total_columnas}</p>
                        </div>
                        <div className="bg-white/5 rounded-xl p-3">
                          <p className="text-[8px] font-black text-neutral-500 uppercase tracking-widest">Delimitador</p>
                          <p className="text-[11px] font-bold mt-1">{llmAnalysis.mapeo.delimitador}</p>
                        </div>
                      </div>
                      <div className="flex gap-3 mb-4">
                        {llmAnalysis.mapeo.tiene_descuento && (
                          <span className="text-[9px] font-bold px-2 py-1 rounded-lg bg-yellow-500/20 text-yellow-300">TIENE DESCUENTOS</span>
                        )}
                        {llmAnalysis.mapeo.tiene_iva && (
                          <span className="text-[9px] font-bold px-2 py-1 rounded-lg bg-blue-500/20 text-blue-300">TIENE IVA</span>
                        )}
                        {llmAnalysis.reintentado_con && (
                          <span className="text-[9px] font-bold px-2 py-1 rounded-lg bg-purple-500/20 text-purple-300">REINTENTADO CON 1.7B</span>
                        )}
                      </div>
                      <div className="bg-white/5 rounded-xl p-4">
                        <p className="text-[8px] font-black text-neutral-500 uppercase tracking-widest mb-2">Notas del Análisis</p>
                        <p className="text-[11px] text-neutral-300 leading-relaxed">{llmAnalysis.notas}</p>
                      </div>
                      <div className="mt-3 bg-white/5 rounded-xl p-3">
                        <p className="text-[8px] font-black text-neutral-500 uppercase tracking-widest mb-2">Mapeo de Columnas</p>
                        <div className="flex flex-wrap gap-2">
                          {Object.entries(llmAnalysis.mapeo.columnas as Record<string, any>).map(([key, val]) => (
                            <span key={key} className={`text-[9px] font-bold px-2 py-1 rounded-lg ${val !== null ? 'bg-white/10 text-white' : 'bg-white/5 text-neutral-600 line-through'}`}>
                              {key}: {val !== null ? `col ${val}` : 'N/A'}
                            </span>
                          ))}
                        </div>
                      </div>
                    </div>
                  </div>
                )}

                {parserMode === 'catalogo' && (
                  <CatalogosParseados />
                )}

                {parserMode !== 'entrenar IA' && (
                <div className="flex justify-center pt-2">
                  <button
                    disabled={!selectedPath || parsedItems.length === 0 || isAnalyzing}
                    onClick={handleTrainIA}
                    className={`w-full py-5 rounded-2xl text-[11px] font-black uppercase tracking-[0.3em] transition-all shadow-2xl flex items-center justify-center gap-3 ${
                      selectedPath && parsedItems.length > 0 && !isAnalyzing
                        ? 'bg-neutral-900 text-white hover:scale-[1.02] active:scale-95 shadow-neutral-200'
                        : 'bg-neutral-100 text-neutral-300 cursor-not-allowed shadow-none'
                    }`}
                  >
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"><path d="m12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275L12 3Z"/></svg>
                    {parserMode === 'catalogo' ? 'Entrenar IA con Catálogo' : 'Insertar Carpeta'}
                  </button>
                </div>
                )}
          </div>
        </div>
        )}
      </div>
        </>
      )}
    </div>
  );
};

export default Configuracion;
