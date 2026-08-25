// ═══════════════════════════════════════════════════════════════════════════
// SERVICIO DE INVENTARIO — Única fuente de verdad para el tipo InventoryItem
// y para los comandos Tauri de inventario/catálogos. Los componentes deben
// consumir estas funciones en lugar de invocar `invoke` directamente.
// ═══════════════════════════════════════════════════════════════════════════

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

export interface CatalogoImportado {
  id: number;
  hash: string;
  ruta_archivo: string;
  fecha_importacion: string;
  total_productos: number;
}

export async function obtenerInventario(): Promise<InventoryItem[]> {
  return invoke<InventoryItem[]>("get_inventory");
}

export async function agregarProductoInventario(item: InventoryItem): Promise<void> {
  await invoke("add_inventory_item", { item });
}

export async function actualizarProductoInventario(item: InventoryItem): Promise<void> {
  await invoke("update_inventory_item", { item });
}

export async function eliminarProductoInventario(id: number): Promise<void> {
  await invoke("delete_inventory_item", { id });
}

export async function importarCatalogo(
  items: InventoryItem[],
  rutaArchivo?: string,
  contenidoArchivo?: string,
): Promise<string> {
  return invoke<string>("importar_catalogo", {
    items,
    rutaArchivo,
    contenidoArchivo,
  });
}

export async function obtenerCatalogosImportados(): Promise<CatalogoImportado[]> {
  return invoke<CatalogoImportado[]>("get_catalogos_importados");
}

export async function obtenerProductosPorCatalogo(catalogoId: number): Promise<InventoryItem[]> {
  return invoke<InventoryItem[]>("get_productos_por_catalogo", { catalogoId });
}
