// Zona central de mensajes del chat.
// Muestra las burbujas de usuario/asistente con markdown, el streaming en vivo del modelo,
// el bloque "pensando", el estado vacío con sugerencias rotativas y autoscroll al fondo.
import { useEffect, useRef, useState } from "react";
import Markdown from "react-markdown";
import remarkGfm from "remark-gfm";
import type { Message } from "../ChatWidget";

interface ChatMessagesProps {
  messages: Message[];
  isStreaming: boolean;
  streamingText: string;
  streamingModel: string;
  thinkingText: string;
  expandedThinking: Set<number>;
  onToggleThinking: (index: number) => void;
  suggestions: string[];
  modelLoadingLabel: string | null;
  currentSelectionLabel: string;
}

function modelDotClass(model: string): string {
  if (model === "1.7B" || model.toLowerCase().includes("qwen") || model.toLowerCase().includes("gguf")) {
    return "bg-emerald-500";
  }
  return "bg-sky-500";
}

const ChatMessages = ({
  messages,
  isStreaming,
  streamingText,
  streamingModel,
  thinkingText,
  expandedThinking,
  onToggleThinking,
  suggestions,
  modelLoadingLabel,
  currentSelectionLabel,
}: ChatMessagesProps) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const endRef = useRef<HTMLDivElement>(null);
  const stickToBottomRef = useRef(true);
  const [currentSuggestion, setCurrentSuggestion] = useState(0);

  useEffect(() => {
    if (!stickToBottomRef.current) return;
    endRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, streamingText, thinkingText]);

  useEffect(() => {
    if (!messages.length && !isStreaming) {
      const interval = window.setInterval(() => {
        setCurrentSuggestion((value) => (value + 1) % Math.max(suggestions.length, 1));
      }, 2600);
      return () => window.clearInterval(interval);
    }
  }, [messages.length, isStreaming, suggestions.length]);

  const handleScroll = () => {
    const element = containerRef.current;
    if (!element) return;
    stickToBottomRef.current = element.scrollHeight - element.scrollTop - element.clientHeight < 120;
  };

  return (
    <div ref={containerRef} onScroll={handleScroll} className="custom-scrollbar flex-1 overflow-y-auto px-5 py-6 sm:px-8">
      {!messages.length && !isStreaming && (
        <div className="flex h-full flex-col items-center justify-center animate-in fade-in duration-500">
          <div className="yarvis-panel-soft yarvis-border mb-6 flex h-16 w-16 items-center justify-center rounded-2xl border">
            <svg xmlns="http://www.w3.org/2000/svg" width="30" height="30" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" className="yarvis-muted"><path d="M12 8V4H8" /><rect width="16" height="12" x="4" y="8" rx="2" /><path d="M2 14h2M20 14h2M15 13v2M9 13v2" /></svg>
          </div>
          <p className="yarvis-muted max-w-md text-center text-base font-bold leading-relaxed">{suggestions[currentSuggestion] || "Pregúntale algo a Y.A.R.V.I.S."}</p>
          <p className="yarvis-faint mt-3 text-[10px] font-black uppercase tracking-[0.2em]">{currentSelectionLabel}</p>
        </div>
      )}

      <div className="mx-auto max-w-4xl space-y-5">
        {messages.map((message, index) => (
          <div key={`${message.timestamp}-${index}`} className={`flex animate-in fade-in slide-in-from-bottom-1 duration-300 ${message.role === "user" ? "justify-end" : "justify-start"}`}>
            <div className={`max-w-[88%] rounded-2xl border px-5 py-4 sm:max-w-[76%] ${message.role === "user" ? "yarvis-primary border-transparent" : "yarvis-panel-soft yarvis-border"}`}>
              {message.role === "assistant" ? <div className="chat-markdown yarvis-text text-[15px] font-bold leading-relaxed"><Markdown remarkPlugins={[remarkGfm]}>{message.content}</Markdown></div> : <p className="text-[15px] font-bold leading-relaxed whitespace-pre-wrap">{message.content}</p>}
              {message.role === "assistant" && message.model && <div className="yarvis-border yarvis-muted mt-3 flex items-center gap-2 border-t pt-2 text-[10px] font-black uppercase tracking-widest"><span className={`h-2 w-2 rounded-full ${modelDotClass(message.model)}`} />{message.model}</div>}
              {message.role === "assistant" && message.thinking && <div className="mt-3"><button onClick={() => onToggleThinking(index)} className="yarvis-muted text-[10px] font-black uppercase tracking-widest hover:text-current">{expandedThinking.has(index) ? "Ocultar razonamiento" : "Ver razonamiento"}</button>{expandedThinking.has(index) && <p className="yarvis-muted yarvis-panel mt-2 max-h-64 overflow-y-auto rounded-xl border border-dashed px-4 py-3 text-xs italic leading-relaxed">{message.thinking}</p>}</div>}
            </div>
          </div>
        ))}

        {thinkingText && <div className="flex justify-start"><div className="yarvis-panel-soft yarvis-border max-w-[88%] rounded-2xl border border-dashed px-5 py-4 sm:max-w-[76%]"><p className="yarvis-muted mb-2 text-[10px] font-black uppercase tracking-widest">{streamingModel || "Modelo"} está pensando…</p><p className="yarvis-muted max-h-48 overflow-y-auto whitespace-pre-wrap text-xs italic leading-relaxed">{thinkingText}</p></div></div>}
        {isStreaming && streamingText && <div className="flex justify-start"><div className="yarvis-panel-soft yarvis-border max-w-[88%] rounded-2xl border px-5 py-4 sm:max-w-[76%]"><div className="chat-markdown yarvis-text text-[15px] font-bold leading-relaxed"><Markdown remarkPlugins={[remarkGfm]}>{streamingText}</Markdown><span className="ml-1 inline-block h-4 w-1 animate-pulse rounded-sm bg-current align-middle" /></div><p className="yarvis-muted mt-3 text-[10px] font-black uppercase tracking-widest">{streamingModel || "Generando"}</p></div></div>}
        {isStreaming && !streamingText && !thinkingText && <div className="yarvis-muted text-xs font-black uppercase tracking-widest animate-pulse">{modelLoadingLabel ? `Cargando ${modelLoadingLabel}…` : `Conectando con ${currentSelectionLabel}…`}</div>}
        <div ref={endRef} />
      </div>
    </div>
  );
};

export default ChatMessages;