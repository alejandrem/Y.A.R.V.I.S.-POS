import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import ChatWidget, { type ModelKey } from "../../../front-admin/ventanas/adminyarvis/ChatWidget";

const yarvisNav = {
  id: "yarvis",
  label: "Y.A.R.V.I.S.",
  icon: (
    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M12 8V4H8" />
      <rect width="16" height="12" x="4" y="8" rx="2" />
      <path d="M2 14h2" />
      <path d="M20 14h2" />
      <path d="M15 13v2" />
      <path d="M9 13v2" />
    </svg>
  ),
};

function pickBestModel(ramGb: number): ModelKey {
  if (ramGb >= 4.0) return "1.7B";
  if (ramGb >= 1.0) return "0.8B";
  return "0.5B";
}

export default function Yarvis() {
  const [selectedModel, setSelectedModel] = useState<ModelKey>("0.5B");
  const [loadingModel, setLoadingModel] = useState<string | null>(null);
  const [loadedModels, setLoadedModels] = useState<Record<string, boolean>>({
    "0.5B": false, "0.8B": false, "1.7B": false,
  });
  const [ramGb, setRamGb] = useState(0);
  const [showModelPicker, setShowModelPicker] = useState(false);
  const [clearTrigger, setClearTrigger] = useState(0);
  const [modelAutoSelected, setModelAutoSelected] = useState(false);
  const [ramWarning, setRamWarning] = useState("");

  const fetchModelStatus = useCallback(async () => {
    try {
      const status = await invoke<{ models: Record<string, boolean>; ram_gb: number }>("get_model_status");
      setLoadedModels(status.models);
      setRamGb(status.ram_gb);
      if (!modelAutoSelected && status.ram_gb > 0) {
        const best = pickBestModel(status.ram_gb);
        setSelectedModel(best);
        setModelAutoSelected(true);
      }
    } catch { /* ignore */ }
  }, [modelAutoSelected]);

  useEffect(() => {
    fetchModelStatus();
    const interval = window.setInterval(() => {
      fetchModelStatus();
    }, 5000);
    return () => window.clearInterval(interval);
  }, [fetchModelStatus]);

  const handleModelSelect = async (model: ModelKey) => {
    setShowModelPicker(false);
    setRamWarning("");

    if (loadedModels[model]) {
      setSelectedModel(model);
      return;
    }

    const MODEL_RAM: Record<ModelKey, number> = { "0.5B": 0, "0.8B": 1.0, "1.7B": 4.0 };
    const needed = MODEL_RAM[model];
    if (ramGb > 0 && ramGb < needed) {
      setRamWarning(`RAM insuficiente para Qwen ${model}: tienes ${ramGb.toFixed(1)}GB, necesitas ≥${needed}GB`);
      setTimeout(() => setRamWarning(""), 5000);
      return;
    }

    const currentLoaded = (["1.7B", "0.8B", "0.5B"] as ModelKey[]).find((m) => loadedModels[m]);
    setLoadingModel(model);
    setSelectedModel(model);
    try {
      if (currentLoaded) {
        await invoke("unload_chat_model", { model: currentLoaded });
      }
      const result = await invoke<{ status: string; models: Record<string, boolean>; ram_gb: number }>("load_chat_model", { model });
      setLoadedModels(result.models);
      setRamGb(result.ram_gb);
    } catch {
      setSelectedModel("0.5B");
    } finally {
      setLoadingModel(null);
    }
  };

  return (
    <div className="h-full animate-in fade-in duration-500 flex flex-col">
      <div className="flex-shrink-0 px-6 pt-4 pb-2 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="w-9 h-9 bg-neutral-900 rounded-xl flex items-center justify-center text-white font-black text-sm">Y</div>
          <div>
            <h2 className="text-sm font-black text-neutral-900 uppercase tracking-tight leading-none">Y.A.R.V.I.S.</h2>
            <p className="text-[10px] font-bold text-neutral-400 uppercase tracking-widest mt-0.5">Asistente</p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => setClearTrigger((t) => t + 1)}
            className="px-3 py-1.5 bg-neutral-900 text-white rounded-lg text-[10px] font-black uppercase tracking-widest hover:bg-neutral-800 transition-all"
          >
            Limpiar
          </button>
          <button
            onClick={() => setShowModelPicker(!showModelPicker)}
            disabled={!!loadingModel}
            className="flex items-center gap-2 px-3 py-1.5 bg-neutral-100 rounded-lg transition-all disabled:opacity-50"
          >
            <div className={`w-2 h-2 rounded-full ${loadingModel ? "bg-amber-500 animate-pulse" : selectedModel === "1.7B" ? "bg-emerald-500" : selectedModel === "0.8B" ? "bg-amber-500" : "bg-neutral-400"}`}></div>
            <span className="text-[10px] font-black text-neutral-600 uppercase tracking-widest">
              {loadingModel ? `Cargando...` : `Qwen ${selectedModel}`}
            </span>
          </button>
          {showModelPicker && (
            <div className="absolute right-4 top-14 w-60 bg-white border border-neutral-200 rounded-xl shadow-2xl z-50 p-1.5">
              {(["1.7B", "0.8B", "0.5B"] as ModelKey[]).map((m) => (
                <button
                  key={m}
                  onClick={() => handleModelSelect(m)}
                  className={`w-full flex items-center gap-2 px-3 py-2 rounded-lg text-left text-[11px] font-black transition-all ${selectedModel === m ? "bg-neutral-900 text-white" : "hover:bg-neutral-50 text-neutral-700"}`}
                >
                  <div className={`w-2 h-2 rounded-full ${selectedModel === m ? "bg-white" : m === "1.7B" ? "bg-emerald-500" : m === "0.8B" ? "bg-amber-500" : "bg-neutral-400"}`}></div>
                  Qwen {m}
                </button>
              ))}
            </div>
          )}
        </div>
      </div>

      {loadingModel && (
        <div className="px-6 py-2 bg-amber-50 border-b border-amber-200">
          <div className="flex items-center gap-3">
            <div className="flex-1">
              <div className="flex items-center justify-between mb-1">
                <span className="text-[10px] font-black text-amber-700 uppercase tracking-widest">
                  Cargando Qwen {loadingModel}
                </span>
                <span className="text-[9px] font-bold text-amber-500">10-30 seg...</span>
              </div>
              <div className="h-1.5 bg-amber-200 rounded-full overflow-hidden">
                <div className="h-full bg-amber-500 rounded-full animate-loading-bar"></div>
              </div>
            </div>
          </div>
        </div>
      )}
      {ramWarning && (
        <div className="px-6 py-2 bg-red-50 border-b border-red-200">
          <div className="flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="text-red-500 flex-shrink-0"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
            <span className="text-[10px] font-black text-red-600 uppercase tracking-widest">{ramWarning}</span>
          </div>
        </div>
      )}

      <div className="flex-1 min-h-0">
        <ChatWidget
          role="empleado"
          userId="empleado"
          suggestions={[
            "¿Qué productos tengo de limpieza?",
            "¿Cuánto stock hay de Coca-Cola?",
            "¿Qué es lo más vendido esta semana?",
            "¿Qué productos no tienen sal?",
          ]}
          modelState={{ selectedModel, loadingModel, loadedModels, ramGb, showPicker: showModelPicker }}
          onModelSelect={handleModelSelect}
          onTogglePicker={() => setShowModelPicker(!showModelPicker)}
          clearTrigger={clearTrigger}
        />
      </div>
    </div>
  );
}

export { yarvisNav };
