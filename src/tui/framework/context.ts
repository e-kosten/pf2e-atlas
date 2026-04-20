import React from "react";

import type { DerivedTagTerminalContextValue } from "./types.js";

export const DerivedTagTerminalContext = React.createContext<DerivedTagTerminalContextValue | null>(null);

export function ensureTerminalContext(): DerivedTagTerminalContextValue {
  const context = React.useContext(DerivedTagTerminalContext);
  if (!context) {
    throw new Error("DerivedTagTerminalContext is not available.");
  }
  return context;
}

export function useDerivedTagTerminalApp(): DerivedTagTerminalContextValue {
  return ensureTerminalContext();
}

export function useDerivedTagTerminalSize(): { width: number; height: number } {
  const terminal = ensureTerminalContext();
  return {
    width: terminal.getTerminalWidth(),
    height: terminal.getTerminalHeight(),
  };
}
