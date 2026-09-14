// ═══════════════════════════════════════════════════════════════════════════
// TARJETA ROJA — Sin match: captura manual que alimenta el aprendizaje.
// [ASIGNAR A EXISTENTE] pega el ean a un producto (buscador top-6).
// [DAR DE ALTA] crea el producto con ficha completa (marca + presentación)
// para que la próxima vez caiga mínimo en amarillo. Todo con morphicons.
// ═══════════════════════════════════════════════════════════════════════════

import { useMemo, useState } from "react";
import { MorphIcon } from "morphicons/react";
import {
  ICONO_ALERTA_CIRCULO,
  ICONO_BUSCAR,
  ICONO_CHECK,
  ICONO_CODIGO_BARRAS,
  ICONO_MAS,
} from "../../../icons";
import { notificarExito } from "../../../components/notificaciones";
import { reportarError } from "../../../services/tauri";
import type { InventoryItem } from "../../../services/inventario";
import {
  resolverRojoAlta,
  resolverRojoAsignando,
  type PendienteCodigo,
} from "../../../services/semaforo";

interface TarjetaRojaProps {
  pendiente: PendienteCodigo;
  inventario: InventoryItem[];
  onResuelto: () => void;
}

const UNIDADES = ["ml", "l", "g", "kg", "pzs"];

const TarjetaRoja = ({ pendiente, inventario, onResuelto }: TarjetaRojaProps) => {
  const [modo, setModo] = useState<"ninguno" | "asignar" | "alta">("ninguno");
  const [busqueda, setBusqueda] = useState("");
  const [resolviendo, setResolviendo] = useState(false);
  const [nombre, setNombre] = useState(pendiente.nombre_crudo);
  const [marca, setMarca] = useState("");
  const [cantidad, setCantidad] = useState("");
  const [unidad, setUnidad] = useState("ml");
  const [categoria, setCategoria] = useState("");

  const resultados = useMemo(() => {
    const q = busqueda.trim().toLowerCase();
    if (q.length < 2) return [];
    return inventario
      .filter(
        (it) =>
          it.nombre.toLowerCase().includes(q) ||
          (it.codigo_barras ?? "").toLowerCase().includes(q),
      )
      .slice(0, 6);
  }, [busqueda, inventario]);

  const asignar = async (productoId: number) => {
    setResolviendo(true);
    try {
      await resolverRojoAsignando(pendiente.id, productoId, pendiente.ean ?? undefined);
      notificarExito("Código asignado y aprendido");
      onResuelto();
    } catch (e) {
      reportarError("No se pudo asignar el código", e);
    } finally {
      setResolviendo(false);
    }
  };

  const darDeAlta = async () => {
    if (nombre.trim().length === 0) {
      reportarError("El nombre del producto es obligatorio", "Sin nombre");
      return;
    }
    const cant = cantidad.trim() ? Number(cantidad.replace(",", ".")) : undefined;
    if (cantidad.trim() && !Number.isFinite(cant)) {
      reportarError("La cantidad no es un número válido", cantidad);
      return;
    }
    setResolviendo(true);
    try {
      await resolverRojoAlta(
        pendiente.id,
        nombre.trim().toUpperCase(),
        pendiente.ean ?? undefined,
        categoria.trim() || undefined,
        marca.trim() || undefined,
        cant,
        cant !== undefined ? unidad : undefined,
      );
      notificarExito("Producto dado de alta");
      onResuelto();
    } catch (e) {
      reportarError("No se pudo dar de alta", e);
    } finally {
      setResolviendo(false);
    }
  };

  return (
    <div className="overflow-hidden rounded-2xl border border-red-200 bg-white shadow-sm">
      <div className="flex items-stretch gap-0">
        <div className="w-1.5 shrink-0 bg-red-400" />
        <div className="flex-1 p-4">
          <div className="flex items-start justify-between gap-3">
            <div>
              <p className="flex items-center gap-1.5 font-mono text-xs font-black text-neutral-900">
                <MorphIcon icon={ICONO_CODIGO_BARRAS} size={14} strokeWidth={2.5} className="text-red-500" />
                {pendiente.ean ?? "SIN EAN"}
              </p>
              <p className="mt-1 text-[11px] font-bold uppercase text-neutral-500">
                Ticket: <span className="text-neutral-900">‘{pendiente.nombre_crudo}’</span>
              </p>
            </div>
            <span className="flex items-center gap-1 rounded-lg bg-red-50 px-2 py-1 text-[9px] font-black uppercase tracking-widest text-red-600">
              <MorphIcon icon={ICONO_ALERTA_CIRCULO} size={12} strokeWidth={2.5} />
              {pendiente.veces_visto}× visto
            </span>
          </div>

          <div className="mt-3 flex gap-2">
            <button
              onClick={() => setModo(modo === "asignar" ? "ninguno" : "asignar")}
              className="flex flex-1 items-center justify-center gap-1.5 rounded-xl bg-neutral-900 px-3 py-2.5 text-[10px] font-black uppercase tracking-widest text-white hover:bg-neutral-700"
            >
              <MorphIcon icon={ICONO_BUSCAR} size={14} strokeWidth={2.5} /> Asignar a existente
            </button>
            <button
              onClick={() => setModo(modo === "alta" ? "ninguno" : "alta")}
              className="flex flex-1 items-center justify-center gap-1.5 rounded-xl bg-red-500 px-3 py-2.5 text-[10px] font-black uppercase tracking-widest text-white hover:bg-red-600"
            >
              <MorphIcon icon={ICONO_MAS} size={14} strokeWidth={3} /> Dar de alta
            </button>
          </div>

          {modo === "asignar" && (
            <div className="mt-3 rounded-xl border border-neutral-100 bg-neutral-50/60 p-3">
              <input
                autoFocus
                value={busqueda}
                onChange={(e) => setBusqueda(e.target.value)}
                placeholder="Buscar producto por nombre o código…"
                className="w-full rounded-xl border border-neutral-200 bg-white px-3 py-2 text-xs font-bold outline-none focus:border-neutral-900"
              />
              <div className="mt-2 space-y-1.5">
                {resultados.map((it) => (
                  <div key={it.id} className="flex items-center justify-between gap-2 rounded-lg bg-white border border-neutral-100 px-3 py-2">
                    <p className="truncate text-[11px] font-black uppercase text-neutral-900">{it.nombre}</p>
                    <button
                      onClick={() => it.id !== undefined && asignar(it.id)}
                      disabled={resolviendo}
                      className="shrink-0 rounded-lg bg-neutral-900 px-3 py-1.5 text-[9px] font-black uppercase tracking-widest text-white hover:bg-neutral-700 disabled:opacity-50"
                    >
                      Pegar código
                    </button>
                  </div>
                ))}
                {busqueda.trim().length >= 2 && resultados.length === 0 && (
                  <p className="text-[10px] font-bold uppercase text-neutral-400">Sin coincidencias, dalo de alta</p>
                )}
              </div>
            </div>
          )}

          {modo === "alta" && (
            <div className="mt-3 space-y-2 rounded-xl border border-neutral-100 bg-neutral-50/60 p-3">
              <input
                value={nombre}
                onChange={(e) => setNombre(e.target.value)}
                placeholder="Nombre del producto *"
                className="w-full rounded-xl border border-neutral-200 bg-white px-3 py-2 text-xs font-bold uppercase outline-none focus:border-neutral-900"
              />
              <div className="grid grid-cols-2 gap-2">
                <input
                  value={marca}
                  onChange={(e) => setMarca(e.target.value)}
                  placeholder="Marca"
                  className="rounded-xl border border-neutral-200 bg-white px-3 py-2 text-xs font-bold outline-none focus:border-neutral-900"
                />
                <input
                  value={categoria}
                  onChange={(e) => setCategoria(e.target.value)}
                  placeholder="Categoría"
                  className="rounded-xl border border-neutral-200 bg-white px-3 py-2 text-xs font-bold outline-none focus:border-neutral-900"
                />
                <input
                  value={cantidad}
                  onChange={(e) => setCantidad(e.target.value)}
                  inputMode="decimal"
                  placeholder="Cantidad (ej. 600)"
                  className="rounded-xl border border-neutral-200 bg-white px-3 py-2 text-xs font-bold outline-none focus:border-neutral-900"
                />
                <select
                  value={unidad}
                  onChange={(e) => setUnidad(e.target.value)}
                  className="rounded-xl border border-neutral-200 bg-white px-3 py-2 text-xs font-bold outline-none focus:border-neutral-900"
                >
                  {UNIDADES.map((u) => (
                    <option key={u} value={u}>{u}</option>
                  ))}
                </select>
              </div>
              <button
                onClick={darDeAlta}
                disabled={resolviendo}
                className="flex w-full items-center justify-center gap-1.5 rounded-xl bg-red-500 px-3 py-2.5 text-[10px] font-black uppercase tracking-widest text-white hover:bg-red-600 disabled:opacity-50"
              >
                <MorphIcon icon={ICONO_CHECK} size={14} strokeWidth={3} /> {resolviendo ? "Guardando…" : "Guardar y aprender"}
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default TarjetaRoja;
