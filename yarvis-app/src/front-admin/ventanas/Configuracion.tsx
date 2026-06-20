import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

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
  adminPass,
  initialLocation = "",
  initialCp = "",
}: ConfiguracionProps) => {
  const [currentAdminName, setCurrentAdminName] = useState(adminName);
  const [currentStoreName, setCurrentStoreName] = useState(storeName);
  const [currentPass, setCurrentPass] = useState(adminPass);
  const [location, setLocation] = useState(initialLocation);
  const [cp, setCp] = useState(initialCp);
  const [selectedTheme, setSelectedTheme] = useState("claro");
  const [parserMode, setParserMode] = useState<"catalogo" | "ticket" | "insertar">("ticket");
  const [selectedPath, setSelectedPath] = useState("");
  const [fileContent, setFileContent] = useState("");
  const [parsedItems, setParsedItems] = useState<any[]>([]);

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

  const handleFileSelect = async () => {
    try {
      const selected = await open({
        multiple: false,
        directory: parserMode === 'insertar',
        filters: parserMode !== 'insertar' ? [{
          name: 'Tickets',
          extensions: ['txt']
        }] : []
      });

      if (selected) {
        const path = Array.isArray(selected) ? selected[0] : selected;
        setSelectedPath(path);
        
        if (parserMode !== 'insertar') {
          const raw = await invoke("leer_archivo_raw", { path });
          setFileContent(raw as string);

          if (parserMode === 'ticket') {
            const items = await invoke("parsear_ticket", { path });
            setParsedItems(items as any[]);
          } else if (parserMode === 'catalogo') {
            const items = await invoke("parsear_catalogo", { path });
            setParsedItems(items as any[]);
          } else {
            setParsedItems([]);
          }
        } else {
          setFileContent("// Directorio seleccionado: " + path);
          setParsedItems([]);
        }
      }
    } catch (error) {
      console.error("Error en la selección:", error);
    }
  };

  const handleTrainIA = async () => {
    if (!parsedItems || parsedItems.length === 0) return;

    try {
      if (parserMode === 'ticket') {
        const total = parsedItems.reduce((acc, item) => acc + item.total, 0);
        await invoke("guardar_ticket_parseado", { items: parsedItems, total });
        alert("¡IA Entrenada! Ticket guardado en el historial histórico.");
      } else if (parserMode === 'catalogo') {
        await invoke("importar_catalogo", { items: parsedItems });
        alert("¡Catálogo importado con éxito al inventario!");
      } else {
        alert("Modo no soportado aún, patrón.");
      }
      setParsedItems([]);
      setSelectedPath("");
      setFileContent("");
    } catch (error) {
      console.error("Error al entrenar IA:", error);
      alert("Fallo al importar los datos a la base de datos.");
    }
  };

  return (
    <div className="flex-1 space-y-6 animate-in fade-in slide-in-from-bottom-4 duration-500 max-w-5xl mx-auto w-full">
      <header className="mb-8 text-left">
        <h2 className="text-3xl font-black text-neutral-900 uppercase tracking-tight mb-2">Ajustes del Sistema</h2>
        <div className="h-1.5 w-12 bg-neutral-900 rounded-full"></div>
      </header>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
        {/* SECCIÓN: PERFIL Y TIENDA */}
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

            <div className="grid grid-cols-2 gap-4">
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
          </div>
        </div>

        {/* SECCIÓN: SEGURIDAD Y TEMA */}
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
              {['claro', 'oscuro', 'sistema'].map((t) => (
                <button 
                  key={t}
                  onClick={() => setSelectedTheme(t)}
                  className={`flex-1 py-3 rounded-xl text-[9px] font-black uppercase tracking-widest transition-all ${selectedTheme === t ? 'bg-neutral-900 text-white shadow-lg' : 'bg-white text-neutral-400 border border-neutral-100 hover:bg-neutral-100'}`}
                >
                  {t}
                </button>
              ))}
            </div>
          </div>
        </div>
      </div>

      {/* SECCIÓN: PARSEADOR DE TICKETS (EL CEREBRO) */}
      <div className="bg-white rounded-[2.5rem] border border-neutral-100 shadow-xl overflow-hidden mt-8 border-t-4 border-t-neutral-900">
        <div className="p-8 border-b border-neutral-50 flex justify-between items-center bg-neutral-50/30">
          <div>
            <h3 className="text-sm font-black text-neutral-900 uppercase tracking-tighter">Módulo de Importación Inteligente</h3>
            <p className="text-[9px] font-bold text-neutral-400 uppercase tracking-widest">Parseador de Datos Raw & Catálogos</p>
          </div>
          <div className="flex bg-neutral-100 p-1 rounded-xl">
            {(['ticket', 'catalogo', 'insertar'] as const).map((m) => (
              <button 
                key={m}
                onClick={() => { setParserMode(m); setSelectedPath(""); setFileContent(""); setParsedItems([]); }}
                className={`px-4 py-2 rounded-lg text-[9px] font-black uppercase tracking-widest transition-all ${parserMode === m ? 'bg-white text-neutral-900 shadow-sm' : 'text-neutral-400 hover:text-neutral-600'}`}
              >
                {m}
              </button>
            ))}
          </div>
        </div>

        <div className="p-10 grid grid-cols-1 lg:grid-cols-12 gap-10">
          <div className="lg:col-span-12 space-y-8">
                {/* VISUALIZADOR RAW (ARRIBA) */}
                <div className="space-y-3">
                  <div className="flex justify-between items-center px-2">
                    <span className="text-[9px] font-black text-neutral-400 uppercase tracking-widest flex items-center gap-2">
                      <div className="w-1.5 h-1.5 rounded-full bg-neutral-900"></div>
                      Visualizador de Datos Raw
                    </span>
                    <button 
                      onClick={handleFileSelect}
                      className="px-5 py-2 bg-neutral-900 text-white rounded-xl text-[9px] font-black uppercase tracking-widest hover:scale-105 active:scale-95 transition-all shadow-lg shadow-neutral-200"
                    >
                      {parserMode === 'insertar' ? 'Seleccionar Carpeta' : 'Cargar Archivo .txt'}
                    </button>
                  </div>
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
                </div>

                {/* TABLA DE PREVISUALIZACIÓN (ABAJO) */}
                <div className="space-y-3">
                  <span className="text-[9px] font-black text-neutral-400 uppercase tracking-widest px-2 flex items-center gap-2">
                    <div className="w-1.5 h-1.5 rounded-full bg-green-500"></div>
                    Previsualización de Datos Estructurados
                  </span>
                  <div className="bg-neutral-50 rounded-3xl border border-neutral-100 overflow-hidden shadow-sm h-64 overflow-y-auto custom-scrollbar">
                    <table className="w-full text-left border-collapse text-[11px]">
                      <thead className="sticky top-0 bg-neutral-50 shadow-sm z-10">
                        <tr className="border-b border-neutral-100">
                          {parserMode === 'ticket' ? (
                            <>
                              <th className="px-6 py-4 font-black text-neutral-400 uppercase tracking-widest">Producto</th>
                              <th className="px-6 py-4 font-black text-neutral-400 uppercase tracking-widest text-center">Cant</th>
                              <th className="px-6 py-4 font-black text-neutral-400 uppercase tracking-widest text-right">Precio</th>
                              <th className="px-6 py-4 font-black text-neutral-400 uppercase tracking-widest text-right">Total</th>
                            </>
                          ) : parserMode === 'catalogo' ? (
                            <>
                              <th className="px-6 py-4 font-black text-neutral-400 uppercase tracking-widest">Nombre</th>
                              <th className="px-6 py-4 font-black text-neutral-400 uppercase tracking-widest text-right">Costo</th>
                              <th className="px-6 py-4 font-black text-neutral-400 uppercase tracking-widest text-right">Venta</th>
                              <th className="px-6 py-4 font-black text-neutral-400 uppercase tracking-widest text-center">Stock</th>
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
                                <td className="px-6 py-3 text-center font-black text-neutral-400">{item.stock}</td>
                              </tr>
                            ))
                          ) : parserMode === 'ticket' ? (
                            parsedItems.map((item, i) => (
                              <tr key={i} className="hover:bg-white transition-colors group">
                                <td className="px-6 py-3 font-bold text-neutral-900">{item.producto}</td>
                                <td className="px-6 py-3 text-center font-black text-neutral-400">{item.cantidad}</td>
                                <td className="px-6 py-3 text-right font-bold">${item.precio.toFixed(2)}</td>
                                <td className="px-6 py-3 text-right font-black text-neutral-900 group-hover:text-green-600 transition-colors">${item.total.toFixed(2)}</td>
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
                            <td colSpan={parserMode === 'insertar' ? 3 : 4} className="px-6 py-20 text-center opacity-20 font-black uppercase tracking-widest text-[9px]">Sin datos para previsualizar</td>
                          </tr>
                        )}
                      </tbody>
                    </table>
                  </div>
                </div>

                {/* BOTÓN DE ACCIÓN DINÁMICO */}
                <div className="flex justify-center pt-2">
                  <button 
                    disabled={!selectedPath || (parserMode === 'ticket' && parsedItems.length === 0)}
                    onClick={handleTrainIA}
                    className={`w-full py-5 rounded-2xl text-[11px] font-black uppercase tracking-[0.3em] transition-all shadow-2xl flex items-center justify-center gap-3 ${
                      selectedPath 
                        ? 'bg-neutral-900 text-white hover:scale-[1.02] active:scale-95 shadow-neutral-200' 
                        : 'bg-neutral-100 text-neutral-300 cursor-not-allowed shadow-none'
                    }`}
                  >
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"><path d="m12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275L12 3Z"/></svg>
                    {parserMode === 'ticket' ? 'Entrenar IA con Ticket' : parserMode === 'catalogo' ? 'Entrenar IA con Catálogo' : 'Insertar Carpeta'}
                  </button>
                </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Configuracion;
