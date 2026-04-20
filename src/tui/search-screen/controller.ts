import React from "react";

import type { OntologyNodeQuery } from "../../domain/index.js";
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
import { type SearchStructuredEditorSession } from "./query-field-builder-session.js";
import {
  getSearchResultWindowMetrics,
  getSessionBufferRange,
  type SearchQueryFieldPickerSession,
} from "./state.js";
import { useSearchQueryFieldPickerWorkflow } from "./query-picker-workflow.js";
import { useSearchSessionWorkflow } from "./session-workflow.js";
import { useSearchWorkspaceActions } from "./workspace-actions.js";
import type { SearchScreenOrigin } from "./workflow-types.js";
import {
  getRenderedTerminalLineCount,
  getTerminalPaneBodyHeight,
  getTerminalTwoPaneDetailWidth,
  sliceRenderedTerminalLines,
} from "../framework/rendering.js";
import { useDerivedTagTerminalApp, useDerivedTagTerminalSize } from "../framework/context.js";
import type { DerivedTagTerminalTwoPaneScreenProps } from "../framework/types.js";

export type SearchScreenControllerResult = {
  structuredEditorSession: SearchStructuredEditorSession | null;
  selectionPickerSession: SearchQueryFieldPickerSession | null;
  screen: DerivedTagTerminalTwoPaneScreenProps;
};

export function useSearchScreenController({
  initialQuery,
  origin = "app",
  onBack,
}: {
  initialQuery?: OntologyNodeQuery;
  origin?: SearchScreenOrigin;
  onBack: () => void;
}): SearchScreenControllerResult {
  const terminal = useDerivedTagTerminalApp();
  const prompts = useTerminalInteractionContextAdapters();
  const { user } = usePf2eTerminalAppServices();
  const size = useDerivedTagTerminalSize();
  const initialQueryState = React.useMemo(
    () => (initialQuery ? user.search.createQueryFromOntologyQuery(initialQuery) : user.search.createDefaultQuery()),
    [initialQuery, user.search],
  );
  const [state, dispatch] = React.useReducer(searchScreenReducer, initialQueryState, createInitialSearchScreenState);
  const queryRef = React.useRef(initialQueryState);

  const bodyHeight = Math.max(
    1,
    getTerminalPaneBodyHeight(size.height, {
      hasSubtitle: true,
      footerLineCount: 2,
    }),
  );
  const {
    selectionJumpSize,
    pageSize,
    windowLimit: resultWindowLimit,
    preloadThreshold,
  } = getSearchResultWindowMetrics(bodyHeight);

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
    autoExecuteInitialQuery: origin !== "ontology",
    dispatch,
    initialQuery,
    initialQueryState,
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

  const detailWidth = getTerminalTwoPaneDetailWidth(size.width, "split", SEARCH_LEFT_WIDTH);
  const detailLines =
    state.layout === "results" && state.session
      ? selectedResult
        ? buildResultDetailLines(selectedResult, state.session, resultSelectedIndex)
        : buildPendingResultDetailLines(state.session, resultSelectedIndex)
      : selectedWorkspaceEntry
        ? buildWorkspaceEntryDetailLines(selectedWorkspaceEntry, state, countState)
        : buildQuerySummaryLines(state, countState);
  const renderedDetailLineCount = getRenderedTerminalLineCount(detailLines, detailWidth);
  const maxDetailScroll = Math.max(0, renderedDetailLineCount - bodyHeight);
  const detailScroll = Math.min(state.detailScroll, maxDetailScroll);

  const { selectionPickerSession, openQueryFieldPicker } = useSearchQueryFieldPickerWorkflow({
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
    openQueryFieldPicker,
    origin,
    resultCount,
    selectedWorkspaceEntry,
    showSearchHelp,
    state,
    terminal,
    prompts,
    user,
    workspaceEntries,
    chooseResultSort,
  });

  useSearchScreenInteractionRouter({
    enabled: !busy && !selectionPickerSession && !structuredEditorSession,
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

  return {
    structuredEditorSession,
    selectionPickerSession,
    screen: {
      title: "Browse/Search",
      subtitle: buildSearchSubtitle(state, countState),
      left: {
        title:
          state.layout === "editor"
            ? "[EDITOR] Query"
            : state.activePane === "list"
              ? `[RESULTS] ${state.session ? `${formatResultPosition(resultSelectedIndex, state.session.total)} | Buf ${formatCount(state.session.loadedCount)} | ${formatSort(state.session.sort)}` : "No applied session"}`
              : `Results | ${state.session ? `${formatResultPosition(resultSelectedIndex, state.session.total)} | ${formatSort(state.session.sort)}` : "No applied session"}`,
        lines:
          state.layout === "editor"
            ? buildWorkspaceLines(workspaceEntries, workspaceSelectedIndex, bodyHeight)
            : buildResultLines(state.session, resultSelectedIndex, bodyHeight, loadingMore),
        active: state.layout === "results" ? state.activePane === "list" : true,
      },
      right: {
        title:
          state.layout === "results"
            ? state.activePane === "detail"
              ? `[PREVIEW] ${selectedResult?.name ?? "Results"}`
              : `Preview | ${selectedResult?.name ?? "Results"}`
            : "Query Status",
        lines: sliceRenderedTerminalLines(detailLines, detailWidth, detailScroll, bodyHeight),
        active: state.layout === "results" && state.activePane === "detail",
      },
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
      leftWidth: SEARCH_LEFT_WIDTH,
    },
  };
}
