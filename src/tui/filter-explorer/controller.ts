import React from "react";

import {
  getRenderedTerminalLineCount,
  getTerminalPaneBodyHeight,
  getTerminalTwoPaneDetailWidth,
  sliceRenderedTerminalLines,
} from "../framework/rendering.js";
import type { DerivedTagTerminalListNavigationAction } from "../framework/input.js";
import { useDerivedTagTerminalSize } from "../framework/context.js";
import type { DerivedTagTerminalInputEvent } from "../framework/types.js";
import { showTerminalReturnDialog, useTerminalInteractionContextAdapters } from "../interaction-context-adapters.js";
import {
  getTerminalInteractionCycleDirection,
  type TerminalInteractionAction,
  type TerminalTextEntryIntent,
} from "../interaction-bindings.js";
import {
  createTerminalDetailInteractionContext,
  createTerminalListInteractionContext,
  createTerminalTextEntryInteractionContext,
  useTerminalInteractionContextRouter,
} from "../interaction-context-router.js";
import {
  ROUTE_TRANSITION_STATUS_KIND,
  getRouteTransitionFooterLineCount,
} from "../route-transition-status.js";
import {
  getDerivedTagTerminalTwoPaneLayoutMode,
  reduceDerivedTagTerminalTwoPaneState,
  type DerivedTagTerminalTwoPaneAction,
} from "../two-pane-state.js";
import {
  buildFilterExplorerBreadcrumb,
  buildFilterExplorerDetailLines,
  canDrillIntoFilterExplorerNode,
  cloneFilterExplorerBrowserSnapshot,
  createFilterExplorerBrowserUiState,
  drillIntoFilterExplorerBrowser,
  getFilterExplorerBrowserSelection,
  getFilterExplorerDetailTitle,
  jumpFilterExplorerSelection,
  moveFilterExplorerDetailScroll,
  moveFilterExplorerDetailScrollToBoundary,
  moveFilterExplorerSelection,
  moveFilterExplorerSelectionToBoundary,
  normalizeFilterExplorerBrowserState,
  popFilterExplorerDepth,
  setFilterExplorerFilter,
} from "./browser.js";
import {
  createEmptyFilterExplorerComposeDraft,
  cloneFilterExplorerComposeDraft,
  cloneFilterExplorerSelectionMap,
  getFilterExplorerScalarClause,
  getFilterExplorerTargetState,
  isFilterExplorerScalarTarget,
  normalizeFilterExplorerComposeDraft,
  setFilterExplorerScalarClause,
  toggleFilterExplorerTargetSelection,
} from "./compose-state.js";
import {
  buildFilterExplorerCommandEntries,
  buildFilterExplorerComposeDetailLines,
  buildFilterExplorerHelpLines,
  getFilterExplorerInteractionActions,
} from "./screen-models.js";
import type { MetadataFilterNode } from "../../domain/metadata-types.js";
import type {
  FilterExplorerBrowserContext,
  FilterExplorerBrowserSnapshot,
  FilterExplorerBrowserUiState,
  FilterExplorerComposeDraft,
  FilterExplorerComposeMode,
  FilterExplorerControllerContext,
  FilterExplorerInspectAndOpenMode,
  FilterExplorerInspectResult,
  FilterExplorerNode,
  FilterExplorerOptions,
  FilterExplorerQueryTarget,
  FilterExplorerScalarClause,
} from "./types.js";
import {
  FILTER_EXPLORER_LAUNCH_INTENT,
  type FilterExplorerLaunchIntent,
} from "./types.js";

type FilterExplorerAction =
  | DerivedTagTerminalTwoPaneAction
  | { type: "normalize" }
  | { type: "set_search_mode"; searchMode: boolean; searchInput?: string }
  | { type: "append_search"; character: string }
  | { type: "backspace_search" }
  | { type: "clear_search" }
  | { type: "move_selection"; delta: number }
  | { type: "jump_selection"; delta: number }
  | { type: "selection_boundary"; boundary: "start" | "end" }
  | { type: "drill_in" }
  | { type: "pop_depth" }
  | { type: "move_detail"; delta: number; maxDetailScroll: number }
  | { type: "detail_boundary"; boundary: "start" | "end"; maxDetailScroll: number };

type FilterExplorerKeyContext = FilterExplorerBrowserContext & {
  event: DerivedTagTerminalInputEvent;
};

type FilterExplorerInteractionRoute = {
  event: DerivedTagTerminalInputEvent;
  interactionAction?: TerminalInteractionAction;
  searchModeAction?: { id: "cancel" };
  textEntryIntent?: TerminalTextEntryIntent;
  listNavigationAction?: DerivedTagTerminalListNavigationAction;
  detailNavigationAction?: DerivedTagTerminalListNavigationAction;
};

type FilterExplorerBackNavigationOutcome = "leave_detail" | "pop_depth" | "exit";

function createFilterExplorerBrowserSnapshot(
  context: Pick<FilterExplorerBrowserContext, "state" | "effectiveState" | "layoutMode">,
): FilterExplorerBrowserSnapshot {
  return cloneFilterExplorerBrowserSnapshot({
    activePane: context.state.activePane,
    browserState: context.effectiveState,
    layoutMode: context.layoutMode,
    searchInput: context.state.searchInput,
    searchMode: context.state.searchMode,
  });
}

function resolveFilterExplorerBackNavigation(
  context: Pick<FilterExplorerBrowserContext, "state" | "effectiveState">,
): FilterExplorerBackNavigationOutcome {
  if (context.state.activePane === "detail") {
    return "leave_detail";
  }

  const nextState = popFilterExplorerDepth(context.effectiveState);
  return nextState.depth === context.effectiveState.depth ? "exit" : "pop_depth";
}

function reduceExplorerTwoPaneState(
  state: FilterExplorerBrowserUiState,
  action: DerivedTagTerminalTwoPaneAction,
): Pick<FilterExplorerBrowserUiState, "activePane" | "layoutMode" | "browserState"> {
  const next = reduceDerivedTagTerminalTwoPaneState(
    {
      activePane: state.activePane,
      detailScroll: state.browserState.detailScroll,
      layoutMode: state.layoutMode,
    },
    action,
  );

  return {
    activePane: next.activePane,
    layoutMode: next.layoutMode,
    browserState: {
      ...state.browserState,
      detailScroll: next.detailScroll,
    },
  };
}

function filterExplorerReducer(
  model: FilterExplorerOptions["model"],
  state: FilterExplorerBrowserUiState,
  action: FilterExplorerAction,
): FilterExplorerBrowserUiState {
  switch (action.type) {
    case "toggle_focus":
    case "toggle_layout":
    case "leave_detail":
      return {
        ...state,
        ...reduceExplorerTwoPaneState(state, action),
      };
    case "normalize":
      return {
        ...state,
        browserState: normalizeFilterExplorerBrowserState(model, state.browserState),
      };
    case "set_search_mode":
      return {
        ...state,
        searchInput: action.searchInput ?? state.searchInput,
        searchMode: action.searchMode,
      };
    case "append_search": {
      const searchInput = state.searchInput + action.character;
      return {
        ...state,
        browserState: setFilterExplorerFilter(model, state.browserState, searchInput),
        searchInput,
      };
    }
    case "backspace_search": {
      const searchInput = state.searchInput.slice(0, -1);
      return {
        ...state,
        browserState: setFilterExplorerFilter(model, state.browserState, searchInput),
        searchInput,
      };
    }
    case "clear_search":
      return {
        ...state,
        browserState: setFilterExplorerFilter(model, state.browserState, ""),
        searchInput: "",
      };
    case "move_selection":
      return {
        ...state,
        browserState: moveFilterExplorerSelection(model, state.browserState, action.delta),
      };
    case "jump_selection":
      return {
        ...state,
        browserState: jumpFilterExplorerSelection(model, state.browserState, action.delta),
      };
    case "selection_boundary":
      return {
        ...state,
        browserState: moveFilterExplorerSelectionToBoundary(model, state.browserState, action.boundary),
      };
    case "drill_in":
      return {
        ...state,
        activePane: "list",
        browserState: drillIntoFilterExplorerBrowser(model, state.browserState),
        layoutMode: "split",
        searchInput: "",
        searchMode: false,
      };
    case "pop_depth":
      return {
        ...state,
        activePane: "list",
        browserState: popFilterExplorerDepth(state.browserState),
        layoutMode: "split",
        searchInput: "",
        searchMode: false,
      };
    case "move_detail":
      return {
        ...state,
        ...reduceExplorerTwoPaneState(state, action),
        browserState: moveFilterExplorerDetailScroll(state.browserState, action.delta, action.maxDetailScroll),
      };
    case "detail_boundary":
      return {
        ...state,
        ...reduceExplorerTwoPaneState(state, action),
        browserState: moveFilterExplorerDetailScrollToBoundary(
          state.browserState,
          action.boundary,
          action.maxDetailScroll,
        ),
      };
    default:
      return state;
  }
}

function resolveFilterExplorerLaunchIntent(
  mode: FilterExplorerInspectAndOpenMode,
  query: FilterExplorerQueryTarget,
): FilterExplorerLaunchIntent {
  if (query.kind !== "listRecords") {
    return FILTER_EXPLORER_LAUNCH_INTENT.EDITOR;
  }

  return mode.defaultListRecordLaunchIntent ?? FILTER_EXPLORER_LAUNCH_INTENT.RESULTS;
}

function buildInspectResult(
  model: FilterExplorerOptions["model"],
  mode: FilterExplorerInspectAndOpenMode,
  node: FilterExplorerNode | undefined,
): FilterExplorerInspectResult | undefined {
  if (!node?.query) {
    return undefined;
  }

  return {
    node,
    query: node.query,
    target: mode.resolveInspectTarget?.(node),
    launchIntent: resolveFilterExplorerLaunchIntent(mode, node.query),
  };
}

function parseInspectScalarTargetKey(key: string): { field: "actorMetric" | "itemMetric"; metric: string } | null {
  const match = key.match(/^(actorMetric|itemMetric):(.+)$/);
  if (!match) {
    return null;
  }

  return {
    field: match[1] as "actorMetric" | "itemMetric",
    metric: match[2]!,
  };
}

function buildInspectScalarPredicate(target: Exclude<FilterExplorerInspectResult["target"], undefined>, clause: FilterExplorerScalarClause): MetadataFilterNode | null {
  if (!isFilterExplorerScalarTarget(target) || target.valueType !== "number") {
    return null;
  }

  const metricTarget = parseInspectScalarTargetKey(target.key);
  if (!metricTarget) {
    return null;
  }

  if (clause.operator === "between") {
    return {
      and: [
        {
          field: metricTarget.field,
          metric: metricTarget.metric,
          op: ">=",
          value: clause.min,
        },
        {
          field: metricTarget.field,
          metric: metricTarget.metric,
          op: "<=",
          value: clause.max,
        },
      ],
    };
  }

  const operator =
    clause.operator === "eq"
      ? "=="
      : clause.operator === "neq"
        ? "!="
        : clause.operator === "gte"
          ? ">="
          : "<=";

  return {
    field: metricTarget.field,
    metric: metricTarget.metric,
    op: operator,
    value: clause.value as number,
  };
}

function formatInspectScalarClauseSummary(clause: FilterExplorerScalarClause): string {
  if (clause.operator === "between") {
    return `between ${clause.min} and ${clause.max}`;
  }

  const operator =
    clause.operator === "eq"
      ? "="
      : clause.operator === "neq"
        ? "!="
        : clause.operator === "gte"
          ? ">="
          : "<=";
  return `${operator} ${clause.value}`;
}

function buildCompiledInspectResult(
  result: FilterExplorerInspectResult,
  clause: FilterExplorerScalarClause,
): FilterExplorerInspectResult | null {
  if (result.query.kind !== "listRecords" || !isFilterExplorerScalarTarget(result.target)) {
    return null;
  }

  const metadata = buildInspectScalarPredicate(result.target, clause);
  if (!metadata) {
    return null;
  }

  return {
    ...result,
    query: {
      ...result.query,
      label: `Browse records where ${result.target.subjectLabel} ${formatInspectScalarClauseSummary(clause)}`,
      filters: {
        ...result.query.filters,
        metadata,
      },
    },
    launchIntent: FILTER_EXPLORER_LAUNCH_INTENT.RESULTS,
  };
}

function openInspectResultDirect(
  options: FilterExplorerOptions,
  keyContext: FilterExplorerKeyContext,
  result: FilterExplorerInspectResult | undefined,
): boolean {
  if (options.mode.kind !== "inspect-and-open" || !result) {
    return false;
  }

  const snapshot = createFilterExplorerBrowserSnapshot(keyContext);
  if (options.mode.onOpenInspectResult) {
    options.mode.onOpenInspectResult(result, snapshot);
    return true;
  }
  if (options.mode.onOpenQuery) {
    options.mode.onOpenQuery(result.query, snapshot, result.launchIntent);
    return true;
  }
  return false;
}

function openInspectResult(
  options: FilterExplorerOptions,
  keyContext: FilterExplorerKeyContext,
  result: FilterExplorerInspectResult | undefined,
): boolean {
  if (
    options.mode.kind === "inspect-and-open" &&
    result &&
    isFilterExplorerScalarTarget(result.target) &&
    options.mode.onEditScalarTarget
  ) {
    void Promise.resolve(
      options.mode.onEditScalarTarget({
        target: result.target,
        draft: createEmptyFilterExplorerComposeDraft(),
      }),
    ).then((nextClause) => {
      if (nextClause === undefined || nextClause === null) {
        return;
      }

      const compiledResult = buildCompiledInspectResult(result, nextClause);
      if (!compiledResult) {
        return;
      }

      openInspectResultDirect(options, keyContext, compiledResult);
    });
    return true;
  }

  return openInspectResultDirect(options, keyContext, result);
}

function openInspectQuery(
  options: FilterExplorerOptions,
  keyContext: FilterExplorerKeyContext,
  result: FilterExplorerInspectResult | undefined,
): boolean {
  if (options.mode.kind !== "inspect-and-open" || !result) {
    return false;
  }

  const snapshot = createFilterExplorerBrowserSnapshot(keyContext);
  if (options.mode.onOpenQuery) {
    options.mode.onOpenQuery(result.query, snapshot, FILTER_EXPLORER_LAUNCH_INTENT.EDITOR);
    return true;
  }
  if (options.mode.onOpenInspectResult) {
    options.mode.onOpenInspectResult(
      {
        ...result,
        launchIntent: FILTER_EXPLORER_LAUNCH_INTENT.EDITOR,
      },
      snapshot,
    );
    return true;
  }
  return false;
}

function getNodeInspectResult(
  options: FilterExplorerOptions,
  node: FilterExplorerNode | undefined,
): FilterExplorerInspectResult | undefined {
  return options.mode.kind === "inspect-and-open"
    ? buildInspectResult(options.model, options.mode, node)
    : undefined;
}

function shouldOpenImmediateInspectResult(
  model: FilterExplorerOptions["model"],
  node: FilterExplorerNode | undefined,
  result: FilterExplorerInspectResult | undefined,
): boolean {
  return Boolean(
    result &&
      result.query.kind === "listRecords" &&
      node?.kind !== "record",
  );
}

function resolveScreenTitle(options: FilterExplorerOptions): string {
  return options.title ?? (options.mode.kind === "compose" ? "Filter Explorer" : options.model.label);
}

function useComposeSelectionState(mode: FilterExplorerComposeMode | null): [
  FilterExplorerComposeDraft,
  (updater: (current: FilterExplorerComposeDraft) => FilterExplorerComposeDraft) => void,
] {
  const isControlled = mode?.draft !== undefined;
  const [internalDraft, setInternalDraft] = React.useState<FilterExplorerComposeDraft>(() =>
    normalizeFilterExplorerComposeDraft(mode?.draft ?? mode?.initialDraft, mode?.selection ?? mode?.initialSelection),
  );

  React.useEffect(() => {
    setInternalDraft(
      normalizeFilterExplorerComposeDraft(mode?.draft ?? mode?.initialDraft, mode?.selection ?? mode?.initialSelection),
    );
  }, [mode?.draft, mode?.initialDraft, mode?.initialSelection, mode?.selection]);

  const currentDraft = React.useMemo(
    () => normalizeFilterExplorerComposeDraft(mode?.draft ?? internalDraft, mode?.selection),
    [internalDraft, mode?.draft, mode?.selection],
  );
  const updateSelection = React.useCallback(
    (updater: (current: FilterExplorerComposeDraft) => FilterExplorerComposeDraft) => {
      if (!mode) {
        return;
      }

      const next = normalizeFilterExplorerComposeDraft(updater(currentDraft));
      if (!isControlled) {
        setInternalDraft(next);
      }
      mode.onDraftChange?.(cloneFilterExplorerComposeDraft(next));
      mode.onSelectionChange?.(cloneFilterExplorerSelectionMap(next.selection));
    },
    [currentDraft, isControlled, mode],
  );

  return [currentDraft, updateSelection];
}

function createFilterExplorerContext(args: {
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
        ? buildInspectResult(args.options.model, args.options.mode, currentNode)
        : undefined,
    transitionStatus: args.options.transitionStatus,
  };
}

function applyComposeCycleSelection(
  composeMode: FilterExplorerComposeMode,
  keyContext: Pick<FilterExplorerKeyContext, "currentNode" | "event">,
  updateDraft: (updater: (current: FilterExplorerComposeDraft) => FilterExplorerComposeDraft) => void,
): boolean {
  const cycleDirection = getTerminalInteractionCycleDirection(keyContext.event, { id: "cycle" });
  if (!cycleDirection) {
    return false;
  }

  const target = composeMode.resolveSelectionTarget(keyContext.currentNode);
  if (!target || target.kind === "scalar") {
    return false;
  }

  updateDraft((current) => ({
    ...current,
    selection: toggleFilterExplorerTargetSelection(target, current.selection, cycleDirection),
  }));
  return true;
}

function openComposeScalarEditor(
  composeMode: FilterExplorerComposeMode,
  target: ReturnType<FilterExplorerComposeMode["resolveSelectionTarget"]>,
  draft: FilterExplorerComposeDraft,
  updateDraft: (updater: (current: FilterExplorerComposeDraft) => FilterExplorerComposeDraft) => void,
): boolean {
  if (!isFilterExplorerScalarTarget(target) || !composeMode.onEditScalarTarget) {
    return false;
  }

  void Promise.resolve(
    composeMode.onEditScalarTarget({
      target,
      currentClause: getFilterExplorerScalarClause(target, draft),
      draft: cloneFilterExplorerComposeDraft(draft),
    }),
  ).then((nextClause) => {
    if (nextClause === undefined) {
      return;
    }
    updateDraft((current) => setFilterExplorerScalarClause(target, nextClause, current));
  });
  return true;
}

function shouldExitAtRootDepth(options: FilterExplorerOptions, keyContext: FilterExplorerKeyContext): boolean {
  return (
    options.mode.kind === "compose" &&
    options.exitAtRootDepth === true &&
    keyContext.state.activePane === "list" &&
    keyContext.effectiveState.depth === (options.rootDepth ?? 0)
  );
}

export function useFilterExplorerController(options: FilterExplorerOptions): FilterExplorerControllerContext {
  const adapters = useTerminalInteractionContextAdapters();
  const composeMode = options.mode.kind === "compose" ? options.mode : null;
  const [draft, updateDraft] = useComposeSelectionState(composeMode);
  const size = useDerivedTagTerminalSize();
  const [state, dispatch] = React.useReducer(
    (current: FilterExplorerBrowserUiState, action: FilterExplorerAction) =>
      filterExplorerReducer(options.model, current, action),
    { model: options.model, initialSnapshot: options.initialSnapshot },
    ({ model, initialSnapshot }) => createFilterExplorerBrowserUiState(model, initialSnapshot),
  );

  React.useEffect(() => {
    dispatch({ type: "normalize" });
  }, [options.model]);

  const layoutMode = getDerivedTagTerminalTwoPaneLayoutMode({
    activePane: state.activePane,
    detailScroll: state.browserState.detailScroll,
    layoutMode: state.layoutMode,
  });
  const normalizedBrowserState = normalizeFilterExplorerBrowserState(options.model, state.browserState);
  const selection = getFilterExplorerBrowserSelection(options.model, normalizedBrowserState);
  const detailLines =
    composeMode
      ? buildFilterExplorerComposeDetailLines({
          mode: composeMode,
          draft,
          currentNodeLabel: selection.currentNode?.label,
          selectedTarget: composeMode.resolveSelectionTarget(selection.currentNode),
          selectedPolicyState: getFilterExplorerTargetState(
            composeMode.resolveSelectionTarget(selection.currentNode),
            draft.selection,
          ),
          selectedScalarClause: getFilterExplorerScalarClause(
            composeMode.resolveSelectionTarget(selection.currentNode),
            draft,
          ),
          baseDetailLines:
            selection.currentNode?.detailLines ?? [{ text: "No ontology entry selected.", tone: "dim" }],
        })
      : buildFilterExplorerDetailLines(options.model, normalizedBrowserState);
  const detailTitle = composeMode
    ? composeMode.detailTitle ?? "Detail"
    : getFilterExplorerDetailTitle(options.model, normalizedBrowserState);
  const bodyHeight = Math.max(
    1,
    getTerminalPaneBodyHeight(size.height, {
      hasSubtitle: true,
      footerLineCount: 2 + getRouteTransitionFooterLineCount(options.transitionStatus),
    }),
  );
  const detailWidth = getTerminalTwoPaneDetailWidth(size.width, layoutMode, 46);
  const renderedDetailLineCount = getRenderedTerminalLineCount(detailLines, detailWidth);
  const maxDetailScroll = Math.max(0, renderedDetailLineCount - bodyHeight);
  const effectiveState =
    normalizedBrowserState.detailScroll > maxDetailScroll
      ? { ...normalizedBrowserState, detailScroll: maxDetailScroll }
      : normalizedBrowserState;
  const currentNodeHasChildren = canDrillIntoFilterExplorerNode(selection.currentNode);
  const searchIndicator = state.searchMode
    ? ` | /${state.searchInput}`
    : effectiveState.filter
      ? ` | /${effectiveState.filter}`
      : "";

  const browserContext: FilterExplorerBrowserContext = {
    state,
    effectiveState,
    selection,
    currentNode: selection.currentNode,
    currentNodeHasChildren,
    breadcrumb: buildFilterExplorerBreadcrumb(options.model, selection),
    bodyHeight,
    detailWidth,
    detailLines,
    visibleDetailLines: sliceRenderedTerminalLines(detailLines, detailWidth, effectiveState.detailScroll, bodyHeight),
    detailTitle,
    layoutMode,
    maxDetailScroll,
    detailJumpSize: Math.max(1, Math.floor(bodyHeight / 2)),
    detailPageSize: Math.max(1, bodyHeight - 1),
    selectionJumpSize: Math.max(1, Math.floor(bodyHeight / 2)),
    searchIndicator,
  };

  const interactionActions = getFilterExplorerInteractionActions(options.mode, browserContext);

  const handleRoute = React.useCallback(
    (route: FilterExplorerInteractionRoute) => {
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
        if (interactionAction && handleExplorerAction(options, interactionAction, keyContext, draft, updateDraft, adapters)) {
          return;
        }
        if (interactionAction?.id === "quit") {
          options.onExit();
          return;
        }
        if (interactionAction?.id === "focus") {
          dispatch({ type: "toggle_focus" });
          return;
        }
        if (interactionAction?.id === "layout") {
          dispatch({ type: "toggle_layout" });
          return;
        }
        if (interactionAction?.id === "back" || interactionAction?.id === "return") {
          const backNavigation = resolveFilterExplorerBackNavigation(browserContext);
          if (backNavigation === "leave_detail") {
            dispatch({ type: "leave_detail" });
            return;
          }
          if (backNavigation === "pop_depth") {
            dispatch({ type: "pop_depth" });
            return;
          }
          options.onExit();
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
          const inspectResult = getNodeInspectResult(options, keyContext.currentNode);
          if (shouldOpenImmediateInspectResult(options.model, keyContext.currentNode, inspectResult)) {
            if (openInspectResult(options, keyContext, inspectResult)) {
              return;
            }
          } else if (!keyContext.currentNodeHasChildren && inspectResult) {
            if (openInspectResult(options, keyContext, inspectResult)) {
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
      if (interactionAction && handleExplorerAction(options, interactionAction, keyContext, draft, updateDraft, adapters)) {
        return;
      }
      if (interactionAction?.id === "focus") {
        dispatch({ type: "toggle_focus" });
        return;
      }
      if (interactionAction?.id === "layout") {
        dispatch({ type: "toggle_layout" });
        return;
      }
      if (interactionAction?.id === "back" || interactionAction?.id === "return") {
        const backNavigation = resolveFilterExplorerBackNavigation(browserContext);
        if (backNavigation === "pop_depth") {
          dispatch({ type: "pop_depth" });
        } else if (backNavigation === "leave_detail") {
          dispatch({ type: "leave_detail" });
        } else {
          options.onExit();
        }
        return;
      }
      if (interactionAction?.id === "quit") {
        options.onExit();
        return;
      }
      if (interactionAction?.id === "search") {
        dispatch({
          type: "set_search_mode",
          searchMode: true,
          searchInput: browserContext.effectiveState.filter,
        });
      }
    },
    [adapters, browserContext, dispatch, draft, options, updateDraft],
  );

  useTerminalInteractionContextRouter({
    contexts: [
      createTerminalListInteractionContext("list", {
        interactionActions,
        pageSize: browserContext.detailPageSize,
        jumpSize: browserContext.selectionJumpSize,
        includeConfirmKeys: true,
        includeHorizontalConfirmKeys: true,
      }),
      createTerminalDetailInteractionContext("detail", {
        interactionActions,
        pageSize: browserContext.detailPageSize,
        jumpSize: browserContext.detailJumpSize,
        includeCancelKeys: true,
        includeHorizontalCancelKeys: true,
      }),
      createTerminalTextEntryInteractionContext("textEntry", [{ id: "cancel" }]),
    ],
    onRoute: (routes) => {
      if (options.transitionStatus?.kind === ROUTE_TRANSITION_STATUS_KIND.PENDING) {
        return;
      }
      handleRoute({
        event: routes.textEntry.event,
        interactionAction:
          browserContext.state.activePane === "detail" ? routes.detail.interactionAction : routes.list.interactionAction,
        searchModeAction: routes.textEntry.interactionAction?.id === "cancel" ? { id: "cancel" } : undefined,
        textEntryIntent: routes.textEntry.textEntryIntent,
        listNavigationAction: routes.list.navigationAction,
        detailNavigationAction: routes.detail.navigationAction,
      });
    },
  });

  return createFilterExplorerContext({
    options,
    browser: browserContext,
    draft,
  });
}

function handleExplorerAction(
  options: FilterExplorerOptions,
  action: TerminalInteractionAction,
  keyContext: FilterExplorerKeyContext,
  draft: FilterExplorerComposeDraft,
  updateDraft: (updater: (current: FilterExplorerComposeDraft) => FilterExplorerComposeDraft) => void,
  adapters: ReturnType<typeof useTerminalInteractionContextAdapters>,
): boolean {
  const context = createFilterExplorerContext({
    options,
    browser: keyContext,
    draft,
  });

  if (action.id === "back" && shouldExitAtRootDepth(options, keyContext)) {
    options.onExit();
    return true;
  }

  if (action.id === "cycle" && options.mode.kind === "compose") {
    const target = options.mode.resolveSelectionTarget(keyContext.currentNode);
    return (
      openComposeScalarEditor(options.mode, target, draft, updateDraft) ||
      applyComposeCycleSelection(options.mode, keyContext, updateDraft)
    );
  }

  if (action.id === "help") {
    void showTerminalReturnDialog(adapters, `${context.screenTitle} Help`, buildFilterExplorerHelpLines(context));
    return true;
  }

  if (action.id === "commands" && options.mode.kind === "inspect-and-open") {
    const commandEntries = buildFilterExplorerCommandEntries(context);
    if (commandEntries.length === 0) {
      return true;
    }

    void adapters
      .promptCommandPalette({
        title: `${context.screenTitle} Commands`,
        prompt: "Filter explorer commands",
        entries: commandEntries,
      })
      .then((selected) => {
        if (selected === "openSelection" || selected === "openResults") {
          openInspectResult(options, keyContext, getNodeInspectResult(options, keyContext.currentNode));
          return;
        }
        if (selected === "openQuery") {
          openInspectQuery(options, keyContext, getNodeInspectResult(options, keyContext.currentNode));
        }
      });
    return true;
  }

  return false;
}
