import React from "react";

import {
  measureTerminalListDetailPresentation,
  useTerminalListDetailNotification,
  useTerminalListDetailInteractionRouter,
} from "../list-detail-presentation.js";
import { useDerivedTagTerminalApp, useDerivedTagTerminalSize } from "../framework/context.js";
import { useTerminalInteractionContextAdapters } from "../interaction-context-adapters.js";
import { getDerivedTagTerminalTwoPaneLayoutMode } from "../two-pane-state.js";
import {
  buildFilterExplorerBreadcrumb,
  buildFilterExplorerDetailLines,
  canDrillIntoFilterExplorerNode,
  createFilterExplorerBrowserUiState,
  getFilterExplorerBrowserSelection,
  getFilterExplorerDetailTitle,
  normalizeFilterExplorerBrowserState,
} from "./browser.js";
import {
  cloneFilterExplorerComposeDraft,
  cloneFilterExplorerSelectionMap,
  getFilterExplorerScalarClause,
  getFilterExplorerTargetState,
  normalizeFilterExplorerComposeDraft,
} from "./compose-state.js";
import {
  buildFilterExplorerControllerContext,
  handleFilterExplorerInteractionRoute,
} from "./controller-routing.js";
import {
  type FilterExplorerAction,
  filterExplorerReducer,
} from "./controller-state.js";
import type { FilterExplorerInteractionRoute } from "./controller-types.js";
import {
  buildFilterExplorerComposeDetailLines,
  getFilterExplorerInteractionActions,
} from "./screen-models.js";
import type {
  FilterExplorerBrowserContext,
  FilterExplorerBrowserUiState,
  FilterExplorerComposeDraft,
  FilterExplorerComposeMode,
  FilterExplorerControllerContext,
  FilterExplorerOptions,
} from "./types.js";

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

export function useFilterExplorerController(options: FilterExplorerOptions): FilterExplorerControllerContext {
  const adapters = useTerminalInteractionContextAdapters();
  const terminal = useDerivedTagTerminalApp();
  const { notification, showNotification } = useTerminalListDetailNotification();
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
    notification,
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

  const interactionActions = getFilterExplorerInteractionActions(options.mode, browserContext, options.discovery);

  const handleRoute = React.useCallback(
    (route: FilterExplorerInteractionRoute) => {
      handleFilterExplorerInteractionRoute({
        route,
        adapters,
        browserContext,
        options,
        draft,
        updateDraft,
        dispatch,
        showNotification,
      });
    },
    [adapters, browserContext, dispatch, draft, options, showNotification, updateDraft],
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

  return buildFilterExplorerControllerContext({
    options,
    browser: browserContext,
    draft,
    notification,
  });
}
