// Orquestador del módulo Cortes: flujo catálogo → carpeta → parseo →
// completo, más historial. Mismo flujo y mismos componentes de
// catálogo que tickets (el catálogo maestro es compartido: los
// artículos de los cortes se vinculan contra ese inventario).
import { useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import type { ArchivoTicket, CatalogItem } from "./compartido";
import { PasosCortes, errorMessage, normalizeCatalogItem, type PhaseCortes } from "./compartido";
import { reportarError } from "../../../../services/tauri";
import { importarCarpetaCortes, type ResumenCortes } from "../../../../services/cortes";
import Catalogo from "../tickets/catalogo";
import CarpetaCortes from "./carpeta";
import ProgresoCortes from "./progreso";
import CompletoCortes from "./completo";
import HistorialCortes from "./historial";

const Cortes = () => {
  const [phase, setPhase] = useState<PhaseCortes>("catalogo");
  const [catalogPath, setCatalogPath] = useState("");
  const [catalogContent, setCatalogContent] = useState("");
  const [catalogItems, setCatalogItems] = useState<CatalogItem[]>([]);
  const [catalogImported, setCatalogImported] = useState(false);
  const [folderPath, setFolderPath] = useState("");
  const [corteFiles, setCorteFiles] = useState<ArchivoTicket[]>([]);
  const [resumen, setResumen] = useState<ResumenCortes | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

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
    } catch (e) {
      setError(`No se pudo leer el catálogo: ${errorMessage(e)}`);
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
    } catch (e) {
      const msg = errorMessage(e);
      if (msg.includes("ya fue importado")) {
        setError(`Este catálogo ya fue parseado anteriormente: ${catalogPath.split("/").pop()} — ya está en inventario. Puedes subir uno nuevo o continuar a Carpeta de cortes.`);
        setCatalogImported(true);
        setTimeout(() => setPhase("carpeta"), 800);
      } else {
        setError(`No se pudo importar el catálogo: ${msg}`);
      }
    } finally {
      setBusy(false);
    }
  };

  const handleSkipCatalog = () => {
    setPhase("carpeta");
    setError("");
    // Aviso: sin catálogo, los artículos de los cortes se crearán en inventario automáticamente.
    if (!catalogImported) {
      setError("Continuando sin catálogo maestro: los artículos de los cortes se crearán en inventario con su precio y cantidad vendida.");
      setTimeout(() => setError(""), 4000);
    }
  };

  const selectFolder = async () => {
    setError("");
    try {
      const selected = await open({ directory: true, multiple: false });
      if (!selected || Array.isArray(selected)) return;
      const files = await invoke<ArchivoTicket[]>("listar_archivos_carpeta", { carpeta: selected });
      if (!files.length) throw new Error("La carpeta no contiene archivos .txt");
      setFolderPath(selected as string);
      setCorteFiles(files);
      setPhase("carpeta");
    } catch (e) {
      setError(`No se pudo leer la carpeta: ${errorMessage(e)}`);
    }
  };

  const startFlow = async () => {
    if (!folderPath || !corteFiles.length || busy) return;
    setBusy(true);
    setError("");
    setResumen(null);
    setPhase("procesando");
    try {
      const r = await importarCarpetaCortes(folderPath);
      setResumen(r);
      setPhase("completo");
    } catch (e) {
      const msg = `No se pudo procesar: ${errorMessage(e)}`;
      setError(msg);
      reportarError("No se pudieron parsear los cortes", e);
      setPhase("carpeta");
    } finally {
      setBusy(false);
    }
  };

  const reset = () => {
    setPhase("catalogo");
    setCatalogPath("");
    setCatalogContent("");
    setCatalogItems([]);
    setCatalogImported(false);
    setFolderPath("");
    setCorteFiles([]);
    setResumen(null);
    setError("");
  };

  return (
    <div className="space-y-6">
      {error && <div className="rounded-2xl bg-red-50 border border-red-100 text-red-700 px-5 py-4 text-sm font-bold whitespace-pre-line">{error}</div>}
      <PasosCortes phase={phase} onPhaseChange={(next) => { setError(""); setPhase(next); }} />
      {phase === "catalogo" && (
        <>
          <Catalogo catalogPath={catalogPath} catalogItems={catalogItems} busy={busy} onSelectCatalog={selectCatalog} onImportCatalog={importCatalog} />
          <div className="mt-4 flex justify-center">
            <button onClick={handleSkipCatalog} className="text-[11px] font-black uppercase tracking-widest text-neutral-500 hover:text-neutral-900 underline decoration-dotted">
              Saltar catálogo y subir carpeta de cortes directamente →
            </button>
          </div>
          <p className="text-center text-[10px] text-neutral-400 mt-2">Los artículos de los cortes se vincularán contra este catálogo; sin él, cada artículo extraído se creará en inventario automáticamente.</p>
        </>
      )}
      {phase === "carpeta" && <CarpetaCortes folderPath={folderPath} corteFiles={corteFiles} busy={busy} onSelectFolder={selectFolder} onStartFlow={startFlow} />}
      {phase === "procesando" && <ProgresoCortes totalArchivos={corteFiles.length} />}
      {phase === "completo" && <CompletoCortes resumen={resumen} totalArchivos={corteFiles.length} onReset={reset} />}
      {phase === "historial" && <HistorialCortes />}
    </div>
  );
};

export default Cortes;
