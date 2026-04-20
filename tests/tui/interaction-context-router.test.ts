import { describe, expect, it } from "vitest";

import { createDerivedTagTerminalInputEvent } from "../../src/tui/terminal-ui.js";
import {
  createTerminalCommandPaletteInteractionContext,
  createTerminalInteractionContextRouterState,
  createTerminalListInteractionContext,
  createTerminalTextEntryInteractionContext,
  createTerminalInteractionContextStack,
  getActiveTerminalInteractionContext,
  popTerminalInteractionContext,
  pushTerminalInteractionContext,
  routeTerminalInteractionContexts,
} from "../../src/tui/interaction-context-router.js";

describe("interaction context router", () => {
  it("preserves pending boundary navigation state per routed context", () => {
    const contexts = [
      createTerminalListInteractionContext("list", {
        pageSize: 10,
        jumpSize: 5,
      }),
    ] as const;

    const firstG = routeTerminalInteractionContexts(
      createDerivedTagTerminalInputEvent("g", {} as never),
      contexts,
      createTerminalInteractionContextRouterState(),
    );
    expect(firstG.routes.list.navigationAction).toBeUndefined();

    const secondG = routeTerminalInteractionContexts(
      createDerivedTagTerminalInputEvent("g", {} as never),
      contexts,
      firstG.state,
    );
    expect(secondG.routes.list.navigationAction).toEqual({ kind: "boundary", boundary: "start" });
  });

  it("can route command-like and text-entry semantics from the same raw input", () => {
    const routed = routeTerminalInteractionContexts(
      createDerivedTagTerminalInputEvent("q", {} as never),
      [
        createTerminalTextEntryInteractionContext("textEntry", [{ id: "quit" }]),
        createTerminalCommandPaletteInteractionContext(10),
      ],
      createTerminalInteractionContextRouterState(),
    );

    expect(routed.routes.textEntry.textEntryIntent).toEqual({ kind: "append", text: "q" });
    expect(routed.routes.textEntry.interactionAction?.id).toBe("quit");
    expect(routed.routes.commandPalette.textEntryIntent).toEqual({ kind: "append", text: "q" });
  });

  it("supports a shared context stack contract for adapter-backed prompts", () => {
    const stack = createTerminalInteractionContextStack([{ kind: "list", key: "search:list" }]);
    const withPrompt = pushTerminalInteractionContext(stack, { kind: "commandPalette", key: "modal:command" });

    expect(getActiveTerminalInteractionContext(withPrompt)).toEqual({
      kind: "commandPalette",
      key: "modal:command",
    });

    const popped = popTerminalInteractionContext(withPrompt);
    expect(popped.popped).toEqual({
      kind: "commandPalette",
      key: "modal:command",
    });
    expect(getActiveTerminalInteractionContext(popped.stack)).toEqual({
      kind: "list",
      key: "search:list",
    });
  });
});
