import { describe, expect, it } from "vitest";

import { formatTerminalInteractionFooter } from "../../src/tui/interaction-bindings.js";
import {
  resolveOntologyExplorerBackNavigation,
  type OntologyExplorerControllerContext,
} from "../../src/tui/ontology-explorer/controller.js";
import {
  buildOntologyBrowserHelpLines,
  getOntologyBrowserInteractionActions,
} from "../../src/tui/ontology-explorer/screen-models.js";

function createController(
  overrides: Partial<OntologyExplorerControllerContext> = {},
): OntologyExplorerControllerContext {
  return {
    state: {
      activePane: "list",
      browserState: {
        depth: 0,
        selectedNodeIds: [],
        filter: "",
        detailScroll: 0,
      },
      layoutMode: "split",
      searchInput: "",
      searchMode: false,
    },
    effectiveState: {
      depth: 0,
      selectedNodeIds: [],
      filter: "",
      detailScroll: 0,
    },
    selection: {
      ancestors: [],
      currentNodes: [],
      currentNode: undefined,
      currentParent: undefined,
    },
    currentNode: undefined,
    currentNodeHasChildren: false,
    selectedQuery: undefined,
    breadcrumb: "Search Semantics",
    bodyHeight: 10,
    detailWidth: 40,
    detailLines: [],
    visibleDetailLines: [],
    detailTitle: "Detail",
    layoutMode: "split",
    maxDetailScroll: 0,
    detailJumpSize: 5,
    detailPageSize: 9,
    selectionJumpSize: 5,
    searchIndicator: "",
    ...overrides,
  };
}

describe("ontology explorer screen models", () => {
  it("uses return wording for ontology browser back actions at the root list", () => {
    const controller = createController();

    const actions = getOntologyBrowserInteractionActions(controller);
    const backAction = actions.find((action) => action.id === "back");

    expect(backAction?.label).toBe("return");
    expect(formatTerminalInteractionFooter(actions)).toContain("←/Esc return");

    const helpLines = buildOntologyBrowserHelpLines(controller);
    expect(helpLines.some((line) => line.text.includes("return from ontology browsing"))).toBe(true);
    expect(helpLines.some((line) => line.text.includes("move up a level"))).toBe(false);
  });

  it("keeps ontology browser detail back as pane navigation by default", () => {
    const controller = createController({
      state: {
        activePane: "detail",
        browserState: {
          depth: 2,
          selectedNodeIds: ["spell", "spell:security", "spell:alarm"],
          filter: "",
          detailScroll: 0,
        },
        layoutMode: "detail-only",
        searchInput: "",
        searchMode: false,
      },
      effectiveState: {
        depth: 2,
        selectedNodeIds: ["spell", "spell:security", "spell:alarm"],
        filter: "",
        detailScroll: 0,
      },
      layoutMode: "detail-only",
    });

    expect(resolveOntologyExplorerBackNavigation(controller)).toBe("leave_detail");
  });

  it("treats nested ontology detail back as a hierarchy pop when requested", () => {
    const controller = createController({
      state: {
        activePane: "detail",
        browserState: {
          depth: 1,
          selectedNodeIds: ["spell:field:derivedTags", "spell:derivedTags:coastal_setting"],
          filter: "",
          detailScroll: 0,
        },
        layoutMode: "detail-only",
        searchInput: "",
        searchMode: false,
      },
      effectiveState: {
        depth: 1,
        selectedNodeIds: ["spell:field:derivedTags", "spell:derivedTags:coastal_setting"],
        filter: "",
        detailScroll: 0,
      },
      layoutMode: "detail-only",
    });

    expect(resolveOntologyExplorerBackNavigation(controller, { nestedDetailBackAction: "pop_depth" })).toBe(
      "pop_depth",
    );
  });
});
