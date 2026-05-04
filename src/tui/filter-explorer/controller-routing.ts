import type { DerivedTagTerminalActionTargetAction } from "../action-target.js";
import type { TerminalInteractionContextAdapters } from "../interaction-context-adapters.js";
import type { TerminalInteractionAction } from "../interaction-bindings.js";
import {
  applyTerminalListDetailRightBehavior,
  type TerminalListDetailRightBehaviorContract,
} from "../list-detail-behavior.js";
import type { TerminalListDetailNotificationTone } from "../list-detail-presentation.js";
import {
  getFilterExplorerDiscreteClause,
  getFilterExplorerScalarClause,
} from "./compose-state.js";
import { describeFilterExplorerHostNode, resolveFilterExplorerHostTarget } from "./host-adapter.js";
import {
  buildFilterExplorerInspectResult,
  openFilterExplorerInspectQuery,
  openFilterExplorerInspectResult,
  shouldOpenImmediateFilterExplorerInspectResult,
} from "./controller-inspect.js";
import {
  createFilterExplorerBrowserSnapshot,
  type FilterExplorerAction,
  resolveFilterExplorerBackNavigation,
} from "./controller-state.js";
import type {
  FilterExplorerInteractionRoute,
  FilterExplorerKeyContext,
} from "./controller-types.js";
import type {
  FilterExplorerBrowserContext,
  FilterExplorerComposeDraft,
  FilterExplorerControllerContext,
  FilterExplorerOptions,
} from "./types.js";
import {
  applyFilterExplorerActionEntry,
  applyComposeCycleSelection,
  handleFilterExplorerAction,
  openComposeScalarEditor,
} from "./workflow-actions.js";

function resolveScreenTitle(options: FilterExplorerOptions): string {
  return options.title ?? (options.mode.kind === "compose" ? "Filter Explorer" : options.model.label);
}

export function buildFilterExplorerControllerContext(args: {
  options: FilterExplorerOptions;
  browser: FilterExplorerBrowserContext;
  draft: FilterExplorerComposeDraft;
  actionEntries: FilterExplorerControllerContext["actionEntries"];
  actionTargetState: FilterExplorerControllerContext["actionTargetState"];
  onListPointerEvent?: FilterExplorerControllerContext["onListPointerEvent"];
  onDetailPointerEvent?: FilterExplorerControllerContext["onDetailPointerEvent"];
  notification?: FilterExplorerControllerContext["notification"];
}): FilterExplorerControllerContext {
  const effectiveDraft = args.options.host.getDraft?.() ?? args.draft;
  const currentNode = args.browser.currentNode;
  const selectedTarget = resolveFilterExplorerHostTarget(args.options.host, currentNode);

  return {
    model: args.options.model,
    host: args.options.host,
    mode: args.options.mode,
    screenTitle: resolveScreenTitle(args.options),
    browser: args.browser,
    draft: effectiveDraft,
    discreteClauses: effectiveDraft.discreteClauses,
    selectedTarget,
    selectedDiscreteClause: getFilterExplorerDiscreteClause(selectedTarget, effectiveDraft),
    selectedScalarClause: getFilterExplorerScalarClause(selectedTarget, effectiveDraft),
    selectedInspectResult:
      args.options.mode.kind === "inspect-and-open"
        ? buildFilterExplorerInspectResult(args.options.mode, currentNode, selectedTarget)
        : undefined,
    discovery: args.options.discovery,
    actionEntries: args.actionEntries,
    actionTargetState: args.actionTargetState,
    onListPointerEvent: args.onListPointerEvent,
    onDetailPointerEvent: args.onDetailPointerEvent,
    notification: args.notification,
    transitionStatus: args.options.transitionStatus,
  };
}

function handleSharedFilterExplorerAction(args: {
  interactionAction?: TerminalInteractionAction;
  adapters: TerminalInteractionContextAdapters;
  browserContext: FilterExplorerBrowserContext;
  keyContext: FilterExplorerKeyContext;
  options: FilterExplorerOptions;
  draft: FilterExplorerComposeDraft;
  updateDraft: (updater: (current: FilterExplorerComposeDraft) => FilterExplorerComposeDraft) => void;
  dispatch: (action: FilterExplorerAction) => void;
  actionEntries: FilterExplorerControllerContext["actionEntries"];
  actionTargetState: FilterExplorerControllerContext["actionTargetState"];
  allowSearch: boolean;
}): boolean {
  const {
    interactionAction,
    adapters,
    browserContext,
    keyContext,
    options,
    draft,
    updateDraft,
    dispatch,
    actionEntries,
    actionTargetState,
    allowSearch,
  } = args;
  if (!interactionAction) {
    return false;
  }

  const context = buildFilterExplorerControllerContext({
    options,
    browser: keyContext,
    draft,
    actionEntries,
    actionTargetState,
  });

  if (
    handleFilterExplorerAction({
      action: interactionAction,
      adapters,
      context,
      draft,
      keyContext,
      onOpenInspectQuery: (result) => {
        void openFilterExplorerInspectQuery({ options, keyContext, result });
      },
      onOpenInspectResult: (result) => {
        void openFilterExplorerInspectResult({ options, keyContext, result });
      },
      options,
      updateDraft,
    })
  ) {
    return true;
  }

  if (interactionAction.id === "quit") {
    options.onOutcome({ kind: "cancel" }, createFilterExplorerBrowserSnapshot(keyContext));
    return true;
  }

  if (interactionAction.id === "focus") {
    dispatch({ type: "toggle_focus" });
    return true;
  }

  if (interactionAction.id === "layout") {
    dispatch({ type: "toggle_layout" });
    return true;
  }

  if (interactionAction.id === "back" || interactionAction.id === "return") {
    const backNavigation = resolveFilterExplorerBackNavigation(browserContext);
    if (backNavigation === "pop_depth") {
      dispatch({ type: "pop_depth" });
      return true;
    }
    if (backNavigation === "leave_detail") {
      dispatch({ type: "leave_detail" });
      return true;
    }
    options.onOutcome(
      { kind: interactionAction.id === "back" ? "back" : "exitRoot" },
      createFilterExplorerBrowserSnapshot(keyContext),
    );
    return true;
  }

  if (allowSearch && interactionAction.id === "search") {
    dispatch({
      type: "set_search_mode",
      searchMode: true,
      searchInput: browserContext.effectiveState.filter,
    });
    return true;
  }

  return false;
}

function resolveFilterExplorerListRightBehavior(args: {
  browserContext: FilterExplorerBrowserContext;
  dispatch: (action: FilterExplorerAction) => void;
  draft: FilterExplorerComposeDraft;
  keyContext: FilterExplorerKeyContext;
  options: FilterExplorerOptions;
  updateDraft: (updater: (current: FilterExplorerComposeDraft) => FilterExplorerComposeDraft) => void;
}): TerminalListDetailRightBehaviorContract {
  const { browserContext, dispatch, draft, keyContext, options, updateDraft } = args;
  const context = buildFilterExplorerControllerContext({
    options,
    browser: keyContext,
    draft,
    actionEntries: [],
    actionTargetState: { activeTarget: "content", selectedActionIndex: 0 },
  });

  if (options.mode.kind === "compose") {
    const composeMode = options.mode;
    const target = resolveFilterExplorerHostTarget(options.host, keyContext.currentNode);
    const canOpenTarget = Boolean(target && (target.kind !== "scalar" || composeMode.onEditScalarTarget));

    if (canOpenTarget) {
      return {
        rightIntent: "open",
        destination: {
          availability: "available",
          perform: () => {
            void (
              openComposeScalarEditor(composeMode, target, draft, updateDraft) ||
              applyComposeCycleSelection(target, keyContext, updateDraft)
            );
          },
        },
      };
    }

    if (browserContext.currentNodeHasChildren) {
      return {
        rightIntent: "drill",
        destination: {
          availability: "available",
          perform: () => {
            dispatch({ type: "drill_in" });
          },
        },
      };
    }

    return {
      rightIntent: target ? "open" : "none",
      destination: { availability: "unavailable" },
      deadEndPolicy: "notify",
    };
  }

  const inspectTarget = resolveFilterExplorerHostTarget(options.host, keyContext.currentNode);
  const inspectPresentation = describeFilterExplorerHostNode({
    host: options.host,
    node: keyContext.currentNode,
    target: inspectTarget,
    isFocused: true,
    controller: context,
  });
  if (
    inspectTarget &&
    inspectPresentation &&
    (inspectPresentation.activationStyle === "toggle" || inspectPresentation.activationStyle === "edit") &&
    options.host.activateTarget
  ) {
    return {
      rightIntent: "open",
      destination: {
        availability: "available",
        perform: () => {
          options.host.activateTarget?.({ target: inspectTarget, controller: context, reason: "open" });
        },
      },
    };
  }
  const inspectResult = buildFilterExplorerInspectResult(options.mode, keyContext.currentNode, inspectTarget);
  const shouldOpenResult =
    inspectResult !== undefined &&
    (shouldOpenImmediateFilterExplorerInspectResult(keyContext.currentNode, inspectResult) ||
      !keyContext.currentNodeHasChildren);

  if (shouldOpenResult) {
    return {
      rightIntent: "open",
      destination: {
        availability: "available",
        perform: () => {
          void openFilterExplorerInspectResult({ options, keyContext, result: inspectResult });
        },
      },
    };
  }

  if (browserContext.currentNodeHasChildren) {
    return {
      rightIntent: "drill",
      destination: {
        availability: "available",
        perform: () => {
          dispatch({ type: "drill_in" });
        },
      },
    };
  }

  return {
    rightIntent: inspectResult ? "open" : "none",
    destination: { availability: "unavailable" },
    deadEndPolicy: "notify",
  };
}

export function handleFilterExplorerInteractionRoute(args: {
  route: FilterExplorerInteractionRoute;
  adapters: TerminalInteractionContextAdapters;
  browserContext: FilterExplorerBrowserContext;
  options: FilterExplorerOptions;
  draft: FilterExplorerComposeDraft;
  updateDraft: (updater: (current: FilterExplorerComposeDraft) => FilterExplorerComposeDraft) => void;
  dispatch: (action: FilterExplorerAction) => void;
  actionEntries: FilterExplorerControllerContext["actionEntries"];
  actionTargetState: FilterExplorerControllerContext["actionTargetState"];
  dispatchActionTarget: (action: DerivedTagTerminalActionTargetAction) => void;
  showNotification: (options: { message: string; tone?: TerminalListDetailNotificationTone }) => void;
}): void {
  const {
    actionEntries,
    actionTargetState,
    adapters,
    browserContext,
    dispatch,
    dispatchActionTarget,
    draft,
    options,
    route,
    showNotification,
    updateDraft,
  } = args;
  const {
    actionTargetInteractionAction,
    actionTargetIntent,
    detailNavigationAction,
    event,
    interactionAction,
    listNavigationAction,
    searchModeAction,
    textEntryIntent,
  } = route;
  const keyContext: FilterExplorerKeyContext = {
    ...browserContext,
    event,
  };
  const context = buildFilterExplorerControllerContext({
    options,
    browser: keyContext,
    draft,
    actionEntries,
    actionTargetState,
  });

  if (browserContext.state.searchMode) {
    if (textEntryIntent?.kind === "submit") {
      dispatch({ type: "set_search_mode", searchMode: false });
      return;
    }
    if (textEntryIntent?.kind === "deleteBackward") {
      dispatch({ type: "backspace_search" });
      return;
    }
    if (searchModeAction?.id === "cancel") {
      dispatch({ type: "clear_search" });
      dispatch({ type: "set_search_mode", searchMode: false, searchInput: "" });
      return;
    }
    if (textEntryIntent?.kind === "append") {
      dispatch({ type: "append_search", character: textEntryIntent.text });
    }
    return;
  }

  if (actionTargetIntent?.kind === "toggle_target") {
    dispatchActionTarget({ type: "toggle_target" });
    return;
  }
  if (actionTargetIntent?.kind === "leave_actions") {
    dispatchActionTarget({ type: "leave_actions" });
    return;
  }
  if (actionTargetIntent?.kind === "move_action") {
    dispatchActionTarget({
      type: "move_action",
      delta: actionTargetIntent.delta,
      actionCount: actionEntries.length,
    });
    return;
  }
  if (actionTargetIntent?.kind === "apply_action") {
    const selectedAction = actionEntries[actionTargetState.selectedActionIndex];
    if (selectedAction) {
      applyFilterExplorerActionEntry({
        actionEntry: selectedAction,
        context,
        onOpenInspectQuery: (result) => {
          void openFilterExplorerInspectQuery({ options, keyContext, result });
        },
        onOpenInspectResult: (result) => {
          void openFilterExplorerInspectResult({ options, keyContext, result });
        },
      });
    }
    return;
  }
  if (actionTargetState.activeTarget === "actions") {
    if (actionTargetInteractionAction?.id === "help") {
      void handleFilterExplorerAction({
        action: actionTargetInteractionAction,
        adapters,
        context,
        draft,
        keyContext,
        onOpenInspectQuery: (result) => {
          void openFilterExplorerInspectQuery({ options, keyContext, result });
        },
        onOpenInspectResult: (result) => {
          void openFilterExplorerInspectResult({ options, keyContext, result });
        },
        options,
        updateDraft,
      });
    }
    return;
  }

  if (browserContext.state.activePane === "detail") {
    if (
      detailNavigationAction?.kind === "viewportScrollSmall" ||
      detailNavigationAction?.kind === "viewportScrollLarge" ||
      detailNavigationAction?.kind === "viewportPage"
    ) {
      dispatch({
        type: "move_detail",
        delta: detailNavigationAction.delta,
        maxDetailScroll: browserContext.maxDetailScroll,
      });
      return;
    }
    if (detailNavigationAction?.kind === "viewportEdge") {
      dispatch({
        type: "detail_boundary",
        boundary: detailNavigationAction.boundary,
        maxDetailScroll: browserContext.maxDetailScroll,
      });
      return;
    }
    if (
      handleSharedFilterExplorerAction({
        interactionAction,
        adapters,
        browserContext,
        keyContext,
        options,
        draft,
        updateDraft,
        dispatch,
        actionEntries,
        actionTargetState,
        allowSearch: false,
      })
    ) {
      return;
    }
    return;
  }

  if (listNavigationAction?.kind === "cursorMove") {
    const isJump = Math.abs(listNavigationAction.delta) > 1;
    dispatch(
      isJump
        ? { type: "jump_selection", delta: listNavigationAction.delta }
        : { type: "move_selection", delta: listNavigationAction.delta },
    );
    return;
  }
  if (listNavigationAction?.kind === "cursorBoundary") {
    dispatch({ type: "selection_boundary", boundary: listNavigationAction.boundary });
    return;
  }
  if (listNavigationAction?.kind === "confirm") {
    applyTerminalListDetailRightBehavior({
      contract: resolveFilterExplorerListRightBehavior({
        browserContext,
        dispatch,
        draft,
        keyContext,
        options,
        updateDraft,
      }),
      showNotification,
    });
    return;
  }

  if (interactionAction?.id === "cancel") {
    if (browserContext.effectiveState.filter) {
      dispatch({ type: "clear_search" });
      return;
    }
    options.onOutcome({ kind: "cancel" }, createFilterExplorerBrowserSnapshot(keyContext));
    return;
  }
  if (
    handleSharedFilterExplorerAction({
      interactionAction,
      adapters,
      browserContext,
      keyContext,
      options,
      draft,
      updateDraft,
      dispatch,
      actionEntries,
      actionTargetState,
      allowSearch: true,
    })
  ) {
    return;
  }
}
