import { useState, useEffect, useCallback } from "react";

export type Theme = "claro" | "oscuro" | "sistema";

const STORAGE_KEY = "yarvis-theme";

/** Resuelve qué tema aplicar basándose en la preferencia del sistema. */
function resolveSystemTheme(): "claro" | "oscuro" {
  if (typeof window === "undefined") return "claro";
  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "oscuro"
    : "claro";
}

/** Aplica la clase `dark` en <html> según el tema resuelto. */
function applyThemeToDOM(resolved: "claro" | "oscuro") {
  const root = document.documentElement;
  if (resolved === "oscuro") {
    root.classList.add("dark");
  } else {
    root.classList.remove("dark");
  }
}

/** Lee el tema guardado en localStorage (fallback: "sistema"). */
function getStoredTheme(): Theme {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored === "claro" || stored === "oscuro" || stored === "sistema") {
      return stored;
    }
  } catch {}
  return "sistema";
}

export function useTheme() {
  const [theme, setThemeState] = useState<Theme>(getStoredTheme);

  // Aplicar tema al montar y cuando cambie
  useEffect(() => {
    const resolved = theme === "sistema" ? resolveSystemTheme() : theme;
    applyThemeToDOM(resolved);
  }, [theme]);

  // Escuchar cambios del sistema cuando el modo es "sistema"
  useEffect(() => {
    if (theme !== "sistema") return;

    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = () => applyThemeToDOM(resolveSystemTheme());

    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, [theme]);

  const setTheme = useCallback((newTheme: Theme) => {
    setThemeState(newTheme);
    try {
      localStorage.setItem(STORAGE_KEY, newTheme);
    } catch {}
  }, []);

  /** El tema "real" que se está mostrando (no "sistema", sino claro u oscuro). */
  const resolvedTheme = theme === "sistema" ? resolveSystemTheme() : theme;

  return { theme, setTheme, resolvedTheme };
}
