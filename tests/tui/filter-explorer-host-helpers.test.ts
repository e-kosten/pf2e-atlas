import { describe, expect, it, vi } from "vitest";

import { createFilterExplorerOutcomeHandler } from "../../src/tui/filter-explorer/host-helpers.js";

const snapshot = {
  activePane: "list" as const,
  browserState: {
    depth: 0,
    selectedNodeIds: ["spell:trait:illusion"],
    filter: "",
    detailScroll: 0,
  },
  layoutMode: "split" as const,
  searchInput: "",
  searchMode: false,
};

describe("filter explorer host helpers", () => {
  it("preserves distinct shared explorer outcomes for host wiring", () => {
    const onBack = vi.fn();
    const onExitRoot = vi.fn();
    const onCancel = vi.fn();
    const onSelectTarget = vi.fn();
    const handleOutcome = createFilterExplorerOutcomeHandler({
      onBack,
      onExitRoot,
      onCancel,
      onSelectTarget,
    });

    handleOutcome({ kind: "back" }, snapshot);
    handleOutcome({ kind: "exitRoot" }, snapshot);
    handleOutcome({ kind: "cancel" }, snapshot);
    handleOutcome(
      {
        kind: "selectTarget",
        activationStyle: "open",
        result: {
          node: {
            id: "spell:trait:illusion",
            kind: "value",
            label: "Illusion",
            filterText: "illusion",
            detailLines: [{ text: "Illusion" }],
          },
          query: {
            label: "Browse illusion spells",
            request: { mode: "browse", limit: 20 },
          },
          launchIntent: "results",
        },
        queryIntent: {
          query: {
            label: "Browse illusion spells",
            request: { mode: "browse", limit: 20 },
          },
          launchIntent: "results",
        },
      },
      snapshot,
    );

    expect(onBack).toHaveBeenCalledWith(snapshot);
    expect(onExitRoot).toHaveBeenCalledWith(snapshot);
    expect(onCancel).toHaveBeenCalledWith(snapshot);
    expect(onSelectTarget).toHaveBeenCalledTimes(1);
  });
});
