import React from "react";

import { cleanup, render } from "ink-testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";

import {
  FilterExplorerScreen,
  FILTER_EXPLORER_LAUNCH_INTENT,
  createInspectFilterExplorerHostAdapter,
  type FilterExplorerModel,
  type FilterExplorerSelectTargetOutcome,
} from "../../src/tui/filter-explorer/index.js";
import { DerivedTagTerminalProvider } from "../../src/tui/terminal-ui.js";
import { browseQuery, metadataPredicateFilter, scopeFilter } from "../helpers/search-request-fixture.js";

function flushInk(): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, 0);
  });
}

function createModel(): FilterExplorerModel {
  return {
    id: "search-semantics",
    label: "Search Semantics",
    description: "Search semantics ontology",
    rootNodes: [
      {
        id: "spell:trait:illusion",
        kind: "value",
        label: "Illusion",
        filterText: "illusion",
        listLabel: "Illusion | 12",
        detailTitle: "Filter Value",
        detailLines: [{ text: "Illusion", tone: "section" }],
        query: browseQuery("Browse illusion spells", {
          filter: {
            kind: "allOf",
            children: [
              scopeFilter("spell"),
              metadataPredicateFilter({ field: "traits", op: "includes", value: "illusion" }),
            ],
          },
          limit: 20,
        }),
      },
    ],
  };
}

afterEach(() => {
  cleanup();
});

describe("filter explorer launch intent", () => {
  it("passes results intent through an explicit open-intent object for list-record nodes", async () => {
    const onOutcome = vi.fn<(outcome: FilterExplorerSelectTargetOutcome) => void>();
    const app = render(
      <DerivedTagTerminalProvider>
        <FilterExplorerScreen
          title="Search Semantics"
          model={createModel()}
          host={createInspectFilterExplorerHostAdapter({})}
          onOutcome={onOutcome}
          mode={{
            kind: "inspect-and-open",
          }}
        />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    expect(onOutcome).toHaveBeenCalledTimes(1);
    const [outcome] = onOutcome.mock.calls[0] ?? [];
    expect(outcome).toBeDefined();
    expect(outcome?.queryIntent.launchIntent).toBe(FILTER_EXPLORER_LAUNCH_INTENT.RESULTS);
    expect("openInResults" in (outcome?.queryIntent.query ?? {})).toBe(false);
  });

  it("supports explicit editor intent for list-record launches", async () => {
    const onOutcome = vi.fn<(outcome: FilterExplorerSelectTargetOutcome) => void>();
    const app = render(
      <DerivedTagTerminalProvider>
        <FilterExplorerScreen
          title="Search Semantics"
          model={createModel()}
          host={createInspectFilterExplorerHostAdapter({})}
          onOutcome={onOutcome}
          mode={{
            kind: "inspect-and-open",
            defaultListRecordLaunchIntent: FILTER_EXPLORER_LAUNCH_INTENT.EDITOR,
          }}
        />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    expect(onOutcome).toHaveBeenCalledTimes(1);
    const [outcome] = onOutcome.mock.calls[0] ?? [];
    expect(outcome?.queryIntent.launchIntent).toBe(FILTER_EXPLORER_LAUNCH_INTENT.EDITOR);
  });

  it("renders a shared warning banner instead of changing focus when rightward drill hits a dead end", async () => {
    const onOutcome = vi.fn<(outcome: FilterExplorerSelectTargetOutcome) => void>();
    const app = render(
      <DerivedTagTerminalProvider>
        <FilterExplorerScreen
          title="Search Semantics"
          model={{
            id: "search-semantics",
            label: "Search Semantics",
            description: "Search semantics ontology",
            rootNodes: [
              {
                id: "spell:leaf",
                kind: "value",
                label: "Leaf Entry",
                filterText: "leaf entry",
                listLabel: "Leaf Entry",
                detailTitle: "Leaf Entry",
                detailLines: [{ text: "Leaf Entry", tone: "section" }],
              },
            ],
          }}
          host={createInspectFilterExplorerHostAdapter({})}
          onOutcome={onOutcome}
          mode={{
            kind: "inspect-and-open",
          }}
        />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    expect(app.lastFrame()).toContain("[LIST] Explorer Entries");

    app.stdin.write("\u001b[C");
    await flushInk();

    expect(onOutcome).not.toHaveBeenCalled();
    expect(app.lastFrame()).toContain("[LIST] Explorer Entries");
    expect(app.lastFrame()).toContain("No rightward action is available for the focused entry.");
    expect(app.lastFrame()).not.toContain("[DETAIL] Leaf Entry");
  });
});
