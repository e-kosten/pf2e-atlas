import { describe, expect, it } from "vitest";

import { createDerivedTagTerminalInputEvent } from "../../src/tui/terminal-ui.js";
import {
  createOntologyExplorerInteractionRouterState,
  routeOntologyExplorerInteraction,
} from "../../src/tui/ontology-explorer/interactions.js";

describe("ontology explorer interactions", () => {
  it("preserves pending list boundary state across routed inputs", () => {
    const context = {
      searchMode: false,
      activePane: "list" as const,
      detailPageSize: 10,
      selectionJumpSize: 5,
      detailJumpSize: 4,
      interactionActions: [],
    };

    const firstG = routeOntologyExplorerInteraction(
      createDerivedTagTerminalInputEvent("g", {} as never),
      context,
      createOntologyExplorerInteractionRouterState(),
    );
    expect(firstG.route.listNavigationAction).toBeUndefined();

    const secondG = routeOntologyExplorerInteraction(
      createDerivedTagTerminalInputEvent("g", {} as never),
      context,
      firstG.state,
    );
    expect(secondG.route.listNavigationAction).toEqual({ kind: "boundary", boundary: "start" });
  });

  it("routes text entry and interaction actions from the same raw input", () => {
    const context = {
      searchMode: true,
      activePane: "list" as const,
      detailPageSize: 10,
      selectionJumpSize: 5,
      detailJumpSize: 4,
      interactionActions: [{ id: "quit" as const }],
    };

    const typedQ = routeOntologyExplorerInteraction(
      createDerivedTagTerminalInputEvent("q", {} as never),
      context,
      createOntologyExplorerInteractionRouterState(),
    );
    expect(typedQ.route.textEntryIntent).toEqual({ kind: "append", text: "q" });
    expect(typedQ.route.interactionAction?.id).toBe("quit");

    const escape = routeOntologyExplorerInteraction(
      createDerivedTagTerminalInputEvent("\u001b", {} as never),
      context,
      typedQ.state,
    );
    expect(escape.route.searchModeAction?.id).toBe("cancel");
    expect(escape.route.interactionAction?.id).toBe("quit");
  });
});
