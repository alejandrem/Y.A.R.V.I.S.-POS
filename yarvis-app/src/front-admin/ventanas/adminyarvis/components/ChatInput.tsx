// Zona de entrada del chat.
// Barra de contexto usado, errores, textarea con auto-expansión, botón enviar/detener
// con morphing, disclaimer de IA y accesos (historial + atajos) en pantallas chicas.
import { useEffect, useRef, type KeyboardEvent } from "react";
import { MorphIcon, type IconInput } from "morphicons/react";

interface ChatInputProps {
  input: string;
  onInputChange: (value: string) => void;
  isLoading: boolean;
  onSend: (text?: string) => void;
  onStop: () => void;
  error: string;
  onErrorDismiss: () => void;
  contextUsed: number;
  contextPercent: number;
  sendIcon: IconInput;
  showHistory: boolean;
  onToggleHistory: () => void;
}

const ChatInput = ({
  input,
  onInputChange,
  isLoading,
  onSend,
  onStop,
  error,
  onErrorDismiss,
  contextUsed,
  contextPercent,
  sendIcon,
  showHistory,
  onToggleHistory,
}: ChatInputProps) => {
  const inputRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    if (!inputRef.current) return;
    inputRef.current.style.height = "auto";
    inputRef.current.style.height = `${Math.min(inputRef.current.scrollHeight, 120)}px`;
  }, [input]);

  // Al montar oculto (KeepAlive: display:none) scrollHeight mide 0 y el textarea queda
  // de 0px. Cuando se hace visible, el ResizeObserver lo redimensiona a su altura real.
  useEffect(() => {
    const textarea = inputRef.current;
    if (!textarea) return;
    let raf = 0;
    const resize = () => {
      cancelAnimationFrame(raf);
      raf = requestAnimationFrame(() => {
        const height = Math.min(textarea.scrollHeight, 120);
        if (height <= 0) return;
        const next = `${height}px`;
        if (textarea.style.height !== next) {
          textarea.style.height = "auto";
          textarea.style.height = next;
        }
      });
    };
    const observer = new ResizeObserver(resize);
    observer.observe(textarea);
    return () => {
      cancelAnimationFrame(raf);
      observer.disconnect();
    };
  }, []);

  const handleKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>) => {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      onSend();
    }
  };

  return (
    <div className="flex-shrink-0 px-5 pb-5 pt-3 sm:px-8 sm:pb-8">
      <div className="mx-auto max-w-4xl">
        {contextUsed > 0 && <div className="yarvis-muted mb-3 flex items-center gap-3 text-[10px] font-black uppercase tracking-widest"><span className="flex-shrink-0">Contexto usado</span><div className="yarvis-input h-2 flex-1 overflow-hidden rounded-full border"><div className={`h-full rounded-full transition-all duration-300 ${contextPercent > 85 ? "bg-red-500" : contextPercent > 65 ? "bg-amber-500" : "bg-emerald-500"}`} style={{ width: `${contextPercent}%` }} /></div><span className="tabular-nums">{contextPercent}%</span></div>}
        {error && <div className="mb-3 flex items-center gap-2 rounded-xl border border-red-500/30 bg-red-500/10 px-4 py-3 text-xs font-bold text-red-500">{error}<button onClick={onErrorDismiss} className="ml-auto text-lg">×</button></div>}
        <div className="yarvis-panel yarvis-border yarvis-shadow flex items-end gap-3 rounded-2xl border px-4 py-3 transition-all focus-within:border-current">
          <textarea ref={inputRef} value={input} onChange={(event) => onInputChange(event.target.value)} onKeyDown={handleKeyDown} placeholder="Pregúntale a Y.A.R.V.I.S…" rows={1} className="yarvis-text flex-1 resize-none bg-transparent text-[15px] font-bold leading-relaxed outline-none placeholder:opacity-50" />
          <button onClick={() => isLoading ? onStop() : onSend()} disabled={!isLoading && !input.trim()} title={isLoading ? "Detener" : "Enviar"} className={`flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-xl transition-all disabled:cursor-not-allowed disabled:opacity-30 ${isLoading ? "bg-red-600 text-white" : "yarvis-primary"}`}>
            <MorphIcon icon={sendIcon} size={17} strokeWidth={2.5} spring="smooth" />
          </button>
        </div>
        <div className="mt-2.5 flex flex-col gap-1.5">
          <p className="yarvis-muted text-center text-[11px] font-bold">Y.A.R.V.I.S. es una IA y puede cometer errores. Por favor verifica los resultados.</p>
          <div className="flex items-center justify-between lg:hidden">
            <button onClick={onToggleHistory} className="yarvis-muted text-[10px] font-black uppercase tracking-widest">{showHistory ? "Ocultar historial" : "Mostrar historial"}</button>
            <span className="yarvis-faint text-[10px] font-bold">Enter para enviar · Shift + Enter para salto</span>
          </div>
        </div>
      </div>
    </div>
  );
};

export default ChatInput;