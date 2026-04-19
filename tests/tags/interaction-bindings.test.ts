import { describe, expect, it } from "vitest";

import {
  buildTerminalInteractionHelpLines,
  formatTerminalInteractionFooter,
  resolveTerminalInteractionAction,
} from "../../src/tui/interaction-bindings.js";
import { createDerivedTagTerminalInputEvent } from "../../src/tui/terminal-ui.js";

describe("terminal interaction bindings", () => {
  it("falls Escape through cancel, back, and quit based on declared actions", () => {
    const escape = createDerivedTagTerminalInputEvent("\u001b", {} as never);

    expect(resolveTerminalInteractionAction(escape, [{ id: "quit" }])?.id).toBe("quit");
    expect(resolveTerminalInteractionAction(escape, [{ id: "back" }, { id: "quit" }])?.id).toBe("back");
    expect(resolveTerminalInteractionAction(escape, [{ id: "cancel" }, { id: "back" }, { id: "quit" }])?.id).toBe(
      "cancel",
    );
  });

  it("keeps non-Escape back-navigation keys mapped to back", () => {
    const backspace = createDerivedTagTerminalInputEvent("\u007f", {} as never);

    expect(resolveTerminalInteractionAction(backspace, [{ id: "cancel" }, { id: "back" }, { id: "quit" }])?.id).toBe(
      "back",
    );
  });

  it("renders footer keys with the same Escape fallback policy", () => {
    expect(formatTerminalInteractionFooter([{ id: "quit", label: "quit" }])).toBe("Esc/q quit");
    expect(
      formatTerminalInteractionFooter([
        { id: "back", label: "back" },
        { id: "quit", label: "quit" },
      ]),
    ).toBe("←/Esc back  q quit");
    expect(
      formatTerminalInteractionFooter([
        { id: "cancel", label: "cancel" },
        { id: "back", label: "back" },
        { id: "quit", label: "quit" },
      ]),
    ).toBe("Esc cancel  ← back  q quit");
  });

  it("renders help text with the same Escape fallback policy", () => {
    expect(
      buildTerminalInteractionHelpLines([
        {
          title: "Actions",
          actions: [{ id: "quit", label: "quit" }],
        },
      ]),
    ).toEqual([
      { text: "Actions", tone: "section" },
      { text: "Escape / q: quit" },
    ]);

    expect(
      buildTerminalInteractionHelpLines([
        {
          title: "Actions",
          actions: [
            { id: "cancel", label: "cancel" },
            { id: "back", label: "back" },
            { id: "quit", label: "quit" },
          ],
        },
      ]),
    ).toEqual([
      { text: "Actions", tone: "section" },
      { text: "Escape: cancel" },
      { text: "\u2190 or h / Backspace: back" },
      { text: "q: quit" },
    ]);
  });
});
