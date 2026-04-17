import { describe, expect, it } from "vitest";

import {
  createDerivedTagTerminalTwoPaneState,
  getDerivedTagTerminalTwoPaneLayoutMode,
  reduceDerivedTagTerminalTwoPaneState,
} from "../../src/tui/two-pane-state.js";

describe("two pane state", () => {
  it("forces split layout when focus returns to the list pane", () => {
    const detailFocused = {
      ...createDerivedTagTerminalTwoPaneState(),
      activePane: "detail" as const,
      layoutMode: "detail-only" as const,
    };

    const listFocused = reduceDerivedTagTerminalTwoPaneState(detailFocused, { type: "toggle_focus" });

    expect(listFocused.activePane).toBe("list");
    expect(getDerivedTagTerminalTwoPaneLayoutMode(listFocused)).toBe("split");
  });

  it("clamps detail scroll within the current maximum", () => {
    const state = {
      ...createDerivedTagTerminalTwoPaneState(),
      activePane: "detail" as const,
      detailScroll: 2,
    };

    const moved = reduceDerivedTagTerminalTwoPaneState(state, {
      type: "move_detail",
      delta: 10,
      maxDetailScroll: 5,
    });

    expect(moved.detailScroll).toBe(5);
  });
});
