// Panel de administración de Y.A.R.V.I.S.
// Selector de modelo (local GGUF o cloud OpenCode/Gemini), botón "Limpiar chat" y
// "Configurar modelos". Orquesta el ChatWidget con su configuración vigente.
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { MorphIcon, type IconInput } from "morphicons/react";
import { notificarError } from "../../../components/notificaciones";
import ChatWidget, {
  type ChatModelSelection,
  type CloudModel,
  CLOUD_PROVIDERS,
} from "./ChatWidget";
import { setApiKeysCache } from "./ChatWidget";
import {
  ICONO_CHECK,
  ICONO_ENGRANAJE,
  ICONO_PAUSA,
  ICONO_REINICIAR,
  ICONO_ROBOT,
} from "../../../icons";

type ProviderId = "google" | "opencode";

const API_PROVIDERS: { id: ProviderId; name: string; description: string; placeholder: string }[] = [
  { id: "opencode", name: "OpenCode", description: "Modelos gratuitos compatibles con OpenAI", placeholder: "sk-…" },
  { id: "google", name: "Gemini", description: "Modelos de Google AI Studio", placeholder: "AIza…" },
];

interface ModelStatus {
  models: Record<string, boolean>;
  ram_libre_gb?: number;
  local_model_path?: string;
  local_model_name?: string;
  local_context_window?: number;
}

interface AdminYarvisProps {
  active?: boolean;
}

const AdminYarvis = ({ active = true }: AdminYarvisProps) => {
  const [showConfig, setShowConfig] = useState(false);
  const [configSection, setConfigSection] = useState<"opencode" | "google" | "local">("opencode");
  const [showModelMenu, setShowModelMenu] = useState(false);
  // Las API keys viven en disco vía backend (api_keys.json con permisos
  // 0600) — NUNCA en localStorage, que queda en texto plano y es legible
  // por cualquier XSS del webview.
  const [apiKeys, setApiKeys] = useState<Record<string, string>>({});
  const [localModelPath, setLocalModelPath] = useState(() => localStorage.getItem("yarvis_local_model_path") || "");
  const [localModelName, setLocalModelName] = useState("Modelo local");
  const [selectedProvider, setSelectedProvider] = useState<"" | ProviderId>(() => {
    const stored = localStorage.getItem("yarvis_active_provider");
    return stored === "google" || stored === "opencode" ? stored : "";
  });
  const [selectedCloudModels, setSelectedCloudModels] = useState<Record<string, string>>(() => {
    try { return JSON.parse(localStorage.getItem("yarvis_cloud_models_selected") || "{}"); } catch { return {}; }
  });
  const [cloudModels, setCloudModels] = useState<Record<string, CloudModel[]>>({});

  useEffect(() => {
    invoke<Record<string, string>>("leer_api_keys")
      .then((keys) => { if (Object.keys(keys).length > 0) { setApiKeys(keys); setApiKeysCache(keys); } })
      .catch((e) => { console.error("[YARVIS] no se pudieron leer las API keys:", e); notificarError("No se pudieron leer las API keys guardadas", e); });
  }, []);
  const [cloudModelsLoading, setCloudModelsLoading] = useState<Record<string, boolean>>({});
  const [loadedModels, setLoadedModels] = useState<Record<string, boolean>>({ "1.7B": false });
  const [ramGb, setRamGb] = useState(0);
  const [loadingModel, setLoadingModel] = useState<string | null>(null);
  const [ramWarning, setRamWarning] = useState("");
  const [configMessage, setConfigMessage] = useState("");
  const [clearTrigger, setClearTrigger] = useState(0);
  const [clearIcon, setClearIcon] = useState<IconInput>(ICONO_REINICIAR);
  const [configIcon, setConfigIcon] = useState<IconInput>(ICONO_ENGRANAJE);
  const modelMenuRef = useRef<HTMLDivElement>(null);
  const clearIconTimeoutsRef = useRef<number[]>([]);

  useEffect(() => {
    return () => {
      clearIconTimeoutsRef.current.forEach(window.clearTimeout);
    };
  }, []);

  const handleClearChat = () => {
    setClearTrigger((value) => value + 1);
    clearIconTimeoutsRef.current.forEach(window.clearTimeout);
    clearIconTimeoutsRef.current = [
      window.setTimeout(() => setClearIcon(ICONO_PAUSA), 0),
      window.setTimeout(() => setClearIcon(ICONO_CHECK), 180),
      window.setTimeout(() => setClearIcon(ICONO_REINICIAR), 600),
    ];
  };

  const handleConfigHoverEnter = () => {
    setConfigIcon(ICONO_ROBOT);
  };

  const handleConfigHoverLeave = () => {
    setConfigIcon(ICONO_ENGRANAJE);
  };

  const refreshStatus = useCallback(async () => {
    try {
      const status = await invoke<ModelStatus>("get_model_status");
      setLoadedModels(status.models || {});
      setRamGb(status.ram_libre_gb || 0);
      if (status.local_model_name && status.local_model_name !== "modelo_no_encontrado.gguf") setLocalModelName(status.local_model_name);
      if (!localModelPath && status.local_model_path && !status.local_model_path.includes("modelo_no_encontrado")) setLocalModelPath(status.local_model_path);
    } catch { /* la pantalla puede abrirse antes de autenticar el backend */ }
  }, [localModelPath]);

  useEffect(() => {
    if (!active) return;
    refreshStatus();
    const timer = window.setInterval(refreshStatus, 5000);
    return () => window.clearInterval(timer);
  }, [refreshStatus, active]);

  useEffect(() => {
    const storedPath = localStorage.getItem("yarvis_local_model_path");
    if (storedPath) {
      invoke("set_local_model_path", { path: storedPath }).catch((e) => { console.error("[YARVIS] no se pudo restaurar la ruta del modelo local:", e); notificarError("No se pudo restaurar el modelo local configurado", e); });
    }
  }, []);

  useEffect(() => {
    const close = (event: MouseEvent) => {
      if (modelMenuRef.current && !modelMenuRef.current.contains(event.target as Node)) setShowModelMenu(false);
    };
    document.addEventListener("mousedown", close);
    return () => document.removeEventListener("mousedown", close);
  }, []);

  const refreshCloudModels = useCallback(async (provider: ProviderId) => {
    const apiKey = (apiKeys[provider] || "").trim();
    if (!apiKey) return;
    setCloudModelsLoading((previous) => ({ ...previous, [provider]: true }));
    try {
      const result = await invoke<{ models: CloudModel[] }>("get_cloud_models", { provider, apiKey });
      const models = result.models || [];
      setCloudModels((previous) => ({ ...previous, [provider]: models }));
      setSelectedCloudModels((previous) => {
        const selected = previous[provider] && models.some((model) => model.id === previous[provider])
          ? previous[provider]
          : models[0]?.id || "";
        const next = { ...previous, [provider]: selected };
        localStorage.setItem("yarvis_cloud_models_selected", JSON.stringify(next));
        return next;
      });
    } catch (error) {
      setConfigMessage(String(error));
    } finally {
      setCloudModelsLoading((previous) => ({ ...previous, [provider]: false }));
    }
  }, [apiKeys]);

  useEffect(() => {
    (Object.keys(apiKeys) as ProviderId[]).filter((provider) => apiKeys[provider]).forEach((provider) => {
      refreshCloudModels(provider);
    });
  // Solo se refresca al montar; el botón de actualizar cubre cambios explícitos.
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const currentCloudModel = selectedProvider
    ? cloudModels[selectedProvider]?.find((model) => model.id === selectedCloudModels[selectedProvider])
    : undefined;

  const currentSelection: ChatModelSelection = useMemo(() => {
    if (selectedProvider) {
      const providerName = selectedProvider === "google" ? "Gemini" : "OpenCode";
      const model = selectedCloudModels[selectedProvider] || CLOUD_PROVIDERS.find((provider) => provider.id === selectedProvider)?.defaultModel || "";
      return {
        provider: selectedProvider,
        apiKey: (apiKeys[selectedProvider] || "").trim(),
        model,
        label: `${providerName} · ${model || "sin modelo"}`,
        contextWindow: currentCloudModel?.context_window || 131072,
      };
    }
    return {
      provider: "",
      apiKey: "",
      model: "1.7B",
      label: localModelName || "Modelo local",
      contextWindow: 4096,
    };
  }, [selectedProvider, selectedCloudModels, apiKeys, currentCloudModel, localModelName]);

  const saveLocalPath = async (path: string) => {
    setConfigMessage("");
    try {
      const result = await invoke<{ name: string; path: string }>("set_local_model_path", { path });
      setLocalModelPath(result.path);
      setLocalModelName(result.name);
      localStorage.setItem("yarvis_local_model_path", result.path);
      setConfigMessage("Modelo local configurado. Cárgalo desde el selector superior.");
    } catch (error) {
      setConfigMessage(String(error));
    }
  };

  const chooseLocalModel = async () => {
    const selected = await open({ multiple: false, filters: [{ name: "Modelo GGUF", extensions: ["gguf"] }] });
    if (typeof selected === "string") {
      setLocalModelPath(selected);
      await saveLocalPath(selected);
    }
  };

  const loadLocalModel = async () => {
    if (!localModelPath) {
      setShowConfig(true);
      setConfigSection("local");
      return;
    }
    setLoadingModel("local");
    setRamWarning("");
    try {
      await invoke("set_local_model_path", { path: localModelPath });
      const result = await invoke<ModelStatus>("load_chat_model", { model: "1.7B" });
      setLoadedModels(result.models || { "1.7B": true });
      setRamGb(result.ram_libre_gb || 0);
      setSelectedProvider("");
      localStorage.removeItem("yarvis_active_provider");
    } catch (error) {
      setRamWarning(String(error));
    } finally {
      setLoadingModel(null);
    }
  };

  const selectProvider = (provider: ProviderId) => {
    if (!apiKeys[provider]) {
      setShowConfig(true);
      setConfigSection(provider);
      return;
    }
    setSelectedProvider(provider);
    localStorage.setItem("yarvis_active_provider", provider);
    setShowModelMenu(false);
    if (!cloudModels[provider]?.length) refreshCloudModels(provider);
  };

  const selectCloudModel = (provider: ProviderId, model: CloudModel) => {
    const next = { ...selectedCloudModels, [provider]: model.id };
    setSelectedCloudModels(next);
    localStorage.setItem("yarvis_cloud_models_selected", JSON.stringify(next));
    localStorage.setItem(`yarvis_cloud_model_${provider}`, model.id);
    setSelectedProvider(provider);
    localStorage.setItem("yarvis_active_provider", provider);
    setShowModelMenu(false);
  };

  const saveApiConfig = async () => {
    try {
      await invoke("guardar_api_keys", { keys: apiKeys });
      setApiKeysCache(apiKeys);
    } catch (e) {
      console.error("[YARVIS] no se pudieron guardar las API keys:", e);
      setConfigMessage(`Error guardando claves: ${e}`);
      return;
    }
    if (selectedProvider && !apiKeys[selectedProvider]) {
      setSelectedProvider("");
      localStorage.removeItem("yarvis_active_provider");
    }
    if (localModelPath) await saveLocalPath(localModelPath);
    (Object.keys(apiKeys) as ProviderId[]).filter((provider) => apiKeys[provider]).forEach((provider) => refreshCloudModels(provider));
    setShowConfig(false);
    setConfigMessage("Configuración guardada.");
  };

  const isLocalLoaded = Object.values(loadedModels).some(Boolean);

  return (
    <div className="yarvis-shell flex h-full min-h-0 flex-col animate-in fade-in duration-500">
      <header className="flex flex-shrink-0 flex-wrap items-center justify-between gap-4 px-6 pb-4 pt-6 sm:px-8 sm:pt-8">
        <div>
          <h2 className="yarvis-text mb-2 text-4xl font-black uppercase tracking-tight">Y.A.R.V.I.S.</h2>
          <div className="h-1.5 w-12 rounded-full bg-neutral-900 dark:bg-neutral-200" />
          <p className="yarvis-muted mt-2 text-[11px] font-black uppercase tracking-[0.3em]">Asistente Inteligente de Negocio</p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <button onClick={handleClearChat} className="yarvis-primary flex items-center gap-2 rounded-xl px-4 py-3 text-[10px] font-black uppercase tracking-widest transition-all"><MorphIcon icon={clearIcon} size={16} strokeWidth={2.5} spring="smooth" /> Limpiar chat</button>
          <button onMouseEnter={handleConfigHoverEnter} onMouseLeave={handleConfigHoverLeave} onClick={() => { setShowConfig(true); setConfigMessage(""); }} className="yarvis-panel yarvis-border yarvis-text flex items-center gap-2 rounded-xl border px-4 py-3 text-[10px] font-black uppercase tracking-widest transition-all"><MorphIcon icon={configIcon} size={16} strokeWidth={2} spring="smooth" /> Configurar modelos</button>
          <div ref={modelMenuRef} className="relative">
            <button onClick={() => setShowModelMenu((value) => !value)} className="yarvis-panel yarvis-border yarvis-text flex items-center gap-2 rounded-xl border px-4 py-3 text-left text-[10px] font-black uppercase tracking-widest">
              <span className={`h-2.5 w-2.5 rounded-full ${loadingModel ? "animate-pulse bg-amber-500" : selectedProvider ? "bg-sky-500" : isLocalLoaded ? "bg-emerald-500" : "bg-zinc-500"}`} />
              <span className="max-w-[180px] truncate">{loadingModel ? "Cargando modelo…" : currentSelection.label}</span><span className="text-xs opacity-50">⌄</span>
            </button>
            {showModelMenu && <div className="yarvis-panel yarvis-border yarvis-shadow absolute right-0 top-full z-50 mt-2 w-[min(360px,calc(100vw-2rem))] overflow-hidden rounded-2xl border p-2">
              <p className="yarvis-faint px-3 py-2 text-[9px] font-black uppercase tracking-[0.2em]">Modelo para este chat</p>
              <button onClick={loadLocalModel} disabled={loadingModel === "local"} className={`yarvis-panel-soft flex w-full items-center gap-3 rounded-xl p-3 text-left ${!selectedProvider ? "ring-1 ring-emerald-500" : ""}`}>
                <span className="h-2.5 w-2.5 rounded-full bg-emerald-500" /><span className="min-w-0 flex-1"><span className="yarvis-text block truncate text-xs font-black">{localModelName}</span><span className="yarvis-faint block truncate text-[10px] font-bold">{localModelPath || "Configura una ruta GGUF"}</span><span className="yarvis-faint block text-[9px] font-bold">RAM libre: {ramGb > 0 ? `${ramGb.toFixed(1)} GB` : "…"}</span></span><span className="text-[9px] font-black uppercase text-emerald-500">{isLocalLoaded ? "Listo" : "Cargar"}</span>
              </button>
              {API_PROVIDERS.map((provider) => <div key={provider.id} className="mt-1">
                <button onClick={() => selectProvider(provider.id)} disabled={!apiKeys[provider.id]} className={`yarvis-hover-panel flex w-full items-center gap-3 rounded-xl p-3 text-left disabled:cursor-not-allowed disabled:opacity-40 ${selectedProvider === provider.id ? "yarvis-panel-soft ring-1 ring-sky-500" : ""}`}>
                  <span className="h-2.5 w-2.5 rounded-full bg-sky-500" /><span className="min-w-0 flex-1"><span className="yarvis-text block text-xs font-black">{provider.name}</span><span className="yarvis-faint block text-[10px] font-bold">{apiKeys[provider.id] ? `${cloudModels[provider.id]?.length || 0} modelos detectados` : "Agrega una API para activar"}</span></span><span className="text-xs opacity-50">›</span>
                </button>
                {selectedProvider === provider.id && <div className="max-h-44 overflow-y-auto px-2 pb-2">
                  <button onClick={() => refreshCloudModels(provider.id)} className="yarvis-muted mb-1 flex w-full justify-end text-[9px] font-black uppercase tracking-widest">{cloudModelsLoading[provider.id] ? "Actualizando…" : "↻ Actualizar lista"}</button>
                  {(cloudModels[provider.id] || []).map((model) => <button key={model.id} onClick={() => selectCloudModel(provider.id, model)} className={`yarvis-hover-panel flex w-full items-center gap-2 rounded-lg px-3 py-2 text-left ${selectedCloudModels[provider.id] === model.id ? "yarvis-primary" : ""}`}><span className="min-w-0 flex-1"><span className="block truncate text-[10px] font-black">{model.name}</span><span className="block truncate text-[9px] opacity-60">{model.id}</span></span>{selectedCloudModels[provider.id] === model.id && <span>✓</span>}</button>)}
                </div>}
              </div>)}
              <div className="yarvis-border mt-2 border-t px-3 pt-3"><p className="yarvis-faint text-[9px] font-bold">Contexto: {Math.round(currentSelection.contextWindow / 1000)}k posiciones aprox. · se muestra como porcentaje en el chat.</p></div>
            </div>}
          </div>
          <span className="yarvis-panel-soft yarvis-border yarvis-muted flex items-center gap-2 rounded-xl border px-4 py-3 text-[10px] font-black uppercase tracking-widest"><span className={`h-2 w-2 rounded-full ${selectedProvider ? "bg-sky-500" : isLocalLoaded ? "bg-emerald-500" : "bg-zinc-500"}`} />{selectedProvider ? "API en línea" : isLocalLoaded ? "Local listo" : "Sin modelo"}</span>
        </div>
      </header>

      {loadingModel && <div className="mx-6 mb-3 flex-shrink-0 rounded-xl border border-amber-500/30 bg-amber-500/10 px-5 py-3 sm:mx-8"><div className="flex items-center justify-between"><span className="text-[10px] font-black uppercase tracking-widest text-amber-500">Cargando modelo local…</span><span className="text-[10px] font-bold text-amber-500">Puede tardar unos segundos</span></div><div className="mt-2 h-1.5 overflow-hidden rounded-full bg-amber-500/20"><div className="h-full animate-loading-bar rounded-full bg-amber-500" /></div></div>}
      {ramWarning && <div className="mx-6 mb-3 flex-shrink-0 rounded-xl border border-red-500/30 bg-red-500/10 px-5 py-3 text-xs font-bold text-red-500 sm:mx-8">{ramWarning}</div>}

      <div className="min-h-0 flex-1 px-4 pb-4 sm:px-8 sm:pb-8">
        <div className="yarvis-panel yarvis-border yarvis-shadow h-full min-h-0 overflow-hidden rounded-[2rem] border">
          <ChatWidget role="admin" userId="admin" suggestions={["¿Hubo algo raro hoy?", "¿Cuánto gané libre hoy quitando el costo de los productos?", "¿Qué debería comprar para el fin de semana?", "¿Qué productos están por agotarse?", "Resumen de ventas de hoy", "¿Qué empleados tienen más reembolsos?"]} modelState={{ loadingModel }} modelSelection={currentSelection} clearTrigger={clearTrigger} />
        </div>
      </div>

      {showConfig && <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4 backdrop-blur-sm">
        <div className="yarvis-shell yarvis-panel yarvis-border yarvis-shadow max-h-[90vh] w-full max-w-2xl overflow-hidden rounded-3xl border">
          <div className="yarvis-border flex items-center justify-between border-b px-6 py-5 sm:px-8"><div><h3 className="yarvis-text text-lg font-black uppercase tracking-tight">Fuentes de inteligencia</h3><p className="yarvis-muted mt-1 text-[10px] font-bold uppercase tracking-widest">Configura API cloud o tu modelo local</p></div><button onClick={() => setShowConfig(false)} className="yarvis-panel-soft yarvis-muted flex h-9 w-9 items-center justify-center rounded-xl text-xl">×</button></div>
          <div className="flex gap-2 overflow-x-auto px-6 pt-5 sm:px-8">{[{ id: "opencode", label: "OpenCode" }, { id: "google", label: "Gemini" }, { id: "local", label: "Modelo local" }].map((item) => <button key={item.id} onClick={() => setConfigSection(item.id as typeof configSection)} className={`flex-shrink-0 rounded-xl px-4 py-2.5 text-[10px] font-black uppercase tracking-widest ${configSection === item.id ? "yarvis-primary" : "yarvis-panel-soft yarvis-muted"}`}>{item.label}</button>)}</div>
          <div className="custom-scrollbar max-h-[55vh] overflow-y-auto px-6 py-6 sm:px-8">
            {configSection !== "local" ? <div className="space-y-5"><div className="yarvis-panel-soft yarvis-border rounded-2xl border p-5"><p className="yarvis-text text-sm font-black">{configSection === "opencode" ? "OpenCode" : "Gemini"}</p><p className="yarvis-muted mt-1 text-xs leading-relaxed">{API_PROVIDERS.find((provider) => provider.id === configSection)?.description}</p><label className="yarvis-muted mt-5 block text-[10px] font-black uppercase tracking-widest">API key</label><input type="password" value={apiKeys[configSection] || ""} onChange={(event) => setApiKeys({ ...apiKeys, [configSection]: event.target.value })} placeholder={API_PROVIDERS.find((provider) => provider.id === configSection)?.placeholder} className="yarvis-input mt-2 w-full rounded-xl border px-4 py-3 text-sm outline-none focus:border-sky-500" /><div className="mt-5 flex items-center justify-between"><span className="yarvis-faint text-[10px] font-bold">{cloudModels[configSection]?.length || 0} modelos disponibles</span><button onClick={() => refreshCloudModels(configSection)} className="yarvis-muted text-[10px] font-black uppercase tracking-widest">{cloudModelsLoading[configSection] ? "Actualizando…" : "Actualizar modelos"}</button></div></div></div> : <div className="space-y-5"><div className="yarvis-panel-soft yarvis-border rounded-2xl border p-5"><p className="yarvis-text text-sm font-black">Cualquier modelo GGUF</p><p className="yarvis-muted mt-1 text-xs leading-relaxed">Selecciona Qwen 0.5B, 1.5B, 1.7B, 1.9B u otro modelo compatible con llama.cpp. El contexto local usa un valor seguro de 4096.</p><label className="yarvis-muted mt-5 block text-[10px] font-black uppercase tracking-widest">Ruta del archivo .gguf</label><div className="mt-2 flex gap-2"><input value={localModelPath} onChange={(event) => setLocalModelPath(event.target.value)} placeholder="/home/ale/Modelos/Qwen.gguf" className="yarvis-input min-w-0 flex-1 rounded-xl border px-4 py-3 text-sm outline-none focus:border-emerald-500" /><button onClick={chooseLocalModel} className="yarvis-primary rounded-xl px-4 text-[10px] font-black uppercase tracking-widest">Buscar</button></div>{localModelName && <p className="yarvis-muted mt-3 truncate text-[10px] font-bold">Actual: {localModelName}</p>}</div></div>}
            {configMessage && <p className="mt-4 rounded-xl border border-sky-500/30 bg-sky-500/10 px-4 py-3 text-xs font-bold text-sky-500">{configMessage}</p>}
          </div>
          <div className="yarvis-border flex gap-3 border-t px-6 py-5 sm:px-8"><button onClick={() => setShowConfig(false)} className="yarvis-panel-soft yarvis-muted flex-1 rounded-xl py-3 text-[10px] font-black uppercase tracking-widest">Cerrar</button><button onClick={saveApiConfig} className="yarvis-primary flex-1 rounded-xl py-3 text-[10px] font-black uppercase tracking-widest">Guardar configuración</button></div>
        </div>
      </div>}
    </div>
  );
};

export default AdminYarvis;
