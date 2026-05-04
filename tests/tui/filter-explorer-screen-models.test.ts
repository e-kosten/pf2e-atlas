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
    pageDocument: null,
    pageInteractionState: { mode: { kind: "section" } },
    focusedPageSection: null,
    selectedPageTarget: null,
    detailInteractionState: { kind: "none" },
    detailTargetActionId: null,
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

  it("shows section-first page bindings in detail contexts with a page document", () => {
    const controller = createControllerContext({
      browser: {
        state: {
          activePane: "detail",
          browserState: { depth: 0, selectedNodeIds: [], filter: "", detailScroll: 0 },
          layoutMode: "split",
          searchInput: "",
          searchMode: false,
        },
        detailInteractionState: { kind: "section", canEnterTargets: true },
      },
    });

    const footer = formatTerminalInteractionFooter(getFilterExplorerInteractionActions(controller));
    const helpLines = buildFilterExplorerHelpLines(controller).map((line) => line.text);

    expect(footer).toContain("↑/↓ section");
    expect(footer).toContain("Enter/→ targets");
    expect(footer).toContain("Ctrl-Y/E scroll");
    expect(helpLines.some((line) => line.includes("move through sections in the preview"))).toBe(true);
    expect(helpLines.some((line) => line.includes("enter link targets inside the active section"))).toBe(true);
  });

  it("shows target-mode bindings when a page target is focused", () => {
    const controller = createControllerContext({
      browser: {
        state: {
          activePane: "detail",
          browserState: { depth: 0, selectedNodeIds: [], filter: "", detailScroll: 0 },
          layoutMode: "split",
          searchInput: "",
          searchMode: false,
        },
        detailInteractionState: { kind: "target" },
        detailTargetActionId: "open",
      },
    });

    const footer = formatTerminalInteractionFooter(getFilterExplorerInteractionActions(controller));
    const helpLines = buildFilterExplorerHelpLines(controller).map((line) => line.text);

    expect(footer).toContain("↑/↓ target");
    expect(footer).toContain("Enter/→ open");
    expect(helpLines.some((line) => line.includes("move through targets in the active section"))).toBe(true);
    expect(helpLines.some((line) => line.includes("open the focused page target"))).toBe(true);
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

  it("uses shared page-section bindings for record detail pages", () => {
    const controller = createControllerContext({
      browser: {
        state: {
          activePane: "detail",
          browserState: { depth: 0, selectedNodeIds: [], filter: "", detailScroll: 0 },
          layoutMode: "split",
          searchInput: "",
          searchMode: false,
        },
        detailInteractionState: { kind: "section", canEnterTargets: true },
      },
    });

    const footer = formatTerminalInteractionFooter(getFilterExplorerInteractionActions(controller));
    const helpLines = buildFilterExplorerHelpLines(controller).map((line) => line.text);

    expect(footer).toContain("↑/↓ section");
    expect(footer).toContain("Ctrl-Y/E scroll");
    expect(helpLines.some((line) => line.includes("move through sections in the preview"))).toBe(true);
    expect(helpLines.some((line) => line.includes("enter link targets inside the active section"))).toBe(true);
  });

  it("updates back help while record detail target mode is active", () => {
    const controller = createControllerContext({
      browser: {
        state: {
          activePane: "detail",
          browserState: { depth: 0, selectedNodeIds: [], filter: "", detailScroll: 0 },
          layoutMode: "split",
          searchInput: "",
          searchMode: false,
        },
        detailInteractionState: { kind: "target" },
        detailTargetActionId: "open",
      },
    });

    const helpLines = buildFilterExplorerHelpLines(controller).map((line) => line.text);

    expect(helpLines.some((line) => line.includes("leave target mode and return to section navigation"))).toBe(true);
    expect(helpLines.some((line) => line.includes("open the focused page target"))).toBe(true);
  });
});
