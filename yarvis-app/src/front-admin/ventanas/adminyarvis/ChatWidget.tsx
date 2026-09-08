// Orquestador del widget de chat de Y.A.R.V.I.S.
// Compone el estado del ChatProvider (sesiones + streaming persistentes)
// con la UI (sidebar, mensajes e input) y define los tipos compartidos
// (Message, ChatSession, selección de modelo).
import { reportarError } from "../../../services/tauri";
import { leerApiKeys } from "../../../services/yarvis";
import ChatSidebar from "./components/ChatSidebar";
import ChatMessages from "./components/ChatMessages";
import ChatInput from "./components/ChatInput";
import { useChat } from "./ChatProvider";

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
  { id: "google", display: "Gemini", defaultModel: "gemini-3.6-flash" },
  { id: "opencode", display: "OpenCode", defaultModel: "mimo-v2.5-free" },
];

export type { CloudModel } from "../../../services/yarvis";

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
    apiKeysCache = await leerApiKeys();
  } catch (e) {
    reportarError("No se pudo refrescar el caché de API keys", e);
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

interface ChatWidgetProps {
  suggestions: string[];
}

const ChatWidget = ({ suggestions }: ChatWidgetProps) => {
  const {
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
    stream,
    sessionsSorted,
  } = useChat();

  const currentSelection = stream.currentSelection;

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