import { describe, expect, it } from "vitest";

import { createDerivedTagTerminalActionTargetState } from "../../src/tui/action-target.js";
import { createDerivedTagTerminalInputEvent } from "../../src/tui/terminal-ui.js";
import {
  createTerminalActionTargetInteractionContext,
  createTerminalCommandPaletteInteractionContext,
  createTerminalInteractionContextRouterState,
  createTerminalListInteractionContext,
  createTerminalSelectPromptInteractionContext,
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

  it("routes Escape as cancel while preserving left-arrow back semantics for select prompts", () => {
    const escape = routeTerminalInteractionContexts(
      createDerivedTagTerminalInputEvent("\u001b", {} as never),
      [createTerminalSelectPromptInteractionContext("selectPrompt", 8, false)],
      createTerminalInteractionContextRouterState(),
    );
    expect(escape.routes.selectPrompt.interactionAction?.id).toBe("cancel");

    const left = routeTerminalInteractionContexts(
      createDerivedTagTerminalInputEvent("\u001b[D", { leftArrow: true } as never),
      [createTerminalSelectPromptInteractionContext("selectPrompt", 8, false)],
      createTerminalInteractionContextRouterState(),
    );
    expect(left.routes.selectPrompt.interactionAction?.id).toBe("back");
  });

  it("resolves action-target intents as part of the shared routing result", () => {
    const state = createDerivedTagTerminalActionTargetState();
    const toggle = routeTerminalInteractionContexts(
      createDerivedTagTerminalInputEvent(":", {} as never),
      [
        createTerminalActionTargetInteractionContext("actionTarget", {
          state,
          orientation: "horizontal",
          interactionActions: [{ id: "actions" }],
        }),
      ],
      createTerminalInteractionContextRouterState(),
    );

    expect(toggle.routes.actionTarget.actionTargetIntent).toEqual({ kind: "toggle_target" });

    const focusedState = {
      ...state,
      activeTarget: "actions" as const,
    };
    const move = routeTerminalInteractionContexts(
      createDerivedTagTerminalInputEvent("\u001b[C", { rightArrow: true } as never),
      [
        createTerminalActionTargetInteractionContext("actionTarget", {
          state: focusedState,
          orientation: "horizontal",
          interactionActions: [{ id: "actions" }, { id: "apply" }],
        }),
      ],
      createTerminalInteractionContextRouterState(),
    );

    expect(move.routes.actionTarget.actionTargetIntent).toEqual({ kind: "move_action", delta: 1 });
  });
});
