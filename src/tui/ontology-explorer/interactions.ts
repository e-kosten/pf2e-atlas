import React from "react";

import {
  createDerivedTagTerminalListNavigationState,
  resolveDerivedTagTerminalListNavigationAction,
  useDerivedTagTerminalInput,
  type DerivedTagTerminalInputEvent,
  type DerivedTagTerminalListNavigationAction,
  type DerivedTagTerminalListNavigationState,
} from "../terminal-ui.js";
import {
  resolveTerminalInteractionAction,
  resolveTerminalTextEntryIntent,
  type TerminalInteractionAction,
  type TerminalTextEntryIntent,
} from "../interaction-bindings.js";

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
  listNavigation: DerivedTagTerminalListNavigationState;
  detailNavigation: DerivedTagTerminalListNavigationState;
};

export function createOntologyExplorerInteractionRouterState(): OntologyExplorerInteractionRouterState {
  return {
    listNavigation: createDerivedTagTerminalListNavigationState(),
    detailNavigation: createDerivedTagTerminalListNavigationState(),
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
  const listNavigation = resolveDerivedTagTerminalListNavigationAction(
    event,
    {
      pageSize: context.detailPageSize,
      jumpSize: context.selectionJumpSize,
      includeConfirmKeys: true,
      includeHorizontalConfirmKeys: true,
    },
    state.listNavigation,
  );
  const detailNavigation = resolveDerivedTagTerminalListNavigationAction(
    event,
    {
      pageSize: context.detailPageSize,
      jumpSize: context.detailJumpSize,
      includeCancelKeys: true,
      includeHorizontalCancelKeys: true,
    },
    state.detailNavigation,
  );

  return {
    route: {
      event,
      interactionAction: resolveTerminalInteractionAction(event, context.interactionActions),
      searchModeAction: resolveTerminalInteractionAction(event, [{ id: "cancel" }]),
      textEntryIntent: resolveTerminalTextEntryIntent(event),
      listNavigationAction: listNavigation.action,
      detailNavigationAction: detailNavigation.action,
    },
    state: {
      listNavigation: listNavigation.state,
      detailNavigation: detailNavigation.state,
    },
  };
}

export function useOntologyExplorerInteractionRouter(options: {
  enabled?: boolean;
  context: OntologyExplorerInteractionContext;
  onRoute: (route: OntologyExplorerInteractionRoute) => void;
}): void {
  const stateRef = React.useRef(createOntologyExplorerInteractionRouterState());

  useDerivedTagTerminalInput(
    (event) => {
      const routed = routeOntologyExplorerInteraction(event, options.context, stateRef.current);
      stateRef.current = routed.state;
      options.onRoute(routed.route);
    },
    options.enabled,
  );
}
