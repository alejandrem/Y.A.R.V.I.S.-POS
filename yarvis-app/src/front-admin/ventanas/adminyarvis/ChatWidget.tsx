// Orquestador del widget de chat de Y.A.R.V.I.S.
// Compone los hooks de sesiones y streaming con la UI (sidebar, mensajes e input),
// define los tipos compartidos (Message, ChatSession, selección de modelo) y resuelve
// los efectos transversales: cambio de modelo, switch de chat, acción "Limpiar chat".
import { useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import ChatSidebar from "./components/ChatSidebar";
import ChatMessages from "./components/ChatMessages";
import ChatInput from "./components/ChatInput";
import { useChatSessions } from "./hooks/useChatSessions";
import { useChatStream } from "./hooks/useChatStream";

export interface Message {
  role: "user" | "assistant";
  content: string;
  model?: string;
  thinking?: string;
  timestamp: number;
}

export interface ChatSession {
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

// Caché EN MEMORIA de las API keys, alimentada desde el backend
// (api_keys.json con permisos 0600). Las keys NUNCA residen en
// localStorage del webview: quedarían en texto plano y legibles por XSS.
let apiKeysCache: Record<string, string> = {};

/** Refresca el caché desde el disco (vía backend). Llamar al montar el panel. */
export async function refrescarApiKeysCache(): Promise<void> {
  try {
    apiKeysCache = await invoke<Record<string, string>>("leer_api_keys");
  } catch (e) {
    console.error("[YARVIS] no se pudo refrescar el caché de API keys:", e);
  }
}

/** Actualiza el caché tras guardar en el backend. */
export function setApiKeysCache(keys: Record<string, string>): void {
  apiKeysCache = keys ?? {};
}

export function getActiveCloud(): ActiveCloud {
  const empty: ActiveCloud = {
    provider: "",
    apiKey: "",
    model: "1.7B",
    label: "Modelo local",
    contextWindow: 4096,
  };

  try {
    const keys = apiKeysCache;
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

const ChatWidget = ({ role, userId, suggestions, modelState, modelSelection, clearTrigger }: ChatWidgetProps) => {
  const fallbackSelection = modelSelection || getActiveCloud();
  const localLoading = modelState.loadingModel;

  const {
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
  } = useChatSessions(userId);

  const stream = useChatStream({
    role,
    activeSession,
    messages,
    fallbackSelection,
    modelLoadingLabel: localLoading,
    clearTrigger,
    commitSession,
  });

  // Un cambio explícito desde el selector del encabezado actualiza solo el chat abierto.
  const initializedSelectionRef = useRef(false);
  const selectionKey = `${fallbackSelection.provider}:${fallbackSelection.model}:${fallbackSelection.label}`;
  useEffect(() => {
    if (!initializedSelectionRef.current) {
      initializedSelectionRef.current = true;
      return;
    }
    updateActiveSession({ modelSelection: fallbackSelection });
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

  const handleCreateChat = () => {
    if (stream.isLoading) return;
    createChat();
    stream.clearError();
  };

  const handleDeleteChat = (id: string) => {
    if (stream.isLoading) return;
    deleteChat(id);
  };

  return (
    <div className="yarvis-shell flex h-full min-h-0">
      {showHistory && (
        <ChatSidebar
          sessions={sessionsSorted}
          activeChatId={activeChatId}
          isLoading={stream.isLoading}
          renamingId={renamingId}
          renameValue={renameValue}
          setRenameValue={setRenameValue}
          onFinishRename={finishRename}
          onCancelRename={cancelRename}
          menuForId={menuForId}
          onToggleMenu={(id) => setMenuForId(menuForId === id ? null : id)}
          onStartRename={startRename}
          onDelete={handleDeleteChat}
          newChatIcon={newChatIcon}
          onCreateChat={handleCreateChat}
          onSelectChat={setActiveChatId}
          modelLabel={currentSelection.label}
        />
      )}

      <section className="flex min-w-0 flex-1 flex-col">
        <ChatMessages
          messages={messages}
          isStreaming={stream.isStreaming}
          streamingText={stream.streamingText}
          streamingModel={stream.streamingModel}
          thinkingText={stream.thinkingText}
          expandedThinking={stream.expandedThinking}
          onToggleThinking={stream.toggleThinking}
          suggestions={suggestions}
          modelLoadingLabel={stream.modelLoadingLabel}
          currentSelectionLabel={currentSelection.label}
        />
        <ChatInput
          input={stream.input}
          onInputChange={(value) => {
            stream.setInput(value);
            if (stream.error) stream.clearError();
          }}
          isLoading={stream.isLoading}
          onSend={stream.handleSend}
          onStop={stream.handleStop}
          error={stream.error}
          onErrorDismiss={stream.clearError}
          contextUsed={stream.contextUsed}
          contextPercent={stream.contextPercent}
          sendIcon={stream.sendIcon}
          showHistory={showHistory}
          onToggleHistory={() => setShowHistory((value) => !value)}
        />
      </section>
    </div>
  );
};

export default ChatWidget;