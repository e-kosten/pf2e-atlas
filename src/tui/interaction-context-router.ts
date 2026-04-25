import React from "react";

import type {
  DerivedTagTerminalActionTargetIntent,
  DerivedTagTerminalActionTargetOrientation,
  DerivedTagTerminalActionTargetState,
} from "./action-target.js";
import { resolveDerivedTagTerminalActionTargetIntent } from "./action-target.js";
import {
  getTerminalInteractionCycleDirection,
  resolveTerminalInteractionAction,
  resolveTerminalTextEntryIntent,
  type TerminalInteractionAction,
  type TerminalTextEntryIntent,
} from "./interaction-bindings.js";
import {
  createDerivedTagTerminalListNavigationState,
  resolveDerivedTagTerminalListNavigationAction,
  useDerivedTagTerminalInput,
  type DerivedTagTerminalListNavigationAction,
  type DerivedTagTerminalListNavigationState,
} from "./framework/input.js";
import type { DerivedTagTerminalInputEvent } from "./framework/types.js";

export type TerminalInteractionContextKind =
  | "list"
  | "detail"
  | "textEntry"
  | "actionTarget"
  | "message"
  | "commandPalette"
  | "selectPrompt"
  | "multiSelectPrompt"
  | "textPrompt"
  | "dialog";

export type TerminalInteractionContextNavigation = {
  pageSize: number;
  jumpSize: number;
  includeConfirmKeys?: boolean;
  includeHorizontalConfirmKeys?: boolean;
  includeCancelKeys?: boolean;
  includeHorizontalCancelKeys?: boolean;
};

export type TerminalInteractionContextConfig<TContextId extends string = string> = {
  id: TContextId;
  kind: TerminalInteractionContextKind;
  interactionActions?: TerminalInteractionAction[];
  navigation?: TerminalInteractionContextNavigation;
  textEntry?: boolean;
  actionTarget?: {
    state: DerivedTagTerminalActionTargetState;
    orientation: DerivedTagTerminalActionTargetOrientation;
  };
};

export type TerminalInteractionContextRoute<TContextId extends string = string> = {
  contextId: TContextId;
  kind: TerminalInteractionContextKind;
  event: DerivedTagTerminalInputEvent;
  interactionAction?: TerminalInteractionAction;
  textEntryIntent?: TerminalTextEntryIntent;
  navigationAction?: DerivedTagTerminalListNavigationAction;
  actionTargetIntent?: DerivedTagTerminalActionTargetIntent;
  cycleDirection?: 1 | -1;
};

export type TerminalInteractionContextRoutes<TContextId extends string = string> = Record<
  TContextId,
  TerminalInteractionContextRoute<TContextId>
>;

export type TerminalInteractionContextRouterState<TContextId extends string = string> = {
  navigationStates: Partial<Record<TContextId, DerivedTagTerminalListNavigationState>>;
};

export type TerminalInteractionContextStackEntry = {
  kind: TerminalInteractionContextKind;
  key: string;
};

export function createTerminalInteractionContextRouterState<
  TContextId extends string = string,
>(): TerminalInteractionContextRouterState<TContextId> {
  return {
    navigationStates: {},
  };
}

export function createTerminalInteractionContextStack(
  initialEntries: TerminalInteractionContextStackEntry[] = [],
): TerminalInteractionContextStackEntry[] {
  return [...initialEntries];
}

export function pushTerminalInteractionContext(
  stack: TerminalInteractionContextStackEntry[],
  entry: TerminalInteractionContextStackEntry,
): TerminalInteractionContextStackEntry[] {
  return [...stack, entry];
}

export function popTerminalInteractionContext(stack: TerminalInteractionContextStackEntry[]): {
  stack: TerminalInteractionContextStackEntry[];
  popped?: TerminalInteractionContextStackEntry;
} {
  const popped = stack.at(-1);
  return {
    stack: stack.length > 0 ? stack.slice(0, -1) : stack,
    popped,
  };
}

export function getActiveTerminalInteractionContext(
  stack: TerminalInteractionContextStackEntry[],
): TerminalInteractionContextStackEntry | undefined {
  return stack.at(-1);
}

function getNavigationState<TContextId extends string>(
  state: TerminalInteractionContextRouterState<TContextId>,
  contextId: TContextId,
): DerivedTagTerminalListNavigationState {
  return state.navigationStates[contextId] ?? createDerivedTagTerminalListNavigationState();
}

export function routeTerminalInteractionContext<TContextId extends string>(
  event: DerivedTagTerminalInputEvent,
  context: TerminalInteractionContextConfig<TContextId>,
  state: TerminalInteractionContextRouterState<TContextId> = createTerminalInteractionContextRouterState<TContextId>(),
): {
  route: TerminalInteractionContextRoute<TContextId>;
  state: TerminalInteractionContextRouterState<TContextId>;
} {
  let nextState = state;
  let navigationAction: DerivedTagTerminalListNavigationAction | undefined;

  if (context.navigation) {
    const navigation = resolveDerivedTagTerminalListNavigationAction(
      event,
      {
        pageSize: context.navigation.pageSize,
        jumpSize: context.navigation.jumpSize,
        includeConfirmKeys: context.navigation.includeConfirmKeys,
        includeHorizontalConfirmKeys: context.navigation.includeHorizontalConfirmKeys,
        includeCancelKeys: context.navigation.includeCancelKeys,
        includeHorizontalCancelKeys: context.navigation.includeHorizontalCancelKeys,
      },
      getNavigationState(state, context.id),
    );

    navigationAction = navigation.action;
    nextState = {
      navigationStates: {
        ...state.navigationStates,
        [context.id]: navigation.state,
      },
    };
  }

  const interactionAction = resolveTerminalInteractionAction(event, context.interactionActions ?? []);
  const textEntryIntent = context.textEntry ? resolveTerminalTextEntryIntent(event) : undefined;
  const actionTargetIntent = context.actionTarget
    ? resolveDerivedTagTerminalActionTargetIntent(
        event,
        context.actionTarget.state,
        context.actionTarget.orientation,
      )
    : undefined;

  return {
    route: {
      contextId: context.id,
      kind: context.kind,
      event,
      interactionAction,
      textEntryIntent,
      navigationAction,
      actionTargetIntent,
      cycleDirection: getTerminalInteractionCycleDirection(event, interactionAction),
    },
    state: nextState,
  };
}

export function routeTerminalInteractionContexts<TContextId extends string>(
  event: DerivedTagTerminalInputEvent,
  contexts: readonly TerminalInteractionContextConfig<TContextId>[],
  state: TerminalInteractionContextRouterState<TContextId> = createTerminalInteractionContextRouterState<TContextId>(),
): {
  routes: TerminalInteractionContextRoutes<TContextId>;
  state: TerminalInteractionContextRouterState<TContextId>;
} {
  let nextState = state;
  const routes = {} as TerminalInteractionContextRoutes<TContextId>;

  for (const context of contexts) {
    const routed = routeTerminalInteractionContext(event, context, nextState);
    nextState = routed.state;
    routes[context.id] = routed.route;
  }

  return {
    routes,
    state: nextState,
  };
}

export function useTerminalInteractionContextRouter<TContextId extends string>(options: {
  enabled?: boolean;
  contexts: readonly TerminalInteractionContextConfig<TContextId>[];
  onRoute: (routes: TerminalInteractionContextRoutes<TContextId>) => void;
}): void {
  const stateRef = React.useRef(createTerminalInteractionContextRouterState<TContextId>());

  useDerivedTagTerminalInput((event) => {
    const routed = routeTerminalInteractionContexts(event, options.contexts, stateRef.current);
    stateRef.current = routed.state;
    options.onRoute(routed.routes);
  }, options.enabled);
}

export function createTerminalListInteractionContext<TContextId extends string>(
  id: TContextId,
  options: {
    interactionActions?: TerminalInteractionAction[];
    pageSize: number;
    jumpSize: number;
    includeConfirmKeys?: boolean;
    includeHorizontalConfirmKeys?: boolean;
  },
): TerminalInteractionContextConfig<TContextId> {
  return {
    id,
    kind: "list",
    interactionActions: options.interactionActions,
    navigation: {
      pageSize: options.pageSize,
      jumpSize: options.jumpSize,
      includeConfirmKeys: options.includeConfirmKeys,
      includeHorizontalConfirmKeys: options.includeHorizontalConfirmKeys,
    },
  };
}

export function createTerminalDetailInteractionContext<TContextId extends string>(
  id: TContextId,
  options: {
    interactionActions?: TerminalInteractionAction[];
    pageSize: number;
    jumpSize: number;
    includeCancelKeys?: boolean;
    includeHorizontalCancelKeys?: boolean;
  },
): TerminalInteractionContextConfig<TContextId> {
  return {
    id,
    kind: "detail",
    interactionActions: options.interactionActions,
    navigation: {
      pageSize: options.pageSize,
      jumpSize: options.jumpSize,
      includeCancelKeys: options.includeCancelKeys,
      includeHorizontalCancelKeys: options.includeHorizontalCancelKeys,
    },
  };
}

export function createTerminalTextEntryInteractionContext<TContextId extends string>(
  id: TContextId,
  interactionActions?: TerminalInteractionAction[],
): TerminalInteractionContextConfig<TContextId> {
  return {
    id,
    kind: "textEntry",
    interactionActions,
    textEntry: true,
  };
}

export function createTerminalActionTargetInteractionContext<TContextId extends string>(
  id: TContextId,
  options: {
    interactionActions?: TerminalInteractionAction[];
    state: DerivedTagTerminalActionTargetState;
    orientation: DerivedTagTerminalActionTargetOrientation;
  },
): TerminalInteractionContextConfig<TContextId> {
  return {
    id,
    kind: "actionTarget",
    interactionActions: options.interactionActions,
    actionTarget: {
      state: options.state,
      orientation: options.orientation,
    },
  };
}

export function createTerminalCommandPaletteInteractionContext(
  pageSize: number,
): TerminalInteractionContextConfig<"commandPalette"> {
  return {
    id: "commandPalette",
    kind: "commandPalette",
    interactionActions: [{ id: "select" }, { id: "back", label: "cancel" }],
    navigation: {
      pageSize,
      jumpSize: 5,
      includeCancelKeys: true,
      includeHorizontalCancelKeys: true,
    },
    textEntry: true,
  };
}

export function createTerminalTextPromptInteractionContext(): TerminalInteractionContextConfig<"textPrompt"> {
  return {
    id: "textPrompt",
    kind: "textPrompt",
    textEntry: true,
  };
}

export function createTerminalSelectPromptInteractionContext(
  pageSize: number,
  supportsCommands = false,
): TerminalInteractionContextConfig<"selectPrompt"> {
  return {
    id: "selectPrompt",
    kind: "selectPrompt",
    interactionActions: [
      { id: "select" },
      ...(supportsCommands ? [{ id: "commands" as const }] : []),
      { id: "cancel" as const },
      { id: "back" as const },
    ],
    navigation: {
      pageSize,
      jumpSize: 5,
      includeCancelKeys: true,
      includeHorizontalCancelKeys: true,
    },
  };
}

export function createTerminalMultiSelectPromptInteractionContext(
  pageSize: number,
  supportsCommands = false,
): TerminalInteractionContextConfig<"multiSelectPrompt"> {
  return {
    id: "multiSelectPrompt",
    kind: "multiSelectPrompt",
    interactionActions: [
      { id: "toggle" },
      ...(supportsCommands ? [{ id: "commands" as const }] : []),
      { id: "return" },
      { id: "cancel" as const },
      { id: "back" as const },
    ],
    navigation: {
      pageSize,
      jumpSize: 5,
      includeCancelKeys: true,
      includeHorizontalCancelKeys: true,
    },
  };
}
