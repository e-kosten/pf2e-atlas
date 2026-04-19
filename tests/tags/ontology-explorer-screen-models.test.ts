import { describe, expect, it } from "vitest";

import type { OntologyDomainModel } from "../../src/types.js";
import { formatTerminalInteractionFooter } from "../../src/tui/interaction-bindings.js";
import {
  resolveOntologyExplorerBackNavigation,
  type OntologyExplorerControllerContext,
} from "../../src/tui/ontology-explorer/controller.js";
import {
  buildFacetPickerHelpLines,
  buildFacetPickerScreenModel,
  buildOntologyBrowserHelpLines,
  getFacetPickerInteractionActions,
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

const model: OntologyDomainModel = {
  id: "searchSemantics",
  label: "Search Semantics",
  description: "Test ontology model",
  rootNodes: [],
};

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

  it("renames the root facet picker list to fields and updates root help copy", () => {
    const controller = createController();

    const actions = getFacetPickerInteractionActions(controller);
    const backAction = actions.find((action) => action.id === "back");

    expect(backAction?.label).toBe("return");
    expect(formatTerminalInteractionFooter(actions)).toContain("←/Esc return");

    const helpLines = buildFacetPickerHelpLines(controller);
    expect(helpLines.some((line) => line.text.includes("return to the query editor"))).toBe(true);
    expect(helpLines.some((line) => line.text.includes("switch focus between query fields and detail"))).toBe(true);

    const screen = buildFacetPickerScreenModel({
      model,
      controller,
      leftLines: [],
      focusedPolicyLabel: "off",
    });

    expect(screen.kind).toBe("two-pane");
    if (screen.kind !== "two-pane") {
      throw new Error("Expected a two-pane facet picker screen.");
    }
    expect(screen.props.left.title).toBe("[QUERY FIELDS]");
  });

  it("uses field-list wording instead of values when backing out of root detail", () => {
    const controller = createController({
      state: {
        activePane: "detail",
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
    });

    const actions = getFacetPickerInteractionActions(controller);
    const backAction = actions.find((action) => action.id === "back");

    expect(backAction?.label).toBeUndefined();
    expect(formatTerminalInteractionFooter(actions)).not.toContain("values");

    const helpLines = buildFacetPickerHelpLines(controller);
    expect(helpLines.some((line) => line.text.includes("return to the query field list"))).toBe(true);
    expect(helpLines.some((line) => line.text.includes("return to the value list"))).toBe(false);
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

  it("treats nested picker detail back as a hierarchy pop", () => {
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

    const helpLines = buildFacetPickerHelpLines(controller);
    expect(helpLines.some((line) => line.text.includes("return to the previous level"))).toBe(true);
    expect(helpLines.some((line) => line.text.includes("return to the value list"))).toBe(false);
  });
});
