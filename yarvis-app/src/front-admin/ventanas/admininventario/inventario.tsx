import { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface InventoryItem {
  id?: number;
  nombre: string;
  descripcion?: string;
  precio_costo: number;
  precio_venta: number;
  stock: number;
  stock_minimo: number;
  vendido: number;
  codigo_barras?: string;
  categoria?: string;
}

interface InventarioProps {
  activeTab: string;
}

const Inventario = ({ activeTab }: InventarioProps) => {
  const [inventory, setInventory] = useState<InventoryItem[]>([]);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [inventoryFilter, setInventoryFilter] = useState("A-Z");
  const [searchQuery, setSearchQuery] = useState("");

  useEffect(() => {
    if (activeTab === "inventario") {
      loadInventory();
    }
  }, [activeTab]);

  const loadInventory = async () => {
    try {
      const items = await invoke<InventoryItem[]>("get_inventory");
      setInventory(items);
    } catch (error) {
      console.error("Error al cargar inventario:", error);
    }
  };

  const sortedInventory = useMemo(() => {
    let sorted = [...inventory].filter(item => 
      item.nombre.toLowerCase().includes(searchQuery.toLowerCase())
    );

    switch (inventoryFilter) {
      case "A-Z": sorted.sort((a, b) => a.nombre.localeCompare(b.nombre)); break;
      case "Z-A": sorted.sort((a, b) => b.nombre.localeCompare(a.nombre)); break;
      case "Barato-Caro": sorted.sort((a, b) => (a.precio_venta || 0) - (b.precio_venta || 0)); break;
      case "Caro-Barato": sorted.sort((a, b) => (b.precio_venta || 0) - (a.precio_venta || 0)); break;
      case "Margen High-Low": sorted.sort((a, b) => {
        const m1 = a.precio_venta > 0 ? (a.precio_venta - a.precio_costo) / a.precio_venta : 0;
        const m2 = b.precio_venta > 0 ? (b.precio_venta - b.precio_costo) / b.precio_venta : 0;
        return m2 - m1;
      }); break;
      case "Margen Low-High": sorted.sort((a, b) => {
        const m1 = a.precio_venta > 0 ? (a.precio_venta - a.precio_costo) / a.precio_venta : 0;
        const m2 = b.precio_venta > 0 ? (b.precio_venta - b.precio_costo) / b.precio_venta : 0;
        return m1 - m2;
      }); break;
    }
    return sorted;
  }, [inventory, inventoryFilter, searchQuery]);

  const handleUpdateItem = async (item: InventoryItem) => {
    try {
      const cleanedItem = {
        ...item,
        precio_costo: item.precio_costo || 0,
        precio_venta: item.precio_venta || 0,
        stock: item.stock || 0,
        stock_minimo: item.stock_minimo || 0,
        vendido: item.vendido || 0,
      };

      if (cleanedItem.id) {
        await invoke("update_inventory_item", { item: cleanedItem });
      } else {
        await invoke("add_inventory_item", { item: cleanedItem });
      }
      
      alert("¡Producto guardado con éxito!");
      loadInventory();
      setEditingId(null);
    } catch (error) {
      console.error("Error al guardar producto:", error);
      alert("Error al guardar en la DB: " + error);
    }
  };

  const handleDeleteItem = async (id: number) => {
    if (confirm("¿Estás seguro de eliminar este producto?")) {
      try {
        await invoke("delete_inventory_item", { id });
        loadInventory();
      } catch (error) {
        console.error("Error al eliminar producto:", error);
      }
    }
  };

  const handleAddRow = () => {
    const tempId = -Date.now();
    const newItem: InventoryItem = {
      id: tempId,
      nombre: "NUEVO PRODUCTO",
      precio_costo: 0,
      precio_venta: 0,
      stock: 0,
      stock_minimo: 5,
      vendido: 0,
    };
    setInventory(prev => [newItem, ...prev]);
    setEditingId(tempId);
  };

  return (
    <div className="max-w-6xl animate-in fade-in slide-in-from-bottom-2 duration-500 mx-auto space-y-12">
      <header className="flex justify-between items-end mb-8">
        <div>
          <h2 className="text-3xl font-black text-neutral-900 uppercase tracking-tight mb-1">Inventario General</h2>
          <p className="text-[10px] font-black text-neutral-400 uppercase tracking-[0.3em]">Gestión de Stock y Predicciones IA</p>
        </div>
        <div className="flex gap-3">
          <div className="relative group">
            <input 
              type="text" 
              placeholder="Buscar producto..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="pl-10 pr-4 py-2.5 bg-neutral-50 border border-neutral-100 rounded-xl text-xs font-bold focus:outline-none focus:ring-4 focus:ring-neutral-900/5 focus:border-neutral-900 transition-all w-64"
            />
            <div className="absolute left-3.5 top-1/2 -translate-y-1/2 text-neutral-300 group-focus-within:text-neutral-900 transition-colors">
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>
            </div>
          </div>
          
          <select 
            value={inventoryFilter}
            onChange={(e) => setInventoryFilter(e.target.value)}
            className="px-4 py-2.5 bg-neutral-50 border border-neutral-100 rounded-xl text-[10px] font-black uppercase tracking-widest focus:outline-none focus:ring-4 focus:ring-neutral-900/5 appearance-none cursor-pointer pr-10 relative"
          >
            <option value="A-Z">A-Z</option>
            <option value="Z-A">Z-A</option>
            <option value="Barato-Caro">$↓</option>
            <option value="Caro-Barato">$↑</option>
            <option value="Margen High-Low">Margen %↑</option>
            <option value="Margen Low-High">Margen %↓</option>
          </select>

          <div className="flex bg-neutral-900 p-1 rounded-xl shadow-xl shadow-neutral-200">
            <button onClick={handleAddRow} className="px-4 py-2 text-[9px] font-black text-white hover:bg-white/10 rounded-lg transition-all flex items-center gap-2">
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"><path d="M5 12h14"/><path d="M12 5v14"/></svg>
              AGREGAR
            </button>
          </div>
        </div>
      </header>

      <div className="bg-white rounded-[2.5rem] border border-neutral-200 shadow-sm overflow-hidden">
        <table className="w-full text-left border-collapse">
          <thead>
            <tr className="bg-neutral-50/50 border-b border-neutral-100">
              <th className="px-8 py-5 text-[10px] font-black text-neutral-400 uppercase tracking-widest">Producto</th>
              <th className="px-6 py-5 text-[10px] font-black text-neutral-400 uppercase tracking-widest text-center">Stock</th>
              <th className="px-6 py-5 text-[10px] font-black text-neutral-400 uppercase tracking-widest text-center">Vendido</th>
              <th className="px-6 py-5 text-[10px] font-black text-neutral-400 uppercase tracking-widest">Costo</th>
              <th className="px-6 py-5 text-[10px] font-black text-neutral-400 uppercase tracking-widest">Venta</th>
              <th className="px-6 py-5 text-[10px] font-black text-neutral-400 uppercase tracking-widest text-right">Margen %</th>
              <th className="px-6 py-5 text-[10px] font-black text-neutral-400 uppercase tracking-widest text-right">Acciones</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-neutral-50">
            {sortedInventory.map((item, idx) => {
              const isEditing = editingId === item.id;
              const margin = item.precio_venta > 0 ? ((item.precio_venta - (item.precio_costo || 0)) / item.precio_venta * 100).toFixed(1) : "0.0";
              
              return (
                <tr key={item.id || idx} className={`group hover:bg-neutral-50/30 transition-all ${isEditing ? 'bg-neutral-50 ring-2 ring-inset ring-neutral-900/5' : ''}`}>
                  <td className="px-8 py-4">
                    {isEditing ? (
                      <input 
                        autoFocus
                        className="bg-white border border-neutral-200 px-3 py-1.5 rounded-lg text-xs font-bold w-full focus:outline-none focus:border-neutral-900"
                        value={item.nombre}
                        onChange={(e) => {
                          const newInv = [...inventory];
                          newInv[inventory.indexOf(item)].nombre = e.target.value.toUpperCase();
                          setInventory(newInv);
                        }}
                      />
                    ) : (
                      <span className="text-xs font-bold text-neutral-900 uppercase">{item.nombre}</span>
                    )}
                  </td>
                  <td className="px-6 py-4 text-center">
                    {isEditing ? (
                      <input 
                        type="number"
                        className="bg-white border border-neutral-200 px-3 py-1.5 rounded-lg text-xs font-bold w-20 text-center focus:outline-none focus:border-neutral-900"
                        value={item.stock}
                        onChange={(e) => {
                          const newInv = [...inventory];
                          newInv[inventory.indexOf(item)].stock = parseFloat(e.target.value);
                          setInventory(newInv);
                        }}
                      />
                    ) : (
                      <div className="flex flex-col items-center">
                        <span className={`text-xs font-black ${item.stock < 0 ? 'text-red-600 bg-red-50 px-2 py-0.5 rounded-full border border-red-200' : item.stock <= item.stock_minimo ? 'text-red-500' : 'text-neutral-900'}`}>
                          {item.stock < 0 ? `-${Math.abs(item.stock)}` : item.stock}
                        </span>
                        <div className="h-1 w-8 bg-neutral-100 rounded-full mt-1 overflow-hidden">
                          <div className={`h-full rounded-full ${item.stock < 0 ? 'bg-red-600' : item.stock <= item.stock_minimo ? 'bg-red-500' : 'bg-neutral-900'}`} style={{ width: `${Math.min(100, Math.max(0, (item.stock / 50) * 100))}%` }}></div>
                        </div>
                      </div>
                    )}
                  </td>
                  <td className="px-6 py-4 text-center">
                    {isEditing ? (
                      <input 
                        type="number"
                        className="bg-white border border-neutral-200 px-3 py-1.5 rounded-lg text-xs font-bold w-20 text-center focus:outline-none focus:border-neutral-900"
                        value={item.vendido}
                        onChange={(e) => {
                          const newInv = [...inventory];
                          newInv[inventory.indexOf(item)].vendido = parseFloat(e.target.value);
                          setInventory(newInv);
                        }}
                      />
                    ) : (
                      <span className="text-xs font-bold text-neutral-900">{item.vendido}</span>
                    )}
                  </td>
                  <td className="px-6 py-4">
                    {isEditing ? (
                      <input 
                        type="number"
                        className="bg-white border border-neutral-200 px-3 py-1.5 rounded-lg text-xs font-bold w-24 focus:outline-none focus:border-neutral-900"
                        value={item.precio_costo}
                        onChange={(e) => {
                          const newInv = [...inventory];
                          newInv[inventory.indexOf(item)].precio_costo = parseFloat(e.target.value);
                          setInventory(newInv);
                        }}
                      />
                    ) : (
                      <span className="text-xs font-bold text-neutral-400">${item.precio_costo.toFixed(2)}</span>
                    )}
                  </td>
                  <td className="px-6 py-4 font-black text-neutral-900">
                    {isEditing ? (
                      <input 
                        type="number"
                        className="bg-white border border-neutral-200 px-3 py-1.5 rounded-lg text-xs font-bold w-24 focus:outline-none focus:border-neutral-900"
                        value={item.precio_venta}
                        onChange={(e) => {
                          const newInv = [...inventory];
                          newInv[inventory.indexOf(item)].precio_venta = parseFloat(e.target.value);
                          setInventory(newInv);
                        }}
                      />
                    ) : (
                      <span className="text-xs font-black text-neutral-900">${item.precio_venta.toFixed(2)}</span>
                    )}
                  </td>
                  <td className="px-6 py-4 text-right">
                    <span className={`text-[10px] font-black px-2 py-1 rounded-md ${parseFloat(margin) > 30 ? 'bg-green-50 text-green-600' : 'bg-neutral-50 text-neutral-400'}`}>
                      {margin}%
                    </span>
                  </td>
                  <td className="px-6 py-4 text-right">
                    <div className="flex justify-end gap-2">
                      {isEditing ? (
                        <button onClick={() => handleUpdateItem(item)} className="p-2 bg-green-500 text-white rounded-lg hover:scale-110 shadow-lg shadow-green-200 transition-all">
                          <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="4" strokeLinecap="round" strokeLinejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
                        </button>
                      ) : (
                        <>
                          <button onClick={() => item.id !== undefined && setEditingId(item.id)} className="p-2 bg-neutral-50 text-neutral-400 rounded-lg hover:text-neutral-900 hover:bg-neutral-100 transition-all">
                            <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"><path d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z"/></svg>
                          </button>
                          <button onClick={() => item.id && handleDeleteItem(item.id)} className="p-2 bg-neutral-50 text-neutral-400 rounded-lg hover:text-red-500 hover:bg-red-50 transition-all">
                            <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round"><path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/></svg>
                          </button>
                        </>
                      )}
                    </div>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>

      <div className="grid grid-cols-2 gap-8">
        <div className="bg-white rounded-[2.5rem] border border-neutral-200 p-8 shadow-sm">
          <div className="flex items-center justify-between mb-8">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 bg-red-50 text-red-500 rounded-2xl flex items-center justify-center text-lg">⚠️</div>
              <div>
                <h3 className="text-xs font-black text-neutral-900 uppercase tracking-widest">Alerta de Stock Bajo</h3>
                <p className="text-[9px] text-neutral-400 font-bold uppercase tracking-tighter">Productos por agotarse</p>
              </div>
            </div>
            <span className="px-3 py-1 bg-red-500 text-white text-[9px] font-black rounded-lg">
              {inventory.filter(i => i.stock <= i.stock_minimo).length} CRÍTICOS
            </span>
          </div>
          
          <div className="space-y-3 max-h-64 overflow-y-auto pr-2 custom-scrollbar">
            {inventory.filter(i => i.stock <= i.stock_minimo).length > 0 ? (
              inventory.filter(i => i.stock <= i.stock_minimo).map((item, idx) => (
                <div key={idx} className="flex items-center justify-between p-4 bg-neutral-50 rounded-2xl border border-neutral-100 group hover:border-red-200 transition-all">
                  <div className="flex items-center gap-3">
                    <div className="w-2 h-2 rounded-full bg-red-500 animate-pulse"></div>
                    <span className="text-[11px] font-bold text-neutral-700 uppercase">{item.nombre}</span>
                  </div>
                  <div className="text-right">
                    <p className={`text-[10px] font-black uppercase ${item.stock < 0 ? 'text-red-600' : 'text-red-500'}`}>{item.stock < 0 ? `FALTANTE: -${Math.abs(item.stock)}` : `Quedan: ${item.stock}`}</p>
                    <p className="text-[8px] text-neutral-400 font-bold uppercase">Min: {item.stock_minimo}</p>
                  </div>
                </div>
              ))
            ) : (
              <div className="py-12 text-center text-[10px] font-black text-neutral-300 uppercase tracking-widest italic">
                Stock saludable. ¡Buen trabajo!
              </div>
            )}
          </div>
        </div>

        <div className="bg-neutral-900 rounded-[2.5rem] p-8 shadow-2xl relative overflow-hidden">
          <div className="absolute top-0 right-0 w-32 h-32 bg-white/5 rounded-full -translate-y-16 translate-x-16 blur-3xl"></div>
          
          <div className="flex items-center justify-between mb-8 relative z-10">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 bg-white/10 text-white rounded-2xl flex items-center justify-center text-lg">✨</div>
              <div>
                <h3 className="text-xs font-black text-white uppercase tracking-widest">Predicciones de Compra</h3>
                <p className="text-[9px] text-neutral-500 font-bold uppercase tracking-tighter">Sugerencias Inteligentes</p>
              </div>
            </div>
          </div>

          <div className="space-y-3 relative z-10">
            <div className="p-4 bg-white/5 border border-white/10 rounded-2xl flex items-center justify-between">
              <span className="text-[10px] font-bold text-neutral-300 uppercase">Sugerencia de compra...</span>
              <span className="text-[10px] font-black text-white uppercase italic opacity-30">Analizando mercado...</span>
            </div>
            <p className="text-[8px] text-neutral-600 uppercase font-black tracking-widest text-center mt-6">
              * El motor de IA se activará al conectar el Sidecar de Python.
            </p>
          </div>
        </div>
      </div>

      <div className="bg-white rounded-[2.5rem] border border-neutral-200 p-10 shadow-sm space-y-8">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="text-xl font-black text-neutral-900 uppercase tracking-tight">Conciliación de Inventario</h3>
            <p className="text-[9px] text-neutral-400 uppercase font-black tracking-widest">Físico vs Sistema</p>
          </div>
        </div>

        <div className="border border-neutral-100 rounded-3xl overflow-hidden">
          <table className="w-full text-left border-collapse">
            <thead>
              <tr className="bg-neutral-50 text-neutral-400 border-b border-neutral-100">
                <th className="px-8 py-4 text-[9px] font-black uppercase tracking-widest">Producto</th>
                <th className="px-6 py-4 text-[9px] font-black uppercase tracking-widest text-center">Físico</th>
                <th className="px-6 py-4 text-[9px] font-black uppercase tracking-widest text-center">Sistema</th>
                <th className="px-6 py-4 text-[9px] font-black uppercase tracking-widest text-center">Dif.</th>
                <th className="px-8 py-4 text-[9px] font-black uppercase tracking-widest text-right">Estado</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-neutral-50">
              <tr className="opacity-40">
                <td colSpan={5} className="px-8 py-12 text-center text-[10px] font-black text-neutral-300 uppercase tracking-widest italic">
                  Módulo de auditoría listo para sincronización manual.
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
};

export default Inventario;
