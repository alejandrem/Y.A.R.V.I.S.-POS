// Shell del módulo Parseador: header + pastilla TICKETS/CORTES con slider negro.
// Guarda el estado y el flujo completos de tickets; cortes es provisional.
// El contenido de tickets vive en ./tickets y el de cortes en ./cortes.
import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import type { ArchivoTicket, BatchProgress, CatalogItem, DeteccionMapeo, Phase } from "./tickets/compartido";
import { PasosGrid, errorMessage, normalizeCatalogItem } from "./tickets/compartido";
import Catalogo from "./tickets/catalogo";
import Carpeta from "./tickets/carpeta";
import Progreso from "./tickets/progreso";
import Completo from "./tickets/completo";
import Historial from "./tickets/historial";
import Cortes from "./cortes/cortes";

type View = "tickets" | "cortes";

const tabs: { id: View; label: string }[] = [
  { id: "tickets", label: "Tickets" },
  { id: "cortes", label: "Cortes" },
];

const Parseador = () => {
  const [view, setView] = useState<View>("tickets");
  const [phase, setPhase] = useState<Phase>("catalogo");
  const [catalogPath, setCatalogPath] = useState("");
  const [catalogContent, setCatalogContent] = useState("");
  const [catalogItems, setCatalogItems] = useState<CatalogItem[]>([]);
  const [catalogImported, setCatalogImported] = useState(false);
  const [folderPath, setFolderPath] = useState("");
  const [ticketFiles, setTicketFiles] = useState<ArchivoTicket[]>([]);
  const [deteccion, setDeteccion] = useState<DeteccionMapeo | null>(null);
  const [batch, setBatch] = useState<BatchProgress | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const unlistenBatch = useRef<(() => void) | null>(null);

  const cleanupListeners = useCallback(() => {
    unlistenBatch.current?.();
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
      const msg = errorMessage(importError);
      if (msg.includes("ya fue importado")) {
        setError(`Este catálogo ya fue parseado anteriormente: ${catalogPath.split("/").pop()} — ya está en inventario. Puedes subir uno nuevo o continuar a Carpeta de tickets.`);
        setCatalogImported(true);
        // No bloquea: permitir saltar a carpeta aunque sea duplicado
        setTimeout(() => setPhase("carpeta"), 800);
      } else {
        setError(`No se pudo importar el catálogo: ${msg}`);
      }
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
    if (!folderPath || !ticketFiles.length) return;
    setBusy(true);
    setError("");
    setDeteccion(null);
    setBatch(null);
    cleanupListeners();

    try {
      // 1. Detección ESTADÍSTICA del formato (sin IA): el mapeo queda
      //    verificado matemáticamente contra cantidad×precio≈total en una
      //    muestra espaciada de la carpeta. Instantáneo, no carga modelos.
      const deteccionResult = await invoke<DeteccionMapeo>("detectar_mapeo_estadistico", { carpeta: folderPath });
      setDeteccion(deteccionResult);

      // 2. Parseo del lote con ese mapeo (per-file fallback incluido en el
      //    núcleo: archivos de otro formato se rescatan solos).
      setPhase("procesando");
      let completeReceived = false;
      unlistenBatch.current = await listen<BatchProgress>("batch-progress", (event) => {
        setBatch(event.payload);
        if (event.payload.type === "complete") completeReceived = true;
      });

      const dbPath = await invoke<string>("get_db_path");
      await invoke("parsear_carpeta_stream", {
        carpeta: folderPath,
        mapeo: deteccionResult.mapeo,
        dbPath,
      });

      if (!completeReceived) {
        await new Promise((resolve) => setTimeout(resolve, 700));
      }
      setPhase("completo");
      unlistenBatch.current?.();
      unlistenBatch.current = null;
    } catch (flowError) {
      setError(`No se pudo procesar: ${errorMessage(flowError)}`);
      setPhase("carpeta");
      cleanupListeners();
    } finally {
      setBusy(false);
    }
  };

  const handlePhaseChange = (next: Phase) => {
    if (next === "catalogo") {
      setPhase("catalogo");
      setError("");
      return;
    }
    if (next === "carpeta") {
      // Permitir saltar el catálogo: si ya está importado o si el usuario quiere ir directo a tickets
      setPhase("carpeta");
      setError("");
      return;
    }
    if (next === "historial") {
      setPhase("historial");
      setError("");
      return;
    }
    if (next === "completo") {
      setPhase("completo");
      setError("");
      return;
    }
  };

  const handleSkipCatalog = () => {
    setPhase("carpeta");
    setError("");
    // Aviso: sin catálogo, los productos de los tickets se crearán en inventario automáticamente
    if (!catalogImported) {
      setError("Continuando sin catálogo maestro: los productos de los tickets se crearán en inventario con su precio y cantidad vendida.");
      setTimeout(() => setError(""), 4000);
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
    setDeteccion(null);
    setBatch(null);
    setError("");
  };

  const batchTotal = batch?.total ?? batch?.total_archivos ?? ticketFiles.length;
  const batchPercent = batchTotal ? Math.min(100, Math.round(((batch?.procesados ?? 0) / batchTotal) * 100)) : 0;

  return (
    <div className="max-w-5xl animate-in fade-in slide-in-from-bottom-2 duration-500 mx-auto w-full">
      <header className="mb-8 text-left relative">
        <h2 className="text-3xl font-black text-neutral-900 uppercase tracking-tight mb-2">Parseador de Tickets</h2>
        <div className="h-1.5 w-12 bg-neutral-900 rounded-full"></div>
      </header>

      <div className="mb-8 flex justify-center">
        <nav className="relative inline-flex rounded-full border border-neutral-200 bg-neutral-100 p-1.5" aria-label="Secciones del parseador">
          <span
            aria-hidden="true"
            className="absolute inset-y-1.5 left-1.5 w-[calc(50%-0.375rem)] rounded-full bg-neutral-950 shadow-lg transition-transform duration-300 ease-out"
            style={{ transform: view === "tickets" ? "translateX(0)" : "translateX(100%)" }}
          />
          {tabs.map((tab) => {
            const active = view === tab.id;
            return (
              <button
                key={tab.id}
                onClick={() => setView(tab.id)}
                className={`relative z-10 w-40 sm:w-48 rounded-full py-3.5 text-[11px] sm:text-xs font-black uppercase tracking-widest transition-colors duration-300 ${active ? "text-neutral-50" : "text-neutral-950 hover:text-neutral-600"}`}
              >
                {tab.label}
              </button>
            );
          })}
        </nav>
      </div>

      {view === "tickets" ? (
        <>
          {error && <div className="mb-6 rounded-2xl bg-red-50 border border-red-100 text-red-700 px-5 py-4 text-sm font-bold">{error}</div>}
          <PasosGrid phase={phase} onPhaseChange={handlePhaseChange} />
          {phase === "catalogo" && (
            <>
              <Catalogo catalogPath={catalogPath} catalogItems={catalogItems} busy={busy} onSelectCatalog={selectCatalog} onImportCatalog={importCatalog} />
              <div className="mt-4 flex justify-center">
                <button onClick={handleSkipCatalog} className="text-[11px] font-black uppercase tracking-widest text-neutral-500 hover:text-neutral-900 underline decoration-dotted">
                  Saltar catálogo y subir carpeta de tickets directamente →
                </button>
              </div>
              <p className="text-center text-[10px] text-neutral-400 mt-2">Si subes 30 tickets sin catálogo, cada producto extraído (nombre, precio, cantidad vendida) se creará en inventario automáticamente.</p>
            </>
          )}
          {phase === "carpeta" && <Carpeta folderPath={folderPath} ticketFiles={ticketFiles} busy={busy} catalogImported={catalogImported} onSelectFolder={selectFolder} onStartFlow={startFlow} />}
          {phase === "procesando" && <Progreso batch={batch} batchTotal={batchTotal} batchPercent={batchPercent} />}
          {phase === "completo" && <Completo batch={batch} ticketFiles={ticketFiles} deteccion={deteccion} onReset={reset} />}
          {phase === "historial" && <Historial />}
        </>
      ) : (
        <Cortes />
      )}
    </div>
  );
};

export default Parseador;