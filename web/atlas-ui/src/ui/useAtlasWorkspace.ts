import { useEffect, useMemo, useReducer, useState } from "react";
import { useQueries, useQuery, type UseQueryResult } from "@tanstack/react-query";
import {
  discoverFilterFields,
  discoverFilterValues,
  getReadiness,
  getRecordDetail,
  openResultWindow,
  readResultWindowPage,
} from "../api/atlasApi";
import type {
  FilterFieldListView,
  FilterValueListView,
  RecordDetailView,
  ResultWindowPage,
} from "../generated/atlas";
import {
  buildFilterDiscoveryContext,
  buildOpenRequest,
  DEFAULT_SEARCH_STATE,
  encodeSearchExecutionState,
  encodeSearchState,
  type SearchFormState,
} from "../state/searchState";
import {
  initialWorkspaceInteractionState,
  recordKeyFromPath,
  workspaceInteractionReducer,
} from "./workspaceState";

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

export type AtlasWorkspaceState = {
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
  filterFields: FilterFieldListView | undefined;
  filterValuesByField: Record<string, FilterValueListView | undefined>;
  readiness: UseQueryResult<Awaited<ReturnType<typeof getReadiness>>, Error>;
  resultsLoading: boolean;
  detailLoading: boolean;
  filterDiscoveryLoading: boolean;
  diagnostics: AtlasWorkspaceDiagnostics;
  errorMessage: string | null;
  refresh: () => void;
};

export function useAtlasWorkspace(): AtlasWorkspaceState {
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
  useEffect(() => {
    const onPopState = () => {
      dispatch({
        type: "url.restored",
        search: initialWorkspaceInteractionState().search,
        selectedRecordKey: recordKeyFromPath(window.location.pathname),
      });
    };
    window.addEventListener("popstate", onPopState);
    return () => window.removeEventListener("popstate", onPopState);
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

  const filterFieldsQuery = useQuery({
    queryKey: ["filter-fields", activeSearchExecutionToken],
    queryFn: () => discoverFilterFields({ context: filterDiscoveryContext }),
  });

  const valueFieldIds = useMemo(() => {
    const fields = filterFieldsQuery.data?.fields ?? [];
    const countBackedFields = new Set(
      fields.filter((field) => field.supports_counts).map((field) => field.id),
    );
    return search.visibleFilterIds.filter((fieldId) => countBackedFields.has(fieldId));
  }, [filterFieldsQuery.data, search.visibleFilterIds]);

  const filterValueQueries = useQueries({
    queries: valueFieldIds.map((fieldId) => ({
      queryKey: ["filter-values", activeSearchExecutionToken, fieldId],
      queryFn: () =>
        discoverFilterValues({
          context: filterDiscoveryContext,
          field_id: fieldId,
        }),
    })),
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
    enabled: selectedRecordKey !== null,
  });

  const resultRows = useMemo(
    () => resultsQuery.data?.rows ?? [],
    [resultsQuery.data?.rows],
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
    const url = selectedRecordKey
      ? `/records/${encodeURIComponent(selectedRecordKey)}?s=${encodeSearchState(next)}`
      : `/?s=${encodeSearchState(next)}`;
    history.replaceState(null, "", url);
  }

  function selectRecord(recordKey: string | null) {
    dispatch({ type: "record.selected", recordKey });
    const url =
      recordKey === null
        ? `/?s=${searchToken}`
        : `/records/${encodeURIComponent(recordKey)}?s=${searchToken}`;
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
    messageFromError(filterFieldsQuery.error) ??
    filterValueQueries.map((query) => messageFromError(query.error)).find(Boolean) ??
    messageFromError(readiness.error);
  const searchDebouncing = activeSearchExecutionToken !== searchExecutionToken;
  const activeWindowId =
    resultWindow?.searchExecutionToken === activeSearchExecutionToken
      ? resultWindow.windowId.toString()
      : null;
  const filterValuesByField = Object.fromEntries(
    valueFieldIds.map((fieldId, index) => [fieldId, filterValueQueries[index]?.data]),
  );

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
    resultPage: resultsQuery.data,
    recordDetail: detailQuery.data,
    filterFields: filterFieldsQuery.data,
    filterValuesByField,
    readiness,
    resultsLoading:
      resultsQuery.isLoading || resultsQuery.isFetching || searchDebouncing,
    detailLoading: detailQuery.isLoading || detailQuery.isFetching,
    filterDiscoveryLoading:
      filterFieldsQuery.isLoading ||
      filterFieldsQuery.isFetching ||
      filterValueQueries.some((query) => query.isLoading || query.isFetching),
    diagnostics: {
      activeWindowId,
      detailRequest: lastDetailRequest,
      resultRequest: lastResultRequest,
      searchDebouncing,
    },
    errorMessage,
    refresh: () => {
      void readiness.refetch();
      void resultsQuery.refetch();
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
