import React from "react";

import {
  measureTerminalListDetailPresentation,
  useTerminalListDetailInteractionRouter,
} from "../list-detail-presentation.js";
import type { DerivedTagTerminalListNavigationAction } from "../framework/input.js";
import { useDerivedTagTerminalApp, useDerivedTagTerminalSize } from "../framework/context.js";
import type { DerivedTagTerminalInputEvent } from "../framework/types.js";
import { useTerminalInteractionContextAdapters } from "../interaction-context-adapters.js";
import {
  type TerminalInteractionAction,
  type TerminalTextEntryIntent,
} from "../interaction-bindings.js";
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
} from "./compose-state.js";
import {
  buildFilterExplorerComposeDetailLines,
  getFilterExplorerInteractionActions,
} from "./screen-models.js";
import type { MetadataFilterNode } from "../../search/filters/types.js";
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
  FilterExplorerQueryOpenIntent,
  FilterExplorerQueryTarget,
  FilterExplorerScalarClause,
} from "./types.js";
import {
  FILTER_EXPLORER_LAUNCH_INTENT,
  type FilterExplorerLaunchIntent,
} from "./types.js";
import {
  applyComposeCycleSelection,
  handleFilterExplorerAction,
  openComposeScalarEditor,
} from "./workflow-actions.js";

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
  if (options.mode.onOpenQueryIntent) {
    options.mode.onOpenQueryIntent(buildFilterExplorerQueryOpenIntent(result.query, result.launchIntent), snapshot);
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
  if (options.mode.onOpenQueryIntent) {
    options.mode.onOpenQueryIntent(
      buildFilterExplorerQueryOpenIntent(result.query, FILTER_EXPLORER_LAUNCH_INTENT.EDITOR),
      snapshot,
    );
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

export function useFilterExplorerController(options: FilterExplorerOptions): FilterExplorerControllerContext {
  const adapters = useTerminalInteractionContextAdapters();
  const terminal = useDerivedTagTerminalApp();
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
  const presentationMetrics = measureTerminalListDetailPresentation({
    terminalWidth: size.width,
    terminalHeight: size.height,
    footerLineCount: 2,
    transitionStatus: options.transitionStatus,
    detailLines,
    detailScroll: normalizedBrowserState.detailScroll,
    layoutMode,
    leftWidth: 46,
    hyperlinkSupport: terminal.capabilities.hyperlinkSupport,
  });
  const maxDetailScroll = presentationMetrics.maxDetailScroll;
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
    bodyHeight: presentationMetrics.bodyHeight,
    detailWidth: presentationMetrics.detailWidth,
    detailLines,
    visibleDetailLines: presentationMetrics.visibleDetailLines,
    detailTitle,
    layoutMode,
    maxDetailScroll,
    detailJumpSize: presentationMetrics.detailJumpSize,
    detailPageSize: presentationMetrics.pageSize,
    selectionJumpSize: presentationMetrics.selectionJumpSize,
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
        if (
          interactionAction &&
          handleFilterExplorerAction({
            action: interactionAction,
            adapters,
            context: createFilterExplorerContext({
              options,
              browser: keyContext,
              draft,
            }),
            draft,
            keyContext,
            onOpenInspectQuery: (result) => {
              void openInspectQuery(options, keyContext, result);
            },
            onOpenInspectResult: (result) => {
              void openInspectResult(options, keyContext, result);
            },
            options,
            updateDraft,
          })
        ) {
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
      if (
        interactionAction &&
        handleFilterExplorerAction({
          action: interactionAction,
          adapters,
          context: createFilterExplorerContext({
            options,
            browser: keyContext,
            draft,
          }),
          draft,
          keyContext,
          onOpenInspectQuery: (result) => {
            void openInspectQuery(options, keyContext, result);
          },
          onOpenInspectResult: (result) => {
            void openInspectResult(options, keyContext, result);
          },
          options,
          updateDraft,
        })
      ) {
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

  useTerminalListDetailInteractionRouter({
    list: {
      interactionActions,
      pageSize: browserContext.detailPageSize,
      jumpSize: browserContext.selectionJumpSize,
      includeConfirmKeys: true,
      includeHorizontalConfirmKeys: true,
    },
    detail: {
      interactionActions,
      pageSize: browserContext.detailPageSize,
      jumpSize: browserContext.detailJumpSize,
      includeCancelKeys: true,
      includeHorizontalCancelKeys: true,
    },
    textEntry: {
      interactionActions: [{ id: "cancel" }],
    },
    transitionStatus: options.transitionStatus,
    onRoute: (routes) => {
      handleRoute({
        event: routes.textEntry?.event ?? routes.list.event,
        interactionAction:
          browserContext.state.activePane === "detail" ? routes.detail.interactionAction : routes.list.interactionAction,
        searchModeAction: routes.textEntry?.interactionAction?.id === "cancel" ? { id: "cancel" } : undefined,
        textEntryIntent: routes.textEntry?.textEntryIntent,
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
function buildFilterExplorerQueryOpenIntent(
  query: FilterExplorerQueryTarget,
  launchIntent: FilterExplorerLaunchIntent,
): FilterExplorerQueryOpenIntent {
  return {
    query,
    launchIntent,
  };
}
