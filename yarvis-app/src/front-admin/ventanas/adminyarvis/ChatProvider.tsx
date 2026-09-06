// Estado del chat que SOBREVIVE al cambio de pestaña.
//
// El problema: los dashboards desmontan cada módulo al cambiar de tab. Las
// sesiones se salvaban (localStorage), pero el stream en curso moría con el
// componente: sus listeners de Tauri se desmontaban y la respuesta que el
// backend seguía generando se perdía al llegar `chat-complete` sin nadie
// que la recibiera (misma clase de bug que el progreso del parseador).
//
// La solución: este provider vive ARRIBA del cambio de pestaña (montado en
// el dashboard) y es dueño de useChatSessions + useChatStream + la config
// de modelo vigente. Los listeners se registran por envío como siempre,
// pero ya nada los desmonta a media respuesta: al volver al tab, el texto
// en streaming o la respuesta completa están ahí. ChatWidget y PanelYarvis
// solo consumen el contexto.
//
// No hay DOM oculto ni doble montaje: una sola instancia por dashboard
// (admin y empleado tienen cada uno la suya, con sus propias sesiones).

import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState } from "react";
import type { ReactNode } from "react";
import { useChatSessions } from "./hooks/useChatSessions";
import { useChatStream } from "./hooks/useChatStream";
import type { ChatModelSelection } from "./ChatWidget";

type SessionsApi = ReturnType<typeof useChatSessions>;
type StreamApi = ReturnType<typeof useChatStream>;

export interface ChatStreamConfig {
  fallbackSelection: ChatModelSelection;
  modelLoadingLabel: string | null;
}

export interface ChatContextValue extends SessionsApi {
  stream: StreamApi;
  /** Actualiza la config de envío (la llama PanelYarvis cuando cambia el modelo). */
  syncConfig: (patch: Partial<ChatStreamConfig>) => void;
  /** Dispara el "Limpiar chat" (vacía el chat abierto vía trigger). */
  clearChat: () => void;
}

const DEFAULT_CONFIG: ChatStreamConfig = {
  fallbackSelection: { provider: "", apiKey: "", model: "1.7B", label: "Modelo local", contextWindow: 4096 },
  modelLoadingLabel: null,
};

const ChatContext = createContext<ChatContextValue | null>(null);

interface ChatProviderProps {
  role: "admin" | "empleado";
  userId: string;
  children: ReactNode;
}

export const ChatProvider = ({ role, userId, children }: ChatProviderProps) => {
  const sessions = useChatSessions(userId);
  const { activeSession, messages, commitSession, activeChatId, updateActiveSession } = sessions;

  const [config, setConfig] = useState<ChatStreamConfig>(DEFAULT_CONFIG);
  const [clearTrigger, setClearTrigger] = useState(0);

  const stream = useChatStream({
    role,
    activeSession,
    messages,
    fallbackSelection: config.fallbackSelection,
    modelLoadingLabel: config.modelLoadingLabel,
    clearTrigger,
    commitSession,
  });

  // Un cambio explícito desde el selector del encabezado actualiza solo el chat abierto.
  const initializedSelectionRef = useRef(false);
  const selectionKey = `${config.fallbackSelection.provider}:${config.fallbackSelection.model}:${config.fallbackSelection.label}`;
  useEffect(() => {
    if (!initializedSelectionRef.current) {
      initializedSelectionRef.current = true;
      return;
    }
    updateActiveSession({ modelSelection: config.fallbackSelection });
    // La clave representa una elección manual; no dependemos del objeto mutable.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectionKey]);

  // Al cambiar de chat se reencuadra el contexto mostrado.
  const currentSelection = stream.currentSelection;
  useEffect(() => {
    stream.resetContext(currentSelection.contextWindow || (currentSelection.provider ? 131072 : 4096));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [activeChatId, currentSelection.contextWindow, currentSelection.model]);

  // El trigger de "Limpiar chat" vacía los mensajes del chat abierto.
  useEffect(() => {
    if (clearTrigger > 0) {
      updateActiveSession({ messages: [], modelSelection: stream.currentSelection });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [clearTrigger]);

  const syncConfig = useCallback((patch: Partial<ChatStreamConfig>) => {
    setConfig((prev) => ({ ...prev, ...patch }));
  }, []);

  const clearChat = useCallback(() => {
    setClearTrigger((value) => value + 1);
  }, []);

  const value = useMemo<ChatContextValue>(
    () => ({ ...sessions, stream, syncConfig, clearChat }),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [sessions, stream, syncConfig, clearChat],
  );
  return <ChatContext.Provider value={value}>{children}</ChatContext.Provider>;
};

export const useChat = (): ChatContextValue => {
  const ctx = useContext(ChatContext);
  if (!ctx) throw new Error("useChat fuera de ChatProvider");
  return ctx;
};
