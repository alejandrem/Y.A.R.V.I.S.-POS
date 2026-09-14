// ═══════════════════════════════════════════════════════════════════════════
// PANEL CÓDIGOS — Cola semáforo dentro de Inventario (issue #13).
// Contadores por color, buscador por ean/nombre, filtro por estado y
// resolución en lote. Las tarjetas viven en TarjetaAmarilla/TarjetaRoja;
// aquí solo orquesta: carga, filtra y recarga. Todo con morphicons.
// ═══════════════════════════════════════════════════════════════════════════

import { useEffect, useMemo, useState } from "react";
import { MorphIcon } from "morphicons/react";
import {
  ICONO_ALERTA_CIRCULO,
  ICONO_BUSCAR,
  ICONO_CHECK,
  ICONO_CHECK_CIRCULO,
  ICONO_CODIGO_BARRAS,
  ICONO_EQUIS,
  ICONO_INFO,
  ICONO_RELOJ,
} from "../../../icons";
import { notificarExito } from "../../../components/notificaciones";
import { reportarError } from "../../../services/tauri";
import { obtenerInventario, type InventoryItem } from "../../../services/inventario";
import {
  confirmarAmarillo,
  contarPendientes,
  contarVerdeHoy,
  listarPendientes,
  ningunoAmarillo,
  type ConteosPendientes,
  type ConteosVerde,
  type PendienteCodigo,
} from "../../../services/semaforo";
import TarjetaAmarilla from "./TarjetaAmarilla";
import TarjetaRoja from "./TarjetaRoja";

type FiltroEstado = "todas" | "amarillo" | "rojo" | "conflicto";

const FILTROS: { id: FiltroEstado; label: string }[] = [
  { id: "todas", label: "Todas" },
  { id: "amarillo", label: "Amarillas" },
  { id: "rojo", label: "Rojas" },
  { id: "conflicto", label: "Conflicto" },
];

const PanelCodigos = () => {
  const [conteos, setConteos] = useState<ConteosPendientes>({ rojo: 0, amarillo: 0, conflicto: 0, resuelto: 0 });
  const [verdes, setVerdes] = useState<ConteosVerde>({ verdes_hoy: 0, total_vinculos: 0 });
  const [pendientes, setPendientes] = useState<PendienteCodigo[]>([]);
  const [inventario, setInventario] = useState<InventoryItem[]>([]);
  const [busqueda, setBusqueda] = useState("");
  const [filtro, setFiltro] = useState<FiltroEstado>("todas");
  const [seleccionadas, setSeleccionadas] = useState<number[]>([]);
  const [cargando, setCargando] = useState(true);
  const [enLote, setEnLote] = useState(false);

  const cargar = async () => {
    setCargando(true);
    try {
      const [c, v, lista, inv] = await Promise.all([
        contarPendientes(),
        contarVerdeHoy(),
        listarPendientes(undefined, 200),
        obtenerInventario(),
      ]);
      setConteos(c);
      setVerdes(v);
      setPendientes(lista);
      setInventario(inv);
      setSeleccionadas([]);
    } catch (e) {
      reportarError("No se pudo cargar la cola de códigos", e);
    } finally {
      setCargando(false);
    }
  };

  useEffect(() => {
    cargar();
  }, []);

  const porId = useMemo(() => {
    const m = new Map<number, InventoryItem>();
    for (const it of inventario) {
      if (it.id !== undefined) m.set(it.id, it);
    }
    return m;
  }, [inventario]);

  const filtrados = useMemo(() => {
    const q = busqueda.trim().toLowerCase();
    return pendientes.filter((p) => {
      if (filtro !== "todas" && p.estado !== filtro) return false;
      if (!q) return true;
      return (
        p.nombre_crudo.toLowerCase().includes(q) ||
        (p.ean ?? "").toLowerCase().includes(q)
      );
    });
  }, [pendientes, busqueda, filtro]);

  const amarillos = filtrados.filter((p) => p.estado === "amarillo");
  const rojos = filtrados.filter((p) => p.estado === "rojo");
  const conflictos = filtrados.filter((p) => p.estado === "conflicto");

  const toggleSeleccion = (id: number) => {
    setSeleccionadas((prev) => (prev.includes(id) ? prev.filter((x) => x !== id) : [...prev, id]));
  };

  const confirmarLote = async () => {
    const candidatas = amarillos.filter((p) => seleccionadas.includes(p.id) && p.mejor_candidato_id);
    if (candidatas.length === 0) return;
    setEnLote(true);
    let ok = 0;
    let mal = 0;
    for (const p of candidatas) {
      try {
        await confirmarAmarillo(p.id, p.mejor_candidato_id!, p.ean ?? undefined);
        ok += 1;
      } catch {
        mal += 1;
      }
    }
    setEnLote(false);
    if (ok > 0) notificarExito(`${ok} código(s) confirmados en lote`);
    if (mal > 0) reportarError(`${mal} no se pudieron confirmar`, "Revisa la cola");
    await cargar();
  };

  const pasarARojo = async (id: number) => {
    try {
      await ningunoAmarillo(id);
      notificarExito("Pasó a rojo para captura manual");
      await cargar();
    } catch (e) {
      reportarError("No se pudo pasar a rojo", e);
    }
  };

  const tiles = [
    { label: "Verde auto hoy", valor: verdes.verdes_hoy, icon: ICONO_CHECK_CIRCULO, cls: "text-emerald-600 bg-emerald-50 border-emerald-200" },
    { label: "Amarillo por confirmar", valor: conteos.amarillo, icon: ICONO_ALERTA_CIRCULO, cls: "text-amber-600 bg-amber-50 border-amber-200" },
    { label: "Rojo sin match", valor: conteos.rojo, icon: ICONO_EQUIS, cls: "text-red-600 bg-red-50 border-red-200" },
    { label: "Conflicto", valor: conteos.conflicto, icon: ICONO_INFO, cls: "text-purple-600 bg-purple-50 border-purple-200" },
  ];

  return (
    <div className="w-full max-w-[1200px] animate-in fade-in slide-in-from-bottom-2 duration-500 mx-auto space-y-6">
      <header className="flex justify-between items-end">
        <div>
          <h2 className="text-3xl font-black text-neutral-900 uppercase tracking-tight mb-1">Códigos de barras</h2>
          <p className="text-[10px] font-black text-neutral-400 uppercase tracking-[0.3em]">
            Cola semáforo · {verdes.total_vinculos} vínculos · {conteos.resuelto} resueltos
          </p>
        </div>
        <button
          onClick={cargar}
          disabled={cargando}
          className="flex items-center gap-2 px-4 py-2.5 bg-neutral-900 text-white rounded-xl text-[10px] font-black uppercase tracking-widest hover:bg-neutral-700 disabled:opacity-50"
        >
          <MorphIcon icon={ICONO_RELOJ} size={14} strokeWidth={2.5} /> {cargando ? "Cargando…" : "Actualizar"}
        </button>
      </header>

      <div className="grid grid-cols-2 lg:grid-cols-4 gap-3">
        {tiles.map((t) => (
          <div key={t.label} className={`rounded-2xl border p-4 bg-white shadow-sm flex items-center gap-3 ${t.cls}`}>
            <MorphIcon icon={t.icon} size={22} strokeWidth={2.5} />
            <div>
              <p className="text-2xl font-black leading-none">{t.valor}</p>
              <p className="mt-1 text-[9px] font-black uppercase tracking-widest opacity-70">{t.label}</p>
            </div>
          </div>
        ))}
      </div>

      <div className="flex flex-col sm:flex-row gap-3 sm:items-center">
        <div className="relative group flex-1">
          <input
            type="text"
            placeholder="Buscar por ean o nombre…"
            value={busqueda}
            onChange={(e) => setBusqueda(e.target.value)}
            className="pl-10 pr-4 py-2.5 bg-white border border-neutral-200 rounded-xl text-xs font-bold focus:outline-none focus:ring-4 focus:ring-neutral-900/5 focus:border-neutral-900 transition-all w-full"
          />
          <div className="absolute left-3.5 top-1/2 -translate-y-1/2 text-neutral-300 group-focus-within:text-neutral-900 transition-colors">
            <MorphIcon icon={ICONO_BUSCAR} size={14} strokeWidth={3} />
          </div>
        </div>
        <div className="flex bg-white border border-neutral-200 p-1 rounded-xl">
          {FILTROS.map((f) => (
            <button
              key={f.id}
              onClick={() => setFiltro(f.id)}
              className={`px-3 py-2 text-[9px] font-black uppercase tracking-widest rounded-lg transition-all ${
                filtro === f.id ? "bg-neutral-900 text-white" : "text-neutral-400 hover:text-neutral-900"
              }`}
            >
              {f.label}
            </button>
          ))}
        </div>
      </div>

      {seleccionadas.length > 0 && (
        <div className="flex items-center justify-between rounded-2xl bg-neutral-900 text-white px-5 py-3">
          <p className="text-[10px] font-black uppercase tracking-widest">{seleccionadas.length} seleccionadas</p>
          <div className="flex gap-2">
            <button
              onClick={() => setSeleccionadas([])}
              className="px-3 py-2 text-[9px] font-black uppercase tracking-widest rounded-lg bg-white/10 hover:bg-white/20"
            >
              Limpiar
            </button>
            <button
              onClick={confirmarLote}
              disabled={enLote}
              className="flex items-center gap-1.5 px-3 py-2 text-[9px] font-black uppercase tracking-widest rounded-lg bg-emerald-500 hover:bg-emerald-600 disabled:opacity-50"
            >
              <MorphIcon icon={ICONO_CHECK} size={13} strokeWidth={3} /> {enLote ? "Confirmando…" : "Confirmar lote"}
            </button>
          </div>
        </div>
      )}

      {cargando ? (
        <p className="py-12 text-center text-[10px] font-black text-neutral-300 uppercase tracking-widest animate-pulse">
          Cargando la cola…
        </p>
      ) : filtrados.length === 0 ? (
        <div className="rounded-[2rem] border border-neutral-200 bg-white p-12 text-center">
          <MorphIcon icon={ICONO_CODIGO_BARRAS} size={32} strokeWidth={2} className="mx-auto text-neutral-300" />
          <p className="mt-3 text-xs font-black uppercase tracking-widest text-neutral-900">Cola vacía</p>
          <p className="mt-1 text-[10px] font-bold uppercase text-neutral-400">Pita o importa para estrenar el semáforo</p>
        </div>
      ) : (
        <div className="space-y-6">
          {amarillos.length > 0 && (
            <section className="space-y-3">
              <h3 className="text-[10px] font-black uppercase tracking-[0.3em] text-amber-600">Amarillas por confirmar ({amarillos.length})</h3>
              {amarillos.map((p) => (
                <TarjetaAmarilla
                  key={p.id}
                  pendiente={p}
                  candidatoNombre={p.mejor_candidato_id ? (porId.get(p.mejor_candidato_id)?.nombre ?? null) : null}
                  seleccionada={seleccionadas.includes(p.id)}
                  onToggleSeleccion={() => toggleSeleccion(p.id)}
                  onResuelto={cargar}
                />
              ))}
            </section>
          )}
          {rojos.length > 0 && (
            <section className="space-y-3">
              <h3 className="text-[10px] font-black uppercase tracking-[0.3em] text-red-500">Rojas sin match ({rojos.length})</h3>
              {rojos.map((p) => (
                <TarjetaRoja key={p.id} pendiente={p} inventario={inventario} onResuelto={cargar} />
              ))}
            </section>
          )}
          {conflictos.length > 0 && (
            <section className="space-y-3">
              <h3 className="text-[10px] font-black uppercase tracking-[0.3em] text-purple-600">Conflicto ({conflictos.length})</h3>
              {conflictos.map((p) => (
                <div key={p.id} className="flex items-center justify-between gap-3 rounded-2xl border border-purple-200 bg-white p-4 shadow-sm">
                  <div className="flex items-center gap-3">
                    <MorphIcon icon={ICONO_INFO} size={18} strokeWidth={2.5} className="text-purple-500 shrink-0" />
                    <div>
                      <p className="font-mono text-xs font-black text-neutral-900">{p.ean ?? "SIN EAN"}</p>
                      <p className="text-[11px] font-bold uppercase text-neutral-500">‘{p.nombre_crudo}’ · 2 productos lo reclaman</p>
                    </div>
                  </div>
                  <button
                    onClick={() => pasarARojo(p.id)}
                    className="shrink-0 rounded-xl bg-neutral-900 px-3 py-2 text-[9px] font-black uppercase tracking-widest text-white hover:bg-neutral-700"
                  >
                    Pasar a rojo
                  </button>
                </div>
              ))}
            </section>
          )}
        </div>
      )}
    </div>
  );
};

export default PanelCodigos;
