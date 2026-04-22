import React from "react";

import { cleanup, render } from "ink-testing-library";
import { afterEach, describe, expect, it, vi } from "vitest";

import {
  FilterExplorerScreen,
  FILTER_EXPLORER_LAUNCH_INTENT,
  type FilterExplorerModel,
  type FilterExplorerQueryOpenIntent,
} from "../../src/tui/filter-explorer/index.js";
import { DerivedTagTerminalProvider } from "../../src/tui/terminal-ui.js";
import { browseQuery } from "../helpers/search-request-fixture.js";

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
          category: "spell",
          metadata: { field: "traits", op: "includesAny", values: ["illusion"] },
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
    const onOpenQueryIntent = vi.fn<(intent: FilterExplorerQueryOpenIntent) => void>();
    const app = render(
      <DerivedTagTerminalProvider>
        <FilterExplorerScreen
          title="Search Semantics"
          model={createModel()}
          onExit={vi.fn()}
          mode={{
            kind: "inspect-and-open",
            onOpenQueryIntent,
          }}
        />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    expect(onOpenQueryIntent).toHaveBeenCalledTimes(1);
    const [intent] = onOpenQueryIntent.mock.calls[0] ?? [];
    expect(intent).toBeDefined();
    expect(intent?.launchIntent).toBe(FILTER_EXPLORER_LAUNCH_INTENT.RESULTS);
    expect("openInResults" in (intent?.query ?? {})).toBe(false);
  });

  it("supports explicit editor intent for list-record launches", async () => {
    const onOpenQueryIntent = vi.fn<(intent: FilterExplorerQueryOpenIntent) => void>();
    const app = render(
      <DerivedTagTerminalProvider>
        <FilterExplorerScreen
          title="Search Semantics"
          model={createModel()}
          onExit={vi.fn()}
          mode={{
            kind: "inspect-and-open",
            defaultListRecordLaunchIntent: FILTER_EXPLORER_LAUNCH_INTENT.EDITOR,
            onOpenQueryIntent,
          }}
        />
      </DerivedTagTerminalProvider>,
    );

    await flushInk();
    app.stdin.write("\r");
    await flushInk();

    expect(onOpenQueryIntent).toHaveBeenCalledTimes(1);
    const [intent] = onOpenQueryIntent.mock.calls[0] ?? [];
    expect(intent?.launchIntent).toBe(FILTER_EXPLORER_LAUNCH_INTENT.EDITOR);
  });
});
