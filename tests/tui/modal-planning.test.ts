import { describe, expect, it } from "vitest";

import { planTerminalModalStateLayout } from "../../src/tui/framework/modal-planning.js";
import type { TerminalModalState } from "../../src/tui/framework/types.js";

describe("planTerminalModalStateLayout", () => {
  it("assigns a bounded centered panel width for overlay text prompts", () => {
    const modal: TerminalModalState = {
      kind: "text",
      options: {
        title: "Search Text",
        prompt: "Enter search text for the current query.",
        defaultValue: "ghost ship captain",
        hint: "Example: ghost ship captain",
        presentation: "centered",
      },
      value: "ghost ship captain",
      resolve: () => undefined,
    };

    const result = planTerminalModalStateLayout(modal, 120, 32);

    expect(result).not.toBeNull();
    expect(result?.presentation).toBe("inline");
    expect(result?.centeredPromptBackground).toBe("overlay");
    expect(result?.panelWidth).toBeDefined();
    expect(result!.panelWidth).toBeLessThan(120);
    expect(result!.bodyHeight).toBeGreaterThan(0);
  });

  it("keeps overlay horizontal prompts at a stable size as the focused option changes", () => {
    const first: TerminalModalState = {
      kind: "select",
      options: {
        title: "Choose Search Mode",
        prompt: "",
        presentation: "centered",
        choiceLayout: "horizontal",
        filtering: false,
        entries: [
          { kind: "selected", value: "browse", label: "Browse", description: "Short detail." },
          {
            kind: "selected",
            value: "search",
            label: "Search",
            description: "Much longer detail text that should determine the stable centered panel size.",
          },
          { kind: "selected", value: "lookup", label: "Lookup", description: "Medium detail." },
        ],
        supportsCommands: false,
      },
      selectedIndex: 0,
      filterText: "",
      filterMode: false,
      resolve: () => undefined,
    };
    const second: TerminalModalState = { ...first, selectedIndex: 1 };

    const firstLayout = planTerminalModalStateLayout(first, 100, 24);
    const secondLayout = planTerminalModalStateLayout(second, 100, 24);

    expect(firstLayout?.presentation).toBe("inline");
    expect(firstLayout?.centeredPromptBackground).toBe("overlay");
    expect(secondLayout?.presentation).toBe("inline");
    expect(secondLayout?.centeredPromptBackground).toBe("overlay");
    expect(firstLayout?.panelWidth).toBe(secondLayout?.panelWidth);
    expect(firstLayout?.totalHeight).toBe(secondLayout?.totalHeight);
  });

  it("preserves blanked-background presentation for first-entry prompts", () => {
    const modal: TerminalModalState = {
      kind: "select",
      options: {
        title: "Choose Search Mode",
        prompt: "",
        presentation: "centered-screen",
        choiceLayout: "horizontal",
        filtering: false,
        entries: [
          { kind: "selected", value: "browse", label: "Browse", description: "Explore records." },
          { kind: "selected", value: "search", label: "Search", description: "Run ranked search." },
        ],
        supportsCommands: false,
      },
      selectedIndex: 0,
      filterText: "",
      filterMode: false,
      resolve: () => undefined,
    };

    const result = planTerminalModalStateLayout(modal, 100, 24);

    expect(result?.presentation).toBe("inline");
    expect(result?.centeredPromptBackground).toBe("blanked");
    expect(result?.panelWidth).toBeDefined();
    expect(result?.totalHeight).toBeGreaterThan(0);
  });
});
