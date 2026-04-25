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
import {
  buildDerivedTagTerminalActionTargetLine,
  createDerivedTagTerminalActionTargetState,
  reduceDerivedTagTerminalActionTargetState,
  shouldRenderDerivedTagTerminalActionTarget,
  type DerivedTagTerminalActionTargetOption,
} from "../action-target.js";
import { useDerivedTagTerminalApp, useDerivedTagTerminalSize } from "../framework/context.js";
import type { DerivedTagTerminalTwoPaneScreenProps } from "../framework/types.js";
import type { SearchScreenProps } from "./entry-props.js";
import type { DerivedTagTerminalActionTargetState } from "../action-target.js";

type SearchEditorActionId = "openSelected" | "executeQuery" | "resetQuery" | "discardResults";

export type SearchScreenControllerResult = {
  structuredEditorSession: SearchStructuredEditorSession | null;
  filterExplorerSession: SearchFilterExplorerSession | null;
  screen: DerivedTagTerminalTwoPaneScreenProps;
};

export function useSearchScreenController({
  entry = "editor",
  transitionStatus,
  origin = "app",
  promptForInitialMode = false,
  onBack,
  ...routeEntry
}: SearchScreenProps): SearchScreenControllerResult {
  const initialRequest = entry === "results" ? undefined : ("initialRequest" in routeEntry ? routeEntry.initialRequest : undefined);
  const initialSession = entry === "results" ? routeEntry.initialSession : undefined;
  const terminal = useDerivedTagTerminalApp();
  const prompts = useTerminalInteractionContextAdapters();
  const { user } = usePf2eTerminalAppServices();
  const size = useDerivedTagTerminalSize();
  const { notification, showNotification } = useTerminalListDetailNotification();
  const initialQueryState = React.useMemo(
    () =>
      initialSession?.query ??
      (initialRequest ? user.search.normalizeQuery(initialRequest) : user.search.createDefaultQuery()),
    [initialRequest, initialSession, user.search],
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
  const [editorActionTargetState, dispatchEditorActionTarget] = React.useReducer(
    reduceDerivedTagTerminalActionTargetState<DerivedTagTerminalActionTargetState>,
    undefined,
    () => createDerivedTagTerminalActionTargetState(),
  );
  const queryRef = React.useRef(initialQueryState);
  const promptedForInitialModeRef = React.useRef(false);

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

  React.useEffect(() => {
    if (entry !== "editor" || initialRequest || !promptForInitialMode || promptedForInitialModeRef.current) {
      return;
    }

    promptedForInitialModeRef.current = true;
    void prompts
      .promptSelectOption({
        title: "Choose Search Mode",
        prompt: "",
        presentation: "blanked",
        choiceLayout: "horizontal",
        filtering: false,
        selectedValue: state.query.mode,
        entries: user.search.getModeOptions().map((option) => ({
          value: option.value,
          label: option.label,
          description: option.description,
          detailLines: [{ text: option.description }],
        })),
      })
      .then((result) => {
        if (result.kind !== "selected") {
          onBack();
          return;
        }
        if (result.value === queryRef.current.mode) {
          return;
        }
        applyQueryUpdate((query) => ({
          ...user.search.createDefaultQuery(result.value),
          limit: query.limit,
        }));
      });
  }, [applyQueryUpdate, entry, initialRequest, onBack, promptForInitialMode, prompts, state.query.mode, user.search]);

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

  const renderOptions = React.useMemo(
    () => ({
      packLabelResolver: user.search.getPackLabel,
    }),
    [user.search],
  );
  const workspaceEntries = buildWorkspaceEntries(state, countState, renderOptions);
  const workspaceSelectedIndex = Math.max(
    0,
    Math.min(state.workspaceSelectedIndex, Math.max(0, workspaceEntries.length - 1)),
  );
  const selectedWorkspaceEntry = workspaceEntries[workspaceSelectedIndex] ?? workspaceEntries[0];
  const resultSelectedIndex = clampAbsoluteSelection(state.resultSelectedIndex, resultCount);

  const detailLines =
    state.layout === "results" && state.session
      ? selectedResult
        ? buildResultDetailLines(selectedResult, state.session, resultSelectedIndex)
        : buildPendingResultDetailLines(state.session, resultSelectedIndex)
      : selectedWorkspaceEntry
        ? buildWorkspaceEntryDetailLines(selectedWorkspaceEntry, state, countState, renderOptions)
        : buildQuerySummaryLines(state, countState, renderOptions);
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

  const editorActionEntries = React.useMemo<DerivedTagTerminalActionTargetOption<SearchEditorActionId>[]>(() => {
    const entries: DerivedTagTerminalActionTargetOption<SearchEditorActionId>[] = [
      {
        id: "openSelected",
        label: "Open Focused Row",
        description: selectedWorkspaceEntry?.disabled
          ? selectedWorkspaceEntry.disabledReason ?? "The focused row is unavailable."
          : selectedWorkspaceEntry?.description ?? "Open or edit the focused query row.",
      },
      {
        id: "executeQuery",
        label: "Execute Query",
        description: "Apply the current query editor state and switch to results.",
      },
      {
        id: "resetQuery",
        label: "Reset Query",
        description: "Restore the default query state.",
      },
    ];

    if (state.session) {
      entries.push({
        id: "discardResults",
        label: "Discard Applied Results",
        description: "Clear the applied result reader while leaving the editor state intact.",
      });
    }

    return entries;
  }, [selectedWorkspaceEntry, state.session]);

  const showSearchHelp = React.useCallback(() => {
    void showTerminalReturnDialog(
      prompts,
      state.layout === "editor" ? "Search Editor Help" : "Search Results Help",
      buildSearchHelpLines(state, workspaceEntries, origin, editorActionEntries),
    );
  }, [editorActionEntries, origin, prompts, state, workspaceEntries]);

  const { handleIntent, runWorkspaceAction, structuredEditorSession } = useSearchWorkspaceActions({
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
    state,
    terminal,
    prompts,
    user,
    workspaceEntries,
    chooseResultSort,
  });
  const runEditorAction = React.useCallback(
    (actionId: SearchEditorActionId) => {
      if (actionId === "openSelected") {
        if (!selectedWorkspaceEntry || selectedWorkspaceEntry.disabled) {
          return;
        }
        runWorkspaceAction(selectedWorkspaceEntry.action);
        return;
      }
      if (actionId === "executeQuery") {
        runWorkspaceAction("execute");
        return;
      }
      if (actionId === "resetQuery") {
        runWorkspaceAction("reset");
        return;
      }
      if (actionId === "discardResults" && state.session) {
        runWorkspaceAction("clearResults");
      }
    },
    [runWorkspaceAction, selectedWorkspaceEntry, state.session],
  );

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
    showNotification,
    onIntent: handleIntent,
    editorActionTarget:
      state.layout === "editor"
        ? {
            state: editorActionTargetState,
            actionCount: editorActionEntries.length,
            onToggle: () => dispatchEditorActionTarget({ type: "toggle_target" }),
            onLeave: () => dispatchEditorActionTarget({ type: "leave_actions" }),
            onMove: (delta) =>
              dispatchEditorActionTarget({
                type: "move_action",
                delta,
                actionCount: editorActionEntries.length,
              }),
            onApply: () => {
              const selectedAction = editorActionEntries[editorActionTargetState.selectedActionIndex];
              if (selectedAction) {
                runEditorAction(selectedAction.id);
              }
            },
          }
        : undefined,
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
        text: buildSearchFooterText(state, loadingMore, origin, {
          editorActionTargetState,
        }),
        tone: "dim",
      },
      state.layout === "editor" &&
      shouldRenderDerivedTagTerminalActionTarget(editorActionTargetState, "onDemand")
        ? buildDerivedTagTerminalActionTargetLine(editorActionEntries, editorActionTargetState)
        : {
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
