import { useEffect, useMemo, useRef, useState, type KeyboardEvent } from "react";
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

interface ChatSession {
  id: string;
  title: string;
  messages: Message[];
  createdAt: number;
  updatedAt: number;
  modelSelection?: ChatModelSelection;
}

export type ModelKey = string;

export const CLOUD_PROVIDERS: { id: "google" | "opencode"; display: string; defaultModel: string }[] = [
  { id: "google", display: "Gemini", defaultModel: "gemini-2.0-flash" },
  { id: "opencode", display: "OpenCode", defaultModel: "mimo-v2.5-free" },
];

export interface CloudModel {
  id: string;
  name: string;
  context_window?: number;
}

export interface ChatModelSelection {
  provider: "" | "google" | "opencode";
  apiKey: string;
  model: string;
  label: string;
  contextWindow: number;
}

export interface ActiveCloud extends ChatModelSelection {}

export function getActiveCloud(): ActiveCloud {
  const empty: ActiveCloud = {
    provider: "",
    apiKey: "",
    model: "1.7B",
    label: "Modelo local",
    contextWindow: 4096,
  };

  try {
    const keys = JSON.parse(localStorage.getItem("yarvis_api_keys") || "{}") as Record<string, string>;
    const activeProvider = localStorage.getItem("yarvis_active_provider") as "google" | "opencode" | null;
    const provider = activeProvider && (keys[activeProvider] || "").trim()
      ? activeProvider
      : CLOUD_PROVIDERS.find((p) => (keys[p.id] || "").trim())?.id;
    if (!provider) return empty;

    const storedModel = localStorage.getItem(`yarvis_cloud_model_${provider}`) ||
      localStorage.getItem("yarvis_cloud_model");
    const model = storedModel || CLOUD_PROVIDERS.find((p) => p.id === provider)?.defaultModel || "";
    return {
      provider,
      apiKey: keys[provider].trim(),
      model,
      label: `${provider === "google" ? "Gemini" : "OpenCode"} · ${model}`,
      contextWindow: 131072,
    };
  } catch {
    return empty;
  }
}

interface ModelPickerState {
  loadingModel: string | null;
}

interface ChatWidgetProps {
  role: "admin" | "empleado";
  userId: string;
  suggestions: string[];
  modelState: ModelPickerState;
  modelSelection?: ChatModelSelection;
  clearTrigger: number;
}

function sessionKey(userId: string) {
  return `yarvis_chat_sessions_${userId}`;
}

function newSession(): ChatSession {
  const now = Date.now();
  return {
    id: `chat-${now}-${Math.random().toString(36).slice(2, 8)}`,
    title: "Nuevo chat",
    messages: [],
    createdAt: now,
    updatedAt: now,
  };
}

function loadSessions(userId: string): ChatSession[] {
  try {
    const stored = localStorage.getItem(sessionKey(userId));
    if (stored) {
      const parsed = JSON.parse(stored) as ChatSession[];
      if (Array.isArray(parsed) && parsed.length) return parsed;
    }

    // Migración silenciosa del único chat que existía antes del historial.
    const legacy = localStorage.getItem(`yarvis_chat_${userId}`);
    const migrated = newSession();
    if (legacy) migrated.messages = JSON.parse(legacy) as Message[];
    return [migrated];
  } catch {
    return [newSession()];
  }
}

function modelDotClass(model: string): string {
  if (model === "1.7B" || model.toLowerCase().includes("qwen") || model.toLowerCase().includes("gguf")) {
    return "bg-emerald-500";
  }
  return "bg-sky-500";
}

const ChatWidget = ({ role, userId, suggestions, modelState, modelSelection, clearTrigger }: ChatWidgetProps) => {
  const fallbackSelection = modelSelection || getActiveCloud();
  const localLoading = modelState.loadingModel;
  const [sessions, setSessions] = useState<ChatSession[]>(() => loadSessions(userId));
  const [activeChatId, setActiveChatId] = useState("");
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
  const [contextMax, setContextMax] = useState(fallbackSelection.contextWindow || 4096);
  const [showHistory, setShowHistory] = useState(true);
  const [renamingId, setRenamingId] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState("");

  const messagesContainerRef = useRef<HTMLDivElement>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const stickToBottomRef = useRef(true);
  const streamingTextRef = useRef("");
  const streamingModelRef = useRef("");
  const thinkingTextRef = useRef("");
  const usageRealRef = useRef(false);
  const listenersRef = useRef<(() => void)[]>([]);
  const stopRef = useRef<(() => void) | null>(null);
  const initializedSelectionRef = useRef(false);

  const activeSession = useMemo(
    () => sessions.find((session) => session.id === activeChatId) || sessions[0],
    [sessions, activeChatId],
  );
  const messages = activeSession?.messages || [];
  const currentSelection = activeSession?.modelSelection || fallbackSelection;
  const selectionKey = `${fallbackSelection.provider}:${fallbackSelection.model}:${fallbackSelection.label}`;
  const contextPercent = contextMax > 0 ? Math.min(100, Math.round((contextUsed / contextMax) * 100)) : 0;

  const persistSessions = (next: ChatSession[]) => {
    setSessions(next);
    try {
      localStorage.setItem(sessionKey(userId), JSON.stringify(next));
    } catch { /* localStorage can be unavailable in restricted webviews */ }
  };

  const updateActiveSession = (patch: Partial<ChatSession>) => {
    if (!activeSession) return;
    persistSessions(sessions.map((session) => (
      session.id === activeSession.id ? { ...session, ...patch, updatedAt: Date.now() } : session
    )));
  };

  useEffect(() => {
    if (!sessions.some((session) => session.id === activeChatId)) {
      setActiveChatId(sessions[0]?.id || "");
    }
  }, [sessions, activeChatId]);

  // Un cambio explícito desde el selector del encabezado actualiza solo el chat abierto.
  useEffect(() => {
    if (!initializedSelectionRef.current) {
      initializedSelectionRef.current = true;
      return;
    }
    updateActiveSession({ modelSelection: fallbackSelection });
    // La clave representa una elección manual; no dependemos del objeto mutable.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectionKey]);

  useEffect(() => {
    setContextMax(currentSelection.contextWindow || (currentSelection.provider ? 131072 : 4096));
    usageRealRef.current = false;
    setContextUsed(0);
    setExpandedThinking(new Set());
  }, [activeChatId, currentSelection.contextWindow, currentSelection.model]);

  useEffect(() => {
    if (clearTrigger > 0) {
      updateActiveSession({ messages: [], modelSelection: currentSelection });
      setExpandedThinking(new Set());
      usageRealRef.current = false;
      setContextUsed(0);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [clearTrigger]);

  useEffect(() => {
    return () => {
      listenersRef.current.forEach((fn) => fn());
      listenersRef.current = [];
    };
  }, []);

  useEffect(() => {
    if (usageRealRef.current) return;
    const estimated = Math.round(messages.reduce((total, message) => total + message.content.length, 0) / 4);
    setContextUsed(estimated);
  }, [messages, streamingText]);

  useEffect(() => {
    if (!stickToBottomRef.current) return;
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, streamingText, thinkingText]);

  useEffect(() => {
    if (!messages.length && !isStreaming) {
      const interval = window.setInterval(() => {
        setCurrentSuggestion((value) => (value + 1) % Math.max(suggestions.length, 1));
      }, 2600);
      return () => window.clearInterval(interval);
    }
  }, [messages.length, isStreaming, suggestions.length]);

  useEffect(() => {
    if (!inputRef.current) return;
    inputRef.current.style.height = "auto";
    inputRef.current.style.height = `${Math.min(inputRef.current.scrollHeight, 120)}px`;
  }, [input]);

  const createChat = () => {
    if (isLoading) return;
    const chat = newSession();
    persistSessions([chat, ...sessions]);
    setActiveChatId(chat.id);
    setError("");
  };

  const deleteChat = (id: string) => {
    if (isLoading) return;
    const remaining = sessions.filter((session) => session.id !== id);
    const next = remaining.length ? remaining : [newSession()];
    persistSessions(next);
    if (id === activeSession?.id) setActiveChatId(next[0].id);
  };

  const startRename = (session: ChatSession) => {
    setRenamingId(session.id);
    setRenameValue(session.title);
  };

  const finishRename = () => {
    if (!renamingId) return;
    const title = renameValue.trim() || "Nuevo chat";
    persistSessions(sessions.map((session) => session.id === renamingId ? { ...session, title } : session));
    setRenamingId(null);
  };

  const handleScroll = () => {
    const element = messagesContainerRef.current;
    if (!element) return;
    stickToBottomRef.current = element.scrollHeight - element.scrollTop - element.clientHeight < 120;
  };

  const toggleThinking = (index: number) => {
    setExpandedThinking((previous) => {
      const next = new Set(previous);
      if (next.has(index)) next.delete(index); else next.add(index);
      return next;
    });
  };

  const handleSend = async (text?: string) => {
    const messageText = (text || input).trim();
    if (!messageText || isLoading || !activeSession) return;

    const selection = currentSelection;
    const userMessage: Message = { role: "user", content: messageText, timestamp: Date.now() };
    const updatedMessages = [...messages, userMessage];
    const nextTitle = activeSession.title === "Nuevo chat" ? messageText.slice(0, 42) : activeSession.title;
    persistSessions(sessions.map((session) => session.id === activeSession.id ? {
      ...session,
      title: nextTitle,
      messages: updatedMessages,
      modelSelection: selection,
      updatedAt: Date.now(),
    } : session));

    setInput("");
    setError("");
    setIsLoading(true);
    setIsStreaming(true);
    setStreamingText("");
    setThinkingText("");
    streamingTextRef.current = "";
    streamingModelRef.current = "";
    thinkingTextRef.current = "";
    stickToBottomRef.current = true;

    let settled = false;
    let timeoutId = 0;
    const cleanup = () => {
      listenersRef.current.forEach((fn) => fn());
      listenersRef.current = [];
    };
    const saveMessages = (nextMessages: Message[]) => {
      persistSessions(sessions.map((session) => session.id === activeSession.id ? {
        ...session,
        title: nextTitle,
        messages: nextMessages,
        modelSelection: selection,
        updatedAt: Date.now(),
      } : session));
    };
    const finish = (response: string, model: string) => {
      if (settled) return;
      settled = true;
      window.clearTimeout(timeoutId);
      cleanup();
      saveMessages([...updatedMessages, {
        role: "assistant", content: response, model, thinking: thinkingTextRef.current.trim() || undefined, timestamp: Date.now(),
      }]);
      setIsLoading(false); setIsStreaming(false); setStreamingText(""); setStreamingModel(""); setThinkingText("");
    };
    const fail = (reason: string) => {
      if (settled) return;
      settled = true;
      window.clearTimeout(timeoutId);
      cleanup();
      saveMessages(updatedMessages);
      setError(reason); setIsLoading(false); setIsStreaming(false); setStreamingText(""); setStreamingModel(""); setThinkingText("");
    };
    const stop = () => {
      if (settled) return;
      settled = true;
      window.clearTimeout(timeoutId);
      cleanup();
      const partial = streamingTextRef.current.trim();
      saveMessages(partial ? [...updatedMessages, {
        role: "assistant", content: partial, model: streamingModelRef.current || undefined,
        thinking: thinkingTextRef.current.trim() || undefined, timestamp: Date.now(),
      }] : updatedMessages);
      setIsLoading(false); setIsStreaming(false); setStreamingText(""); setStreamingModel(""); setThinkingText("");
      invoke("stop_chat_stream").catch(() => {});
    };
    stopRef.current = stop;
    timeoutId = window.setTimeout(() => fail("El motor tardó demasiado en responder. Inténtalo de nuevo."), STREAM_TIMEOUT_MS);

    try {
      listenersRef.current.push(await listen<{ token: string; model: string }>("chat-think", (event) => {
        if (settled) return;
        thinkingTextRef.current += event.payload.token;
        setThinkingText(thinkingTextRef.current);
        streamingModelRef.current = event.payload.model; setStreamingModel(event.payload.model);
      }));
      listenersRef.current.push(await listen<{ token: string; model: string }>("chat-token", (event) => {
        if (settled) return;
        streamingTextRef.current += event.payload.token;
        setStreamingText(streamingTextRef.current);
        streamingModelRef.current = event.payload.model; setStreamingModel(event.payload.model);
      }));
      listenersRef.current.push(await listen<{ usage: { prompt_tokens?: number; total_tokens?: number } }>("chat-usage", (event) => {
        const usage = event.payload.usage;
        const total = usage.total_tokens || usage.prompt_tokens || 0;
        if (total) { usageRealRef.current = true; setContextUsed(total); }
      }));
      listenersRef.current.push(await listen<{ response: string; model: string }>("chat-complete", (event) => finish(event.payload.response, event.payload.model)));
      listenersRef.current.push(await listen<{ error: string }>("chat-error", (event) => fail(event.payload.error)));

      await invoke("send_chat_stream", {
        messages: updatedMessages.slice(-12).map((message) => ({ role: message.role, content: message.content })),
        role,
        model: selection.model,
        provider: selection.provider,
        apiKey: selection.apiKey,
      });
    } catch (err) {
      fail(String(err));
    }
  };

  const handleKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      handleSend();
    }
  };

  return (
    <div className="yarvis-shell flex h-full min-h-0">
      {showHistory && (
        <aside className="yarvis-panel-soft hidden w-64 flex-shrink-0 flex-col border-r lg:flex">
          <div className="flex items-center justify-between border-b yarvis-border px-5 py-5">
            <div>
              <p className="yarvis-faint text-[9px] font-black uppercase tracking-[0.22em]">Conversaciones</p>
              <p className="yarvis-text mt-1 text-xs font-black">Historial de Y.A.R.V.I.S.</p>
            </div>
            <button onClick={createChat} title="Nuevo chat" className="yarvis-primary flex h-8 w-8 items-center justify-center rounded-xl text-lg transition-transform hover:scale-105">+</button>
          </div>
          <div className="custom-scrollbar flex-1 space-y-1 overflow-y-auto p-3">
            {[...sessions].sort((a, b) => b.updatedAt - a.updatedAt).map((session) => (
              <div key={session.id} className={`group rounded-xl border p-2 transition-all ${session.id === activeSession?.id ? "yarvis-panel yarvis-border" : "border-transparent yarvis-hover-panel"}`}>
                {renamingId === session.id ? (
                  <input autoFocus value={renameValue} onChange={(event) => setRenameValue(event.target.value)} onBlur={finishRename} onKeyDown={(event) => { if (event.key === "Enter") finishRename(); if (event.key === "Escape") setRenamingId(null); }} className="yarvis-input w-full rounded-lg border px-2 py-1 text-[11px] font-bold outline-none" />
                ) : (
                  <button disabled={isLoading} onClick={() => setActiveChatId(session.id)} className="w-full text-left">
                    <p className="yarvis-text truncate text-[11px] font-black">{session.title}</p>
                    <p className="yarvis-faint mt-1 text-[9px] font-bold">{session.messages.length} mensajes</p>
                  </button>
                )}
                <div className="mt-1 hidden items-center justify-end gap-1 group-hover:flex">
                  <button onClick={() => startRename(session)} title="Renombrar" className="yarvis-muted rounded px-1 text-[10px] hover:text-current">✎</button>
                  <button onClick={() => deleteChat(session.id)} title="Eliminar chat" className="rounded px-1 text-[10px] text-red-400 hover:text-red-500">×</button>
                </div>
              </div>
            ))}
          </div>
          <div className="border-t yarvis-border px-5 py-4">
            <p className="yarvis-faint truncate text-[9px] font-bold" title={currentSelection.label}>Modelo: {currentSelection.label}</p>
          </div>
        </aside>
      )}

      <section className="flex min-w-0 flex-1 flex-col">
        <div ref={messagesContainerRef} onScroll={handleScroll} className="custom-scrollbar flex-1 overflow-y-auto px-5 py-6 sm:px-8">
          {!messages.length && !isStreaming && (
            <div className="flex h-full flex-col items-center justify-center animate-in fade-in duration-500">
              <div className="yarvis-panel-soft yarvis-border mb-6 flex h-16 w-16 items-center justify-center rounded-2xl border">
                <svg xmlns="http://www.w3.org/2000/svg" width="30" height="30" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" className="yarvis-muted"><path d="M12 8V4H8" /><rect width="16" height="12" x="4" y="8" rx="2" /><path d="M2 14h2M20 14h2M15 13v2M9 13v2" /></svg>
              </div>
              <p className="yarvis-muted max-w-md text-center text-base font-bold leading-relaxed">{suggestions[currentSuggestion] || "Pregúntale algo a Y.A.R.V.I.S."}</p>
              <p className="yarvis-faint mt-3 text-[10px] font-black uppercase tracking-[0.2em]">{currentSelection.label}</p>
            </div>
          )}

          <div className="mx-auto max-w-4xl space-y-5">
            {messages.map((message, index) => (
              <div key={`${message.timestamp}-${index}`} className={`flex animate-in fade-in slide-in-from-bottom-1 duration-300 ${message.role === "user" ? "justify-end" : "justify-start"}`}>
                <div className={`max-w-[88%] rounded-2xl border px-5 py-4 sm:max-w-[76%] ${message.role === "user" ? "yarvis-primary border-transparent" : "yarvis-panel-soft yarvis-border"}`}>
                  {message.role === "assistant" ? <div className="chat-markdown yarvis-text text-[15px] font-bold leading-relaxed"><Markdown remarkPlugins={[remarkGfm]}>{message.content}</Markdown></div> : <p className="text-[15px] font-bold leading-relaxed whitespace-pre-wrap">{message.content}</p>}
                  {message.role === "assistant" && message.model && <div className="yarvis-border yarvis-muted mt-3 flex items-center gap-2 border-t pt-2 text-[10px] font-black uppercase tracking-widest"><span className={`h-2 w-2 rounded-full ${modelDotClass(message.model)}`} />{message.model}</div>}
                  {message.role === "assistant" && message.thinking && <div className="mt-3"><button onClick={() => toggleThinking(index)} className="yarvis-muted text-[10px] font-black uppercase tracking-widest hover:text-current">{expandedThinking.has(index) ? "Ocultar razonamiento" : "Ver razonamiento"}</button>{expandedThinking.has(index) && <p className="yarvis-muted yarvis-panel mt-2 max-h-64 overflow-y-auto rounded-xl border border-dashed px-4 py-3 text-xs italic leading-relaxed">{message.thinking}</p>}</div>}
                </div>
              </div>
            ))}

            {thinkingText && <div className="flex justify-start"><div className="yarvis-panel-soft yarvis-border max-w-[88%] rounded-2xl border border-dashed px-5 py-4 sm:max-w-[76%]"><p className="yarvis-muted mb-2 text-[10px] font-black uppercase tracking-widest">{streamingModel || "Modelo"} está pensando…</p><p className="yarvis-muted max-h-48 overflow-y-auto whitespace-pre-wrap text-xs italic leading-relaxed">{thinkingText}</p></div></div>}
            {isStreaming && streamingText && <div className="flex justify-start"><div className="yarvis-panel-soft yarvis-border max-w-[88%] rounded-2xl border px-5 py-4 sm:max-w-[76%]"><div className="chat-markdown yarvis-text text-[15px] font-bold leading-relaxed"><Markdown remarkPlugins={[remarkGfm]}>{streamingText}</Markdown><span className="ml-1 inline-block h-4 w-1 animate-pulse rounded-sm bg-current align-middle" /></div><p className="yarvis-muted mt-3 text-[10px] font-black uppercase tracking-widest">{streamingModel || "Generando"}</p></div></div>}
            {isStreaming && !streamingText && !thinkingText && <div className="yarvis-muted text-xs font-black uppercase tracking-widest animate-pulse">{localLoading ? `Cargando ${localLoading}…` : `Conectando con ${currentSelection.label}…`}</div>}
            <div ref={messagesEndRef} />
          </div>
        </div>

        <div className="flex-shrink-0 px-5 pb-5 pt-3 sm:px-8 sm:pb-8">
          <div className="mx-auto max-w-4xl">
            {contextUsed > 0 && <div className="yarvis-muted mb-3 flex items-center gap-3 text-[10px] font-black uppercase tracking-widest"><span className="flex-shrink-0">Contexto usado</span><div className="yarvis-input h-2 flex-1 overflow-hidden rounded-full border"><div className={`h-full rounded-full transition-all duration-300 ${contextPercent > 85 ? "bg-red-500" : contextPercent > 65 ? "bg-amber-500" : "bg-emerald-500"}`} style={{ width: `${contextPercent}%` }} /></div><span className="tabular-nums">{contextPercent}%</span></div>}
            {error && <div className="mb-3 flex items-center gap-2 rounded-xl border border-red-500/30 bg-red-500/10 px-4 py-3 text-xs font-bold text-red-500">{error}<button onClick={() => setError("")} className="ml-auto text-lg">×</button></div>}
            <div className="yarvis-panel yarvis-border yarvis-shadow flex items-end gap-3 rounded-2xl border px-4 py-3 transition-all focus-within:border-current">
              <textarea ref={inputRef} value={input} onChange={(event) => { setInput(event.target.value); if (error) setError(""); }} onKeyDown={handleKeyDown} placeholder="Pregúntale a Y.A.R.V.I.S…" rows={1} className="yarvis-text flex-1 resize-none bg-transparent text-[15px] font-bold leading-relaxed outline-none placeholder:opacity-50" />
              <button onClick={() => isLoading ? stopRef.current?.() : handleSend()} disabled={!isLoading && !input.trim()} title={isLoading ? "Detener" : "Enviar"} className={`flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-xl transition-all disabled:cursor-not-allowed disabled:opacity-30 ${isLoading ? "bg-red-600 text-white" : "yarvis-primary"}`}>
                {isLoading ? <svg xmlns="http://www.w3.org/2000/svg" width="17" height="17" viewBox="0 0 24 24" fill="currentColor"><rect x="5" y="5" width="14" height="14" rx="2" /></svg> : <svg xmlns="http://www.w3.org/2000/svg" width="17" height="17" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5"><line x1="22" y1="2" x2="11" y2="13" /><polygon points="22 2 15 22 11 13 2 9 22 2" /></svg>}
              </button>
            </div>
            <div className="mt-2 flex items-center justify-between lg:hidden"><button onClick={() => setShowHistory((value) => !value)} className="yarvis-muted text-[10px] font-black uppercase tracking-widest">{showHistory ? "Ocultar historial" : "Mostrar historial"}</button><span className="yarvis-faint text-[10px] font-bold">Enter para enviar · Shift + Enter para salto</span></div>
          </div>
        </div>
      </section>
    </div>
  );
};

export default ChatWidget;
