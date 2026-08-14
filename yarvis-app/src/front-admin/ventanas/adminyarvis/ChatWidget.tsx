import { useState, useEffect, useRef, type KeyboardEvent } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import Markdown from "react-markdown";
import remarkGfm from "remark-gfm";

const STREAM_TIMEOUT_MS = 180_000;

interface Message {
  role: "user" | "assistant";
  content: string;
  model?: string;
  thinking?: string;
  timestamp: number;
}

export type ModelKey = "0.5B" | "0.8B" | "1.7B";

export const MODEL_OPTIONS: { key: ModelKey; label: string; desc: string; minRam: number }[] = [
  { key: "1.7B", label: "1.7B", desc: "El más capaz", minRam: 4.0 },
  { key: "0.8B", label: "0.8B", desc: "Balance ideal", minRam: 1.0 },
  { key: "0.5B", label: "0.5B", desc: "Rápido y ligero", minRam: 0 },
];

export const CLOUD_PROVIDERS: { id: string; display: string; defaultModel: string }[] = [
  { id: "google", display: "Gemini", defaultModel: "gemini-2.0-flash" },
  { id: "opencode", display: "OpenCode", defaultModel: "mimo-v2.5-free" },
];

export interface ActiveCloud {
  provider: string;
  apiKey: string;
  display: string;
  model: string;
}

export function getActiveCloud(): ActiveCloud {
  const empty: ActiveCloud = { provider: "", apiKey: "", display: "", model: "" };
  try {
    const raw = localStorage.getItem("yarvis_api_keys");
    if (!raw) return empty;
    const keys = JSON.parse(raw) as Record<string, string>;
    let stored: { provider?: string; model?: string } | null = null;
    try {
      stored = JSON.parse(localStorage.getItem("yarvis_cloud_model") || "null");
    } catch { /* ignore */ }
    for (const p of CLOUD_PROVIDERS) {
      if ((keys[p.id] || "").trim()) {
        const model =
          stored && stored.provider === p.id && stored.model
            ? stored.model
            : p.defaultModel;
        return { provider: p.id, apiKey: keys[p.id].trim(), display: p.display, model };
      }
    }
  } catch { /* ignore */ }
  return empty;
}

function modelDotClass(model: string): string {
  if (model === "1.7B") return "bg-emerald-500";
  if (model === "0.8B") return "bg-amber-500";
  if (model === "0.5B") return "bg-neutral-400";
  return "bg-blue-500";
}

function contextLimit(_model: string): number {
  return 131072;
}

function fmtTokens(n: number): string {
  return n >= 1000 ? `${(n / 1000).toFixed(1)}k` : `${Math.round(n)}`;
}

interface ModelPickerState {
  selectedModel: ModelKey;
  loadingModel: string | null;
  loadedModels: Record<string, boolean>;
  ramGb: number;
  showPicker: boolean;
}

interface ChatWidgetProps {
  role: "admin" | "empleado";
  userId: string;
  suggestions: string[];
  modelState: ModelPickerState;
  onModelSelect: (model: ModelKey) => void;
  onTogglePicker: () => void;
  clearTrigger: number;
}

const ChatWidget = ({ role, userId, suggestions, modelState, clearTrigger }: ChatWidgetProps) => {
  const { selectedModel, loadingModel } = modelState;
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState("");

  const [streamingText, setStreamingText] = useState("");
  const [streamingModel, setStreamingModel] = useState("");
  const [isStreaming, setIsStreaming] = useState(false);
  const [thinkingText, setThinkingText] = useState("");
  const [expandedThinking, setExpandedThinking] = useState<Set<number>>(new Set());

  const [currentSuggestion, setCurrentSuggestion] = useState(0);

  const [contextUsed, setContextUsed] = useState(0);
  const [contextMax, setContextMax] = useState(131072);
  const usageRealRef = useRef(false);

  const messagesEndRef = useRef<HTMLDivElement>(null);
  const messagesContainerRef = useRef<HTMLDivElement>(null);
  const stickToBottomRef = useRef(true);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const streamingTextRef = useRef("");
  const streamingModelRef = useRef("");
  const thinkingTextRef = useRef("");
  const unlistenRef = useRef<(() => void)[]>([]);
  const stopRef = useRef<(() => void) | null>(null);

  useEffect(() => {
    return () => {
      unlistenRef.current.forEach((fn) => fn());
      unlistenRef.current = [];
    };
  }, []);

  const storageKey = `yarvis_chat_${userId}`;

  useEffect(() => {
    try {
      const saved = localStorage.getItem(storageKey);
      if (saved) setMessages(JSON.parse(saved));
    } catch { /* ignore */ }
  }, [storageKey]);

  useEffect(() => {
    if (clearTrigger > 0) {
      setMessages([]);
      setExpandedThinking(new Set());
      localStorage.removeItem(storageKey);
      usageRealRef.current = false;
      setContextUsed(0);
    }
  }, [clearTrigger, storageKey]);

  useEffect(() => {
    if (usageRealRef.current) return;
    const chars =
      messages.reduce((acc, m) => acc + (m.content || "").length, 0) + streamingText.length;
    setContextUsed(Math.round(chars / 4));
    setContextMax(contextLimit(getActiveCloud().model || streamingModel));
  }, [messages, streamingText, streamingModel]);

  const toggleThinking = (idx: number) => {
    setExpandedThinking((prev) => {
      const next = new Set(prev);
      if (next.has(idx)) next.delete(idx);
      else next.add(idx);
      return next;
    });
  };

  const handleScroll = () => {
    const el = messagesContainerRef.current;
    if (!el) return;
    const distanceFromBottom = el.scrollHeight - el.scrollTop - el.clientHeight;
    stickToBottomRef.current = distanceFromBottom < 120;
  };

  useEffect(() => {
    if (!stickToBottomRef.current) return;
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, streamingText, thinkingText]);

  useEffect(() => {
    if (inputRef.current) {
      inputRef.current.style.height = "auto";
      inputRef.current.style.height = Math.min(inputRef.current.scrollHeight, 120) + "px";
    }
  }, [input]);

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
    stickToBottomRef.current = true;
    setIsLoading(true);
    setIsStreaming(true);
    setStreamingText("");
    streamingTextRef.current = "";
    streamingModelRef.current = "";
    setThinkingText("");
    thinkingTextRef.current = "";

    let settled = false;
    let timeoutId = 0;

    const cleanupListeners = () => {
      unlistenRef.current.forEach((fn) => fn());
      unlistenRef.current = [];
    };

    const finish = (response: string, model: string) => {
      if (settled) return;
      settled = true;
      stopRef.current = null;
      window.clearTimeout(timeoutId);
      cleanupListeners();
      const assistantMessage: Message = {
        role: "assistant",
        content: response,
        model,
        thinking: thinkingTextRef.current.trim() || undefined,
        timestamp: Date.now(),
      };
      const finalMessages = [...updatedMessages, assistantMessage];
      setMessages(finalMessages);
      saveHistory(finalMessages);
      setIsLoading(false);
      setIsStreaming(false);
      setStreamingText("");
      setStreamingModel("");
      setThinkingText("");
      thinkingTextRef.current = "";
    };

    const fail = (errorMessage: string) => {
      if (settled) return;
      settled = true;
      stopRef.current = null;
      window.clearTimeout(timeoutId);
      cleanupListeners();
      saveHistory(updatedMessages);
      setError(errorMessage);
      setIsLoading(false);
      setIsStreaming(false);
      setStreamingText("");
      setStreamingModel("");
      setThinkingText("");
      thinkingTextRef.current = "";
    };

    const stop = () => {
      if (settled) return;
      settled = true;
      stopRef.current = null;
      window.clearTimeout(timeoutId);
      cleanupListeners();
      const partial = streamingTextRef.current;
      const finalMessages = partial.trim()
        ? [
            ...updatedMessages,
            {
              role: "assistant" as const,
              content: partial,
              model: streamingModelRef.current || undefined,
              thinking: thinkingTextRef.current.trim() || undefined,
              timestamp: Date.now(),
            },
          ]
        : updatedMessages;
      setMessages(finalMessages);
      saveHistory(finalMessages);
      setIsLoading(false);
      setIsStreaming(false);
      setStreamingText("");
      setStreamingModel("");
      setThinkingText("");
      thinkingTextRef.current = "";
      invoke("stop_chat_stream").catch(() => {});
    };
    stopRef.current = stop;

    timeoutId = window.setTimeout(() => {
      fail("El motor de IA tardó demasiado en responder. Inténtalo de nuevo.");
    }, STREAM_TIMEOUT_MS);

    try {
      unlistenRef.current.push(await listen<{ token: string; model: string }>("chat-think", (event) => {
        if (settled) return;
        thinkingTextRef.current += event.payload.token;
        setThinkingText(thinkingTextRef.current);
        streamingModelRef.current = event.payload.model;
        setStreamingModel(event.payload.model);
      }));

      unlistenRef.current.push(await listen<{ token: string; model: string }>("chat-token", (event) => {
        if (settled) return;
        streamingTextRef.current += event.payload.token;
        setStreamingText(streamingTextRef.current);
        streamingModelRef.current = event.payload.model;
        setStreamingModel(event.payload.model);
      }));

      unlistenRef.current.push(await listen<{
        usage: { prompt_tokens?: number; completion_tokens?: number; total_tokens?: number };
      }>("chat-usage", (event) => {
        if (settled) return;
        const u = event.payload.usage;
        const total = u.total_tokens || (u.prompt_tokens || 0) + (u.completion_tokens || 0);
        if (total) {
          usageRealRef.current = true;
          setContextUsed(total);
          setContextMax(contextLimit(getActiveCloud().model || streamingModelRef.current));
        }
      }));

      unlistenRef.current.push(await listen<{ response: string; model: string }>("chat-complete", (event) => {
        finish(event.payload.response, event.payload.model);
      }));

      unlistenRef.current.push(await listen<{ error: string }>("chat-error", (event) => {
        fail(event.payload.error);
      }));

      const cloud = getActiveCloud();
      await invoke("send_chat_stream", {
        messages: updatedMessages.slice(-10).map((m) => ({ role: m.role, content: m.content })),
        role,
        model: cloud.provider ? cloud.model : selectedModel,
        provider: cloud.provider,
        apiKey: cloud.apiKey,
      });
    } catch (err) {
      fail(String(err));
    }
  };

  const handleKeyDown = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <div className="flex flex-col h-full bg-white">
      {/* MESSAGES */}
      <div ref={messagesContainerRef} onScroll={handleScroll} className="flex-1 overflow-y-auto px-8 py-5 space-y-5 custom-scrollbar">
        {messages.length === 0 && !isStreaming && (
          <div className="flex flex-col items-center justify-center h-full animate-in fade-in duration-500">
            <div className="w-16 h-16 bg-neutral-100 rounded-full flex items-center justify-center mb-6">
              <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" className="text-neutral-500">
                <path d="M12 8V4H8" /><rect width="16" height="12" x="4" y="8" rx="2" /><path d="M2 14h2" /><path d="M20 14h2" /><path d="M15 13v2" /><path d="M9 13v2" />
              </svg>
            </div>
            <p className="text-[17px] text-neutral-400 font-bold text-center max-w-md leading-relaxed h-8">
              {suggestions[currentSuggestion]}
              <span className="inline-block w-0.5 h-4 bg-neutral-400 ml-0.5 animate-pulse rounded-sm align-middle"></span>
            </p>
          </div>
        )}

        {messages.map((msg, idx) => (
          <div key={idx} className={`flex ${msg.role === "user" ? "justify-end" : "justify-start"} animate-in fade-in slide-in-from-bottom-1 duration-300`}>
            <div className={`max-w-[75%] ${msg.role === "user" ? "bg-neutral-900 text-white rounded-2xl rounded-br-md px-6 py-4" : "bg-neutral-50 border border-neutral-200 text-neutral-800 rounded-2xl rounded-bl-md px-6 py-4"}`}>
              {msg.role === "assistant" ? (
                <div className="chat-markdown text-[16px] font-bold leading-relaxed"><Markdown remarkPlugins={[remarkGfm]}>{msg.content}</Markdown></div>
              ) : (
                <p className="text-[16px] font-bold leading-relaxed whitespace-pre-wrap">{msg.content}</p>
              )}
              {msg.role === "assistant" && msg.model && (
                <div className="mt-2 pt-2 border-t border-neutral-200/50 flex items-center gap-1.5">
                  <div className={`w-2 h-2 rounded-full ${modelDotClass(msg.model)}`}></div>
                  <span className="text-[12px] font-black text-neutral-400 uppercase tracking-widest">{msg.model}</span>
                </div>
              )}
              {msg.role === "assistant" && msg.thinking && (
                <div className="mt-2">
                  <button
                    onClick={() => toggleThinking(idx)}
                    className="flex items-center gap-1.5 text-[13px] font-black text-neutral-400 uppercase tracking-widest hover:text-neutral-600 transition-colors"
                  >
                    <svg xmlns="http://www.w3.org/2000/svg" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round" className={`transition-transform ${expandedThinking.has(idx) ? "rotate-180" : ""}`}><path d="m6 9 6 6 6-6" /></svg>
                    {expandedThinking.has(idx) ? "Ocultar hilo de pensamiento" : "Ver hilo de pensamiento"}
                  </button>
                  {expandedThinking.has(idx) && (
                    <div className="mt-2 bg-neutral-100 border border-dashed border-neutral-300 rounded-xl px-4 py-3">
                      <p className="text-[15px] font-medium text-neutral-400 italic leading-relaxed whitespace-pre-wrap max-h-64 overflow-y-auto custom-scrollbar">{msg.thinking}</p>
                    </div>
                  )}
                </div>
              )}
            </div>
          </div>
        ))}

        {/* THINKING BOX */}
        {thinkingText && (
          <div className="flex justify-start animate-in fade-in duration-300">
            <div className="max-w-[75%] w-full bg-neutral-100 border border-dashed border-neutral-300 rounded-2xl rounded-bl-md px-6 py-4">
              <div className="flex items-center gap-2.5 mb-2.5">
                <div className="flex gap-1">
                  <div className="w-2 h-2 bg-amber-400 rounded-full animate-bounce [animation-delay:-0.3s]"></div>
                  <div className="w-2 h-2 bg-amber-400 rounded-full animate-bounce [animation-delay:-0.15s]"></div>
                  <div className="w-2 h-2 bg-amber-400 rounded-full animate-bounce"></div>
                </div>
                <span className="text-[17px] font-black text-neutral-500 uppercase tracking-widest">
                  El modelo está pensando...
                </span>
                <span className="text-[12px] font-bold text-neutral-400 uppercase tracking-widest ml-auto">
                  {streamingModel} · no es la respuesta final
                </span>
              </div>
              <p className="text-[15px] font-medium text-neutral-400 italic leading-relaxed whitespace-pre-wrap max-h-64 overflow-y-auto custom-scrollbar">
                {thinkingText}
              </p>
            </div>
          </div>
        )}

        {/* STREAMING RESPONSE */}
        {isStreaming && streamingText && (
          <div className="flex justify-start animate-in fade-in duration-200">
            <div className="max-w-[75%] bg-neutral-50 border border-neutral-200 text-neutral-800 rounded-2xl rounded-bl-md px-6 py-4">
              <div className="chat-markdown text-[16px] font-bold leading-relaxed">
                <Markdown remarkPlugins={[remarkGfm]}>{streamingText}</Markdown>
                <span className="inline-block w-1.5 h-5 bg-neutral-900 ml-0.5 animate-pulse rounded-sm align-middle"></span>
              </div>
              <div className="mt-2 pt-2 border-t border-neutral-200/50 flex items-center gap-1.5">
                <div className={`w-2 h-2 rounded-full animate-pulse ${modelDotClass(streamingModel)}`}></div>
                <span className="text-[12px] font-black text-neutral-400 uppercase tracking-widest">
                  {streamingModel || "..."} generando
                </span>
              </div>
            </div>
          </div>
        )}

        {/* WAITING FOR FIRST TOKEN */}
        {isStreaming && !streamingText && !thinkingText && (
          <div className="flex justify-start animate-in fade-in duration-300">
            <div className="bg-neutral-50 border border-neutral-200 rounded-2xl rounded-bl-md px-6 py-4">
              <div className="flex items-center gap-3">
                <div className="flex gap-1.5">
                  <div className="w-2 h-2 bg-neutral-400 rounded-full animate-bounce [animation-delay:-0.3s]"></div>
                  <div className="w-2 h-2 bg-neutral-400 rounded-full animate-bounce [animation-delay:-0.15s]"></div>
                  <div className="w-2 h-2 bg-neutral-400 rounded-full animate-bounce"></div>
                </div>
                <span className="text-[13px] font-bold text-neutral-400 uppercase tracking-widest animate-pulse">
                  {streamingModel ? `${streamingModel} escribiendo...` : "Escribiendo..."}
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
                <span className="text-[13px] font-bold text-neutral-400 uppercase tracking-widest">
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
        {contextUsed > 0 && (
          <div className="mb-3 flex items-center gap-2.5">
            <span className="text-[9px] font-black uppercase tracking-widest text-neutral-400 flex-shrink-0">Contexto</span>
            <div className="flex-1 h-1.5 bg-neutral-100 rounded-full overflow-hidden">
              <div
                className={`h-full rounded-full transition-all duration-300 ${contextUsed / contextMax > 0.85 ? "bg-red-500" : "bg-neutral-900"}`}
                style={{ width: `${Math.min(100, (contextUsed / contextMax) * 100)}%` }}
              />
            </div>
            <span className="text-[9px] font-bold text-neutral-400 flex-shrink-0 tabular-nums">
              {fmtTokens(contextUsed)} / {fmtTokens(contextMax)} tok
            </span>
          </div>
        )}
        {error && (
          <div className="mb-3 px-5 py-3 bg-red-50 border border-red-200 rounded-2xl text-[14px] font-bold text-red-600 flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="12" cy="12" r="10" /><line x1="15" y1="9" x2="9" y2="15" /><line x1="9" y1="9" x2="15" y2="15" /></svg>
            {error}
            <button onClick={() => setError("")} className="ml-auto text-red-400 hover:text-red-600">×</button>
          </div>
        )}
        <div className="flex items-end gap-3 bg-white border border-neutral-200 rounded-3xl px-5 py-4 shadow-lg shadow-neutral-200/50 focus-within:border-neutral-400 focus-within:shadow-xl focus-within:shadow-neutral-300/50 transition-all duration-300">
          <textarea
            ref={inputRef}
            value={input}
            onChange={(e) => {
              setInput(e.target.value);
              if (error) setError("");
            }}
            onKeyDown={handleKeyDown}
            placeholder="Pregúntale a Y.A.R.V.I.S..."
            rows={1}
            className="flex-1 bg-transparent text-[17px] font-bold text-neutral-900 placeholder:text-neutral-400 resize-none outline-none leading-relaxed max-h-[120px]"
          />
          <button
            onClick={() => (isLoading ? stopRef.current?.() : handleSend())}
            disabled={!isLoading && !input.trim()}
            title={isLoading ? "Detener" : "Enviar"}
            className={`flex-shrink-0 w-10 h-10 text-white rounded-full flex items-center justify-center transition-all duration-200 disabled:cursor-not-allowed hover:scale-105 active:scale-95 shadow-md ${isLoading ? "bg-red-600 hover:bg-red-500" : "bg-neutral-900 hover:bg-neutral-800 disabled:bg-neutral-200"}`}
          >
            {isLoading ? (
              <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><rect x="5" y="5" width="14" height="14" rx="2" /></svg>
            ) : (
              <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round"><line x1="22" y1="2" x2="11" y2="13" /><polygon points="22 2 15 22 11 13 2 9 22 2" /></svg>
            )}
          </button>
        </div>
      </div>
    </div>
  );
};

export default ChatWidget;
