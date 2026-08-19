// Tabla interactiva que previsualiza cómo el sistema interpretó los datos
// (producto, precio, cantidad) para que el usuario valide antes de guardarlos.

import { useParserContext } from "../../../../../hooks/ParserContext";

interface PreviewTableProps {
  isAnalyzing: boolean;
}

const PreviewTable = ({ isAnalyzing }: PreviewTableProps) => {
  const { parsedItems, parserMode } = useParserContext();

  return (
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
  );
};

export default PreviewTable;