import type { DerivedTagTerminalListNavigationAction } from "../framework/input.js";
import type { DerivedTagTerminalInputEvent } from "../framework/types.js";
import type { TerminalInteractionAction, TerminalTextEntryIntent } from "../interaction-bindings.js";
import {
  createTerminalDetailInteractionContext,
  createTerminalInteractionContextRouterState,
  createTerminalListInteractionContext,
  createTerminalTextEntryInteractionContext,
  routeTerminalInteractionContexts,
  useTerminalInteractionContextRouter,
} from "../interaction-context-router.js";

export type OntologyExplorerInteractionContext = {
  searchMode: boolean;
  activePane: "list" | "detail";
  detailPageSize: number;
  selectionJumpSize: number;
  detailJumpSize: number;
  interactionActions: TerminalInteractionAction[];
};

export type OntologyExplorerInteractionRoute = {
  event: DerivedTagTerminalInputEvent;
  interactionAction?: TerminalInteractionAction;
  searchModeAction?: TerminalInteractionAction;
  textEntryIntent?: TerminalTextEntryIntent;
  listNavigationAction?: DerivedTagTerminalListNavigationAction;
  detailNavigationAction?: DerivedTagTerminalListNavigationAction;
};

export type OntologyExplorerInteractionRouterState = {
  router: ReturnType<typeof createTerminalInteractionContextRouterState<"list" | "detail" | "textEntry">>;
};

export function createOntologyExplorerInteractionRouterState(): OntologyExplorerInteractionRouterState {
  return {
    router: createTerminalInteractionContextRouterState(),
  };
}

export function routeOntologyExplorerInteraction(
  event: DerivedTagTerminalInputEvent,
  context: OntologyExplorerInteractionContext,
  state: OntologyExplorerInteractionRouterState = createOntologyExplorerInteractionRouterState(),
): {
  route: OntologyExplorerInteractionRoute;
  state: OntologyExplorerInteractionRouterState;
} {
  const routed = routeTerminalInteractionContexts(
    event,
    [
      createTerminalListInteractionContext("list", {
        interactionActions: context.interactionActions,
        pageSize: context.detailPageSize,
        jumpSize: context.selectionJumpSize,
        includeConfirmKeys: true,
        includeHorizontalConfirmKeys: true,
      }),
      createTerminalDetailInteractionContext("detail", {
        interactionActions: context.interactionActions,
        pageSize: context.detailPageSize,
        jumpSize: context.detailJumpSize,
        includeCancelKeys: true,
        includeHorizontalCancelKeys: true,
      }),
      createTerminalTextEntryInteractionContext("textEntry", [{ id: "cancel" }]),
    ],
    state.router,
  );

  return {
    route: {
      event,
      interactionAction:
        context.activePane === "detail" ? routed.routes.detail.interactionAction : routed.routes.list.interactionAction,
      searchModeAction: routed.routes.textEntry.interactionAction,
      textEntryIntent: routed.routes.textEntry.textEntryIntent,
      listNavigationAction: routed.routes.list.navigationAction,
      detailNavigationAction: routed.routes.detail.navigationAction,
    },
    state: {
      router: routed.state,
    },
  };
}

export function useOntologyExplorerInteractionRouter(options: {
  enabled?: boolean;
  context: OntologyExplorerInteractionContext;
  onRoute: (route: OntologyExplorerInteractionRoute) => void;
}): void {
  useTerminalInteractionContextRouter({
    enabled: options.enabled,
    contexts: [
      createTerminalListInteractionContext("list", {
        interactionActions: options.context.interactionActions,
        pageSize: options.context.detailPageSize,
        jumpSize: options.context.selectionJumpSize,
        includeConfirmKeys: true,
        includeHorizontalConfirmKeys: true,
      }),
      createTerminalDetailInteractionContext("detail", {
        interactionActions: options.context.interactionActions,
        pageSize: options.context.detailPageSize,
        jumpSize: options.context.detailJumpSize,
        includeCancelKeys: true,
        includeHorizontalCancelKeys: true,
      }),
      createTerminalTextEntryInteractionContext("textEntry", [{ id: "cancel" }]),
    ],
    onRoute: (routes) => {
      options.onRoute({
        event: routes.textEntry.event,
        interactionAction:
          options.context.activePane === "detail" ? routes.detail.interactionAction : routes.list.interactionAction,
        searchModeAction: routes.textEntry.interactionAction,
        textEntryIntent: routes.textEntry.textEntryIntent,
        listNavigationAction: routes.list.navigationAction,
        detailNavigationAction: routes.detail.navigationAction,
      });
    },
  });
}
