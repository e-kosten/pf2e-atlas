import React from "react";

import type { OntologyNodeQuery } from "../types.js";
import { usePf2eTerminalAppServices } from "./app-service-context.js";
import { TERMINAL_DIALOG_RETURN_FOOTER } from "./interaction-bindings.js";
import { buildSearchFacetPickerModel } from "./ontology-explorer/facet-picker-model.js";
import type { OntologyPickerSelectionMap } from "./ontology-explorer/picker-screen.js";
import type { Pf2eTerminalSearchRequest, Pf2eTerminalSearchSession } from "./search-service.js";
import {
  type SearchFacetPickerSession,
  type SearchCountState,
  type SearchWorkspaceAction,
  SEARCH_LEFT_WIDTH,
  LIVE_COUNT_DEBOUNCE_MS,
  RESULT_WINDOW_FETCH_DEBOUNCE_MS,
  applyFacetPickerSelectionsToRequest,
  buildDraftCommandPaletteEntries,
  buildDraftSummaryLines,
  buildFacetPickerInitialSelections,
  buildFacetRemovalEntries,
  buildPendingResultDetailLines,
  buildResultCommandPaletteEntries,
  buildResultDetailLines,
  buildResultLines,
  buildSearchFooterText,
  buildSearchHelpLines,
  type SearchScreenIntent,
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
  getExecuteAvailability,
  getSearchResultWindowMetrics,
  getSearchResultWindowTarget,
  getSessionBufferRange,
  getSessionRecordAtIndex,
  parseJumpToResultInput,
  parseLevelRangeInput,
  searchScreenReducer,
  useSearchScreenInteractionRouter,
} from "./search-screen-model.js";
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
  facetPickerSession: SearchFacetPickerSession | null;
  screen: DerivedTagTerminalTwoPaneScreenProps;
};

export function useSearchScreenController({
  initialQuery,
  onBack,
}: {
  initialQuery?: OntologyNodeQuery;
  onBack: () => void;
}): SearchScreenControllerResult {
  const terminal = useDerivedTagTerminalApp();
  const { user } = usePf2eTerminalAppServices();
  const size = useDerivedTagTerminalSize();
  const [busy, setBusy] = React.useState(false);
  const [loadingMore, setLoadingMore] = React.useState(false);
  const [facetPickerSession, setFacetPickerSession] = React.useState<SearchFacetPickerSession | null>(null);
  const [countState, setCountState] = React.useState<SearchCountState>({
    status: "idle",
    result: null,
    message: null,
  });
  const initialRequest = React.useMemo(
    () =>
      initialQuery ? user.search.createRequestFromOntologyQuery(initialQuery) : user.search.createDefaultRequest(),
    [initialQuery, user.search],
  );
  const [state, dispatch] = React.useReducer(searchScreenReducer, initialRequest, createInitialSearchScreenState);
  const draftRef = React.useRef(initialRequest);
  const autoRanInitialQuery = React.useRef(false);
  const loadMoreSessionKeyRef = React.useRef<string | null>(null);
  const loadMoreTimerRef = React.useRef<ReturnType<typeof setTimeout> | null>(null);
  const activeSessionRef = React.useRef<Pf2eTerminalSearchSession | null>(null);

  const workspaceEntries = buildWorkspaceEntries(state, countState);
  const workspaceSelectedIndex = Math.max(
    0,
    Math.min(state.workspaceSelectedIndex, Math.max(0, workspaceEntries.length - 1)),
  );
  const selectedWorkspaceEntry = workspaceEntries[workspaceSelectedIndex] ?? workspaceEntries[0];

  const resultCount = state.session?.total ?? 0;
  const resultSelectedIndex = clampAbsoluteSelection(state.resultSelectedIndex, resultCount);
  const selectedResult = resultCount > 0 ? getSessionRecordAtIndex(state.session, resultSelectedIndex) : null;

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

  const applyDraftUpdate = React.useCallback(
    (update: (request: Pf2eTerminalSearchRequest) => Pf2eTerminalSearchRequest) => {
      const nextRequest = user.search.normalizeRequest(update(draftRef.current));
      draftRef.current = nextRequest;
      dispatch({ type: "set_draft", request: nextRequest });
    },
    [user.search],
  );

  const disposeSession = React.useCallback(
    (session: Pf2eTerminalSearchSession | null) => {
      if (!session) {
        return;
      }
      user.search.disposeSession(session);
    },
    [user.search],
  );

  const exitSearchScreen = React.useCallback(() => {
    const activeSession = activeSessionRef.current;
    if (activeSession) {
      disposeSession(activeSession);
      activeSessionRef.current = null;
    }
    onBack();
  }, [disposeSession, onBack]);

  const executeRequest = React.useCallback(
    async (request: Pf2eTerminalSearchRequest) => {
      const availability = getExecuteAvailability(request);
      if (availability.disabled) {
        await terminal.pauseForAnyKey(availability.reason ?? "This query setup cannot be executed yet.");
        return;
      }

      setBusy(true);
      try {
        const sort =
          state.session && state.session.request.mode === request.mode
            ? state.session.sort
            : user.search.getDefaultSort(request.mode);
        const session = await user.search.executeQuery(request, {
          sort,
          limit: Math.max(request.limit, resultWindowLimit),
        });
        dispatch({ type: "set_session", session });
      } catch (error) {
        await terminal.pauseForAnyKey(`Workspace query failed.\n\n${(error as Error).message}`);
      } finally {
        setBusy(false);
      }
    },
    [resultWindowLimit, state.session, terminal, user.search],
  );

  const chooseResultSort = React.useCallback(async () => {
    if (!state.session) {
      return;
    }

    const result = await terminal.promptSelectOption({
      title: "Result Sort",
      prompt: "Choose how the current result reader should be ordered",
      entries: user.search.getResultSortOptions(state.session.request.mode).map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
      selectedValue: state.session.sort,
    });

    if (result.kind !== "selected" || result.value === state.session.sort) {
      return;
    }

    setBusy(true);
    try {
      const session = await user.search.changeSort(state.session, result.value);
      dispatch({ type: "set_session", session });
    } catch (error) {
      await terminal.pauseForAnyKey(`Result sort failed.\n\n${(error as Error).message}`);
    } finally {
      setBusy(false);
    }
  }, [state.session, terminal, user.search]);

  const jumpToResultPosition = React.useCallback(async () => {
    if (!state.session || state.session.total <= 0) {
      return;
    }

    const input = await terminal.promptTextInput({
      title: "Jump To Result",
      prompt: `Enter a result number between 1 and ${state.session.total}.`,
      hint: `Current: ${resultSelectedIndex + 1}  Example: 6000`,
    });

    if (input === undefined) {
      return;
    }

    const parsed = parseJumpToResultInput(input, state.session.total);
    if (typeof parsed === "string") {
      await terminal.pauseForAnyKey(parsed);
      return;
    }

    dispatch({
      type: "set_result_selection",
      index: parsed,
      itemCount: state.session.total,
    });
  }, [resultSelectedIndex, state.session, terminal]);

  React.useEffect(() => {
    if (!initialQuery || autoRanInitialQuery.current) {
      return;
    }
    autoRanInitialQuery.current = true;
    void executeRequest(initialRequest);
  }, [executeRequest, initialQuery, initialRequest]);

  React.useEffect(() => {
    draftRef.current = state.draft;
  }, [state.draft]);

  React.useEffect(() => {
    const previousSession = activeSessionRef.current;
    const nextSession = state.session;
    if (previousSession && previousSession.windowId !== nextSession?.windowId) {
      disposeSession(previousSession);
    }
    activeSessionRef.current = nextSession;
  }, [disposeSession, state.session]);

  React.useEffect(
    () => () => {
      const activeSession = activeSessionRef.current;
      if (activeSession) {
        disposeSession(activeSession);
        activeSessionRef.current = null;
      }
    },
    [disposeSession],
  );

  React.useEffect(() => {
    const availability = getExecuteAvailability(state.draft);
    if (availability.disabled) {
      setCountState({
        status: "idle",
        result: null,
        message: availability.reason,
      });
      return;
    }

    let cancelled = false;
    setCountState((current) => ({
      status: "loading",
      result: current.result,
      message: null,
    }));

    const timeout = setTimeout(() => {
      void user.search
        .countQuery(state.draft)
        .then((result) => {
          if (cancelled) {
            return;
          }
          setCountState({
            status: "ready",
            result,
            message: null,
          });
        })
        .catch((error) => {
          if (cancelled) {
            return;
          }
          setCountState({
            status: "error",
            result: null,
            message: (error as Error).message,
          });
        });
    }, LIVE_COUNT_DEBOUNCE_MS);

    return () => {
      cancelled = true;
      clearTimeout(timeout);
    };
  }, [state.draft, user.search]);

  React.useEffect(() => {
    if (loadMoreTimerRef.current) {
      clearTimeout(loadMoreTimerRef.current);
      loadMoreTimerRef.current = null;
    }
    if (!state.session || state.layout !== "results" || state.session.total <= 0) {
      return;
    }

    const currentSession = state.session;
    const targetWindow = getSearchResultWindowTarget(currentSession, resultSelectedIndex, {
      windowLimit: resultWindowLimit,
      preloadThreshold,
    });
    if (!targetWindow) {
      return;
    }

    const sessionKey =
      `${currentSession.windowId}:${currentSession.sort}:${currentSession.windowOffset}:${currentSession.loadedCount}:` +
      `${targetWindow.offset}:${targetWindow.limit}`;
    if (loadingMore || loadMoreSessionKeyRef.current === sessionKey) {
      return;
    }

    let cancelled = false;
    loadMoreTimerRef.current = setTimeout(() => {
      if (cancelled) {
        return;
      }

      loadMoreSessionKeyRef.current = sessionKey;
      setLoadingMore(true);
      void user.search
        .readResultWindow(currentSession, targetWindow)
        .then((session) => {
          if (cancelled) {
            return;
          }
          dispatch({ type: "set_session", session, showResults: true, preserveSelection: true });
        })
        .catch(async (error) => {
          if (cancelled) {
            return;
          }
          await terminal.pauseForAnyKey(`Loading more results failed.\n\n${(error as Error).message}`);
        })
        .finally(() => {
          if (loadMoreSessionKeyRef.current === sessionKey) {
            loadMoreSessionKeyRef.current = null;
            setLoadingMore(false);
          }
        });
    }, RESULT_WINDOW_FETCH_DEBOUNCE_MS);

    return () => {
      cancelled = true;
      if (loadMoreTimerRef.current) {
        clearTimeout(loadMoreTimerRef.current);
        loadMoreTimerRef.current = null;
      }
    };
  }, [
    loadingMore,
    preloadThreshold,
    resultSelectedIndex,
    resultWindowLimit,
    state.layout,
    state.session,
    terminal,
    user.search,
  ]);

  const editQueryText = React.useCallback(async () => {
    const queryText = await terminal.promptTextInput({
      title: "Query Text",
      prompt:
        state.draft.mode === "lookup"
          ? "Enter an exact or near-exact record name"
          : "Enter search text for the current query setup",
      defaultValue: state.draft.queryText,
      hint: state.draft.mode === "lookup" ? "Example: Raise Shield" : "Example: ghost ship captain",
    });

    if (queryText === undefined) {
      return;
    }

    applyDraftUpdate((request) => ({
      ...request,
      queryText,
    }));
  }, [applyDraftUpdate, state.draft.mode, state.draft.queryText, terminal]);

  const chooseMode = React.useCallback(async () => {
    const result = await terminal.promptSelectOption({
      title: "Workspace Mode",
      prompt: "Choose how the current query setup should execute",
      entries: user.search.getModeOptions().map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
      selectedValue: state.draft.mode,
    });

    if (result.kind !== "selected") {
      return;
    }

    applyDraftUpdate((request) => ({
      ...request,
      mode: result.value,
    }));
  }, [applyDraftUpdate, state.draft.mode, terminal, user.search]);

  const chooseSearchProfile = React.useCallback(async () => {
    const result = await terminal.promptSelectOption({
      title: "Search Profile",
      prompt: "Choose the current profile for ranked search mode",
      entries: user.search.getProfileOptions().map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
      selectedValue: state.draft.searchProfile,
    });

    if (result.kind === "selected") {
      applyDraftUpdate((request) => ({
        ...request,
        searchProfile: result.value,
      }));
    }
  }, [applyDraftUpdate, state.draft.searchProfile, terminal, user.search]);

  const chooseCategoryFilter = React.useCallback(async () => {
    const [allCategoryOption, ...categoryEntries] = user.search.getCategoryOptions();
    const result = await terminal.promptOptionalSelectOption({
      title: "Category Scope",
      prompt: "Choose the current category boundary",
      allOption: {
        label: allCategoryOption?.label ?? "Any Category",
        description: allCategoryOption?.description,
      },
      entries: categoryEntries.map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
      selectedValue: state.draft.filters.category ?? null,
    });

    if (result.kind === "cancelled") {
      return;
    }

    applyDraftUpdate((request) => ({
      ...request,
      filters: {
        ...request.filters,
        category: result.kind === "all" ? null : result.value,
        subcategory: null,
      },
    }));
  }, [applyDraftUpdate, state.draft.filters.category, terminal, user.search]);

  const chooseSubcategoryFilter = React.useCallback(async () => {
    if (!state.draft.filters.category) {
      await terminal.pauseForAnyKey("Choose a category before selecting a subcategory.");
      return;
    }

    const [allSubcategoryOption, ...subcategoryEntries] = user.search.getSubcategoryOptions(
      state.draft.filters.category,
    );
    const result = await terminal.promptOptionalSelectOption({
      title: "Subcategory Scope",
      prompt: "Choose the current subcategory boundary",
      allOption: {
        label: allSubcategoryOption?.label ?? "Any Subcategory",
        description: allSubcategoryOption?.description,
      },
      entries: subcategoryEntries.map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
      selectedValue: state.draft.filters.subcategory ?? null,
    });

    if (result.kind === "cancelled") {
      return;
    }

    applyDraftUpdate((request) => ({
      ...request,
      filters: {
        ...request.filters,
        subcategory: result.kind === "all" ? null : result.value,
      },
    }));
  }, [applyDraftUpdate, state.draft.filters.category, state.draft.filters.subcategory, terminal, user.search]);

  const chooseRarityFilter = React.useCallback(async () => {
    const options = user.search.getRarityOptions(state.draft.filters.category, state.draft.filters.subcategory);
    const selected = await terminal.promptPolicySelectOption({
      title: "Rarity Filter",
      prompt: "Cycle rarities through include and exclude. Press Esc or Left when finished.",
      allowedStates: ["any", "exclude"],
      entries: options.map((option) => ({
        value: option.value,
        label: option.label,
        description: option.description,
      })),
      selectedValues: state.draft.filters.rarity,
    });

    applyDraftUpdate((request) => ({
      ...request,
      filters: {
        ...request.filters,
        rarity: {
          any: selected.any,
          all: [],
          exclude: selected.exclude,
        },
      },
    }));
  }, [
    applyDraftUpdate,
    state.draft.filters.category,
    state.draft.filters.rarity,
    state.draft.filters.subcategory,
    terminal,
    user.search,
  ]);

  const editLevelRange = React.useCallback(async () => {
    const input = await terminal.promptTextInput({
      title: "Level Range",
      prompt: "Enter `3-8`, `5`, `5+`, or `<=10`. Leave blank to clear.",
      defaultValue:
        state.draft.filters.levelMin === null && state.draft.filters.levelMax === null
          ? ""
          : state.draft.filters.levelMin !== null && state.draft.filters.levelMax !== null
            ? state.draft.filters.levelMin === state.draft.filters.levelMax
              ? String(state.draft.filters.levelMin)
              : `${state.draft.filters.levelMin}-${state.draft.filters.levelMax}`
            : state.draft.filters.levelMin !== null
              ? `${state.draft.filters.levelMin}+`
              : `<=${state.draft.filters.levelMax}`,
      hint: "Examples: 3-8 or <=5",
    });

    if (input === undefined) {
      return;
    }

    const parsed = parseLevelRangeInput(input);
    if (typeof parsed === "string") {
      await terminal.pauseForAnyKey(parsed);
      return;
    }

    applyDraftUpdate((request) => ({
      ...request,
      filters: {
        ...request.filters,
        levelMin: parsed.levelMin,
        levelMax: parsed.levelMax,
      },
    }));
  }, [applyDraftUpdate, state.draft.filters.levelMax, state.draft.filters.levelMin, terminal]);

  const addFacetFilter = React.useCallback(async () => {
    if (!state.draft.filters.category) {
      await terminal.pauseForAnyKey("Choose a category before editing a discoverable facet filter.");
      return;
    }

    const fieldOptions = user.search.getFacetFieldOptions(
      state.draft.filters.category,
      state.draft.filters.subcategory,
    );
    if (fieldOptions.length === 0) {
      await terminal.pauseForAnyKey("No discoverable facet fields are available for the current browse scope.");
      return;
    }

    const searchSemanticsDomain = user.ontology.loadDomain("searchSemantics");
    const model = buildSearchFacetPickerModel(searchSemanticsDomain, {
      category: state.draft.filters.category,
      subcategory: state.draft.filters.subcategory,
      fieldOptions,
    });
    if (model.rootNodes.length === 0) {
      await terminal.pauseForAnyKey("No ontology-backed facet hierarchy is available for that scope.");
      return;
    }

    setFacetPickerSession({
      model,
      initialSelections: buildFacetPickerInitialSelections(
        state.draft,
        fieldOptions.map((option) => option.value),
      ),
      scopedFields: fieldOptions.map((option) => option.value),
    });
  }, [state.draft, terminal, user.ontology, user.search]);

  const removeFacetFilter = React.useCallback(async () => {
    if (
      state.draft.filters.facets.length === 0 &&
      state.draft.filters.actionCost.any.length === 0 &&
      state.draft.filters.actionCost.all.length === 0 &&
      state.draft.filters.actionCost.exclude.length === 0
    ) {
      await terminal.pauseForAnyKey("There are no facet policies to clear from the current query setup.");
      return;
    }

    const selected = await terminal.promptMultiSelectOption({
      title: "Clear Facet Filter",
      prompt: "Toggle facet fields to clear. Press Esc or Left when finished.",
      entries: buildFacetRemovalEntries(state.draft.filters.facets, state.draft.filters.actionCost),
      selectedValues: [],
    });
    applyDraftUpdate((request) => ({
      ...request,
      filters: {
        ...request.filters,
        actionCost: selected.includes("actionCost") ? { any: [], all: [], exclude: [] } : request.filters.actionCost,
        facets: request.filters.facets.filter((facet) => !selected.includes(facet.field)),
      },
    }));
  }, [applyDraftUpdate, state.draft.filters.actionCost, state.draft.filters.facets, terminal]);

  const applyFacetPicker = React.useCallback(
    (selection: OntologyPickerSelectionMap) => {
      if (!facetPickerSession) {
        return;
      }
      applyDraftUpdate((request) =>
        applyFacetPickerSelectionsToRequest(request, selection, facetPickerSession.scopedFields),
      );
      setFacetPickerSession(null);
    },
    [applyDraftUpdate, facetPickerSession],
  );

  const resetDraftWorkspace = React.useCallback(() => {
    const defaultRequest = user.search.createDefaultRequest();
    draftRef.current = defaultRequest;
    dispatch({ type: "set_draft", request: defaultRequest });
    dispatch({ type: "set_layout", layout: "draft", pane: "list" });
  }, [user.search]);

  const runWorkspaceAction = React.useCallback(
    (action: SearchWorkspaceAction) => {
      const entry = workspaceEntries.find((candidate) => candidate.action === action);
      if (entry?.disabled) {
        return;
      }

      switch (action) {
        case "execute":
          void executeRequest(state.draft);
          return;
        case "mode":
          void chooseMode();
          return;
        case "query":
          void editQueryText();
          return;
        case "profile":
          void chooseSearchProfile();
          return;
        case "category":
          void chooseCategoryFilter();
          return;
        case "subcategory":
          void chooseSubcategoryFilter();
          return;
        case "levels":
          void editLevelRange();
          return;
        case "rarity":
          void chooseRarityFilter();
          return;
        case "addFacet":
          void addFacetFilter();
          return;
        case "removeFacet":
          void removeFacetFilter();
          return;
        case "reset":
          resetDraftWorkspace();
          return;
        case "clearResults":
          dispatch({ type: "clear_results" });
          return;
      }
    },
    [
      addFacetFilter,
      chooseCategoryFilter,
      chooseMode,
      chooseRarityFilter,
      chooseSearchProfile,
      chooseSubcategoryFilter,
      editLevelRange,
      editQueryText,
      executeRequest,
      removeFacetFilter,
      resetDraftWorkspace,
      state.draft,
      workspaceEntries,
    ],
  );

  const openSelectedWorkspaceEntry = React.useCallback(() => {
    if (!selectedWorkspaceEntry || selectedWorkspaceEntry.disabled) {
      return;
    }
    runWorkspaceAction(selectedWorkspaceEntry.action);
  }, [runWorkspaceAction, selectedWorkspaceEntry]);

  const openDraftCommandPalette = React.useCallback(async () => {
    const selected = await terminal.promptCommandPalette({
      title: "Search Setup Commands",
      prompt: "Filter setup commands",
      entries: buildDraftCommandPaletteEntries(workspaceEntries),
    });
    if (!selected) {
      return;
    }
    if (workspaceEntries.find((entry) => entry.action === selected)?.disabled) {
      return;
    }
    runWorkspaceAction(selected);
  }, [runWorkspaceAction, terminal, workspaceEntries]);

  const openResultCommandPalette = React.useCallback(async () => {
    const selected = await terminal.promptCommandPalette({
      title: "Result Commands",
      prompt: "Filter result commands",
      entries: buildResultCommandPaletteEntries(state),
    });
    if (selected === "jumpToResult") {
      void jumpToResultPosition();
      return;
    }
    if (selected === "sortResults") {
      void chooseResultSort();
    }
  }, [chooseResultSort, jumpToResultPosition, state, terminal]);

  const showSearchHelp = React.useCallback(() => {
    void terminal.showDialog({
      title: state.layout === "draft" ? "Search Setup Help" : "Search Results Help",
      body: buildSearchHelpLines(state, workspaceEntries),
      footer: [{ text: TERMINAL_DIALOG_RETURN_FOOTER, tone: "dim" }],
    });
  }, [state, terminal, workspaceEntries]);

  const handleIntent = React.useCallback(
    (intent: SearchScreenIntent) => {
      switch (intent.type) {
        case "show_help":
          showSearchHelp();
          return;
        case "quit":
          exitSearchScreen();
          return;
        case "edit_query":
          void editQueryText();
          return;
        case "open_setup_commands":
          void openDraftCommandPalette();
          return;
        case "execute":
          void executeRequest(state.draft);
          return;
        case "back_to_app":
          exitSearchScreen();
          return;
        case "move_workspace_selection":
          dispatch({
            type: "move_workspace_selection",
            delta: intent.delta,
            itemCount: workspaceEntries.length,
          });
          return;
        case "workspace_selection_boundary":
          dispatch({
            type: "workspace_selection_boundary",
            boundary: intent.boundary,
            itemCount: workspaceEntries.length,
          });
          return;
        case "edit_selected_workspace":
          openSelectedWorkspaceEntry();
          return;
        case "open_result_commands":
          void openResultCommandPalette();
          return;
        case "toggle_pane":
          dispatch({ type: "set_active_pane", pane: state.activePane === "list" ? "detail" : "list" });
          return;
        case "return_to_setup":
          dispatch({ type: "set_layout", layout: "draft", pane: "list" });
          return;
        case "open_preview":
          dispatch({ type: "set_active_pane", pane: "detail" });
          return;
        case "move_result_selection":
          dispatch({ type: "move_result_selection", delta: intent.delta, itemCount: resultCount });
          return;
        case "result_selection_boundary":
          dispatch({
            type: "result_selection_boundary",
            boundary: intent.boundary,
            itemCount: resultCount,
          });
          return;
        case "return_to_result_list":
          dispatch({ type: "set_active_pane", pane: "list" });
          return;
        case "move_detail":
          dispatch({ type: "move_detail", delta: intent.delta, maxDetailScroll });
          return;
        case "detail_boundary":
          dispatch({ type: "detail_boundary", boundary: intent.boundary, maxDetailScroll });
          return;
      }
    },
    [
      editQueryText,
      executeRequest,
      exitSearchScreen,
      maxDetailScroll,
      openDraftCommandPalette,
      openResultCommandPalette,
      openSelectedWorkspaceEntry,
      resultCount,
      showSearchHelp,
      state.activePane,
      state.draft,
      workspaceEntries.length,
    ],
  );

  useSearchScreenInteractionRouter({
    enabled: !busy && !facetPickerSession,
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
          text: buildSearchFooterText(state, loadingMore),
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
