import React from "react";

import { usePf2eTerminalAppServices } from "../app-service-context.js";
import { showTerminalReturnDialog, useTerminalInteractionContextAdapters } from "../interaction-context-adapters.js";
import {
  SEARCH_LEFT_WIDTH,
  buildQuerySummaryLines,
  buildPendingResultDetailLines,
  buildResultDetailLines,
  buildResultLines,
  buildSearchSubtitle,
  buildWorkspaceEntries,
  buildWorkspaceEntryDetailLines,
  buildWorkspaceLines,
  clampAbsoluteSelection,
  createInitialSearchScreenState,
  formatCount,
  formatCountSummary,
  formatQueryStatus,
  formatResultPosition,
  formatSort,
  searchScreenReducer,
} from "./model.js";
import {
  buildSearchFooterText,
  buildSearchHelpLines,
  useSearchScreenInteractionRouter,
} from "./interactions.js";
import { type SearchStructuredEditorSession } from "./query-field-builder/query-field-builder-session.js";
import {
  getSearchResultWindowMetrics,
  getSessionBufferRange,
  type SearchFilterExplorerSession,
} from "./state.js";
import { useSearchFilterExplorerWorkflow } from "./filter-explorer-workflow.js";
import { useSearchSessionWorkflow } from "./session-workflow.js";
import { useSearchWorkspaceActions } from "./workspace/workspace-actions.js";
import {
  buildTerminalListDetailScreenModel,
  measureTerminalListDetailPresentation,
  useTerminalListDetailNotification,
} from "../list-detail-presentation.js";
import { useDerivedTagTerminalApp, useDerivedTagTerminalSize } from "../framework/context.js";
import type { DerivedTagTerminalTwoPaneScreenProps } from "../framework/types.js";
import type { SearchScreenProps } from "./entry-props.js";

export type SearchScreenControllerResult = {
  structuredEditorSession: SearchStructuredEditorSession | null;
  filterExplorerSession: SearchFilterExplorerSession | null;
  screen: DerivedTagTerminalTwoPaneScreenProps;
};

export function useSearchScreenController({
  entry = "editor",
  transitionStatus,
  origin = "app",
  onBack,
  ...routeEntry
}: SearchScreenProps): SearchScreenControllerResult {
  const initialQuery = entry === "results" ? undefined : routeEntry.initialQuery;
  const initialSession = entry === "results" ? routeEntry.initialSession : undefined;
  const terminal = useDerivedTagTerminalApp();
  const prompts = useTerminalInteractionContextAdapters();
  const { user } = usePf2eTerminalAppServices();
  const size = useDerivedTagTerminalSize();
  const { notification, showNotification } = useTerminalListDetailNotification();
  const initialQueryState = React.useMemo(
    () =>
      initialSession?.query ??
      (initialQuery ? user.search.createQueryFromOntologyQuery(initialQuery) : user.search.createDefaultQuery()),
    [initialQuery, initialSession, user.search],
  );
  const initialLayout: import("./state.js").SearchScreenLayout = entry === "results" ? "results" : "editor";
  const [state, dispatch] = React.useReducer(
    searchScreenReducer,
    {
      initialQuery: initialQueryState,
      initialLayout,
      initialSession,
    },
    ({ initialQuery, initialLayout, initialSession }) =>
      createInitialSearchScreenState(initialQuery, { layout: initialLayout, session: initialSession }),
  );
  const queryRef = React.useRef(initialQueryState);

  const {
    selectionJumpSize,
    pageSize,
    windowLimit: resultWindowLimit,
    preloadThreshold,
  } = getSearchResultWindowMetrics(
    measureTerminalListDetailPresentation({
      terminalWidth: size.width,
      terminalHeight: size.height,
      footerLineCount: 2,
      notification,
      transitionStatus,
      detailLines: [{ text: "" }],
      detailScroll: 0,
      layoutMode: "split",
      leftWidth: SEARCH_LEFT_WIDTH,
    }).bodyHeight,
  );

  const applyQueryUpdate = React.useCallback(
    (
      update: (
        query: import("../search/service.js").Pf2eTerminalSearchQuery,
      ) => import("../search/service.js").Pf2eTerminalSearchQuery,
    ) => {
      const nextQuery = user.search.normalizeQuery(update(queryRef.current));
      queryRef.current = nextQuery;
      dispatch({ type: "set_query", query: nextQuery });
    },
    [user.search],
  );

  React.useEffect(() => {
    queryRef.current = state.query;
  }, [state.query]);

  const {
    busy,
    countState,
    executeRequest,
    jumpToResultPosition,
    loadingMore,
    selectedResult,
    resultCount,
    chooseResultSort,
    exitSearchScreen,
  } = useSearchSessionWorkflow({
    dispatch,
    onExit: onBack,
    preloadThreshold,
    prompts,
    resultSelectedIndex: clampAbsoluteSelection(state.resultSelectedIndex, state.session?.total ?? 0),
    resultWindowLimit,
    state,
    terminal,
    user,
  });

  const workspaceEntries = buildWorkspaceEntries(state, countState);
  const workspaceSelectedIndex = Math.max(
    0,
    Math.min(state.workspaceSelectedIndex, Math.max(0, workspaceEntries.length - 1)),
  );
  const selectedWorkspaceEntry = workspaceEntries[workspaceSelectedIndex] ?? workspaceEntries[0];
  const resultSelectedIndex = clampAbsoluteSelection(state.resultSelectedIndex, resultCount);

  const showSearchHelp = React.useCallback(() => {
    void showTerminalReturnDialog(
      prompts,
      state.layout === "editor" ? "Search Editor Help" : "Search Results Help",
      buildSearchHelpLines(state, workspaceEntries, origin),
    );
  }, [origin, prompts, state, workspaceEntries]);

  const detailLines =
    state.layout === "results" && state.session
      ? selectedResult
        ? buildResultDetailLines(selectedResult, state.session, resultSelectedIndex)
        : buildPendingResultDetailLines(state.session, resultSelectedIndex)
      : selectedWorkspaceEntry
        ? buildWorkspaceEntryDetailLines(selectedWorkspaceEntry, state, countState)
        : buildQuerySummaryLines(state, countState);
  const presentationMetrics = measureTerminalListDetailPresentation({
    terminalWidth: size.width,
    terminalHeight: size.height,
    footerLineCount: 2,
    notification,
    transitionStatus,
    detailLines,
    detailScroll: state.detailScroll,
    layoutMode: "split",
    leftWidth: SEARCH_LEFT_WIDTH,
    hyperlinkSupport: terminal.capabilities.hyperlinkSupport,
  });
  const maxDetailScroll = presentationMetrics.maxDetailScroll;

  const { filterExplorerSession, openFilterExplorer } = useSearchFilterExplorerWorkflow({
    query: state.query,
    services: user,
    onUnavailable: terminal.pauseForAnyKey,
  });

  const { handleIntent, structuredEditorSession } = useSearchWorkspaceActions({
    applyQueryUpdate,
    dispatch,
    executeRequest,
    exitSearchScreen,
    jumpToResultPosition,
    maxDetailScroll,
    openFilterExplorer,
    origin,
    resultCount,
    selectedWorkspaceEntry,
    showSearchHelp,
    showNotification,
    state,
    terminal,
    prompts,
    user,
    workspaceEntries,
    chooseResultSort,
  });

  useSearchScreenInteractionRouter({
    enabled: !busy && !filterExplorerSession && !structuredEditorSession,
    origin,
    state,
    workspaceEntryCount: workspaceEntries.length,
    resultCount,
    selectionJumpSize,
    pageSize,
    maxDetailScroll,
    hasSelectedResult: Boolean(selectedResult),
    onIntent: handleIntent,
  });

  const screenModel = buildTerminalListDetailScreenModel({
    title: "Browse/Search",
    subtitle: buildSearchSubtitle(state, countState),
    activePane: state.layout === "results" ? state.activePane : "list",
    layoutMode: "split",
    leftWidth: SEARCH_LEFT_WIDTH,
    leftPane: {
      title:
        state.layout === "editor"
          ? "[EDITOR] Query"
          : state.activePane === "list"
            ? `[RESULTS] ${state.session ? `${formatResultPosition(resultSelectedIndex, state.session.total)} | Buf ${formatCount(state.session.loadedCount)} | ${formatSort(state.session.sort)}` : "No applied session"}`
            : `Results | ${state.session ? `${formatResultPosition(resultSelectedIndex, state.session.total)} | ${formatSort(state.session.sort)}` : "No applied session"}`,
      lines:
        state.layout === "editor"
          ? buildWorkspaceLines(workspaceEntries, workspaceSelectedIndex, presentationMetrics.bodyHeight)
          : buildResultLines(state.session, resultSelectedIndex, presentationMetrics.bodyHeight, loadingMore),
    },
    rightPane: {
      title:
        state.layout === "results"
          ? state.activePane === "detail"
            ? `[PREVIEW] ${selectedResult?.name ?? "Results"}`
            : `Preview | ${selectedResult?.name ?? "Results"}`
          : "Query Status",
    },
    metrics: presentationMetrics,
    footer: [
      {
        text: buildSearchFooterText(state, loadingMore, origin),
        tone: "dim",
      },
      {
        text:
          state.layout === "results" && state.session
            ? `${formatQueryStatus(state)} | ${formatResultPosition(resultSelectedIndex, state.session.total)} | Buf ${formatCount(state.session.loadedCount)} | Win ${getSessionBufferRange(state.session)}`
            : `${formatQueryStatus(state)} | ${formatCountSummary(countState, state.query)} | Query Editor`,
        tone: "accent",
      },
    ],
    notification,
    transitionStatus,
  });
  if (screenModel.kind !== "two-pane") {
    throw new Error("Browse/search screen must render as a two-pane presentation.");
  }

  return {
    structuredEditorSession,
    filterExplorerSession,
    screen: screenModel.props,
  };
}
