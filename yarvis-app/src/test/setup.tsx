// ═══════════════════════════════════════════════════════════════════════════
// SETUP GLOBAL DE TESTS — Entorno jsdom preparado para el frontend de Tauri.
// Tarea única: mockear la capa nativa (@tauri-apps) para que ningún test
// toque el backend real, polyfills que recharts/jsdom esperan, y un mock
// ligero de morphicons. Cada test puede sobreescribir el invoke con
// mockInvoke.mockImplementation(...) según sus necesidades.
// ═══════════════════════════════════════════════════════════════════════════

import { vi, beforeAll } from "vitest";
import "@testing-library/jest-dom/vitest";

// ── Mock de invoke: por defecto resuelve undefined; cada test lo ajusta ────
export const mockInvoke = vi.fn().mockResolvedValue(undefined);

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
  convertFileSrc: (p: string) => p,
}));

// ── Mock de eventos Tauri (usado por useChatStream) ─────────────────────────
export const mockListen = vi.fn().mockResolvedValue(() => {});
vi.mock("@tauri-apps/api/event", () => ({
  listen: (...args: unknown[]) => mockListen(...args),
}));

// ── Mock ligero de morphicons: evita framer-motion en jsdom ────────────────
vi.mock("morphicons/react", () => ({
  MorphIcon: ({ icon }: { icon?: unknown }) => (
    <svg data-testid="morphicon" data-icon={typeof icon === "string" ? icon : "icon"} />
  ),
  IconInput: undefined,
}));

// ── Polyfills que recharts / libs animadas esperan en jsdom ────────────────
beforeAll(() => {
  // jsdom de vitest no expone localStorage por defecto
  if (typeof globalThis.localStorage === "undefined") {
    const store = new Map<string, string>();
    globalThis.localStorage = {
      getItem: (k: string) => store.get(k) ?? null,
      setItem: (k: string, v: string) => void store.set(k, String(v)),
      removeItem: (k: string) => void store.delete(k),
      clear: () => void store.clear(),
      key: (i: number) => [...store.keys()][i] ?? null,
      get length() { return store.size; },
    };
  }
  if (!window.matchMedia) {
    Object.defineProperty(window, "matchMedia", {
      writable: true,
      value: (query: string) => ({
        matches: false,
        media: query,
        onchange: null,
        addListener: () => {},
        removeListener: () => {},
        addEventListener: () => {},
        removeEventListener: () => {},
        dispatchEvent: () => false,
      }),
    });
  }
  if (!("ResizeObserver" in window)) {
    class ResizeObserverMock {
      observe() {}
      unobserve() {}
      disconnect() {}
    }
    (window as unknown as { ResizeObserver: unknown }).ResizeObserver = ResizeObserverMock;
  }
  if (!Element.prototype.scrollIntoView) {
    Element.prototype.scrollIntoView = () => {};
  }
});
