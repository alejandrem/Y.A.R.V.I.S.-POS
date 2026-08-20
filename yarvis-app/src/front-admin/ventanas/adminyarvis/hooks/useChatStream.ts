// Hook del envío y streaming del chat.
// Escucha los eventos de Tauri (chat-think/token/usage/complete/error), maneja los
// estados de carga/error, el contexto usado y el morphing del botón de enviar.
import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ICONO_CHECK, ICONO_ENVIAR, ICONO_PAUSA } from "../../../../icons";
import { useIconSequence } from "./useIconSequence";
import type { ChatModelSelection, ChatSession, Message } from "../ChatWidget";

const STREAM_TIMEOUT_MS = 180_000;

interface UseChatStreamArgs {
  role: "admin" | "empleado";
  activeSession: ChatSession | undefined;
  messages: Message[];
  fallbackSelection: ChatModelSelection;
  modelLoadingLabel: string | null;
  clearTrigger: number;
  commitSession: (id: string, updater: (session: ChatSession) => ChatSession) => void;
}

/**
 * Lógica de envío y streaming del chat: escuchas de eventos,
 * estados de carga/error y el morphing del botón de enviar.
 */
export function useChatStream({
  role,
  activeSession,
  messages,
  fallbackSelection,
  modelLoadingLabel,
  clearTrigger,
  commitSession,
}: UseChatStreamArgs) {
  const [input, setInput] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState("");
  const [streamingText, setStreamingText] = useState("");
  const [streamingModel, setStreamingModel] = useState("");
  const [isStreaming, setIsStreaming] = useState(false);
  const [thinkingText, setThinkingText] = useState("");
  const [expandedThinking, setExpandedThinking] = useState<Set<number>>(new Set());
  const [contextUsed, setContextUsed] = useState(0);
  const [contextMax, setContextMax] = useState(fallbackSelection.contextWindow || 4096);

  const currentSelection = activeSession?.modelSelection || fallbackSelection;

  const streamingTextRef = useRef("");
  const streamingModelRef = useRef("");
  const thinkingTextRef = useRef("");
  const usageRealRef = useRef(false);
  const listenersRef = useRef<(() => void)[]>([]);
  const stopRef = useRef<(() => void) | null>(null);
  const { icon: sendIcon, play: playSend, jump: jumpSend } = useIconSequence(ICONO_ENVIAR);

  const contextPercent = contextMax > 0 ? Math.min(100, Math.round((contextUsed / contextMax) * 100)) : 0;

  const resetContext = (max?: number) => {
    if (typeof max === "number") setContextMax(max);
    usageRealRef.current = false;
    setContextUsed(0);
    setExpandedThinking(new Set());
  };

  useEffect(() => {
    if (usageRealRef.current) return;
    const estimated = Math.round(messages.reduce((total, message) => total + message.content.length, 0) / 4);
    setContextUsed(estimated);
  }, [messages, streamingText]);

  useEffect(() => {
    if (clearTrigger > 0) {
      usageRealRef.current = false;
      setContextUsed(0);
      setExpandedThinking(new Set());
    }
  }, [clearTrigger]);

  useEffect(() => {
    return () => {
      listenersRef.current.forEach((fn) => fn());
      listenersRef.current = [];
    };
  }, []);

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
    const chatId = activeSession.id;

    const selection = currentSelection;
    const userMessage: Message = { role: "user", content: messageText, timestamp: Date.now() };
    const updatedMessages = [...messages, userMessage];
    const nextTitle = activeSession.title === "Nuevo chat" ? messageText.slice(0, 42) : activeSession.title;

    commitSession(chatId, (session) => ({
      ...session,
      title: nextTitle,
      messages: updatedMessages,
      modelSelection: selection,
      updatedAt: Date.now(),
    }));

    setInput("");
    setError("");
    setIsLoading(true);
    setIsStreaming(true);
    setStreamingText("");
    setThinkingText("");
    jumpSend(ICONO_PAUSA);
    streamingTextRef.current = "";
    streamingModelRef.current = "";
    thinkingTextRef.current = "";

    let settled = false;
    let timeoutId = 0;
    const cleanup = () => {
      listenersRef.current.forEach((fn) => fn());
      listenersRef.current = [];
    };
    const saveMessages = (nextMessages: Message[]) => {
      commitSession(chatId, (session) => ({
        ...session,
        title: nextTitle,
        messages: nextMessages,
        modelSelection: selection,
        updatedAt: Date.now(),
      }));
    };
    const settleFinish = () => {
      setIsLoading(false);
      setIsStreaming(false);
      setStreamingText("");
      setStreamingModel("");
      setThinkingText("");
      playSend([
        { icon: ICONO_CHECK, delay: 0 },
        { icon: ICONO_ENVIAR, delay: 1000 },
      ]);
    };
    const finish = (response: string, model: string) => {
      if (settled) return;
      settled = true;
      window.clearTimeout(timeoutId);
      cleanup();
      saveMessages([...updatedMessages, {
        role: "assistant", content: response, model, thinking: thinkingTextRef.current.trim() || undefined, timestamp: Date.now(),
      }]);
      settleFinish();
    };
    const fail = (reason: string) => {
      if (settled) return;
      settled = true;
      window.clearTimeout(timeoutId);
      cleanup();
      saveMessages(updatedMessages);
      setError(reason);
      setIsLoading(false);
      setIsStreaming(false);
      setStreamingText("");
      setStreamingModel("");
      setThinkingText("");
      jumpSend(ICONO_ENVIAR);
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
      setIsLoading(false);
      setIsStreaming(false);
      setStreamingText("");
      setStreamingModel("");
      setThinkingText("");
      playSend([
        { icon: ICONO_CHECK, delay: 0 },
        { icon: ICONO_ENVIAR, delay: 1000 },
      ]);
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

  const handleStop = () => stopRef.current?.();

  return {
    input,
    setInput,
    error,
    clearError: () => setError(""),
    isLoading,
    isStreaming,
    streamingText,
    streamingModel,
    thinkingText,
    expandedThinking,
    toggleThinking,
    contextUsed,
    contextPercent,
    resetContext,
    currentSelection,
    modelLoadingLabel,
    sendIcon,
    handleSend,
    handleStop,
  };
}