import { describe, expect, it } from "vitest";

import type { OntologyDomainModel } from "../../src/types.js";
import {
  buildOntologyBrowserBreadcrumb,
  buildOntologyBrowserListLines,
  createOntologyBrowserState,
  drillIntoOntologyBrowser,
  getOntologyBrowserSelection,
  moveOntologyBrowserDetailScroll,
  moveOntologyBrowserDetailScrollToBoundary,
  popOntologyBrowserDepth,
  setOntologyBrowserFilter,
} from "../../src/tui/ontology-explorer/ui.js";

function createTestOntologyModel(): OntologyDomainModel {
  return {
    id: "derivedTags",
    label: "Derived Tags",
    description: "Test ontology domain",
    rootNodes: [
      {
        id: "spell",
        kind: "category",
        label: "Spell",
        filterText: "spell",
        listLabel: "spell | 2 families",
        detailTitle: "Category Details",
        detailLines: [{ text: "Spell", tone: "section" }],
        childPresentation: {
          mode: "grouped",
          groupBy: "axis",
          render: "inline",
        },
        children: [
          {
            id: "spell:security",
            kind: "family",
            label: "security",
            filterText: "security utility alarm",
            listLabel: "security | 1 tag",
            detailTitle: "Family Details",
            detailLines: [{ text: "security", tone: "section" }],
            groupValues: { axis: "utility" },
            children: [
              {
                id: "spell:alarm",
                kind: "tag",
                label: "alarm",
                filterText: "alarm warning",
                listLabel: "alarm | 2 live records",
                detailTitle: "Tag Details",
                detailLines: [{ text: "alarm", tone: "section" }],
              },
            ],
          },
          {
            id: "spell:revelation",
            kind: "family",
            label: "revelation",
            filterText: "revelation information truth",
            listLabel: "revelation | 1 tag",
            detailTitle: "Family Details",
            detailLines: [{ text: "revelation", tone: "section" }],
            groupValues: { axis: "information" },
            children: [
              {
                id: "spell:truth_reveal",
                kind: "tag",
                label: "truth_reveal",
                filterText: "truth reveal honesty",
                listLabel: "truth_reveal | 1 live record",
                detailTitle: "Tag Details",
                detailLines: [{ text: "truth_reveal", tone: "section" }],
              },
            ],
          },
        ],
      },
      {
        id: "creature",
        kind: "category",
        label: "Creature",
        filterText: "creature",
        listLabel: "creature | 1 family",
        detailTitle: "Category Details",
        detailLines: [{ text: "Creature", tone: "section" }],
      },
    ],
  };
}

describe("ontology browser ui", () => {
  it("renders grouped children inline when the parent requests inline grouping", () => {
    const model = createTestOntologyModel();
    const state = drillIntoOntologyBrowser(model, createOntologyBrowserState(model));

    expect(buildOntologyBrowserListLines(model, state, 10)).toEqual([
      { text: "Utility", tone: "section", noWrap: true },
      { text: "security | 1 tag", tone: "selected", noWrap: true },
      { text: "" },
      { text: "Information", tone: "section", noWrap: true },
      { text: "revelation | 1 tag", tone: "default", noWrap: true },
    ]);
  });

  it("filters within the current depth and updates breadcrumb selection", () => {
    const model = createTestOntologyModel();
    const drilledState = drillIntoOntologyBrowser(model, createOntologyBrowserState(model));
    const filteredState = setOntologyBrowserFilter(model, drilledState, "revelation");
    const selection = getOntologyBrowserSelection(model, filteredState);

    expect(selection.currentNodes.map((node) => node.id)).toEqual(["spell:revelation"]);
    expect(selection.currentNode?.id).toBe("spell:revelation");
    expect(buildOntologyBrowserBreadcrumb(model, filteredState)).toBe("Derived Tags > Spell > revelation");
  });

  it("drills down, pops back up, and clamps detail scrolling", () => {
    const model = createTestOntologyModel();
    let state = createOntologyBrowserState(model);

    state = drillIntoOntologyBrowser(model, state);
    state = drillIntoOntologyBrowser(model, state);
    expect(state.depth).toBe(2);
    expect(state.selectedNodeIds).toEqual(["spell", "spell:security", "spell:alarm"]);

    state = moveOntologyBrowserDetailScroll(state, 5, 8);
    expect(state.detailScroll).toBe(5);

    state = moveOntologyBrowserDetailScroll(state, 10, 8);
    expect(state.detailScroll).toBe(8);

    state = moveOntologyBrowserDetailScrollToBoundary(state, "start", 8);
    expect(state.detailScroll).toBe(0);

    state = popOntologyBrowserDepth(state);
    expect(state.depth).toBe(1);
    expect(state.selectedNodeIds).toEqual(["spell", "spell:security"]);
    expect(state.filter).toBe("");
  });

  it("resolves lazy children when drilling into a node", () => {
    const model: OntologyDomainModel = {
      id: "searchSemantics",
      label: "Search Semantics",
      description: "Lazy ontology test",
      rootNodes: [
        {
          id: "spell",
          kind: "category",
          label: "Spell",
          filterText: "spell",
          detailLines: [{ text: "Spell", tone: "section" }],
          loadChildren: () => [
            {
              id: "spell:saveType",
              kind: "field",
              label: "saveType",
              filterText: "saveType",
              detailLines: [{ text: "saveType", tone: "section" }],
            },
          ],
        },
      ],
    };

    const state = drillIntoOntologyBrowser(model, createOntologyBrowserState(model));
    const selection = getOntologyBrowserSelection(model, state);

    expect(selection.currentNode?.id).toBe("spell:saveType");
    expect(model.rootNodes[0]?.children?.map((node) => node.id)).toEqual(["spell:saveType"]);
  });
});
