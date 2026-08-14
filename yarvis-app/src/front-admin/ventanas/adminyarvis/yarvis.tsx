import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import ChatWidget, { type ModelKey, CLOUD_PROVIDERS, MODEL_OPTIONS, getActiveCloud } from "./ChatWidget";

const API_PROVIDERS = [
  { id: "google", name: "Google AI", placeholder: "AIza..." },
  { id: "opencode", name: "OpenCode", placeholder: "sk-..." },
];

function pickBestModel(ramGb: number): ModelKey {
  if (ramGb >= 4.0) return "1.7B";
  if (ramGb >= 1.0) return "0.8B";
  return "0.5B";
}

const AdminYarvis = () => {
  const [showApiModal, setShowApiModal] = useState(false);
  const [apiKeys, setApiKeys] = useState<Record<string, string>>(() => {
    try {
      return JSON.parse(localStorage.getItem("yarvis_api_keys") || "{}");
    } catch {
      return {};
    }
  });

  const [cloudModels, setCloudModels] = useState<{ id: string; name: string }[]>([]);
  const [cloudModel, setCloudModel] = useState("");
  const [cloudModelsLoading, setCloudModelsLoading] = useState(false);

  const activeCloud = getActiveCloud();

  const [selectedModel, setSelectedModel] = useState<ModelKey>("0.5B");
  const [showModelPicker, setShowModelPicker] = useState(false);
  const [loadingModel, setLoadingModel] = useState<string | null>(null);
  const [loadedModels, setLoadedModels] = useState<Record<string, boolean>>({
    "0.5B": false, "0.8B": false, "1.7B": false,
  });
  const [ramGb, setRamGb] = useState(0);
  const [clearTrigger, setClearTrigger] = useState(0);
  const [modelAutoSelected, setModelAutoSelected] = useState(false);
  const [ramWarning, setRamWarning] = useState("");

  const modelPickerRef = useRef<HTMLDivElement>(null);
  const retryTimeoutRef = useRef<number>(0);
  const ramWarningTimeoutRef = useRef<number>(0);
  const mountedRef = useRef(true);

  useEffect(() => {
    mountedRef.current = true;
    return () => {
      mountedRef.current = false;
      window.clearTimeout(retryTimeoutRef.current);
      window.clearTimeout(ramWarningTimeoutRef.current);
    };
  }, []);

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
    } catch {
      if (mountedRef.current) {
        retryTimeoutRef.current = window.setTimeout(() => {
          if (mountedRef.current) fetchModelStatus();
        }, 5000);
      }
    }
  }, [modelAutoSelected]);

  useEffect(() => {
    fetchModelStatus();
    const interval = window.setInterval(() => {
      if (mountedRef.current) fetchModelStatus();
    }, 5000);
    return () => window.clearInterval(interval);
  }, [fetchModelStatus]);

  const refreshModelStatus = async () => {
    try {
      const status = await invoke<{ models: Record<string, boolean>; ram_gb: number }>("get_model_status");
      setLoadedModels(status.models);
      setRamGb(status.ram_gb);
      const loaded = (["1.7B", "0.8B", "0.5B"] as ModelKey[]).find((m) => status.models[m]);
      setSelectedModel(loaded || "0.5B");
    } catch {
      setSelectedModel("0.5B");
    }
  };

  useEffect(() => {
    const handleClick = (e: MouseEvent) => {
      if (modelPickerRef.current && !modelPickerRef.current.contains(e.target as Node)) {
        setShowModelPicker(false);
      }
    };
    document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  }, []);

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
      window.clearTimeout(ramWarningTimeoutRef.current);
      ramWarningTimeoutRef.current = window.setTimeout(() => setRamWarning(""), 5000);
      return;
    }

    const currentLoaded = (["1.7B", "0.8B", "0.5B"] as ModelKey[]).find((m) => loadedModels[m]);
    if (currentLoaded) {
      setLoadingModel(model);
      setSelectedModel(model);
      try {
        await invoke("unload_chat_model", { model: currentLoaded });
        const result = await invoke<{
          status: string;
          models: Record<string, boolean>;
          ram_gb: number;
        }>("load_chat_model", { model });
        setLoadedModels(result.models);
        setRamGb(result.ram_gb);
      } catch {
        await refreshModelStatus();
      } finally {
        setLoadingModel(null);
      }
    } else {
      setLoadingModel(model);
      setSelectedModel(model);
      try {
        const result = await invoke<{
          status: string;
          models: Record<string, boolean>;
          ram_gb: number;
        }>("load_chat_model", { model });
        setLoadedModels(result.models);
        setRamGb(result.ram_gb);
      } catch {
        await refreshModelStatus();
      } finally {
        setLoadingModel(null);
      }
    }
  };

  const currentModel = MODEL_OPTIONS.find((m) => m.key === selectedModel) || MODEL_OPTIONS[0];

  const handleSaveApiKeys = () => {
    localStorage.setItem("yarvis_api_keys", JSON.stringify(apiKeys));
    setShowApiModal(false);
    const first = CLOUD_PROVIDERS.find((p) => (apiKeys[p.id] || "").trim());
    refreshCloudModels(first ? first.id : undefined, first ? apiKeys[first.id] : undefined);
  };

  const refreshCloudModels = async (provider?: string, apiKey?: string) => {
    const p = provider ?? activeCloud.provider;
    const k = apiKey ?? activeCloud.apiKey;
    if (!p) {
      setCloudModels([]);
      setCloudModel("");
      return;
    }
    setCloudModelsLoading(true);
    try {
      const res = await invoke<{ models: { id: string; name: string }[] }>("get_cloud_models", {
        provider: p,
        apiKey: k,
      });
      setCloudModels(res.models);
      let stored: { provider?: string; model?: string } | null = null;
      try {
        stored = JSON.parse(localStorage.getItem("yarvis_cloud_model") || "null");
      } catch { /* ignore */ }
      const selected =
        stored && stored.provider === p && stored.model
          ? stored.model
          : activeCloud.provider === p
            ? activeCloud.model
            : res.models[0]?.id ?? "";
      setCloudModel(selected);
    } catch {
      /* ignore */
    } finally {
      setCloudModelsLoading(false);
    }
  };

  const selectCloudModel = (model: string) => {
    setCloudModel(model);
    try {
      localStorage.setItem(
        "yarvis_cloud_model",
        JSON.stringify({ provider: activeCloud.provider, model })
      );
    } catch { /* ignore */ }
  };

  const toggleModelPicker = () => {
    setShowModelPicker((prev) => {
      if (!prev && activeCloud.provider && cloudModels.length === 0) refreshCloudModels();
      return !prev;
    });
  };

  return (
    <div className="h-full animate-in fade-in duration-500 flex flex-col bg-gradient-to-br from-neutral-50 via-white to-neutral-100">
      <div className="flex-shrink-0 px-8 pt-8 pb-4">
        <header className="flex justify-between items-center mb-6">
          <div>
            <h2 className="text-4xl font-black text-neutral-900 uppercase tracking-tight mb-1">Y.A.R.V.I.S.</h2>
            <p className="text-[11px] font-black text-neutral-400 uppercase tracking-[0.3em]">Asistente Inteligente de Negocio</p>
          </div>
          <div className="flex items-center gap-3">
            <button
              onClick={() => setClearTrigger((t) => t + 1)}
              className="flex items-center gap-2.5 px-5 py-3 bg-neutral-900 rounded-2xl shadow-sm hover:bg-neutral-800 transition-all"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="text-white">
                <path d="M3 6h18" /><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" /><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
              </svg>
              <span className="text-[11px] font-black text-white uppercase tracking-widest">Limpiar</span>
            </button>

            <button
              onClick={() => setShowApiModal(true)}
              className="flex items-center gap-2.5 px-5 py-3 bg-white/80 backdrop-blur-sm border border-neutral-200 rounded-2xl shadow-sm hover:bg-white hover:shadow-md transition-all"
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="text-neutral-500">
                <path d="M15 7h3a5 5 0 0 1 5 5 5 5 0 0 1-5 5h-3m-6 0H6a5 5 0 0 1-5-5 5 5 0 0 1 5-5h3" />
                <line x1="8" y1="12" x2="16" y2="12" />
              </svg>
              <span className="text-[11px] font-black text-neutral-600 uppercase tracking-widest">Agregar API</span>
            </button>

            <div ref={modelPickerRef} className="relative">
              <button
                onClick={toggleModelPicker}
                disabled={!!loadingModel}
                className="flex items-center gap-2.5 px-5 py-3 bg-white/80 backdrop-blur-sm border border-neutral-200 rounded-2xl shadow-sm hover:bg-white hover:shadow-md transition-all disabled:opacity-50"
              >
                <div className={`w-2.5 h-2.5 rounded-full ${loadingModel ? "bg-amber-500 animate-pulse" : activeCloud.provider ? "bg-blue-500" : selectedModel === "1.7B" ? "bg-emerald-500" : selectedModel === "0.8B" ? "bg-amber-500" : "bg-neutral-400"}`}></div>
                <span className="text-[11px] font-black text-neutral-600 uppercase tracking-widest">
                  {loadingModel ? `Cargando...` : activeCloud.provider ? activeCloud.display : `Qwen ${currentModel.label}`}
                </span>
                {!loadingModel && (
                  <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round" className="text-neutral-400"><path d="m6 9 6 6 6-6" /></svg>
                )}
              </button>

              {showModelPicker && (
                <div className="absolute right-0 top-full mt-2 w-72 bg-white border border-neutral-200 rounded-2xl shadow-2xl shadow-neutral-200/50 overflow-hidden z-50 animate-in fade-in slide-in-from-top-2 duration-150">
                  <div className="p-2">
                    <p className="px-4 py-2 text-[10px] font-black text-neutral-400 uppercase tracking-widest">
                      Seleccionar modelo
                    </p>
                    {activeCloud.provider && (
                      <div className="mb-1">
                        <div className="px-4 py-2 flex items-center justify-between">
                          <p className="text-[10px] font-black text-neutral-400 uppercase tracking-widest">
                            Modelo de {activeCloud.display}
                          </p>
                          <button
                            onClick={() => refreshCloudModels()}
                            title="Actualizar lista de modelos"
                            className="flex items-center gap-1 text-[9px] font-black text-neutral-500 uppercase tracking-widest hover:text-neutral-900 transition-colors"
                          >
                            <svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round" className={cloudModelsLoading ? "animate-spin" : ""}><path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8" /><path d="M21 3v5h-5" /></svg>
                            {cloudModelsLoading ? "..." : "Actualizar"}
                          </button>
                        </div>
                        <div className="px-2 pb-1 max-h-40 overflow-y-auto custom-scrollbar space-y-1">
                          {cloudModels.length === 0 && !cloudModelsLoading && (
                            <p className="px-3 py-2 text-[10px] font-bold text-neutral-400">
                              Sin modelos cargados. Pulsa Actualizar.
                            </p>
                          )}
                          {cloudModels.map((m) => (
                            <button
                              key={m.id}
                              onClick={() => selectCloudModel(m.id)}
                              className={`w-full flex items-center gap-2 px-3 py-2 rounded-lg text-left transition-all ${cloudModel === m.id ? "bg-neutral-900 text-white" : "hover:bg-neutral-50 text-neutral-700"}`}
                            >
                              <div className={`w-2 h-2 rounded-full flex-shrink-0 ${cloudModel === m.id ? "bg-white" : "bg-blue-500"}`}></div>
                              <div className="flex-1 min-w-0">
                                <p className="text-[11px] font-black truncate">{m.name}</p>
                                <p className={`text-[9px] font-bold truncate ${cloudModel === m.id ? "text-white/50" : "text-neutral-400"}`}>{m.id}</p>
                              </div>
                              {cloudModel === m.id && (
                                <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round" className="text-white flex-shrink-0"><polyline points="20 6 9 17 4 12" /></svg>
                              )}
                            </button>
                          ))}
                        </div>
                      </div>
                    )}
                    {MODEL_OPTIONS.map((opt) => {
                      const isLoaded = loadedModels[opt.key];
                      const isLoadingThis = loadingModel === opt.key;
                      const canFit = ramGb >= opt.minRam;

                      return (
                        <button
                          key={opt.key}
                          onClick={() => handleModelSelect(opt.key)}
                          disabled={isLoadingThis || (!isLoaded && !canFit)}
                          className={`w-full flex items-center gap-3 px-4 py-3 rounded-xl text-left transition-all ${selectedModel === opt.key
                            ? "bg-neutral-900 text-white"
                            : "hover:bg-neutral-50 text-neutral-700"
                            } ${!isLoaded && !canFit ? "opacity-40 cursor-not-allowed" : ""}`}
                        >
                          <div className={`w-2.5 h-2.5 rounded-full flex-shrink-0 ${isLoadingThis ? "bg-amber-500 animate-pulse"
                            : isLoaded ? "bg-emerald-500"
                              : selectedModel === opt.key ? "bg-white"
                                : opt.key === "1.7B" ? "bg-emerald-500"
                                  : opt.key === "0.8B" ? "bg-amber-500"
                                    : "bg-neutral-400"
                            }`}></div>
                          <div className="flex-1 min-w-0">
                            <div className="flex items-center gap-2">
                              <p className={`text-[12px] font-black ${selectedModel === opt.key ? "text-white" : "text-neutral-900"}`}>
                                Qwen {opt.label}
                              </p>
                              {isLoaded && (
                                <span className={`text-[9px] font-black px-2 py-0.5 rounded-full ${selectedModel === opt.key ? "bg-white/20 text-white" : "bg-emerald-50 text-emerald-600"}`}>
                                  LISTO
                                </span>
                              )}
                              {isLoadingThis && (
                                <span className="text-[9px] font-black px-2 py-0.5 rounded-full bg-amber-50 text-amber-600 animate-pulse">
                                  CARGANDO
                                </span>
                              )}
                              {!isLoaded && !canFit && (
                                <span className="text-[9px] font-black px-2 py-0.5 rounded-full bg-red-50 text-red-500">
                                  RAM INSUF.
                                </span>
                              )}
                            </div>
                            <p className={`text-[10px] font-bold mt-0.5 ${selectedModel === opt.key ? "text-white/50" : "text-neutral-400"}`}>
                              {opt.desc}
                            </p>
                          </div>
                          {selectedModel === opt.key && (
                            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round" className="text-white flex-shrink-0"><polyline points="20 6 9 17 4 12" /></svg>
                          )}
                        </button>
                      );
                    })}
                    <div className="px-4 py-2.5 border-t border-neutral-100 mt-1 space-y-0.5">
                      <p className="text-[10px] font-black text-neutral-500">
                        {activeCloud.provider
                          ? `Usando API: ${activeCloud.display}${cloudModel ? ` · ${cloudModel}` : ""}`
                          : `Modelo local: Qwen ${currentModel.label}`}
                      </p>
                      <p className="text-[10px] font-bold text-neutral-400">
                        RAM: {ramGb > 0 ? `${ramGb.toFixed(1)}GB` : "..."}
                      </p>
                    </div>
                  </div>
                </div>
              )}
            </div>

            <div className={`flex items-center gap-2.5 px-5 py-3 bg-white/80 backdrop-blur-sm border border-neutral-200 rounded-2xl shadow-sm ${loadingModel ? "opacity-100" : "opacity-90"}`}>
              <div className={`w-2.5 h-2.5 rounded-full ${loadingModel
                ? "bg-amber-500 animate-pulse"
                : activeCloud.provider
                  ? "bg-blue-500 animate-pulse shadow-lg shadow-blue-500/50"
                  : Object.values(loadedModels).some(Boolean)
                    ? "bg-emerald-500 animate-pulse shadow-lg shadow-emerald-500/50"
                    : "bg-neutral-300"}`}></div>
              <span className="text-[11px] font-black text-neutral-600 uppercase tracking-widest">
                {loadingModel ? "Cargando..." : activeCloud.provider ? "API en línea" : Object.values(loadedModels).some(Boolean) ? "Activado" : "Desactivado"}
              </span>
            </div>
          </div>
        </header>
      </div>

      <div className="flex-1 min-h-0 px-8 pb-8">
        <div className="h-full bg-white/70 backdrop-blur-md rounded-[3rem] border border-neutral-200/80 shadow-2xl shadow-neutral-300/30 overflow-hidden relative">
          <div className="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-transparent via-neutral-900/10 to-transparent"></div>
          {loadingModel && (
            <div className="absolute top-1 left-0 right-0 z-10 px-8 py-3 bg-amber-50 border-b border-amber-200">
              <div className="flex items-center gap-3">
                <div className="flex-1">
                  <div className="flex items-center justify-between mb-1.5">
                    <span className="text-[11px] font-black text-amber-700 uppercase tracking-widest">
                      Cargando Qwen {loadingModel}
                    </span>
                    <span className="text-[10px] font-bold text-amber-500">Esto puede tardar 10-30 seg...</span>
                  </div>
                  <div className="h-2 bg-amber-200 rounded-full overflow-hidden">
                    <div className="h-full bg-amber-500 rounded-full animate-loading-bar"></div>
                  </div>
                </div>
              </div>
            </div>
          )}
          {ramWarning && (
            <div className="absolute top-1 left-0 right-0 z-10 px-8 py-3 bg-red-50 border-b border-red-200">
              <div className="flex items-center gap-3">
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="text-red-500 flex-shrink-0"><circle cx="12" cy="12" r="10" /><line x1="12" y1="8" x2="12" y2="12" /><line x1="12" y1="16" x2="12.01" y2="16" /></svg>
                <span className="text-[11px] font-black text-red-600 uppercase tracking-widest">{ramWarning}</span>
              </div>
            </div>
          )}
          <ChatWidget
            role="admin"
            userId="admin"
            suggestions={[
              "¿Hubo algo raro hoy?",
              "¿Cuánto gané libre hoy quitando el costo de los productos?",
              "¿Qué debería comprar para el fin de semana?",
              "¿Qué productos están por agotarse?",
              "Resumen de ventas de hoy",
              "¿Qué empleados tienen más reembolsos?",
            ]}
            modelState={{ selectedModel, loadingModel, loadedModels, ramGb, showPicker: showModelPicker }}
            onModelSelect={handleModelSelect}
            onTogglePicker={() => setShowModelPicker(!showModelPicker)}
            clearTrigger={clearTrigger}
          />
        </div>
      </div>

      {/* API KEY MODAL */}
      {showApiModal && (
        <div className="fixed inset-0 bg-black/40 backdrop-blur-sm flex items-center justify-center z-50 animate-in fade-in duration-200">
          <div className="bg-white rounded-3xl shadow-2xl w-full max-w-lg mx-4 overflow-hidden animate-in zoom-in-95 duration-200">
            <div className="px-8 py-6 border-b border-neutral-100 flex items-center justify-between">
              <div>
                <h3 className="text-lg font-black text-neutral-900 uppercase tracking-tight">Configurar API</h3>
                <p className="text-[11px] font-bold text-neutral-400 uppercase tracking-widest mt-1">Agrega tu clave de proveedor IA</p>
              </div>
              <button onClick={() => setShowApiModal(false)} className="w-10 h-10 bg-neutral-100 hover:bg-neutral-200 rounded-xl flex items-center justify-center transition-all">
                <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="text-neutral-500"><line x1="18" y1="6" x2="6" y2="18" /><line x1="6" y1="6" x2="18" y2="18" /></svg>
              </button>
            </div>
            <div className="px-8 py-6 space-y-4 max-h-[60vh] overflow-y-auto custom-scrollbar">
              {API_PROVIDERS.map((provider) => (
                <div key={provider.id} className="space-y-2">
                  <label className="text-[11px] font-black text-neutral-500 uppercase tracking-widest flex items-center gap-2">
                    <div className="w-1.5 h-1.5 rounded-full bg-neutral-900"></div>
                    {provider.name}
                  </label>
                  <input
                    type="password"
                    value={apiKeys[provider.id] || ""}
                    onChange={(e) => setApiKeys({ ...apiKeys, [provider.id]: e.target.value })}
                    placeholder={provider.placeholder}
                    className="w-full bg-neutral-50 border border-neutral-200 px-5 py-3.5 rounded-2xl text-[13px] font-medium text-neutral-900 placeholder:text-neutral-400 focus:outline-none focus:border-neutral-900 focus:ring-4 focus:ring-neutral-900/5 transition-all"
                  />
                </div>
              ))}
            </div>
            <div className="px-8 py-5 border-t border-neutral-100 bg-neutral-50/50 flex gap-3">
              <button
                onClick={() => setShowApiModal(false)}
                className="flex-1 py-3.5 bg-neutral-100 hover:bg-neutral-200 text-neutral-600 rounded-2xl text-[12px] font-black uppercase tracking-widest transition-all"
              >
                Cancelar
              </button>
              <button
                onClick={handleSaveApiKeys}
                className="flex-1 py-3.5 bg-neutral-900 hover:bg-neutral-800 text-white rounded-2xl text-[12px] font-black uppercase tracking-widest transition-all shadow-lg"
              >
                Guardar
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default AdminYarvis;
