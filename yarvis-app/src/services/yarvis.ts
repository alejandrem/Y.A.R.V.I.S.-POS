// ═══════════════════════════════════════════════════════════════════════════
// SERVICIO DE YARVIS (IA) — API keys, modelo local y control del stream.
// Los componentes importan estas funciones en vez de `invoke` crudo.
// ═══════════════════════════════════════════════════════════════════════════

import { invokeTauri } from "./tauri";

export interface ModelStatus {
  models: Record<string, boolean>;
  ram_libre_gb: number;
  local_model_name?: string;
  local_model_path?: string;
}

export interface CloudModel {
  id: string;
  name: string;
  context_window?: number;
}

export const leerApiKeys = () =>
  invokeTauri<Record<string, string>>("leer_api_keys");

export const guardarApiKeys = (keys: Record<string, string>) =>
  invokeTauri("guardar_api_keys", { keys });

export const setLocalModelPath = (path: string) =>
  invokeTauri<{ name: string; path: string }>("set_local_model_path", { path });

export const cargarModeloLocal = () =>
  invokeTauri<ModelStatus>("load_chat_model", { model: "1.7B" });

export const stopChatStream = () =>
  invokeTauri("stop_chat_stream");

export const obtenerModelStatus = () =>
  invokeTauri<ModelStatus>("get_model_status");

export const obtenerCloudModels = (provider: string, apiKey: string) =>
  invokeTauri<{ models: CloudModel[] }>("get_cloud_models", { provider, apiKey });
