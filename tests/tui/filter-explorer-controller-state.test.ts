import { describe, expect, it } from "vitest";

import { createFilterExplorerBrowserUiState } from "../../src/tui/filter-explorer/browser.js";
import {
  createFilterExplorerBrowserSnapshot,
  filterExplorerReducer,
  resolveFilterExplorerBackNavigation,
} from "../../src/tui/filter-explorer/controller-state.js";
import type {
  FilterExplorerBrowserContext,
  FilterExplorerModel,
} from "../../src/tui/filter-explorer/types.js";

function createModel(): FilterExplorerModel {
  return {
    id: "search-semantics",
    label: "Search Semantics",
    description: "Search semantics ontology",
    rootNodes: [
      {
        id: "spells",
        kind: "category",
        label: "Spells",
        filterText: "spells",
        detailLines: [{ text: "Spells" }],
        children: [
          {
            id: "illusion",
            kind: "value",
            label: "Illusion",
            filterText: "illusion",
            detailLines: [{ text: "Illusion" }],
          },
        ],
      },
      {
        id: "items",
        kind: "category",
        label: "Items",
        filterText: "items",
        detailLines: [{ text: "Items" }],
      },
    ],
  };
}

function createBrowserContext(
  overrides: Partial<FilterExplorerBrowserContext> = {},
): FilterExplorerBrowserContext {
  const state = {
    activePane: "list" as const,
    browserState: { depth: 0, selectedNodeIds: [], filter: "", detailScroll: 0 },
    layoutMode: "split" as const,
    searchInput: "",
    searchMode: false,
  };

  return {
    state,
    effectiveState: state.browserState,
    selection: { ancestors: [], currentNodes: [], currentNode: undefined, currentParent: undefined },
    currentNode: undefined,
    currentNodeHasChildren: false,
    breadcrumb: "",
    bodyHeight: 20,
    detailWidth: 40,
    detailLines: [],
    visibleDetailLines: [],
    detailTitle: "Detail",
    layoutMode: "split",
    maxDetailScroll: 0,
    detailJumpSize: 5,
    detailPageSize: 10,
    selectionJumpSize: 5,
    searchIndicator: "",
    pageDocument: null,
    pageInteractionState: { mode: { kind: "section" } },
    focusedPageSection: null,
    selectedPageTarget: null,
    detailInteractionState: { kind: "none" },
    detailTargetActionId: null,
    ...overrides,
  };
}

describe("filter explorer controller state", () => {
  it("updates filter state while typing and clears search mode on drill in", () => {
    const model = createModel();
    let state = createFilterExplorerBrowserUiState(model);

    state = filterExplorerReducer(model, state, { type: "set_search_mode", searchMode: true });
    state = filterExplorerReducer(model, state, { type: "append_search", character: "i" });
    state = filterExplorerReducer(model, state, { type: "append_search", character: "l" });

    expect(state.searchMode).toBe(true);
    expect(state.searchInput).toBe("il");
    expect(state.browserState.filter).toBe("il");

    state = filterExplorerReducer(model, state, { type: "clear_search" });
    state = filterExplorerReducer(model, state, { type: "drill_in" });

    expect(state.searchMode).toBe(false);
    expect(state.searchInput).toBe("");
    expect(state.browserState.depth).toBe(1);
    expect(state.browserState.filter).toBe("");
  });

  it("resolves back navigation by pane and depth", () => {
    const detailContext = createBrowserContext({
      state: {
        activePane: "detail",
        browserState: { depth: 1, selectedNodeIds: ["spells"], filter: "", detailScroll: 0 },
        layoutMode: "split",
        searchInput: "",
        searchMode: false,
      },
      effectiveState: { depth: 1, selectedNodeIds: ["spells"], filter: "", detailScroll: 0 },
    });
    const nestedListContext = createBrowserContext({
      effectiveState: { depth: 1, selectedNodeIds: ["spells"], filter: "", detailScroll: 0 },
    });
    const rootListContext = createBrowserContext();

    expect(resolveFilterExplorerBackNavigation(detailContext)).toBe("leave_detail");
    expect(resolveFilterExplorerBackNavigation(nestedListContext)).toBe("pop_depth");
    expect(resolveFilterExplorerBackNavigation(rootListContext)).toBe("exit");
  });

  it("captures browser snapshots from effective state", () => {
    const context = createBrowserContext({
      state: {
        activePane: "detail",
        browserState: { depth: 0, selectedNodeIds: [], filter: "", detailScroll: 14 },
        layoutMode: "split",
        searchInput: "rare",
        searchMode: true,
      },
      effectiveState: { depth: 0, selectedNodeIds: [], filter: "rare", detailScroll: 6 },
      layoutMode: "single",
    });

    expect(createFilterExplorerBrowserSnapshot(context)).toEqual({
      activePane: "detail",
      browserState: { depth: 0, selectedNodeIds: [], filter: "rare", detailScroll: 6 },
      layoutMode: "single",
      searchInput: "rare",
      searchMode: true,
    });
  });
});
