// Hook personalizado que maneja toda la lógica compleja de análisis e importación de tickets 
// (interacción con la IA local/cloud, parseo de texto y sincronización con la BD).
import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useParserContext } from "../../../../hooks/ParserContext";

export function useAccionesParser() {
  const {
    parsedItems, setParsedItems,
    fileContent, setFileContent,
    selectedPath, setSelectedPath,
    parserMode, setParserMode,
    setShowColumnMapper,
    llmAnalysis, setLlmAnalysis,
    lastCatalogPath, setLastCatalogPath,
    lastCatalogItems, setLastCatalogItems,
    setCatalogParsed,
    setIaTrained,
    setTicketsParsed,
    setTicketsCount,
    setTicketsGuardados,
  } = useParserContext();

  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [showBatchProcessor, setShowBatchProcessor] = useState(false);
  const [isSyncing, setIsSyncing] = useState(false);
  const [syncResult, setSyncResult] = useState("");

  const resetParserUI = useCallback(() => {
    setParsedItems([]);
    setSelectedPath("");
    setFileContent("");
    setLlmAnalysis(null);
    setShowColumnMapper(false);
  }, [setParsedItems, setSelectedPath, setFileContent, setLlmAnalysis, setShowColumnMapper]);

  const handleFileSelect = useCallback(async () => {
    try {
      const selected = await open({
        multiple: false,
        directory: parserMode === 'insertar',
        filters: parserMode !== 'insertar' ? [{
          name: 'Archivos soportados',
          extensions: ['txt', 'csv', 'xlsx']
        }] : []
      });

      if (selected) {
        const path = Array.isArray(selected) ? selected[0] : selected;
        setSelectedPath(path);
        setLlmAnalysis(null);

        if (parserMode !== 'insertar') {
          const ext = path.split('.').pop()?.toLowerCase() || '';

          if (ext === 'xlsx' || ext === 'xls') {
            const bytes = await invoke("leer_archivo_bytes", { path }) as number[];
            const result = await invoke("parsear_excel", { archivo: bytes }) as any;
            if (result.productos && result.productos.length > 0) {
              setParsedItems(result.productos);
              setFileContent(`Excel cargado: ${result.total} productos encontrados`);
            } else {
              setFileContent('No se encontraron productos en el Excel');
            }
          } else {
            const raw = await invoke("leer_archivo_raw", { path });
            setFileContent(raw as string);

            if (parserMode === 'entrenar IA') {
              setShowColumnMapper(true);
            } else if (parserMode === 'catalogo') {
              const result = await invoke("parsear_catalogo_visual", { path }) as any;
              if (result.productos && result.productos.length > 0) {
                setParsedItems(result.productos);
                setLastCatalogPath(path);
                setLastCatalogItems(result.productos);
              } else {
                const items = await invoke("parsear_catalogo_csv", { path });
                setParsedItems(items as any[]);
                setLastCatalogPath(path);
                setLastCatalogItems(items as any[]);
              }
            } else {
              setParsedItems([]);
            }
          }
        } else {
          setShowBatchProcessor(true);
        }
      }
    } catch (error) {
      console.error("Error en la selección:", error);
      setIsAnalyzing(false);
    }
  }, [parserMode, setSelectedPath, setLlmAnalysis, setParsedItems, setFileContent, setShowColumnMapper, setLastCatalogPath, setLastCatalogItems]);

  const handleGuardarTicket = useCallback(async (items: any[], _analysis: any) => {
    if (!items || items.length === 0) return;

    try {
      if (parserMode === 'entrenar IA') {
        const total = items.reduce((acc: number, item: any) => acc + (item.total || 0), 0);
        await invoke("guardar_ticket_parseado", {
          items,
          total,
          fecha: _analysis?.fecha_ticket || null,
          hora: _analysis?.hora_ticket || null
        });
        setIaTrained(true);
        setTicketsParsed(true);
        setTicketsCount((c) => c + items.length);
        setTicketsGuardados((c) => c + 1);
        alert("¡Ticket guardado en el historial histórico!");
      }
      resetParserUI();
    } catch (error) {
      console.error("Error al guardar ticket:", error);
      alert("Fallo al guardar el ticket.");
    }
  }, [parserMode, setIaTrained, setTicketsParsed, setTicketsCount, setTicketsGuardados, resetParserUI]);

  const handleTrainIA = useCallback(async () => {
    if (!parsedItems || parsedItems.length === 0) return;

    try {
      if (parserMode === 'entrenar IA') {
        const total = parsedItems.reduce((acc, item) => acc + (item.total || 0), 0);
        await invoke("guardar_ticket_parseado", {
          items: parsedItems,
          total,
          fecha: llmAnalysis?.fecha_ticket || null,
          hora: llmAnalysis?.hora_ticket || null
        });
        setIaTrained(true);
        setTicketsParsed(true);
        setTicketsCount((c) => c + parsedItems.length);
        setTicketsGuardados((c) => c + 1);
        alert("¡IA Entrenada! Ticket guardado en el historial histórico.");
      } else if (parserMode === 'catalogo') {
        const items = parsedItems.map((item: any) => ({
          id: null,
          nombre: item.nombre || item.producto || "",
          descripcion: null,
          precio_costo: item.precio_costo || 0,
          precio_venta: item.precio_venta || 0, // <-- Corrección: redundancia eliminada
          stock: item.stock || 0,
          vendido: 0,
          stock_minimo: 5,
          codigo_barras: null,
          categoria: item.categoria || null,
        }));

        const result = await invoke("importar_catalogo", {
          items,
          rutaArchivo: selectedPath,
          contenidoArchivo: fileContent
        }) as string;

        setCatalogParsed(true);
        setLastCatalogPath(selectedPath);
        setLastCatalogItems(parsedItems);
        alert(result || "¡Catálogo importado con éxito al inventario!");
      } else {
        alert("Modo no soportado aún, patrón.");
      }
      resetParserUI();
    } catch (error) {
      console.error("Error al entrenar IA:", error);
      const errorMsg = error instanceof Error ? error.message : String(error);
      if (errorMsg.includes("ya fue importado") || errorMsg.includes("duplicados")) {
        alert(errorMsg);
      } else {
        alert("Fallo al importar los datos a la base de datos.");
      }
    }
  }, [parsedItems, parserMode, llmAnalysis, setIaTrained, setTicketsParsed, setTicketsCount, setTicketsGuardados, selectedPath, fileContent, setCatalogParsed, setLastCatalogPath, setLastCatalogItems, resetParserUI]);

  const handleSyncEmbeddings = useCallback(async () => {
    setIsSyncing(true);
    setSyncResult("");
    try {
      const result = await invoke("backfill_embeddings") as {
        status: string;
        total_productos: number;
        insertados: number;
        omitidos: number;
      };
      setSyncResult(
        `✓ Sincronización completa: ${result.insertados} embeddings generados, ${result.omitidos} ya existentes (${result.total_productos} productos).`
      );
    } catch (error) {
      console.error("Error sincronizando embeddings:", error);
      const errorMsg = error instanceof Error ? error.message : String(error);
      setSyncResult(errorMsg.includes("no está listo")
        ? "El motor de IA no está listo aún. Espera un momento e inténtalo de nuevo."
        : `Error: ${errorMsg}`);
    } finally {
      setIsSyncing(false);
    }
  }, []);

  const handleChangeMode = useCallback((m: "catalogo" | "entrenar IA" | "insertar") => {
    if (parserMode === 'catalogo' && parsedItems.length > 0) {
      setLastCatalogPath(selectedPath);
      setLastCatalogItems(parsedItems);
    }
    setParserMode(m);
    setSelectedPath("");
    setFileContent("");
    setParsedItems([]);
    setLlmAnalysis(null);
    setShowColumnMapper(false);
    
    if (m === 'catalogo' && lastCatalogItems.length > 0) {
      setSelectedPath(lastCatalogPath);
      setParsedItems(lastCatalogItems);
    }
  }, [parserMode, parsedItems, selectedPath, setLastCatalogPath, setLastCatalogItems, setParserMode, setSelectedPath, setFileContent, setParsedItems, setLlmAnalysis, setShowColumnMapper, lastCatalogItems, lastCatalogPath]);

  return {
    isAnalyzing,
    showBatchProcessor,
    setShowBatchProcessor,
    isSyncing,
    syncResult,
    handleFileSelect,
    handleGuardarTicket,
    handleTrainIA,
    handleSyncEmbeddings,
    handleChangeMode,
  };
}