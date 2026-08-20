// Barra lateral del historial de conversaciones.
// Lista los chats ordenados por actividad, incluye el botón "Nuevo chat" con morphing,
// renombrado inline y el menú de 3 puntitos (editar con lápiz / borrar con basurero).
import type { IconInput } from "morphicons/react";
import { MorphIcon } from "morphicons/react";
import type { ChatSession } from "../ChatWidget";

interface ChatSidebarProps {
  sessions: ChatSession[];
  activeChatId: string;
  isLoading: boolean;
  renamingId: string | null;
  renameValue: string;
  setRenameValue: (value: string) => void;
  onFinishRename: () => void;
  onCancelRename: () => void;
  menuForId: string | null;
  onToggleMenu: (id: string) => void;
  onStartRename: (session: ChatSession) => void;
  onDelete: (id: string) => void;
  newChatIcon: IconInput;
  onCreateChat: () => void;
  onSelectChat: (id: string) => void;
  modelLabel: string;
}

const ChatSidebar = ({
  sessions,
  activeChatId,
  isLoading,
  renamingId,
  renameValue,
  setRenameValue,
  onFinishRename,
  onCancelRename,
  menuForId,
  onToggleMenu,
  onStartRename,
  onDelete,
  newChatIcon,
  onCreateChat,
  onSelectChat,
  modelLabel,
}: ChatSidebarProps) => (
  <aside className="yarvis-panel-soft hidden w-64 flex-shrink-0 flex-col border-r lg:flex">
    <div className="flex items-center justify-between border-b yarvis-border px-5 py-5">
      <div>
        <p className="yarvis-faint text-[9px] font-black uppercase tracking-[0.22em]">Conversaciones</p>
        <p className="yarvis-text mt-1 text-xs font-black">Historial de Y.A.R.V.I.S.</p>
      </div>
      <button onClick={onCreateChat} title="Nuevo chat" className="yarvis-primary flex h-8 w-8 items-center justify-center rounded-xl transition-transform hover:scale-105">
        <MorphIcon icon={newChatIcon} size={15} strokeWidth={2.5} spring="smooth" />
      </button>
    </div>
    <div className="custom-scrollbar flex-1 space-y-1 overflow-y-auto p-3">
      {sessions.map((session) => (
        <div key={session.id} data-session-id={session.id} className={`group relative rounded-xl border p-2 pr-8 transition-all ${session.id === activeChatId ? "yarvis-panel yarvis-border" : "border-transparent yarvis-hover-panel"}`}>
          {renamingId === session.id ? (
            <input autoFocus value={renameValue} onChange={(event) => setRenameValue(event.target.value)} onBlur={onFinishRename} onKeyDown={(event) => { if (event.key === "Enter") onFinishRename(); if (event.key === "Escape") onCancelRename(); }} className="yarvis-input w-full rounded-lg border px-2 py-1 text-[11px] font-bold outline-none" />
          ) : (
            <button disabled={isLoading} onClick={() => onSelectChat(session.id)} className="w-full text-left">
              <p className="yarvis-text truncate text-[11px] font-black">{session.title}</p>
              <p className="yarvis-faint mt-1 text-[9px] font-bold">{session.messages.length} mensajes</p>
            </button>
          )}
          {renamingId !== session.id && (
            <button
              data-open={menuForId === session.id ? "" : undefined}
              onClick={(event) => { event.stopPropagation(); onToggleMenu(session.id); }}
              title="Opciones del chat"
              className={`session-menu-trigger yarvis-muted absolute right-1.5 top-2 items-center justify-center rounded-lg p-1 transition-all hover:bg-black/5 dark:hover:bg-white/10 ${menuForId === session.id ? "flex opacity-100" : "hidden opacity-0 group-hover:flex group-hover:opacity-100"}`}
            >
              <svg xmlns="http://www.w3.org/2000/svg" width="17" height="17" viewBox="0 0 24 24" fill="currentColor"><circle cx="12" cy="5" r="2.3" /><circle cx="12" cy="12" r="2.3" /><circle cx="12" cy="19" r="2.3" /></svg>
            </button>
          )}
          {menuForId === session.id && (
            <div className="yarvis-panel yarvis-border yarvis-shadow absolute right-1.5 top-9 z-30 w-44 overflow-hidden rounded-xl border p-1 animate-in fade-in zoom-in-95 duration-150">
              <button onClick={() => onStartRename(session)} className="yarvis-hover-panel yarvis-text flex w-full items-center gap-2.5 rounded-lg px-3 py-2.5 text-left text-[10px] font-black uppercase tracking-widest">
                <svg xmlns="http://www.w3.org/2000/svg" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M12 20h9" /><path d="M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4Z" /></svg>
                Editar
              </button>
              <button onClick={() => onDelete(session.id)} className="flex w-full items-center gap-2.5 rounded-lg px-3 py-2.5 text-left text-[10px] font-black uppercase tracking-widest text-red-400 hover:bg-red-500/10">
                <svg xmlns="http://www.w3.org/2000/svg" width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M3 6h18" /><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6" /><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" /><line x1="10" x2="10" y1="11" y2="17" /><line x1="14" x2="14" y1="11" y2="17" /></svg>
                Borrar
              </button>
            </div>
          )}
        </div>
      ))}
    </div>
    <div className="border-t yarvis-border px-5 py-4">
      <p className="yarvis-faint truncate text-[9px] font-bold" title={modelLabel}>Modelo: {modelLabel}</p>
    </div>
  </aside>
);

export default ChatSidebar;