import { createContext, useContext, useState, useEffect, ReactNode } from "react";

interface ParserState {
  parsedItems: any[];
  fileContent: string;
  selectedPath: string;
  parserMode: "catalogo" | "entrenar IA" | "insertar";
  showColumnMapper: boolean;
  llmAnalysis: any | null;
  lastCatalogPath: string;
  lastCatalogItems: any[];
  catalogParsed: boolean;
  iaTrained: boolean;
  ticketsParsed: boolean;
  ticketsCount: number;
  ticketsGuardados: number;
}

interface ParserContextType extends ParserState {
  setParsedItems: (items: any[]) => void;
  setFileContent: (content: string) => void;
  setSelectedPath: (path: string) => void;
  setParserMode: (mode: "catalogo" | "entrenar IA" | "insertar") => void;
  setShowColumnMapper: (show: boolean) => void;
  setLlmAnalysis: (analysis: any | null) => void;
  setLastCatalogPath: (path: string) => void;
  setLastCatalogItems: (items: any[]) => void;
  setCatalogParsed: (parsed: boolean) => void;
  setIaTrained: (trained: boolean) => void;
  setTicketsParsed: (parsed: boolean) => void;
  setTicketsCount: (count: number | ((c: number) => number)) => void;
  setTicketsGuardados: (count: number | ((c: number) => number)) => void;
}

const STORAGE_KEY = "yarvis_parser_state";

const defaultState: ParserState = {
  parsedItems: [],
  fileContent: "",
  selectedPath: "",
  parserMode: "entrenar IA",
  showColumnMapper: false,
  llmAnalysis: null,
  lastCatalogPath: "",
  lastCatalogItems: [],
  catalogParsed: false,
  iaTrained: false,
  ticketsParsed: false,
  ticketsCount: 0,
  ticketsGuardados: 0,
};

function loadFromLocalStorage(): ParserState {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved) {
      const parsed = JSON.parse(saved);
      return { ...defaultState, ...parsed };
    }
  } catch (e) {
    console.error("Error loading parser state from localStorage:", e);
  }
  return defaultState;
}

function saveToLocalStorage(state: ParserState) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
  } catch (e) {
    console.error("Error saving parser state to localStorage:", e);
  }
}

const ParserContext = createContext<ParserContextType | null>(null);

export function ParserProvider({ children }: { children: ReactNode }) {
  const [state, setState] = useState<ParserState>(loadFromLocalStorage);

  useEffect(() => {
    saveToLocalStorage(state);
  }, [state]);

  const setParsedItems = (items: any[]) => setState(s => ({ ...s, parsedItems: items }));
  const setFileContent = (content: string) => setState(s => ({ ...s, fileContent: content }));
  const setSelectedPath = (path: string) => setState(s => ({ ...s, selectedPath: path }));
  const setParserMode = (mode: "catalogo" | "entrenar IA" | "insertar") => setState(s => ({ ...s, parserMode: mode }));
  const setShowColumnMapper = (show: boolean) => setState(s => ({ ...s, showColumnMapper: show }));
  const setLlmAnalysis = (analysis: any | null) => setState(s => ({ ...s, llmAnalysis: analysis }));
  const setLastCatalogPath = (path: string) => setState(s => ({ ...s, lastCatalogPath: path }));
  const setLastCatalogItems = (items: any[]) => setState(s => ({ ...s, lastCatalogItems: items }));
  const setCatalogParsed = (parsed: boolean) => setState(s => ({ ...s, catalogParsed: parsed }));
  const setIaTrained = (trained: boolean) => setState(s => ({ ...s, iaTrained: trained }));
  const setTicketsParsed = (parsed: boolean) => setState(s => ({ ...s, ticketsParsed: parsed }));
  const setTicketsCount = (count: number | ((c: number) => number)) => setState(s => ({
    ...s,
    ticketsCount: typeof count === "function" ? count(s.ticketsCount) : count,
  }));
  const setTicketsGuardados = (count: number | ((c: number) => number)) => setState(s => ({
    ...s,
    ticketsGuardados: typeof count === "function" ? count(s.ticketsGuardados) : count,
  }));
  return (
    <ParserContext.Provider value={{
      ...state,
      setParsedItems,
      setFileContent,
      setSelectedPath,
      setParserMode,
      setShowColumnMapper,
      setLlmAnalysis,
      setLastCatalogPath,
      setLastCatalogItems,
      setCatalogParsed,
      setIaTrained,
      setTicketsParsed,
      setTicketsCount,
      setTicketsGuardados,
    }}>
      {children}
    </ParserContext.Provider>
  );
}

export function useParserContext() {
  const context = useContext(ParserContext);
  if (!context) {
    throw new Error("useParserContext must be used within a ParserProvider");
  }
  return context;
}
