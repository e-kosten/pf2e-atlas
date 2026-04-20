import React from "react";

import type { OntologyNodeQuery } from "../domain/index.js";
import type { Pf2eTerminalAppServices } from "./app-services.js";
import type { SearchTerminalPromptAdapters } from "./interaction-context-adapters.js";
import type { SearchCountState, SearchScreenAction, SearchScreenState } from "./search-screen-state.js";
import type { Pf2eTerminalSearchQuery, Pf2eTerminalSearchSession } from "./search-service.js";
import type { DerivedTagTerminalApp } from "./framework/types.js";
import {
  LIVE_COUNT_DEBOUNCE_MS,
  RESULT_WINDOW_FETCH_DEBOUNCE_MS,
  clampAbsoluteSelection,
  getExecuteAvailability,
  getSearchResultWindowTarget,
  getSessionRecordAtIndex,
  parseJumpToResultInput,
} from "./search-screen-model.js";

export function useSearchSessionWorkflow({
  autoExecuteInitialQuery = true,
  dispatch,
  initialQuery,
  initialQueryState,
  onExit,
  preloadThreshold,
  prompts,
  resultSelectedIndex,
  resultWindowLimit,
  state,
  terminal,
  user,
}: {
  autoExecuteInitialQuery?: boolean;
  dispatch: React.Dispatch<SearchScreenAction>;
  initialQuery?: OntologyNodeQuery;
  initialQueryState: Pf2eTerminalSearchQuery;
  onExit: () => void;
  preloadThreshold: number;
  prompts: Pick<SearchTerminalPromptAdapters, "promptSelectOption" | "promptTextInput">;
  resultSelectedIndex: number;
  resultWindowLimit: number;
  state: SearchScreenState;
  terminal: Pick<DerivedTagTerminalApp, "pauseForAnyKey">;
  user: Pick<Pf2eTerminalAppServices["user"], "search">;
}): {
  busy: boolean;
  countState: SearchCountState;
  executeRequest: (query: Pf2eTerminalSearchQuery) => Promise<void>;
  jumpToResultPosition: () => Promise<void>;
  loadingMore: boolean;
  selectedResult: Pf2eTerminalSearchSession["results"][number] | null;
  resultCount: number;
  chooseResultSort: () => Promise<void>;
  exitSearchScreen: () => void;
} {
  const [busy, setBusy] = React.useState(false);
  const [countState, setCountState] = React.useState<SearchCountState>({
    status: "idle",
    result: null,
    message: null,
  });
  const [loadingMore, setLoadingMore] = React.useState(false);
  const autoRanInitialQuery = React.useRef(false);
  const loadMoreSessionKeyRef = React.useRef<string | null>(null);
  const loadMoreTimerRef = React.useRef<ReturnType<typeof setTimeout> | null>(null);
  const activeSessionRef = React.useRef<Pf2eTerminalSearchSession | null>(null);

  const resultCount = state.session?.total ?? 0;
  const clampedResultSelectedIndex = clampAbsoluteSelection(resultSelectedIndex, resultCount);
  const selectedResult = resultCount > 0 ? getSessionRecordAtIndex(state.session, clampedResultSelectedIndex) : null;

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
    onExit();
  }, [disposeSession, onExit]);

  const executeRequest = React.useCallback(
    async (query: Pf2eTerminalSearchQuery) => {
      const availability = getExecuteAvailability(query);
      if (availability.disabled) {
        await terminal.pauseForAnyKey(availability.reason ?? "This query cannot be executed yet.");
        return;
      }

      setBusy(true);
      try {
        const sort =
          state.session && state.session.query.mode === query.mode
            ? state.session.sort
            : user.search.getDefaultSort(query.mode);
        const session = await user.search.executeQuery(query, {
          sort,
          limit: Math.max(query.limit, resultWindowLimit),
        });
        dispatch({ type: "set_session", session });
      } catch (error) {
        await terminal.pauseForAnyKey(`Query execution failed.\n\n${(error as Error).message}`);
      } finally {
        setBusy(false);
      }
    },
    [dispatch, resultWindowLimit, state.session, terminal, user.search],
  );

  const chooseResultSort = React.useCallback(async () => {
    if (!state.session) {
      return;
    }

    const result = await prompts.promptSelectOption({
      title: "Result Sort",
      prompt: "Choose how the current result reader should be ordered",
      entries: user.search.getResultSortOptions(state.session.query.mode).map((option) => ({
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
  }, [dispatch, prompts, state.session, terminal, user.search]);

  const jumpToResultPosition = React.useCallback(async () => {
    if (!state.session || state.session.total <= 0) {
      return;
    }

    const input = await prompts.promptTextInput({
      title: "Jump To Result",
      prompt: `Enter a result number between 1 and ${state.session.total}.`,
      hint: `Current: ${clampedResultSelectedIndex + 1}  Example: 6000`,
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
  }, [clampedResultSelectedIndex, dispatch, prompts, state.session, terminal]);

  React.useEffect(() => {
    if (!autoExecuteInitialQuery || !initialQuery || autoRanInitialQuery.current) {
      return;
    }
    autoRanInitialQuery.current = true;
    void executeRequest(initialQueryState);
  }, [autoExecuteInitialQuery, executeRequest, initialQuery, initialQueryState]);

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
    const availability = getExecuteAvailability(state.query);
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
        .countQuery(state.query)
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
  }, [state.query, user.search]);

  React.useEffect(() => {
    if (loadMoreTimerRef.current) {
      clearTimeout(loadMoreTimerRef.current);
      loadMoreTimerRef.current = null;
    }
    if (!state.session || state.layout !== "results" || state.session.total <= 0) {
      return;
    }

    const currentSession = state.session;
    const targetWindow = getSearchResultWindowTarget(currentSession, clampedResultSelectedIndex, {
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
    clampedResultSelectedIndex,
    dispatch,
    loadingMore,
    preloadThreshold,
    resultWindowLimit,
    state.layout,
    state.session,
    terminal,
    user.search,
  ]);

  return {
    busy,
    countState,
    executeRequest,
    jumpToResultPosition,
    loadingMore,
    selectedResult,
    resultCount,
    chooseResultSort,
    exitSearchScreen,
  };
}
