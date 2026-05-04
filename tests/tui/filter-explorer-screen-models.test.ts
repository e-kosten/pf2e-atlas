import { describe, expect, it } from "vitest";

import { createDerivedTagTerminalActionTargetState } from "../../src/tui/action-target.js";
import { formatTerminalInteractionFooter } from "../../src/tui/interaction-bindings.js";
import {
  buildFilterExplorerHelpLines,
  getFilterExplorerInteractionActions,
} from "../../src/tui/filter-explorer/screen-models.js";
import type {
  FilterExplorerBrowserContext,
  FilterExplorerControllerContext,
  FilterExplorerModel,
} from "../../src/tui/filter-explorer/types.js";

function createModel(): FilterExplorerModel {
  return {
    id: "search-semantics",
    label: "Search Semantics",
    description: "Search semantics ontology",
    rootNodes: [],
  };
}

function createBrowserContext(overrides: Partial<FilterExplorerBrowserContext> = {}): FilterExplorerBrowserContext {
  const browserState = {
    depth: 0,
    selectedNodeIds: [],
    filter: "",
    detailScroll: 0,
  };

  return {
    state: {
      activePane: "list",
      browserState,
      layoutMode: "split",
      searchInput: "",
      searchMode: false,
    },
    effectiveState: browserState,
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
    ...overrides,
  };
}

function createControllerContext(overrides: {
  browser?: Partial<FilterExplorerBrowserContext>;
} = {}): FilterExplorerControllerContext {
  const browser = createBrowserContext(overrides.browser);
  return {
    model: createModel(),
    host: {
      describeNode: () => undefined,
    },
    mode: {
      kind: "inspect-and-open",
    },
    screenTitle: "Search Semantics",
    browser,
    draft: {
      discreteClauses: [],
      scalarClauses: {},
    },
    discreteClauses: [],
    actionEntries: [],
    actionTargetState: createDerivedTagTerminalActionTargetState(),
  } as FilterExplorerControllerContext;
}

describe("filter explorer screen models", () => {
  it("uses viewport navigation bindings in detail contexts", () => {
    const controller = createControllerContext({
      browser: {
        state: {
          activePane: "detail",
          browserState: { depth: 0, selectedNodeIds: [], filter: "", detailScroll: 0 },
          layoutMode: "split",
          searchInput: "",
          searchMode: false,
        },
      },
    });

    const footer = formatTerminalInteractionFooter(getFilterExplorerInteractionActions(controller));
    const helpLines = buildFilterExplorerHelpLines(controller).map((line) => line.text);

    expect(footer).toContain("Ctrl-Y/E scroll");
    expect(footer).toContain("Home/End edge");
    expect(footer).not.toContain("↑/↓ select");
    expect(helpLines.some((line) => line.includes("Ctrl-Y / Ctrl-E"))).toBe(true);
    expect(helpLines.some((line) => line.includes("Home / End"))).toBe(true);
  });

  it("uses viewport navigation bindings in detail-only layouts", () => {
    const controller = createControllerContext({
      browser: {
        state: {
          activePane: "list",
          browserState: { depth: 0, selectedNodeIds: [], filter: "", detailScroll: 0 },
          layoutMode: "detail-only",
          searchInput: "",
          searchMode: false,
        },
        layoutMode: "detail-only",
      },
    });

    const footer = formatTerminalInteractionFooter(getFilterExplorerInteractionActions(controller));
    const helpLines = buildFilterExplorerHelpLines(controller).map((line) => line.text);

    expect(footer).toContain("Ctrl-Y/E scroll");
    expect(footer).toContain("Home/End edge");
    expect(helpLines.some((line) => line.includes("Ctrl-Y / Ctrl-E"))).toBe(true);
  });

  it("keeps cursor-driven navigation bindings in list contexts", () => {
    const controller = createControllerContext();

    const footer = formatTerminalInteractionFooter(getFilterExplorerInteractionActions(controller));
    const helpLines = buildFilterExplorerHelpLines(controller).map((line) => line.text);

    expect(footer).toContain("↑/↓ move");
    expect(footer).not.toContain("Ctrl-Y/E scroll");
    expect(helpLines.some((line) => line.includes("↑ / ↓ or j / k"))).toBe(true);
  });
});
