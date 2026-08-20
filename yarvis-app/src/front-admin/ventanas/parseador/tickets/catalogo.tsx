// Paso 1 del flujo de tickets: carga e importación del catálogo maestro.
import type { CatalogItem } from "./compartido";

interface CatalogoProps {
  catalogPath: string;
  catalogItems: CatalogItem[];
  busy: boolean;
  onSelectCatalog: () => void;
  onImportCatalog: () => void;
}

const Catalogo = ({ catalogPath, catalogItems, busy, onSelectCatalog, onImportCatalog }: CatalogoProps) => (
  <section className="bg-white rounded-[2.5rem] border border-neutral-100 shadow-xl p-6 sm:p-10 space-y-6">
    <div>
      <p className="text-[10px] font-black uppercase tracking-[0.35em] text-neutral-400">Paso 1 · Fuente de verdad</p>
      <h3 className="text-2xl font-black text-neutral-900 mt-2">Carga tu catálogo maestro</h3>
      <p className="text-sm text-neutral-500 mt-2">Se usará para reconocer productos y mantener el inventario consistente.</p>
    </div>
    <button onClick={onSelectCatalog} className="w-full border-2 border-dashed border-neutral-200 rounded-3xl py-10 hover:border-neutral-900 hover:bg-neutral-50 transition-colors">
      <div className="text-3xl mb-3">▦</div>
      <span className="text-[11px] font-black uppercase tracking-widest text-neutral-500">Seleccionar TXT, CSV o Excel</span>
      {catalogPath && <p className="text-xs text-neutral-900 font-bold mt-3 break-all px-4">{catalogPath}</p>}
    </button>
    {!!catalogItems.length && (
      <div className="rounded-2xl bg-neutral-50 p-5">
        <div className="flex items-center justify-between mb-4">
          <span className="text-[10px] font-black uppercase tracking-widest text-neutral-400">Vista previa</span>
          <span className="text-xs font-black text-neutral-900">{catalogItems.length} productos</span>
        </div>
        <div className="space-y-2 max-h-48 overflow-y-auto">
          {catalogItems.slice(0, 6).map((item, index) => <div key={`${item.nombre}-${index}`} className="flex justify-between gap-4 text-sm"><span className="truncate font-bold text-neutral-700">{item.nombre}</span><span className="text-neutral-400">${item.precio_venta.toFixed(2)}</span></div>)}
        </div>
        <button disabled={busy} onClick={onImportCatalog} className="w-full mt-5 rounded-2xl bg-neutral-950 text-neutral-50 py-4 text-[10px] font-black uppercase tracking-widest disabled:opacity-40">{busy ? "Importando catálogo..." : "Importar catálogo maestro"}</button>
      </div>
    )}
  </section>
);

export default Catalogo;