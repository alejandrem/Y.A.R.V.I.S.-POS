// Hook del historial de conversaciones.
// Gestiona el estado de sesiones en localStorage, el chat activo, el renombrado
// (inline), el menú de 3 puntitos (abrir/cerrar) y la persistencia de mensajes.
import { useEffect, useMemo, useState } from "react";
import { ICONO_CHECK, ICONO_MAS, ICONO_PAUSA } from "../../../../icons";
import type { ChatSession, Message } from "../ChatWidget";
import { useIconSequence } from "./useIconSequence";

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

/**
 * Estado y operaciones del historial de conversaciones:
 * sesiones, chat activo, renombrado y el menú de 3 puntitos.
 */
export function useChatSessions(userId: string) {
  const [sessions, setSessionsState] = useState<ChatSession[]>(() => loadSessions(userId));
  const [activeChatId, setActiveChatId] = useState("");
  const [renamingId, setRenamingId] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState("");
  const [menuForId, setMenuForId] = useState<string | null>(null);
  const [showHistory, setShowHistory] = useState(true);

  const { icon: newChatIcon, play: playNewChat } = useIconSequence(ICONO_MAS);

  // La persistencia vive en un efecto: los updaters de estado quedan puros.
  useEffect(() => {
    try {
      localStorage.setItem(sessionKey(userId), JSON.stringify(sessions));
    } catch { /* localStorage puede no estar disponible en webviews restringidos */ }
  }, [sessions, userId]);

  const replaceSessions = (next: ChatSession[]) => {
    setSessionsState(next);
  };

  const updateSessions = (updater: (prev: ChatSession[]) => ChatSession[]) => {
    setSessionsState((prev) => updater(prev));
  };

  const commitSession = (id: string, updater: (session: ChatSession) => ChatSession) => {
    updateSessions((prev) => prev.map((session) => (session.id === id ? updater(session) : session)));
  };

  const activeSession = useMemo(
    () => sessions.find((session) => session.id === activeChatId) || sessions[0],
    [sessions, activeChatId],
  );
  const messages = activeSession?.messages || [];

  useEffect(() => {
    if (!sessions.some((session) => session.id === activeChatId)) {
      setActiveChatId(sessions[0]?.id || "");
    }
  }, [sessions, activeChatId]);

  useEffect(() => {
    if (!menuForId) return;
    const close = (event: MouseEvent) => {
      const target = event.target as HTMLElement;
      if (target.closest(".session-menu-trigger")) return;
      if (target.closest(`[data-session-id="${menuForId}"]`)) return;
      setMenuForId(null);
    };
    document.addEventListener("mousedown", close);
    return () => document.removeEventListener("mousedown", close);
  }, [menuForId]);

  const createChat = () => {
    const chat = newSession();
    replaceSessions([chat, ...sessions]);
    setActiveChatId(chat.id);
    setMenuForId(null);
    playNewChat([
      { icon: ICONO_PAUSA, delay: 0 },
      { icon: ICONO_CHECK, delay: 180 },
      { icon: ICONO_MAS, delay: 550 },
    ]);
  };

  const deleteChat = (id: string) => {
    const remaining = sessions.filter((session) => session.id !== id);
    const next = remaining.length ? remaining : [newSession()];
    replaceSessions(next);
    if (id === activeSession?.id) setActiveChatId(next[0].id);
  };

  const startRename = (session: ChatSession) => {
    setMenuForId(null);
    setRenamingId(session.id);
    setRenameValue(session.title);
  };

  const finishRename = () => {
    if (!renamingId) return;
    const title = renameValue.trim() || "Nuevo chat";
    setMenuForId(null);
    replaceSessions(sessions.map((session) => session.id === renamingId ? { ...session, title } : session));
    setRenamingId(null);
  };

  const cancelRename = () => setRenamingId(null);

  const updateActiveSession = (patch: Partial<ChatSession>) => {
    if (!activeSession) return;
    commitSession(activeSession.id, (session) => ({ ...session, ...patch, updatedAt: Date.now() }));
  };

  const sessionsSorted = useMemo(
    () => [...sessions].sort((a, b) => b.updatedAt - a.updatedAt),
    [sessions],
  );

  return {
    sessions,
    sessionsSorted,
    activeSession,
    activeChatId,
    setActiveChatId,
    messages,
    renamingId,
    renameValue,
    setRenameValue,
    finishRename,
    cancelRename,
    menuForId,
    setMenuForId,
    newChatIcon,
    showHistory,
    setShowHistory,
    createChat,
    deleteChat,
    startRename,
    updateActiveSession,
    commitSession,
  };
}