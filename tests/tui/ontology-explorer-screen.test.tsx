import React from "react";

import { cleanup, render } from "ink-testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { OntologyDomainModel, OntologyNodeQuery } from "../../src/domain/ontology-types.js";
import { OntologyInspectScreen } from "../../src/tui/ontology-explorer/inspect-screen.js";
import { OntologyBrowserScreen } from "../../src/tui/ontology-explorer/screen.js";
import type { OntologyBrowserSnapshot } from "../../src/tui/ontology-explorer/ui.js";
import * as scalarEditor from "../../src/tui/search-screen/scalar-editor.js";
import { DerivedTagTerminalProvider } from "../../src/tui/terminal-ui.js";

function flushInk(): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, 0);
  });
}

function createTestOntologyModel(): OntologyDomainModel {
  return {
    id: "derivedTags",
    label: "Derived Tags",
    description: "Test ontology domain",
    rootNodes: [
      {
        id: "spell:alarm",
        kind: "tag",
        label: "alarm",
        filterText: "alarm",
        listLabel: "alarm | 1 live record",
        detailTitle: "Tag Details",
        detailLines: [
          { text: "Alarm", tone: "section" },
          ...Array.from({ length: 30 }, (_, index) => ({ text: `Detail line ${index + 1}` })),
        ],
        query: {
          kind: "listRecords",
          label: "Browse this tag",
          filters: {
            category: "spell",
            limit: 20,
          },
        },
      },
    ],
  };
}

function createNumericMetricInspectModel(): OntologyDomainModel {
  return {
    id: "searchSemantics",
    label: "Search Semantics",
    description: "Test ontology inspect domain",
    rootNodes: [
      {
        id: "creature:actorMetrics:perception.mod",
        kind: "metric",
        label: "Perception Modifier",
        shortLabel: "perception.mod",
        filterText: "perception modifier creature actor metric",
        listLabel: "Perception Modifier | 12",
        detailTitle: "Metric Details",
        detailLines: [
          { text: "Perception Modifier", tone: "section" },
          { text: "Inspecting this metric should open the scalar editor." },
        ],
        query: {
          kind: "listRecords",
          label: "Browse records with Perception Modifier",
          filters: {
            category: "creature",
            metadata: {
              field: "actorMetricCompare",
              leftMetric: "perception.mod",
              op: ">=",
              rightMetric: "perception.mod",
            },
            limit: 20,
          },
        },
      },
    ],
  };
}

describe("ontology browser screen", () => {
  afterEach(() => {
    cleanup();
  });

  it("treats q as search input while inline search is active", async () => {
    const model = createTestOntologyModel();
    const onExit = vi.fn();
    const app = render(
      <DerivedTagTerminalProvider>
        <OntologyBrowserScreen model={model} onExit={onExit} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    app.stdin.write("/");
    await flushInk();
    expect(app.lastFrame()).toContain("Search /");

    app.stdin.write("q");
    await flushInk();

    expect(onExit).not.toHaveBeenCalled();
    expect(app.lastFrame()).toContain("Search /q");

    app.unmount();
  });

  it("focuses and scrolls the detail pane from a leaf entry", async () => {
    const model = createTestOntologyModel();
    const onExit = vi.fn();
    const app = render(
      <DerivedTagTerminalProvider>
        <OntologyBrowserScreen model={model} onExit={onExit} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    expect(app.lastFrame()).toContain("list focus | split layout | Detail scroll 0/");

    app.stdin.write("\r");
    await flushInk();
    expect(app.lastFrame()).toContain("detail focus | split layout | Detail scroll 0/");

    app.stdin.write("j");
    await flushInk();

    expect(onExit).not.toHaveBeenCalled();
    expect(app.lastFrame()).toContain("detail focus | split layout | Detail scroll 1/");

    app.unmount();
  });

  it("opens the selected node query in the search workspace when requested", async () => {
    const model = createTestOntologyModel();
    const onOpenQuery = vi.fn<(query: OntologyNodeQuery, snapshot: OntologyBrowserSnapshot) => void>();
    const app = render(
      <DerivedTagTerminalProvider>
        <OntologyBrowserScreen model={model} onExit={vi.fn()} onOpenQuery={onOpenQuery} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    expect(app.lastFrame()).toContain("Ontology Entries");

    app.stdin.write(":");
    await flushInk();
    expect(app.lastFrame()).toContain("Ontology Commands");

    for (const character of "open query") {
      app.stdin.write(character);
    }
    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    expect(onOpenQuery).toHaveBeenCalledWith(
      {
        kind: "listRecords",
        label: "Browse this tag",
        filters: {
          category: "spell",
          limit: 20,
        },
      },
      expect.anything(),
    );
    expect(onOpenQuery.mock.calls[0]?.[1]).toMatchObject({
      activePane: "list",
      browserState: {
        depth: 0,
        selectedNodeIds: ["spell:alarm"],
        filter: "",
      },
    });

    app.unmount();
  });

  it("does not treat old page-specific letters as live ontology commands", async () => {
    const model = createTestOntologyModel();
    const onOpenQuery = vi.fn<(query: OntologyNodeQuery, snapshot: OntologyBrowserSnapshot) => void>();
    const app = render(
      <DerivedTagTerminalProvider>
        <OntologyBrowserScreen model={model} onExit={vi.fn()} onOpenQuery={onOpenQuery} />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("o");
    await flushInk();

    expect(onOpenQuery).not.toHaveBeenCalled();

    app.unmount();
  });

  it("opens list-record queries on confirm for search semantics leaves", async () => {
    const model: OntologyDomainModel = {
      id: "searchSemantics",
      label: "Search Semantics",
      description: "Test search semantics domain",
      rootNodes: [
        {
          id: "creature:publicationTitle:rage-of-elements",
          kind: "value",
          label: "Pathfinder Rage of Elements",
          filterText: "pathfinder rage of elements",
          listLabel: "Pathfinder Rage of Elements | 81",
          detailTitle: "Filter Value",
          detailLines: [
            { text: "Pathfinder Rage of Elements", tone: "section" },
            { text: "Live canonical records: 81" },
            { text: "Press Enter or o to open the full matching set in the shared result reader." },
          ],
          query: {
            kind: "listRecords",
            label: "Browse records with this value",
            filters: {
              category: "creature",
              metadata: { field: "publicationTitle", op: "contains", value: "Pathfinder Rage of Elements" },
              limit: 20,
            },
          },
        },
      ],
    };
    const onOpenQuery = vi.fn<(query: OntologyNodeQuery, snapshot: OntologyBrowserSnapshot) => void>();
    const app = render(
      <DerivedTagTerminalProvider>
        <OntologyBrowserScreen model={model} onExit={vi.fn()} onOpenQuery={onOpenQuery} mode="inspect-and-open" />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    app.stdin.write("\r");
    await flushInk();

    expect(onOpenQuery).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: "listRecords",
        label: "Browse records with this value",
        filters: {
          category: "creature",
          metadata: { field: "publicationTitle", op: "contains", value: "Pathfinder Rage of Elements" },
          limit: 20,
        },
        openInResults: true,
      }),
      expect.anything(),
    );
    expect(onOpenQuery.mock.calls[0]?.[1]).toMatchObject({
      browserState: {
        selectedNodeIds: ["creature:publicationTitle:rage-of-elements"],
      },
    });

    app.unmount();
  });

  it("opens advanced search semantics predicates on confirm when they expose a live query", async () => {
    const model: OntologyDomainModel = {
      id: "searchSemantics",
      label: "Search Semantics",
      description: "Test search semantics domain",
      rootNodes: [
        {
          id: "equipment:advanced:itemMetric",
          kind: "advancedPredicate",
          label: "itemMetric",
          filterText: "itemMetric weapon.reload",
          listLabel: "itemMetric | ==, >=",
          detailTitle: "Advanced Predicate Details",
          detailLines: [
            { text: "itemMetric", tone: "section" },
            { text: "Generic keyed equipment metric predicate." },
            { text: "Press Enter or o to open the full matching set in the shared result reader." },
          ],
          query: {
            kind: "listRecords",
            label: "Browse records matching the itemMetric example",
            filters: {
              category: "equipment",
              metadata: {
                field: "itemMetric",
                metric: "weapon.reload",
                op: "==",
                value: 1,
              },
              limit: 20,
            },
          },
        },
      ],
    };
    const onOpenQuery = vi.fn<(query: OntologyNodeQuery, snapshot: OntologyBrowserSnapshot) => void>();
    const app = render(
      <DerivedTagTerminalProvider>
        <OntologyBrowserScreen model={model} onExit={vi.fn()} onOpenQuery={onOpenQuery} mode="inspect-and-open" />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    app.stdin.write("\r");
    await flushInk();

    expect(onOpenQuery).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: "listRecords",
        label: "Browse records matching the itemMetric example",
        filters: {
          category: "equipment",
          metadata: {
            field: "itemMetric",
            metric: "weapon.reload",
            op: "==",
            value: 1,
          },
          limit: 20,
        },
        openInResults: true,
      }),
      expect.anything(),
    );
    expect(onOpenQuery.mock.calls[0]?.[1]).toMatchObject({
      browserState: {
        selectedNodeIds: ["equipment:advanced:itemMetric"],
      },
    });

    app.unmount();
  });

  it("opens numeric metric inspect targets through the shared scalar editor flow", async () => {
    const promptNumericScalarClause = vi
      .spyOn(scalarEditor, "promptNumericScalarClause")
      .mockResolvedValue({ op: "gte", value: 5 });
    const onOpenQuery = vi.fn<(query: OntologyNodeQuery, snapshot: OntologyBrowserSnapshot) => void>();
    const app = render(
      <DerivedTagTerminalProvider>
        <OntologyInspectScreen model={createNumericMetricInspectModel()} onExit={vi.fn()} onOpenQuery={onOpenQuery} />
      </DerivedTagTerminalProvider>,
    );

    try {
      await flushInk();

      app.stdin.write("\r");
      await flushInk();

      expect(promptNumericScalarClause).toHaveBeenCalledWith(
        expect.anything(),
        expect.anything(),
        expect.objectContaining({
          title: "Actor Metric / Perception Modifier",
          currentClause: null,
        }),
      );
      expect(onOpenQuery).toHaveBeenCalledWith(
        expect.objectContaining({
          kind: "listRecords",
          label: "Browse records where Perception Modifier >= 5",
          filters: {
            category: "creature",
            metadata: {
              field: "actorMetric",
              metric: "perception.mod",
              op: ">=",
              value: 5,
            },
            limit: 20,
          },
          openInResults: true,
        }),
        expect.anything(),
      );
    } finally {
      promptNumericScalarClause.mockRestore();
      app.unmount();
    }
  });

  it("restores the browser from an initial snapshot", async () => {
    const model: OntologyDomainModel = {
      id: "searchSemantics",
      label: "Search Semantics",
      description: "Snapshot test ontology domain",
      rootNodes: [
        {
          id: "creature",
          kind: "category",
          label: "Creature",
          filterText: "creature",
          listLabel: "Creature",
          detailTitle: "Creature Details",
          detailLines: [{ text: "Creature", tone: "section" }],
          children: [
            {
              id: "creature:publicationTitle:rage-of-elements",
              kind: "value",
              label: "Pathfinder Rage of Elements",
              filterText: "pathfinder rage of elements rage",
              listLabel: "Pathfinder Rage of Elements | 81",
              detailTitle: "Filter Value",
              detailLines: [
                { text: "Pathfinder Rage of Elements", tone: "section" },
                ...Array.from({ length: 30 }, (_, index) => ({ text: `Detail line ${index + 1}` })),
              ],
              query: {
                kind: "listRecords",
                label: "Browse records with this value",
                filters: {
                  category: "creature",
                  limit: 20,
                },
              },
            },
          ],
        },
      ],
    };
    const app = render(
      <DerivedTagTerminalProvider>
        <OntologyBrowserScreen
          initialSnapshot={{
            activePane: "detail",
            browserState: {
              depth: 1,
              selectedNodeIds: ["creature", "creature:publicationTitle:rage-of-elements"],
              filter: "rage",
              detailScroll: 4,
            },
            layoutMode: "split",
            searchInput: "rage",
            searchMode: false,
          }}
          model={model}
          onExit={vi.fn()}
        />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();

    expect(app.lastFrame()).toContain("Search Semantics > Creature > Pathfinder Rage of Elements | depth 1 | /rage");
    expect(app.lastFrame()).toContain("[DETAIL] Filter Value");
    expect(app.lastFrame()).toContain("detail focus | split layout | Detail scroll 4/");

    app.unmount();
  });
});
