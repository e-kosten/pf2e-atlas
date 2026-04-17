import React from "react";

import { isBackOrExitKey } from "./keymap.js";
import { TerminalTextScreen, getNormalizedKeyName, useDerivedTagTerminalInput } from "./terminal-ui.js";

export function SearchScreen({
  onBack,
}: {
  onBack: () => void;
}): React.JSX.Element {
  useDerivedTagTerminalInput((input, key) => {
    const normalized = getNormalizedKeyName(input, key);
    if (isBackOrExitKey(normalized)) {
      onBack();
    }
  });

  return (
    <TerminalTextScreen
      title="Search"
      body={[
        { text: "This area is reserved for the future first-class TUI search surface.", tone: "section" },
        { text: "" },
        { text: "Planned capabilities:" },
        { text: "Exact name lookup, category-aware hard filters, deterministic listing, and ranked/semantic search over the same indexed PF2E data surfaced by the MCP server.", indent: 2 },
        { text: "" },
        { text: "Entity pages in Ontology Search are the groundwork for this future surface.", tone: "dim" },
      ]}
      footer={[{ text: "q or Backspace return to top level", tone: "dim" }]}
    />
  );
}
