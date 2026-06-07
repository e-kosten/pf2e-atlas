import { useEffect, useMemo, useState } from "react";
import {
  useQueries,
  useQuery,
  type UseQueryResult,
} from "@tanstack/react-query";
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
  decodeSearchState,
  DEFAULT_SEARCH_STATE,
  encodeSearchExecutionState,
  encodeSearchState,
  type SearchFormState,
} from "../state/searchState";

const SEARCH_REQUEST_DEBOUNCE_MS = 300;

export type AtlasRequestTiming = {
  durationMs: number;
  finishedAt: number;
};

export type AtlasWorkspaceDiagnostics = {
  activeWindowId: string | null;
  detailRequest: AtlasRequestTiming | null;
  resultRequest: (AtlasRequestTiming & {
    kind: "open_window" | "page";
  }) | null;
  searchDebouncing: boolean;
};

export type AtlasWorkspaceState = {
  search: SearchFormState;
  setSearch: (next: SearchFormState) => void;
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
  const [search, setSearchState] = useState<SearchFormState>(() =>
    decodeSearchState(new URLSearchParams(window.location.search).get("s")),
  );
  const [selectedRecordKey, setSelectedRecordKey] = useState<string | null>(
    () => recordKeyFromPath(window.location.pathname),
  );
  const [pageNumber, setPageNumber] = useState(search.pageSize > 0 ? 1 : 1);
  const [windowId, setWindowId] = useState<bigint | null>(null);
  const [activeSearch, setActiveSearch] = useState(search);
  const [lastDetailRequest, setLastDetailRequest] =
    useState<AtlasRequestTiming | null>(null);
  const [lastResultRequest, setLastResultRequest] = useState<
    AtlasWorkspaceDiagnostics["resultRequest"]
  >(null);
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
      setSearchState(
        decodeSearchState(new URLSearchParams(window.location.search).get("s")),
      );
      setSelectedRecordKey(recordKeyFromPath(window.location.pathname));
    };
    window.addEventListener("popstate", onPopState);
    return () => window.removeEventListener("popstate", onPopState);
  }, []);

  useEffect(() => {
    const timeout = window.setTimeout(() => {
      setWindowId(null);
      setPageNumber(1);
      setActiveSearch(search);
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
      if (windowId === null || pageNumber === 1) {
        try {
          const page = await openResultWindow(
            buildOpenRequest(activeSearch, pageNumber),
          );
          setWindowId(page.window_id);
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
        return await readResultWindowPage(windowId, {
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
      fields
        .filter((field) => field.supports_counts)
        .map((field) => field.id),
    );
    return search.visibleFilterIds.filter((fieldId) =>
      countBackedFields.has(fieldId),
    );
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

  function setSearch(next: SearchFormState) {
    setSearchState(next);
    const url = selectedRecordKey
      ? `/records/${encodeURIComponent(selectedRecordKey)}?s=${encodeSearchState(next)}`
      : `/?s=${encodeSearchState(next)}`;
    history.replaceState(null, "", url);
  }

  function selectRecord(recordKey: string | null) {
    setSelectedRecordKey(recordKey);
    const url =
      recordKey === null
        ? `/?s=${searchToken}`
        : `/records/${encodeURIComponent(recordKey)}?s=${searchToken}`;
    history.pushState(null, "", url);
  }

  const errorMessage =
    messageFromError(resultsQuery.error) ??
    messageFromError(detailQuery.error) ??
    messageFromError(filterFieldsQuery.error) ??
    filterValueQueries.map((query) => messageFromError(query.error)).find(Boolean) ??
    messageFromError(readiness.error);
  const searchDebouncing = activeSearchExecutionToken !== searchExecutionToken;
  const filterValuesByField = Object.fromEntries(
    valueFieldIds.map((fieldId, index) => [
      fieldId,
      filterValueQueries[index]?.data,
    ]),
  );

  return {
    search,
    setSearch,
    selectedRecordKey,
    selectRecord,
    pageNumber,
    setPageNumber,
    resultPage: resultsQuery.data,
    recordDetail: detailQuery.data,
    filterFields: filterFieldsQuery.data,
    filterValuesByField,
    readiness,
    resultsLoading:
      resultsQuery.isLoading ||
      resultsQuery.isFetching ||
      searchDebouncing,
    detailLoading: detailQuery.isLoading || detailQuery.isFetching,
    filterDiscoveryLoading:
      filterFieldsQuery.isLoading ||
      filterFieldsQuery.isFetching ||
      filterValueQueries.some((query) => query.isLoading || query.isFetching),
    diagnostics: {
      activeWindowId: windowId?.toString() ?? null,
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

function recordKeyFromPath(pathname: string): string | null {
  const match = pathname.match(/^\/records\/(.+)$/);
  return match ? decodeURIComponent(match[1]) : null;
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
