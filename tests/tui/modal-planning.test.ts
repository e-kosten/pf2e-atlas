import { describe, expect, it } from "vitest";

import { planTerminalModalStateLayout } from "../../src/tui/framework/modal-planning.js";
import type { TerminalModalState } from "../../src/tui/framework/types.js";

describe("planTerminalModalStateLayout", () => {
  it("assigns a bounded centered panel width for centered text prompts", () => {
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
    expect(result?.presentation).toBe("centered");
    expect(result?.panelWidth).toBeDefined();
    expect(result!.panelWidth).toBeLessThan(120);
    expect(result!.bodyHeight).toBeGreaterThan(0);
  });
});
