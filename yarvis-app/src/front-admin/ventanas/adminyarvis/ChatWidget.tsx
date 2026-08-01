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
  timestamp: number;
}

export type ModelKey = "0.5B" | "0.8B" | "1.7B";

export const MODEL_OPTIONS: { key: ModelKey; label: string; desc: string; minRam: number }[] = [
  { key: "1.7B", label: "1.7B", desc: "El más capaz", minRam: 4.0 },
  { key: "0.8B", label: "0.8B", desc: "Balance ideal", minRam: 1.0 },
  { key: "0.5B", label: "0.5B", desc: "Rápido y ligero", minRam: 0 },
];

export const CLOUD_PROVIDERS: { id: string; display: string; defaultModel: string }[] = [
  { id: "openai", display: "ChatGPT", defaultModel: "gpt-4o-mini" },
  { id: "anthropic", display: "Claude", defaultModel: "claude-3-5-haiku-latest" },
  { id: "google", display: "Gemini", defaultModel: "gemini-2.0-flash" },
  { id: "mistral", display: "Mistral", defaultModel: "mistral-small-latest" },
  { id: "groq", display: "Groq", defaultModel: "llama-3.3-70b-versatile" },
  { id: "deepseek", display: "DeepSeek", defaultModel: "deepseek-chat" },
  { id: "ollama", display: "Ollama", defaultModel: "llama3" },
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
    for (const p of CLOUD_PROVIDERS) {
      if ((keys[p.id] || "").trim()) {
        return { provider: p.id, apiKey: keys[p.id].trim(), display: p.display, model: p.defaultModel };
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

  const [currentSuggestion, setCurrentSuggestion] = useState(0);

  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const streamingTextRef = useRef("");
  const unlistenRef = useRef<(() => void)[]>([]);

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
      localStorage.removeItem(storageKey);
    }
  }, [clearTrigger, storageKey]);

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
    setIsLoading(true);
    setIsStreaming(true);
    setStreamingText("");
    streamingTextRef.current = "";

    let settled = false;
    let timeoutId = 0;

    const cleanupListeners = () => {
      unlistenRef.current.forEach((fn) => fn());
      unlistenRef.current = [];
    };

    const finish = (response: string, model: string) => {
      if (settled) return;
      settled = true;
      window.clearTimeout(timeoutId);
      cleanupListeners();
      const assistantMessage: Message = {
        role: "assistant",
        content: response,
        model,
        timestamp: Date.now(),
      };
      const finalMessages = [...updatedMessages, assistantMessage];
      setMessages(finalMessages);
      saveHistory(finalMessages);
      setIsLoading(false);
      setIsStreaming(false);
      setStreamingText("");
      setStreamingModel("");
    };

    const fail = (errorMessage: string) => {
      if (settled) return;
      settled = true;
      window.clearTimeout(timeoutId);
      cleanupListeners();
      saveHistory(updatedMessages);
      setError(errorMessage);
      setIsLoading(false);
      setIsStreaming(false);
      setStreamingText("");
      setStreamingModel("");
    };

    timeoutId = window.setTimeout(() => {
      fail("El motor de IA tardó demasiado en responder. Inténtalo de nuevo.");
    }, STREAM_TIMEOUT_MS);

    try {
      unlistenRef.current.push(await listen<{ token: string; model: string }>("chat-token", (event) => {
        if (settled) return;
        streamingTextRef.current += event.payload.token;
        setStreamingText(streamingTextRef.current);
        setStreamingModel(event.payload.model);
      }));

      unlistenRef.current.push(await listen<{ response: string; model: string }>("chat-complete", (event) => {
        finish(event.payload.response, event.payload.model);
      }));

      unlistenRef.current.push(await listen<{ error: string }>("chat-error", (event) => {
        fail(event.payload.error);
      }));

      const cloud = getActiveCloud();
      await invoke("send_chat_stream", {
        messages: updatedMessages.map((m) => ({ role: m.role, content: m.content })),
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
                  <div className={`w-2 h-2 rounded-full ${modelDotClass(msg.model)}`}></div>
                  <span className="text-[10px] font-black text-neutral-400 uppercase tracking-widest">{msg.model}</span>
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
                <div className={`w-2 h-2 rounded-full animate-pulse ${modelDotClass(streamingModel)}`}></div>
                <span className="text-[10px] font-black text-neutral-400 uppercase tracking-widest">
                  {streamingModel || "..."} generando
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
            onChange={(e) => {
              setInput(e.target.value);
              if (error) setError("");
            }}
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
