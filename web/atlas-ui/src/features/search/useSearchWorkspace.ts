import { useEffect, useMemo, useReducer, useState } from "react";
import { keepPreviousData, useQuery, type UseQueryResult } from "@tanstack/react-query";
import {
  getReadiness,
  getRecordDetail,
  openResultWindow,
  readResultWindowPage,
} from "../../api/atlasApi";
import type {
  FilterEditorView,
  FilterValueListView,
  RecordDetailView,
  ResultWindowPage,
} from "../../generated/atlas";
import {
  buildFilterDiscoveryContext,
  buildOpenRequest,
  DEFAULT_SEARCH_STATE,
  encodeSearchExecutionState,
  encodeSearchState,
  hasExecutableSearch,
  searchStateQueryString,
  type SearchFormState,
} from "../../shared/filters/searchState";
import { useFilterDiscovery } from "../../shared/filters/useFilterDiscovery";
import {
  initialWorkspaceInteractionState,
  workspaceInteractionReducer,
} from "./workspaceState";
import {
  ATLAS_ROUTE_CHANGE_EVENT,
  currentAtlasRoute,
  searchPath,
} from "../../app/routes";

const SEARCH_REQUEST_DEBOUNCE_MS = 300;

export type AtlasRequestTiming = {
  durationMs: number;
  finishedAt: number;
};

export type AtlasWorkspaceDiagnostics = {
  activeWindowId: string | null;
  detailRequest: AtlasRequestTiming | null;
  resultRequest:
    | (AtlasRequestTiming & {
        kind: "open_window" | "page";
      })
    | null;
  searchDebouncing: boolean;
};

type ResultWindowHandle = {
  searchExecutionToken: string;
  windowId: bigint;
};

export type SearchWorkspaceState = {
  search: SearchFormState;
  setSearch: (next: SearchFormState) => void;
  activeResultKey: string | null;
  focusResult: (recordKey: string) => void;
  moveResultSelection: (direction: "next" | "previous") => void;
  openActiveResult: () => void;
  selectedRecordKey: string | null;
  selectRecord: (recordKey: string | null) => void;
  pageNumber: number;
  setPageNumber: (page: number) => void;
  resultPage: ResultWindowPage | undefined;
  recordDetail: RecordDetailView | undefined;
  filterEditor: FilterEditorView | undefined;
  filterValuesByField: Record<string, FilterValueListView | undefined>;
  readiness: UseQueryResult<Awaited<ReturnType<typeof getReadiness>>, Error>;
  resultsLoading: boolean;
  resultsRefreshing: boolean;
  detailLoading: boolean;
  filterDiscoveryLoading: boolean;
  diagnostics: AtlasWorkspaceDiagnostics;
  errorMessage: string | null;
  refresh: () => void;
};

type UseAtlasWorkspaceOptions = {
  enabled?: boolean;
};

export function useSearchWorkspace({
  enabled = true,
}: UseAtlasWorkspaceOptions = {}): SearchWorkspaceState {
  const [interaction, dispatch] = useReducer(
    workspaceInteractionReducer,
    undefined,
    initialWorkspaceInteractionState,
  );
  const { search, selectedRecordKey, focusedResultKey, pageNumber, activeSearch } =
    interaction;
  const [lastDetailRequest, setLastDetailRequest] = useState<AtlasRequestTiming | null>(
    null,
  );
  const [lastResultRequest, setLastResultRequest] =
    useState<AtlasWorkspaceDiagnostics["resultRequest"]>(null);
  const [resultWindow, setResultWindow] = useState<ResultWindowHandle | null>(null);
  const searchToken = useMemo(() => encodeSearchState(search), [search]);
  const searchExecutionToken = useMemo(
    () => encodeSearchExecutionState(search),
    [search],
  );
  const activeSearchExecutionToken = useMemo(
    () => encodeSearchExecutionState(activeSearch),
    [activeSearch],
  );
  const canRunResultSearch = enabled && hasExecutableSearch(activeSearch);
  useEffect(() => {
    const onUrlStateChange = () => {
      const route = currentAtlasRoute();
      dispatch({
        type: "url.restored",
        search: initialWorkspaceInteractionState().search,
        selectedRecordKey: route.kind === "search" ? route.selectedRecordKey : null,
      });
    };
    window.addEventListener("popstate", onUrlStateChange);
    window.addEventListener(ATLAS_ROUTE_CHANGE_EVENT, onUrlStateChange);
    return () => {
      window.removeEventListener("popstate", onUrlStateChange);
      window.removeEventListener(ATLAS_ROUTE_CHANGE_EVENT, onUrlStateChange);
    };
  }, []);

  useEffect(() => {
    const timeout = window.setTimeout(() => {
      dispatch({ type: "search.executionCommitted", search });
    }, SEARCH_REQUEST_DEBOUNCE_MS);
    return () => window.clearTimeout(timeout);
  }, [search, searchToken]);

  const readiness = useQuery({
    queryKey: ["readiness"],
    queryFn: getReadiness,
  });

  const resultsQuery = useQuery({
    queryKey: ["results", activeSearchExecutionToken, pageNumber],
    enabled: canRunResultSearch,
    placeholderData: keepPreviousData,
    queryFn: async () => {
      const startedAt = performance.now();
      if (
        pageNumber === 1 ||
        resultWindow?.searchExecutionToken !== activeSearchExecutionToken
      ) {
        try {
          const page = await openResultWindow(
            buildOpenRequest(activeSearch, pageNumber),
          );
          setResultWindow({
            searchExecutionToken: activeSearchExecutionToken,
            windowId: page.window_id,
          });
          return page;
        } finally {
          setLastResultRequest({
            durationMs: elapsedMilliseconds(startedAt),
            finishedAt: Date.now(),
            kind: "open_window",
          });
        }
      }
      try {
        return await readResultWindowPage(resultWindow.windowId, {
          page: { number: pageNumber, size: activeSearch.pageSize },
        });
      } finally {
        setLastResultRequest({
          durationMs: elapsedMilliseconds(startedAt),
          finishedAt: Date.now(),
          kind: "page",
        });
      }
    },
  });

  const filterDiscoveryContext = useMemo(
    () => buildFilterDiscoveryContext(activeSearch),
    [activeSearch],
  );

  const filterDiscovery = useFilterDiscovery({
    context: filterDiscoveryContext,
    enabled,
    hiddenFieldIds: search.hiddenFilterIds,
    queryKeyPrefix: ["filter-discovery", activeSearchExecutionToken],
    retainedValueQueryKeyPrefix: ["filter-discovery"],
    selectedFieldIds: search.visibleFilterIds,
    visibleFieldIds: search.visibleFilterIds,
  });

  const detailQuery = useQuery({
    queryKey: ["record-detail", selectedRecordKey],
    queryFn: async () => {
      const startedAt = performance.now();
      try {
        return await getRecordDetail(selectedRecordKey!);
      } finally {
        setLastDetailRequest({
          durationMs: elapsedMilliseconds(startedAt),
          finishedAt: Date.now(),
        });
      }
    },
    enabled: enabled && selectedRecordKey !== null,
  });

  const resultRows = useMemo(
    () => (canRunResultSearch ? (resultsQuery.data?.rows ?? []) : []),
    [canRunResultSearch, resultsQuery.data?.rows],
  );

  const activeResultKey = useMemo(() => {
    if (resultRows.length === 0) {
      return null;
    }
    return focusedResultKey &&
      resultRows.some((row) => row.record.record_key === focusedResultKey)
      ? focusedResultKey
      : resultRows[0].record.record_key;
  }, [focusedResultKey, resultRows]);

  function setSearch(next: SearchFormState) {
    dispatch({ type: "search.changed", search: next });
    const url = `${searchPath(selectedRecordKey)}${searchStateQueryString(next)}`;
    history.replaceState(null, "", url);
  }

  function selectRecord(recordKey: string | null) {
    dispatch({ type: "record.selected", recordKey });
    const searchQuery = searchStateQueryString(search);
    const url = `${searchPath(recordKey)}${searchQuery}`;
    history.pushState(null, "", url);
  }

  function moveResultSelection(direction: "next" | "previous") {
    if (resultRows.length === 0) {
      dispatch({ type: "result.focused", recordKey: null });
      return;
    }
    const currentIndex = activeResultKey
      ? resultRows.findIndex((row) => row.record.record_key === activeResultKey)
      : -1;
    const fallbackIndex = direction === "next" ? 0 : resultRows.length - 1;
    const nextIndex =
      currentIndex === -1
        ? fallbackIndex
        : Math.min(
            resultRows.length - 1,
            Math.max(0, currentIndex + (direction === "next" ? 1 : -1)),
          );
    dispatch({
      type: "result.focused",
      recordKey: resultRows[nextIndex].record.record_key,
    });
  }

  function focusResult(recordKey: string) {
    dispatch({ type: "result.focused", recordKey });
  }

  function openActiveResult() {
    if (activeResultKey) {
      selectRecord(activeResultKey);
    }
  }

  const errorMessage =
    messageFromError(resultsQuery.error) ??
    messageFromError(detailQuery.error) ??
    filterDiscovery.errorMessage ??
    messageFromError(readiness.error);
  const searchDebouncing = activeSearchExecutionToken !== searchExecutionToken;
  const activeWindowId =
    resultWindow?.searchExecutionToken === activeSearchExecutionToken
      ? resultWindow.windowId.toString()
      : null;
  const resultsRefreshing =
    canRunResultSearch &&
    (searchDebouncing ||
      resultsQuery.isPlaceholderData ||
      (resultsQuery.isFetching && !resultsQuery.isLoading));

  return {
    search,
    setSearch,
    activeResultKey,
    focusResult,
    moveResultSelection,
    openActiveResult,
    selectedRecordKey,
    selectRecord,
    pageNumber,
    setPageNumber: (pageNumber) => dispatch({ type: "resultPage.changed", pageNumber }),
    resultPage: canRunResultSearch ? resultsQuery.data : undefined,
    recordDetail: detailQuery.data,
    filterEditor: filterDiscovery.filterEditor,
    filterValuesByField: filterDiscovery.filterValuesByField,
    readiness,
    resultsLoading: canRunResultSearch && (resultsQuery.isLoading || searchDebouncing),
    resultsRefreshing,
    detailLoading: detailQuery.isLoading || detailQuery.isFetching,
    filterDiscoveryLoading: filterDiscovery.loading,
    diagnostics: {
      activeWindowId,
      detailRequest: lastDetailRequest,
      resultRequest: lastResultRequest,
      searchDebouncing,
    },
    errorMessage,
    refresh: () => {
      void readiness.refetch();
      if (!enabled) {
        return;
      }
      if (canRunResultSearch) {
        void resultsQuery.refetch();
      }
      void detailQuery.refetch();
    },
  };
}

function messageFromError(error: unknown): string | null {
  if (!error) {
    return null;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return String(error);
}

function elapsedMilliseconds(startedAt: number): number {
  return Math.max(0, Math.round(performance.now() - startedAt));
}

export function resetSearchState(): SearchFormState {
  return DEFAULT_SEARCH_STATE;
}
