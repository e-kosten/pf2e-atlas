import { describe, expect, it } from "vitest";

import type { Pf2eTerminalSearchQuery } from "../../src/tui/search/service.js";
import { createInitialSearchScreenState, searchScreenReducer } from "../../src/tui/search-screen/state.js";

const QUERY: Pf2eTerminalSearchQuery = {
  mode: "browse",
  limit: 20,
};

describe("search screen state", () => {
  it("keeps the selected result stable while detail-pane navigation changes only the viewport", () => {
    const state = {
      ...createInitialSearchScreenState(QUERY, { layout: "results" }),
      activePane: "detail" as const,
      resultSelectedIndex: 2,
      detailScroll: 3,
    };

    const moved = searchScreenReducer(state, {
      type: "move_detail",
      delta: 4,
      maxDetailScroll: 12,
    });
    expect(moved.activePane).toBe("detail");
    expect(moved.detailScroll).toBe(7);
    expect(moved.resultSelectedIndex).toBe(2);

    const bounded = searchScreenReducer(moved, {
      type: "detail_boundary",
      boundary: "end",
      maxDetailScroll: 12,
    });
    expect(bounded.activePane).toBe("detail");
    expect(bounded.detailScroll).toBe(12);
    expect(bounded.resultSelectedIndex).toBe(2);
  });
});
