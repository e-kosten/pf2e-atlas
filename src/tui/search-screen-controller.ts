import React from "react";

import type { OntologyNodeQuery } from "../types.js";
import { usePf2eTerminalAppServices } from "./app-service-context.js";
import { showTerminalReturnDialog, useTerminalInteractionContextAdapters } from "./interaction-context-adapters.js";
import type { OntologyPickerSelectionMap } from "./ontology-explorer/picker-screen.js";
import {
  SEARCH_LEFT_WIDTH,
  buildDraftSummaryLines,
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
  formatDraftStatus,
  formatResultPosition,
  formatSort,
  searchScreenReducer,
} from "./search-screen-model.js";
import { buildSearchFooterText, buildSearchHelpLines, useSearchScreenInteractionRouter } from "./search-screen-interactions.js";
import { getSearchResultWindowMetrics, getSessionBufferRange } from "./search-screen-state.js";
import { useSearchFacetWorkflow } from "./search-screen-facet-workflow.js";
import { useSearchSessionWorkflow } from "./search-screen-session-workflow.js";
import { useSearchWorkspaceActions } from "./search-screen-workspace-actions.js";
import type { SearchScreenOrigin } from "./search-workflow-types.js";
import {
  type DerivedTagTerminalTwoPaneScreenProps,
  getRenderedTerminalLineCount,
  getTerminalPaneBodyHeight,
  getTerminalTwoPaneDetailWidth,
  sliceRenderedTerminalLines,
  useDerivedTagTerminalApp,
  useDerivedTagTerminalSize,
} from "./terminal-ui.js";

export type SearchScreenControllerResult = {
  applyFacetPicker: (selection: OntologyPickerSelectionMap) => void;
  facetPickerSession: ReturnType<typeof useSearchFacetWorkflow>["facetPickerSession"];
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
  const initialRequest = React.useMemo(
    () =>
      initialQuery ? user.search.createRequestFromOntologyQuery(initialQuery) : user.search.createDefaultRequest(),
    [initialQuery, user.search],
  );
  const [state, dispatch] = React.useReducer(searchScreenReducer, initialRequest, createInitialSearchScreenState);
  const draftRef = React.useRef(initialRequest);

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

  const applyDraftUpdate = React.useCallback(
    (update: (request: import("./search-service.js").Pf2eTerminalSearchRequest) => import("./search-service.js").Pf2eTerminalSearchRequest) => {
      const nextRequest = user.search.normalizeRequest(update(draftRef.current));
      draftRef.current = nextRequest;
      dispatch({ type: "set_draft", request: nextRequest });
    },
    [user.search],
  );

  React.useEffect(() => {
    draftRef.current = state.draft;
  }, [state.draft]);

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
    initialRequest,
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
      state.layout === "draft" ? "Search Setup Help" : "Search Results Help",
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
        : buildDraftSummaryLines(state, countState);
  const renderedDetailLineCount = getRenderedTerminalLineCount(detailLines, detailWidth);
  const maxDetailScroll = Math.max(0, renderedDetailLineCount - bodyHeight);
  const detailScroll = Math.min(state.detailScroll, maxDetailScroll);

  const {
    facetPickerSession,
    openFacetPicker,
    applyFacetPicker,
  } = useSearchFacetWorkflow({
    draft: state.draft,
    applyDraftUpdate,
    services: user,
    onUnavailable: terminal.pauseForAnyKey,
  });

  const { handleIntent } = useSearchWorkspaceActions({
    applyDraftUpdate,
    dispatch,
    executeRequest,
    exitSearchScreen,
    jumpToResultPosition,
    maxDetailScroll,
    onOpenFacetPicker: openFacetPicker,
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
    enabled: !busy && !facetPickerSession,
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
    applyFacetPicker,
    facetPickerSession,
    screen: {
      title: "Browse/Search",
      subtitle: buildSearchSubtitle(state, countState),
      left: {
        title:
          state.layout === "draft"
            ? "[SETUP] Scope & Filters"
            : state.activePane === "list"
              ? `[RESULTS] ${state.session ? `${formatResultPosition(resultSelectedIndex, state.session.total)} | Buf ${formatCount(state.session.loadedCount)} | ${formatSort(state.session.sort)}` : "No applied session"}`
              : `Results | ${state.session ? `${formatResultPosition(resultSelectedIndex, state.session.total)} | ${formatSort(state.session.sort)}` : "No applied session"}`,
        lines:
          state.layout === "draft"
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
              ? `${formatDraftStatus(state)} | ${formatResultPosition(resultSelectedIndex, state.session.total)} | Buf ${formatCount(state.session.loadedCount)} | Win ${getSessionBufferRange(state.session)}`
              : `${formatDraftStatus(state)} | ${formatCountSummary(countState, state.draft)} | Scope & Filters`,
          tone: "accent",
        },
      ],
      leftWidth: SEARCH_LEFT_WIDTH,
    },
  };
}
