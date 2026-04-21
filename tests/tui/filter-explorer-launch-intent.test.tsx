import React from "react";

import { cleanup, render } from "ink-testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";

import {
  FilterExplorerScreen,
  FILTER_EXPLORER_LAUNCH_INTENT,
  type FilterExplorerModel,
} from "../../src/tui/filter-explorer/index.js";
import { DerivedTagTerminalProvider } from "../../src/tui/terminal-ui.js";

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
        query: {
          kind: "listRecords",
          label: "Browse illusion spells",
          filters: {
            category: "spell",
            metadata: { field: "traits", op: "includesAny", values: ["illusion"] },
            limit: 20,
          },
        },
      },
    ],
  };
}

afterEach(() => {
  cleanup();
});

describe("filter explorer launch intent", () => {
  it("passes results intent separately from the query payload for list-record nodes", async () => {
    const onOpenQuery = vi.fn();
    const app = render(
      <DerivedTagTerminalProvider>
        <FilterExplorerScreen
          title="Search Semantics"
          model={createModel()}
          onExit={vi.fn()}
          mode={{
            kind: "inspect-and-open",
            onOpenQuery,
          }}
        />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    expect(onOpenQuery).toHaveBeenCalledTimes(1);
    const [query, _snapshot, launchIntent] = onOpenQuery.mock.calls[0]!;
    expect(launchIntent).toBe(FILTER_EXPLORER_LAUNCH_INTENT.RESULTS);
    expect((query as { openInResults?: boolean }).openInResults).toBeUndefined();
  });

  it("supports explicit editor intent for list-record launches", async () => {
    const onOpenQuery = vi.fn();
    const app = render(
      <DerivedTagTerminalProvider>
        <FilterExplorerScreen
          title="Search Semantics"
          model={createModel()}
          onExit={vi.fn()}
          mode={{
            kind: "inspect-and-open",
            defaultListRecordLaunchIntent: FILTER_EXPLORER_LAUNCH_INTENT.EDITOR,
            onOpenQuery,
          }}
        />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    expect(onOpenQuery).toHaveBeenCalledTimes(1);
    expect(onOpenQuery.mock.calls[0]?.[2]).toBe(FILTER_EXPLORER_LAUNCH_INTENT.EDITOR);
  });
});
