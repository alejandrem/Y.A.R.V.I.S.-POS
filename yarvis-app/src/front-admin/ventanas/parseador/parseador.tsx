import { useCallback, useEffect, useRef, useState } from "react";
import type { ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";

interface CatalogItem {
  id: number | null;
  nombre: string;
  descripcion: string | null;
  precio_costo: number;
  precio_venta: number;
  stock: number;
  vendido: number;
  stock_minimo: number;
  codigo_barras: string | null;
  categoria: string | null;
}

interface ArchivoTicket {
  nombre: string;
  ruta: string;
  tamano: number;
  preview: string;
}

interface TrainingProgress {
  indice: number;
  total: number;
  archivo: string;
  estado: "ok" | "error";
  mensaje: string;
}

interface BatchProgress {
  type: "progress" | "complete";
  procesados: number;
  total?: number;
  total_archivos?: number;
  exitosos: number;
  errores: number;
  ventas_creadas?: number;
  items_insertados?: number;
  productos_nuevos?: number;
  productos_existentes?: number;
  duplicados_detectados?: number;
}

interface CalibrationResult {
  mapeo: Record<string, unknown>;
  analizados: number;
  total_muestras: number;
  votos_ganadores: number;
}

type Phase = "catalogo" | "carpeta" | "calibrando" | "procesando" | "completo";

const toNumber = (value: unknown, fallback = 0) => {
  const number = Number(value);
  return Number.isFinite(number) ? number : fallback;
};

const normalizeCatalogItem = (item: any): CatalogItem => ({
  id: item.id ?? null,
  nombre: String(item.nombre ?? item.producto ?? "").trim(),
  descripcion: item.descripcion ?? null,
  precio_costo: toNumber(item.precio_costo),
  precio_venta: toNumber(item.precio_venta),
  stock: toNumber(item.stock),
  vendido: toNumber(item.vendido),
  stock_minimo: toNumber(item.stock_minimo, 5),
  codigo_barras: item.codigo_barras ?? null,
  categoria: item.categoria ?? null,
});

const formatSize = (bytes: number) => {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
};

const errorMessage = (error: unknown) => error instanceof Error ? error.message : String(error);

const Parseador = () => {
  const [phase, setPhase] = useState<Phase>("catalogo");
  const [catalogPath, setCatalogPath] = useState("");
  const [catalogContent, setCatalogContent] = useState("");
  const [catalogItems, setCatalogItems] = useState<CatalogItem[]>([]);
  const [catalogImported, setCatalogImported] = useState(false);
  const [folderPath, setFolderPath] = useState("");
  const [ticketFiles, setTicketFiles] = useState<ArchivoTicket[]>([]);
  const [training, setTraining] = useState<TrainingProgress[]>([]);
  const [trainingResult, setTrainingResult] = useState<CalibrationResult | null>(null);
  const [batch, setBatch] = useState<BatchProgress | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const unlistenTraining = useRef<(() => void) | null>(null);
  const unlistenBatch = useRef<(() => void) | null>(null);

  const cleanupListeners = useCallback(() => {
    unlistenTraining.current?.();
    unlistenBatch.current?.();
    unlistenTraining.current = null;
    unlistenBatch.current = null;
  }, []);

  useEffect(() => cleanupListeners, [cleanupListeners]);

  const selectCatalog = async () => {
    setError("");
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "Catálogos", extensions: ["txt", "csv", "xlsx", "xls"] }],
      });
      if (!selected || Array.isArray(selected)) return;

      const path = selected as string;
      const extension = path.split(".").pop()?.toLowerCase();
      let items: any[] = [];
      let content = "";

      if (extension === "xlsx" || extension === "xls") {
        const bytes = await invoke<number[]>("leer_archivo_bytes", { path });
        const result = await invoke<any>("parsear_excel", { archivo: bytes });
        items = result.productos ?? [];
        content = `Excel cargado: ${items.length} productos`;
      } else {
        content = await invoke<string>("leer_archivo_raw", { path });
        try {
          const visual = await invoke<any>("parsear_catalogo_visual", { path });
          items = visual.productos ?? [];
        } catch {
          items = await invoke<any[]>("parsear_catalogo_csv", { path });
        }
      }

      const normalized = items.map(normalizeCatalogItem).filter((item) => item.nombre);
      if (!normalized.length) throw new Error("No se encontraron productos válidos en el catálogo");
      setCatalogPath(path);
      setCatalogContent(content);
      setCatalogItems(normalized);
      setCatalogImported(false);
    } catch (selectionError) {
      setError(`No se pudo leer el catálogo: ${errorMessage(selectionError)}`);
    }
  };

  const importCatalog = async () => {
    if (!catalogItems.length || !catalogPath) return;
    setBusy(true);
    setError("");
    try {
      await invoke("importar_catalogo", {
        items: catalogItems,
        rutaArchivo: catalogPath,
        contenidoArchivo: catalogContent,
      });
      setCatalogImported(true);
      setPhase("carpeta");
    } catch (importError) {
      setError(`No se pudo importar el catálogo: ${errorMessage(importError)}`);
    } finally {
      setBusy(false);
    }
  };

  const selectFolder = async () => {
    setError("");
    try {
      const selected = await open({ directory: true, multiple: false });
      if (!selected || Array.isArray(selected)) return;
      const path = selected as string;
      const files = await invoke<ArchivoTicket[]>("listar_archivos_carpeta", { carpeta: path });
      if (!files.length) throw new Error("La carpeta no contiene archivos .txt");
      setFolderPath(path);
      setTicketFiles(files);
      setPhase("carpeta");
    } catch (selectionError) {
      setError(`No se pudo leer la carpeta: ${errorMessage(selectionError)}`);
    }
  };

  const startFlow = async () => {
    if (!catalogImported || !folderPath || !ticketFiles.length) return;
    setBusy(true);
    setError("");
    setTraining([]);
    setTrainingResult(null);
    setBatch(null);
    cleanupListeners();

    try {
      setPhase("calibrando");
      unlistenTraining.current = await listen<TrainingProgress>("parser-training-progress", (event) => {
        setTraining((current) => [...current, event.payload]);
      });
      const calibration = await invoke<CalibrationResult>("analizar_muestras_carpeta", { carpeta: folderPath });
      unlistenTraining.current?.();
      unlistenTraining.current = null;
      setTrainingResult(calibration);

      setPhase("procesando");
      let completeReceived = false;
      unlistenBatch.current = await listen<BatchProgress>("batch-progress", (event) => {
        setBatch(event.payload);
        if (event.payload.type === "complete") completeReceived = true;
      });

      const dbPath = await invoke<string>("get_db_path");
      await invoke("parsear_carpeta_stream", {
        carpeta: folderPath,
        mapeo: calibration.mapeo,
        dbPath,
      });

      if (!completeReceived) {
        await new Promise((resolve) => setTimeout(resolve, 700));
      }
      setPhase("completo");
      unlistenBatch.current?.();
      unlistenBatch.current = null;
    } catch (flowError) {
      setError(`El proceso se detuvo: ${errorMessage(flowError)}`);
      setPhase("carpeta");
      cleanupListeners();
    } finally {
      setBusy(false);
    }
  };

  const reset = () => {
    cleanupListeners();
    setPhase("catalogo");
    setCatalogPath("");
    setCatalogContent("");
    setCatalogItems([]);
    setCatalogImported(false);
    setFolderPath("");
    setTicketFiles([]);
    setTraining([]);
    setTrainingResult(null);
    setBatch(null);
    setError("");
  };

  const trainingTotal = trainingResult?.total_muestras ?? Math.min(ticketFiles.length, 5);
  const trainingDone = training.length;
  const trainingPercent = trainingTotal ? Math.min(100, Math.round((trainingDone / trainingTotal) * 100)) : 0;
  const batchTotal = batch?.total ?? batch?.total_archivos ?? ticketFiles.length;
  const batchPercent = batchTotal ? Math.min(100, Math.round(((batch?.procesados ?? 0) / batchTotal) * 100)) : 0;

  return (
    <div className="max-w-5xl animate-in fade-in slide-in-from-bottom-2 duration-500 mx-auto w-full">
      <header className="mb-8 text-left relative">
        <h2 className="text-3xl font-black text-neutral-900 uppercase tracking-tight mb-2">Parseador de Tickets</h2>
        <div className="h-1.5 w-12 bg-neutral-900 rounded-full"></div>
      </header>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-3 mb-8">
        {[
          ["01", "Catálogo maestro", phase !== "catalogo"],
          ["02", "Carpeta de tickets", ["calibrando", "procesando", "completo"].includes(phase)],
          ["03", "Análisis y parseo", phase === "completo" || phase === "procesando" || phase === "calibrando"],
        ].map(([number, label, done]) => (
          <div key={String(number)} className={`rounded-2xl border p-4 flex items-center gap-3 ${done ? "bg-neutral-900 text-white border-neutral-900" : "bg-white border-neutral-100 text-neutral-400"}`}>
            <span className="text-xs font-black opacity-60">{number}</span>
            <span className="text-[10px] font-black uppercase tracking-widest">{label}</span>
          </div>
        ))}
      </div>

      {error && <div className="mb-6 rounded-2xl bg-red-50 border border-red-100 text-red-700 px-5 py-4 text-sm font-bold">{error}</div>}

      {phase === "catalogo" && (
        <section className="bg-white rounded-[2.5rem] border border-neutral-100 shadow-xl p-6 sm:p-10 space-y-6">
          <div>
            <p className="text-[10px] font-black uppercase tracking-[0.35em] text-neutral-400">Paso 1 · Fuente de verdad</p>
            <h3 className="text-2xl font-black text-neutral-900 mt-2">Carga tu catálogo maestro</h3>
            <p className="text-sm text-neutral-500 mt-2">Se usará para reconocer productos y mantener el inventario consistente.</p>
          </div>
          <button onClick={selectCatalog} className="w-full border-2 border-dashed border-neutral-200 rounded-3xl py-10 hover:border-neutral-900 hover:bg-neutral-50 transition-colors">
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
              <button disabled={busy} onClick={importCatalog} className="w-full mt-5 rounded-2xl bg-neutral-900 text-white py-4 text-[10px] font-black uppercase tracking-widest disabled:opacity-40">{busy ? "Importando catálogo..." : "Importar catálogo maestro"}</button>
            </div>
          )}
        </section>
      )}

      {phase === "carpeta" && (
        <section className="bg-white rounded-[2.5rem] border border-neutral-100 shadow-xl p-6 sm:p-10 space-y-6">
          <div className="flex items-start justify-between gap-4">
            <div><p className="text-[10px] font-black uppercase tracking-[0.35em] text-neutral-400">Paso 2 · Lote de tickets</p><h3 className="text-2xl font-black text-neutral-900 mt-2">Selecciona la carpeta completa</h3><p className="text-sm text-neutral-500 mt-2">Primero se analizarán 5 tickets al azar; después se procesará todo el lote.</p></div>
            <span className="rounded-xl bg-emerald-50 text-emerald-700 px-3 py-2 text-[10px] font-black uppercase">Catálogo listo</span>
          </div>
          <button onClick={selectFolder} className="w-full border-2 border-dashed border-neutral-200 rounded-3xl py-10 hover:border-neutral-900 hover:bg-neutral-50 transition-colors"><div className="text-3xl mb-3">▰</div><span className="text-[11px] font-black uppercase tracking-widest text-neutral-500">Seleccionar carpeta de tickets TXT</span>{folderPath && <p className="text-xs text-neutral-900 font-bold mt-3 break-all px-4">{folderPath}</p>}</button>
          {!!ticketFiles.length && <div className="rounded-2xl bg-neutral-50 p-5"><div className="flex justify-between items-center"><span className="text-[10px] font-black uppercase tracking-widest text-neutral-400">Carpeta preparada</span><span className="text-lg font-black text-neutral-900">{ticketFiles.length} tickets</span></div><p className="text-xs text-neutral-500 mt-2">{formatSize(ticketFiles.reduce((sum, file) => sum + file.tamano, 0))} en archivos TXT · se elegirán {Math.min(5, ticketFiles.length)} muestras automáticamente.</p><button disabled={busy} onClick={startFlow} className="w-full mt-5 rounded-2xl bg-neutral-900 text-white py-5 text-[10px] font-black uppercase tracking-widest disabled:opacity-40">{busy ? "Preparando..." : "Analizar y parsear carpeta"}</button></div>}
        </section>
      )}

      {phase === "calibrando" && (
        <ProgressCard title="Calibrando el analizador" subtitle="La IA está comparando 5 tickets aleatorios para encontrar el patrón más estable." current={trainingDone} total={trainingTotal} percent={trainingPercent}>
          <div className="space-y-2">{training.map((ticket, index) => <div key={`${ticket.archivo}-${index}`} className="flex items-center gap-3 rounded-xl bg-neutral-50 px-4 py-3"><span className={`w-2 h-2 rounded-full ${ticket.estado === "ok" ? "bg-emerald-500" : "bg-amber-500"}`}></span><span className="text-xs font-bold text-neutral-700 truncate">{ticket.mensaje}</span><span className="ml-auto text-[10px] text-neutral-400 truncate max-w-[35%]">{ticket.archivo}</span></div>)}</div>
        </ProgressCard>
      )}

      {phase === "procesando" && <ProgressCard title="Parseando la carpeta completa" subtitle="Cada ticket se está convirtiendo e insertando en el inventario y el historial." current={batch?.procesados ?? 0} total={batchTotal} percent={batchPercent}><div className="grid grid-cols-2 sm:grid-cols-4 gap-3">{[["Procesados", batch?.procesados ?? 0], ["Correctos", batch?.exitosos ?? 0], ["Errores", batch?.errores ?? 0], ["Productos nuevos", batch?.productos_nuevos ?? 0]].map(([label, value]) => <div key={String(label)} className="rounded-2xl bg-neutral-50 p-4"><p className="text-[9px] font-black uppercase tracking-widest text-neutral-400">{label}</p><p className="text-2xl font-black text-neutral-900 mt-1">{value}</p></div>)}</div></ProgressCard>}

      {phase === "completo" && <section className="bg-neutral-900 text-white rounded-[2.5rem] shadow-xl p-8 sm:p-12 text-center"><div className="mx-auto w-16 h-16 rounded-full bg-emerald-400 text-neutral-900 flex items-center justify-center text-3xl font-black">✓</div><p className="text-[10px] font-black uppercase tracking-[0.35em] text-neutral-400 mt-6">Proceso terminado</p><h3 className="text-3xl font-black mt-2">Carpeta parseada correctamente</h3><p className="text-neutral-400 text-sm mt-3">El patrón ganador fue aplicado a los {batch?.procesados ?? ticketFiles.length} tickets.</p><div className="grid grid-cols-2 sm:grid-cols-4 gap-3 mt-8 text-left">{[["Tickets", batch?.procesados ?? ticketFiles.length], ["Correctos", batch?.exitosos ?? 0], ["Ventas", batch?.ventas_creadas ?? 0], ["Items", batch?.items_insertados ?? 0]].map(([label, value]) => <div key={String(label)} className="rounded-2xl bg-white/10 p-4"><p className="text-[9px] font-black uppercase tracking-widest text-neutral-400">{label}</p><p className="text-2xl font-black mt-1">{value}</p></div>)}</div><button onClick={reset} className="mt-8 rounded-2xl bg-white text-neutral-900 px-8 py-4 text-[10px] font-black uppercase tracking-widest">Procesar otra carpeta</button>{trainingResult && <p className="text-[10px] text-neutral-500 mt-5">Mapeo elegido por {trainingResult.votos_ganadores} de {trainingResult.total_muestras} muestras válidas.</p>}</section>}
    </div>
  );
};

const ProgressCard = ({ title, subtitle, current, total, percent, children }: { title: string; subtitle: string; current: number; total: number; percent: number; children: ReactNode }) => (
  <section className="bg-white rounded-[2.5rem] border border-neutral-100 shadow-xl p-6 sm:p-10">
    <div className="flex items-center gap-4"><div className="w-12 h-12 rounded-2xl bg-neutral-900 flex items-center justify-center"><div className="w-5 h-5 rounded-full border-2 border-white border-t-transparent animate-spin" /></div><div><p className="text-[10px] font-black uppercase tracking-[0.35em] text-neutral-400">Procesamiento automático</p><h3 className="text-2xl font-black text-neutral-900 mt-1">{title}</h3></div></div>
    <p className="text-sm text-neutral-500 mt-6">{subtitle}</p>
    <div className="mt-8"><div className="flex justify-between text-[10px] font-black uppercase tracking-widest text-neutral-400 mb-2"><span>{current} de {total}</span><span>{percent}%</span></div><div className="h-4 rounded-full bg-neutral-100 overflow-hidden"><div className="h-full rounded-full bg-neutral-900 transition-all duration-500" style={{ width: `${percent}%` }} /></div></div>
    <div className="mt-8">{children}</div>
  </section>
);

export default Parseador;
