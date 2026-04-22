import type { TerminalInteractionContextAdapters } from "../interaction-context-adapters.js";
import type { TerminalInteractionAction } from "../interaction-bindings.js";
import { getFilterExplorerScalarClause, getFilterExplorerTargetState } from "./compose-state.js";
import {
  buildFilterExplorerInspectResult,
  openFilterExplorerInspectQuery,
  openFilterExplorerInspectResult,
  shouldOpenImmediateFilterExplorerInspectResult,
} from "./controller-inspect.js";
import {
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
}): FilterExplorerControllerContext {
  const composeMode = args.options.mode.kind === "compose" ? args.options.mode : null;
  const currentNode = args.browser.currentNode;
  const selectedTarget = composeMode?.resolveSelectionTarget(currentNode);

  return {
    model: args.options.model,
    mode: args.options.mode,
    screenTitle: resolveScreenTitle(args.options),
    browser: args.browser,
    draft: args.draft,
    selection: args.draft.selection,
    selectedTarget,
    selectedPolicyState: getFilterExplorerTargetState(selectedTarget, args.draft.selection),
    selectedScalarClause: getFilterExplorerScalarClause(selectedTarget, args.draft),
    selectedInspectResult:
      args.options.mode.kind === "inspect-and-open"
        ? buildFilterExplorerInspectResult(args.options.mode, currentNode)
        : undefined,
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
    allowSearch,
  } = args;
  if (!interactionAction) {
    return false;
  }

  const context = buildFilterExplorerControllerContext({
    options,
    browser: keyContext,
    draft,
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
    options.onExit();
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
    options.onExit();
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

export function handleFilterExplorerInteractionRoute(args: {
  route: FilterExplorerInteractionRoute;
  adapters: TerminalInteractionContextAdapters;
  browserContext: FilterExplorerBrowserContext;
  options: FilterExplorerOptions;
  draft: FilterExplorerComposeDraft;
  updateDraft: (updater: (current: FilterExplorerComposeDraft) => FilterExplorerComposeDraft) => void;
  dispatch: (action: FilterExplorerAction) => void;
}): void {
  const { adapters, browserContext, dispatch, draft, options, route, updateDraft } = args;
  const {
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

  if (browserContext.state.activePane === "detail") {
    if (detailNavigationAction?.kind === "move") {
      dispatch({
        type: "move_detail",
        delta: detailNavigationAction.delta,
        maxDetailScroll: browserContext.maxDetailScroll,
      });
      return;
    }
    if (detailNavigationAction?.kind === "boundary") {
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
        allowSearch: false,
      })
    ) {
      return;
    }
    return;
  }

  if (listNavigationAction?.kind === "move") {
    const isJump = Math.abs(listNavigationAction.delta) > 1;
    dispatch(
      isJump
        ? { type: "jump_selection", delta: listNavigationAction.delta }
        : { type: "move_selection", delta: listNavigationAction.delta },
    );
    return;
  }
  if (listNavigationAction?.kind === "boundary") {
    dispatch({ type: "selection_boundary", boundary: listNavigationAction.boundary });
    return;
  }
  if (listNavigationAction?.kind === "confirm") {
    if (options.mode.kind === "compose") {
      const target = options.mode.resolveSelectionTarget(keyContext.currentNode);
      if (
        openComposeScalarEditor(options.mode, target, draft, updateDraft) ||
        applyComposeCycleSelection(options.mode, keyContext, updateDraft)
      ) {
        return;
      }
    } else {
      const inspectResult = buildFilterExplorerInspectResult(options.mode, keyContext.currentNode);
      if (shouldOpenImmediateFilterExplorerInspectResult(keyContext.currentNode, inspectResult)) {
        if (openFilterExplorerInspectResult({ options, keyContext, result: inspectResult })) {
          return;
        }
      } else if (!keyContext.currentNodeHasChildren && inspectResult) {
        if (openFilterExplorerInspectResult({ options, keyContext, result: inspectResult })) {
          return;
        }
      }
    }

    if (browserContext.currentNodeHasChildren) {
      dispatch({ type: "drill_in" });
    } else {
      dispatch({ type: "toggle_focus" });
    }
    return;
  }

  if (interactionAction?.id === "cancel") {
    if (browserContext.effectiveState.filter) {
      dispatch({ type: "clear_search" });
      return;
    }
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
      allowSearch: true,
    })
  ) {
    return;
  }
}
