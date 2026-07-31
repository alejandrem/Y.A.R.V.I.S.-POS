import { useState, useEffect, useRef, useCallback, type KeyboardEvent } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import Markdown from "react-markdown";
import remarkGfm from "remark-gfm";

interface Message {
  role: "user" | "assistant";
  content: string;
  model?: string;
  timestamp: number;
}

interface ChatWidgetProps {
  role: "admin" | "empleado";
  userId: string;
  suggestions: string[];
}

type ModelKey = "auto" | "0.5B" | "0.8B" | "1.7B";

const MODEL_OPTIONS: { key: ModelKey; label: string; desc: string }[] = [
  { key: "auto", label: "AUTO", desc: "Siempre usa 0.5B (rápido)" },
  { key: "0.5B", label: "0.5B", desc: "Siempre listo, ~450MB RAM" },
  { key: "0.8B", label: "0.8B", desc: "Balance, se carga bajo demanda" },
  { key: "1.7B", label: "1.7B", desc: "Más capaz, necesita ≥4GB RAM" },
];

const ChatWidget = ({ role, userId, suggestions }: ChatWidgetProps) => {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState("");
  const [selectedModel, setSelectedModel] = useState<ModelKey>("auto");
  const [showModelPicker, setShowModelPicker] = useState(false);

  const [streamingText, setStreamingText] = useState("");
  const [streamingModel, setStreamingModel] = useState("");
  const [isStreaming, setIsStreaming] = useState(false);

  const [loadingModel, setLoadingModel] = useState<string | null>(null);
  const [loadedModels, setLoadedModels] = useState<Record<string, boolean>>({
    "0.5B": true, "0.8B": false, "1.7B": false,
  });
  const [ramGb, setRamGb] = useState(0);
  const [currentSuggestion, setCurrentSuggestion] = useState(0);

  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const modelPickerRef = useRef<HTMLDivElement>(null);
  const streamingTextRef = useRef("");

  const storageKey = `yarvis_chat_${userId}`;

  const fetchModelStatus = useCallback(async () => {
    try {
      const status = await invoke<{ models: Record<string, boolean>; ram_gb: number }>("get_model_status");
      setLoadedModels(status.models);
      setRamGb(status.ram_gb);
    } catch { /* ignore */ }
  }, []);

  useEffect(() => {
    fetchModelStatus();
  }, [fetchModelStatus]);

  useEffect(() => {
    try {
      const saved = localStorage.getItem(storageKey);
      if (saved) setMessages(JSON.parse(saved));
    } catch { /* ignore */ }
  }, [storageKey]);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, streamingText]);

  useEffect(() => {
    if (inputRef.current) {
      inputRef.current.style.height = "auto";
      inputRef.current.style.height = Math.min(inputRef.current.scrollHeight, 120) + "px";
    }
  }, [input]);

  useEffect(() => {
    const handleClick = (e: MouseEvent) => {
      if (modelPickerRef.current && !modelPickerRef.current.contains(e.target as Node)) {
        setShowModelPicker(false);
      }
    };
    document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  }, []);

  useEffect(() => {
    if (messages.length > 0 || isStreaming) return;
    const interval = setInterval(() => {
      setCurrentSuggestion((prev) => (prev + 1) % suggestions.length);
    }, 2500);
    return () => clearInterval(interval);
  }, [messages.length, isStreaming, suggestions.length]);

  const saveHistory = (msgs: Message[]) => {
    try {
      localStorage.setItem(storageKey, JSON.stringify(msgs));
    } catch { /* ignore */ }
  };

  const clearHistory = () => {
    setMessages([]);
    localStorage.removeItem(storageKey);
  };

  const handleModelSelect = async (model: ModelKey) => {
    setShowModelPicker(false);
    if (model === "auto" || model === "0.5B") {
      setSelectedModel(model);
      return;
    }

    if (loadedModels[model]) {
      setSelectedModel(model);
      return;
    }

    if (model === "1.7B" && ramGb < 4.0 && ramGb > 0) {
      setError(`RAM insuficiente para Qwen 1.7B: ${ramGb.toFixed(1)}GB disponibles (mínimo 4GB)`);
      return;
    }

    setLoadingModel(model);
    setError("");
    setSelectedModel(model);

    try {
      const result = await invoke<{
        status: string;
        models: Record<string, boolean>;
        ram_gb: number;
      }>("load_chat_model", { model });
      setLoadedModels(result.models);
      setRamGb(result.ram_gb);
    } catch (err) {
      setError(String(err));
      setSelectedModel("auto");
    } finally {
      setLoadingModel(null);
    }
  };

  const handleSend = async (text?: string) => {
    const msg = (text || input).trim();
    if (!msg || isLoading) return;

    setError("");
    setInput("");

    const userMessage: Message = {
      role: "user",
      content: msg,
      timestamp: Date.now(),
    };

    const updatedMessages = [...messages, userMessage];
    setMessages(updatedMessages);
    setIsLoading(true);
    setIsStreaming(true);
    setStreamingText("");
    streamingTextRef.current = "";

    const unlistenToken = await listen<{ token: string; model: string }>("chat-token", (event) => {
      streamingTextRef.current += event.payload.token;
      setStreamingText(streamingTextRef.current);
      setStreamingModel(event.payload.model);
    });

    const unlistenDone = await listen<{ model: string }>("chat-done", () => { });

    const unlistenComplete = await listen<{ response: string; model: string }>("chat-complete", (event) => {
      const assistantMessage: Message = {
        role: "assistant",
        content: event.payload.response,
        model: event.payload.model,
        timestamp: Date.now(),
      };
      const finalMessages = [...updatedMessages, assistantMessage];
      setMessages(finalMessages);
      saveHistory(finalMessages);
      setIsLoading(false);
      setIsStreaming(false);
      setStreamingText("");
      setStreamingModel("");
      unlistenToken();
      unlistenDone();
      unlistenError();
      fetchModelStatus();
    });

    const unlistenError = await listen<{ error: string }>("chat-error", (event) => {
      setError(event.payload.error);
      setIsLoading(false);
      setIsStreaming(false);
      setStreamingText("");
      unlistenToken();
      unlistenDone();
      unlistenComplete();
    });

    try {
      await invoke("send_chat_stream", {
        messages: updatedMessages.map((m) => ({ role: m.role, content: m.content })),
        role,
        model: selectedModel,
      });
    } catch (err) {
      setError(String(err));
      setIsLoading(false);
      setIsStreaming(false);
      setStreamingText("");
      unlistenToken();
      unlistenDone();
      unlistenComplete();
      unlistenError();
    }
  };

  const handleKeyDown = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const currentModel = MODEL_OPTIONS.find((m) => m.key === selectedModel) || MODEL_OPTIONS[0];

  return (
    <div className="flex flex-col h-full bg-white">
      {/* HEADER */}
      <div className="flex-shrink-0 px-8 py-5 border-b border-neutral-100 bg-white flex items-center justify-between">
        <div className="flex items-center gap-4">
          <div className="w-11 h-11 bg-neutral-900 rounded-2xl flex items-center justify-center text-white font-black text-base">Y</div>
          <div>
            <h2 className="text-base font-black text-neutral-900 uppercase tracking-tight leading-none">Y.A.R.V.I.S.</h2>
            <p className="text-[11px] font-bold text-neutral-400 uppercase tracking-widest mt-1">Asistente Inteligente</p>
          </div>
        </div>

        <div className="flex items-center gap-3">
          {/* MODEL PICKER */}
          <div ref={modelPickerRef} className="relative">
            <button
              onClick={() => setShowModelPicker(!showModelPicker)}
              disabled={!!loadingModel}
              className="flex items-center gap-2.5 px-4 py-2.5 bg-neutral-100 hover:bg-neutral-200 rounded-xl transition-all disabled:opacity-50"
            >
              <div className={`w-2 h-2 rounded-full ${loadingModel ? "bg-amber-500 animate-pulse" : selectedModel === "auto" ? "bg-blue-500" : selectedModel === "1.7B" ? "bg-emerald-500" : selectedModel === "0.8B" ? "bg-amber-500" : "bg-neutral-400"}`}></div>
              <span className="text-[11px] font-black text-neutral-600 uppercase tracking-widest">
                {loadingModel ? `Cargando ${loadingModel}...` : `Qwen ${currentModel.label}`}
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
                  {MODEL_OPTIONS.map((opt) => {
                    const isLoaded = loadedModels[opt.key] || opt.key === "auto";
                    const isLoadingThis = loadingModel === opt.key;
                    const isBlocked = opt.key === "1.7B" && ramGb < 4.0 && ramGb > 0;

                    return (
                      <button
                        key={opt.key}
                        onClick={() => handleModelSelect(opt.key)}
                        disabled={isLoadingThis || (isBlocked && !isLoaded)}
                        className={`w-full flex items-center gap-3 px-4 py-3 rounded-xl text-left transition-all ${selectedModel === opt.key
                          ? "bg-neutral-900 text-white"
                          : "hover:bg-neutral-50 text-neutral-700"
                          } ${isBlocked && !isLoaded ? "opacity-40 cursor-not-allowed" : ""}`}
                      >
                        <div className={`w-2.5 h-2.5 rounded-full flex-shrink-0 ${isLoadingThis ? "bg-amber-500 animate-pulse"
                          : isLoaded ? "bg-emerald-500"
                            : selectedModel === opt.key ? "bg-white"
                              : opt.key === "auto" ? "bg-blue-500"
                                : opt.key === "1.7B" ? "bg-emerald-500"
                                  : opt.key === "0.8B" ? "bg-amber-500"
                                    : "bg-neutral-400"
                          }`}></div>
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2">
                            <p className={`text-[12px] font-black ${selectedModel === opt.key ? "text-white" : "text-neutral-900"}`}>
                              Qwen {opt.label}
                            </p>
                            {isLoaded && opt.key !== "auto" && (
                              <span className={`text-[9px] font-black px-2 py-0.5 rounded-full ${selectedModel === opt.key ? "bg-white/20 text-white" : "bg-emerald-50 text-emerald-600"}`}>
                                LISTO
                              </span>
                            )}
                            {isLoadingThis && (
                              <span className="text-[9px] font-black px-2 py-0.5 rounded-full bg-amber-50 text-amber-600 animate-pulse">
                                CARGANDO
                              </span>
                            )}
                            {isBlocked && !isLoaded && (
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
                  <div className="px-4 py-2.5 border-t border-neutral-100 mt-1">
                    <p className="text-[10px] font-bold text-neutral-400">
                      RAM: {ramGb > 0 ? `${ramGb.toFixed(1)}GB` : "..."} {ramGb >= 4.0 ? "✓" : ramGb > 0 ? `⚠ <4GB` : ""}
                    </p>
                  </div>
                </div>
              </div>
            )}
          </div>

          {messages.length > 0 && (
            <button onClick={clearHistory} className="px-4 py-2.5 text-[11px] font-black text-neutral-400 hover:text-red-500 hover:bg-red-50 rounded-xl uppercase tracking-widest transition-all">
              Limpiar
            </button>
          )}
        </div>
      </div>

      {/* LOADING MODEL BAR */}
      {loadingModel && (
        <div className="flex-shrink-0 px-8 py-3 bg-amber-50 border-b border-amber-200">
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

      {/* MESSAGES */}
      <div className="flex-1 overflow-y-auto px-8 py-5 space-y-5 custom-scrollbar">
        {messages.length === 0 && !isStreaming && (
          <div className="flex flex-col items-center justify-center h-full animate-in fade-in duration-500">
            <div className="w-16 h-16 bg-neutral-100 rounded-full flex items-center justify-center mb-6">
              <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" className="text-neutral-500">
                <path d="M12 8V4H8" /><rect width="16" height="12" x="4" y="8" rx="2" /><path d="M2 14h2" /><path d="M20 14h2" /><path d="M15 13v2" /><path d="M9 13v2" />
              </svg>
            </div>
            <p className="text-[15px] text-neutral-400 font-medium text-center max-w-md leading-relaxed h-6">
              {suggestions[currentSuggestion]}
              <span className="inline-block w-0.5 h-4 bg-neutral-400 ml-0.5 animate-pulse rounded-sm align-middle"></span>
            </p>
          </div>
        )}

        {messages.map((msg, idx) => (
          <div key={idx} className={`flex ${msg.role === "user" ? "justify-end" : "justify-start"} animate-in fade-in slide-in-from-bottom-1 duration-300`}>
            <div className={`max-w-[75%] ${msg.role === "user" ? "bg-neutral-900 text-white rounded-2xl rounded-br-md px-6 py-4" : "bg-neutral-50 border border-neutral-200 text-neutral-800 rounded-2xl rounded-bl-md px-6 py-4"}`}>
              {msg.role === "assistant" ? (
                <div className="chat-markdown text-[14px] leading-relaxed"><Markdown remarkPlugins={[remarkGfm]}>{msg.content}</Markdown></div>
              ) : (
                <p className="text-[14px] leading-relaxed whitespace-pre-wrap">{msg.content}</p>
              )}
              {msg.role === "assistant" && msg.model && (
                <div className="mt-2 pt-2 border-t border-neutral-200/50 flex items-center gap-1.5">
                  <div className={`w-2 h-2 rounded-full ${msg.model === "1.7B" ? "bg-emerald-500" : msg.model === "0.8B" ? "bg-amber-500" : "bg-neutral-400"}`}></div>
                  <span className="text-[10px] font-black text-neutral-400 uppercase tracking-widest">Qwen {msg.model}</span>
                </div>
              )}
            </div>
          </div>
        ))}

        {/* STREAMING RESPONSE */}
        {isStreaming && streamingText && (
          <div className="flex justify-start animate-in fade-in duration-200">
            <div className="max-w-[75%] bg-neutral-50 border border-neutral-200 text-neutral-800 rounded-2xl rounded-bl-md px-6 py-4">
              <div className="chat-markdown text-[14px] leading-relaxed">
                <Markdown remarkPlugins={[remarkGfm]}>{streamingText}</Markdown>
                <span className="inline-block w-1.5 h-4 bg-neutral-900 ml-0.5 animate-pulse rounded-sm align-middle"></span>
              </div>
              <div className="mt-2 pt-2 border-t border-neutral-200/50 flex items-center gap-1.5">
                <div className={`w-2 h-2 rounded-full animate-pulse ${streamingModel === "1.7B" ? "bg-emerald-500" : streamingModel === "0.8B" ? "bg-amber-500" : "bg-blue-500"}`}></div>
                <span className="text-[10px] font-black text-neutral-400 uppercase tracking-widest">
                  Qwen {streamingModel || "..."} generando
                </span>
              </div>
            </div>
          </div>
        )}

        {/* LOADING STATE (before streaming) */}
        {isLoading && !isStreaming && (
          <div className="flex justify-start animate-in fade-in duration-300">
            <div className="bg-neutral-50 border border-neutral-200 rounded-2xl rounded-bl-md px-6 py-4">
              <div className="flex items-center gap-3">
                <div className="flex gap-1.5">
                  <div className="w-2 h-2 bg-neutral-400 rounded-full animate-bounce [animation-delay:-0.3s]"></div>
                  <div className="w-2 h-2 bg-neutral-400 rounded-full animate-bounce [animation-delay:-0.15s]"></div>
                  <div className="w-2 h-2 bg-neutral-400 rounded-full animate-bounce"></div>
                </div>
                <span className="text-[11px] font-bold text-neutral-400 uppercase tracking-widest">
                  {loadingModel ? `Cargando Qwen ${loadingModel}...` : "Conectando..."}
                </span>
              </div>
            </div>
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      {/* INPUT */}
      <div className="flex-shrink-0 px-8 pb-8 pt-3">
        {error && (
          <div className="mb-3 px-5 py-3 bg-red-50 border border-red-200 rounded-2xl text-[12px] font-bold text-red-600 flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10" /><line x1="15" y1="9" x2="9" y2="15" /><line x1="9" y1="9" x2="15" y2="15" /></svg>
            {error}
            <button onClick={() => setError("")} className="ml-auto text-red-400 hover:text-red-600">×</button>
          </div>
        )}
        <div className="flex items-end gap-3 bg-white border border-neutral-200 rounded-3xl px-5 py-4 shadow-lg shadow-neutral-200/50 focus-within:border-neutral-400 focus-within:shadow-xl focus-within:shadow-neutral-300/50 transition-all duration-300">
          <textarea
            ref={inputRef}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Pregúntale a Y.A.R.V.I.S..."
            rows={1}
            className="flex-1 bg-transparent text-[15px] font-medium text-neutral-900 placeholder:text-neutral-400 resize-none outline-none leading-relaxed max-h-[120px]"
          />
          <button
            onClick={() => handleSend()}
            disabled={!input.trim() || isLoading}
            className="flex-shrink-0 w-10 h-10 bg-neutral-900 hover:bg-neutral-800 disabled:bg-neutral-200 text-white rounded-full flex items-center justify-center transition-all duration-200 disabled:cursor-not-allowed hover:scale-105 active:scale-95 shadow-md"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><line x1="22" y1="2" x2="11" y2="13" /><polygon points="22 2 15 22 11 13 2 9 22 2" /></svg>
          </button>
        </div>
      </div>
    </div>
  );
};

export default ChatWidget;
