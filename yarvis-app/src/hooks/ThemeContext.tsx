import { createContext, useContext, ReactNode } from "react";
import { useTheme, Theme } from "../hooks/useTheme";

interface ThemeContextValue {
  theme: Theme;
  setTheme: (t: Theme) => void;
  resolvedTheme: "claro" | "oscuro";
}

const ThemeContext = createContext<ThemeContextValue>({
  theme: "sistema",
  setTheme: () => {},
  resolvedTheme: "claro",
});

export function ThemeProvider({ children }: { children: ReactNode }) {
  const { theme, setTheme, resolvedTheme } = useTheme();

  return (
    <ThemeContext.Provider value={{ theme, setTheme, resolvedTheme }}>
      {children}
    </ThemeContext.Provider>
  );
}

export function useThemeContext() {
  return useContext(ThemeContext);
}
