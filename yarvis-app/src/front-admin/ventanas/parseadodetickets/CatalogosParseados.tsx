import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useParserContext } from "../../../hooks/ParserContext";

interface CatalogoImportado {
  id: number;
  hash: string;
  ruta_archivo: string;
  fecha_importacion: string;
  total_productos: number;
}

const CatalogosParseados = () => {
  const [catalogos, setCatalogos] = useState<CatalogoImportado[]>([]);
  const [loading, setLoading] = useState(true);
  const { setParsedItems, setSelectedPath, setFileContent, setLastCatalogPath, setLastCatalogItems } = useParserContext();

  useEffect(() => {
    loadCatalogos();
  }, []);

  const loadCatalogos = async () => {
    try {
      const result = await invoke<CatalogoImportado[]>("get_catalogos_importados");
      setCatalogos(result);
    } catch (error) {
      console.error("Error cargando catálogos:", error);
    } finally {
      setLoading(false);
    }
  };

  const handleRecargar = async (catalogo: CatalogoImportado) => {
    try {
      const items = await invoke<any[]>("get_productos_por_catalogo", { catalogoId: catalogo.id });
      if (items && items.length > 0) {
        setParsedItems(items);
        setSelectedPath(catalogo.ruta_archivo);
        setFileContent(`Catálogo recargado: ${items.length} productos`);
        setLastCatalogPath(catalogo.ruta_archivo);
        setLastCatalogItems(items);
        alert(`Catálogo recargado: ${items.length} productos`);
      }
    } catch (error) {
      console.error("Error recargando catálogo:", error);
      alert("Error al recargar el catálogo");
    }
  };

  if (loading) {
    return (
      <div className="space-y-3 animate-in fade-in duration-500">
        <span className="text-[9px] font-black text-neutral-400 uppercase tracking-widest px-2 flex items-center gap-2">
          <div className="w-1.5 h-1.5 rounded-full bg-neutral-300 animate-pulse"></div>
          Cargando catálogos...
        </span>
      </div>
    );
  }

  if (catalogos.length === 0) {
    return null;
  }

  return (
    <div className="space-y-3 animate-in fade-in duration-500">
      <span className="text-[9px] font-black text-neutral-400 uppercase tracking-widest px-2 flex items-center gap-2">
        <div className="w-1.5 h-1.5 rounded-full bg-blue-500"></div>
        Catálogos Importados ({catalogos.length})
      </span>
      <div className="bg-neutral-50 rounded-3xl border border-neutral-100 overflow-hidden shadow-sm">
        <table className="w-full text-left border-collapse text-[11px]">
          <thead>
            <tr className="border-b border-neutral-100 bg-neutral-100/50">
              <th className="px-4 py-3 font-black text-neutral-400 uppercase tracking-widest">Fecha</th>
              <th className="px-4 py-3 font-black text-neutral-400 uppercase tracking-widest">Archivo</th>
              <th className="px-4 py-3 font-black text-neutral-400 uppercase tracking-widest text-center">Productos</th>
              <th className="px-4 py-3 font-black text-neutral-400 uppercase tracking-widest text-center">Acción</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-neutral-100">
            {catalogos.map((catalogo) => (
              <tr key={catalogo.id} className="hover:bg-white transition-colors">
                <td className="px-4 py-3 font-bold text-neutral-600">
                  {new Date(catalogo.fecha_importacion).toLocaleDateString('es-MX')}
                </td>
                <td className="px-4 py-3 font-bold text-neutral-900 truncate max-w-[200px]">
                  {catalogo.ruta_archivo.split('/').pop() || catalogo.ruta_archivo}
                </td>
                <td className="px-4 py-3 text-center font-black text-neutral-600">
                  {catalogo.total_productos}
                </td>
                <td className="px-4 py-3 text-center">
                  <button
                    onClick={() => handleRecargar(catalogo)}
                    className="px-3 py-1 rounded-lg text-[9px] font-black uppercase tracking-widest bg-neutral-900 text-white hover:bg-neutral-700 transition-colors"
                  >
                    Recargar
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
};

export default CatalogosParseados;
