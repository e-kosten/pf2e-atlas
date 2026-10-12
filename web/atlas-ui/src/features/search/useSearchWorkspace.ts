import {
  useEffect,
  useLayoutEffect,
  useMemo,
  useReducer,
  useRef,
  useState,
} from "react";
import { useQuery, useQueryClient, type UseQueryResult } from "@tanstack/react-query";
import { getReadiness, getRecordDetail, searchPage } from "../../api/atlasApi";
import type {
  FilterEditorView,
  RecordDetailView,
  SearchResultsPage,
} from "../../generated/atlas";
import {
  buildFilterDiscoveryContext,
  buildSearchRequest,
  DEFAULT_SEARCH_STATE,
  SIMILAR_QUERY_CONFLICT,
  encodeSearchExecutionState,
  encodeSearchState,
  hasExecutableSearch,
  searchStateQueryString,
  type SearchFormState,
} from "../../shared/filters/searchState";
import { useFilterDiscovery } from "../../shared/filters/useFilterDiscovery";
import { commonCondition } from "../../shared/filters/filterControls";
import { recordDetailQueryOptions } from "../../shared/records/recordDetailQuery";
import {
  initialWorkspaceInteractionState,
  rememberKindPreference,
  workspaceInteractionReducer,
} from "./workspaceState";
import {
  ATLAS_ROUTE_CHANGE_EVENT,
  currentAtlasRoute,
  searchPath,
} from "../../app/routes";

import {
  isArtifactGenerationChanged,
  retireImmutableReferenceQueries,
} from "../../shared/queries/immutableReferenceQueries";

const SEARCH_REQUEST_DEBOUNCE_MS = 300;

export type AtlasRequestTiming = {
  durationMs: number;
  finishedAt: number;
};

export type AtlasWorkspaceDiagnostics = {
  detailRequest: AtlasRequestTiming | null;
  resultRequest:
    | (AtlasRequestTiming & {
        kind: "page";
      })
    | null;
  searchDebouncing: boolean;
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
  resultPage: SearchResultsPage | undefined;
  recordDetail: RecordDetailView | undefined;
  detailError: Error | null;
  retryDetail: () => void;
  filterEditor: FilterEditorView | undefined;
  filterCountsByField?: Record<
    string,
    import("../../generated/atlas").FilterCountsView | undefined
  >;
  readiness: UseQueryResult<Awaited<ReturnType<typeof getReadiness>>, Error>;
  resultsLoading: boolean;
  resultsRefreshing: boolean;
  resultsPaused?: boolean;
  resultsRecovering?: boolean;
  refreshResults: () => Promise<void>;
  detailLoading: boolean;
  filterDiscoveryLoading: boolean;
  filterDiscoveryContext: import("../../generated/atlas").FilterDiscoveryContext;
  vocabularyScope: import("../../generated/atlas").FilterVocabularyScope;
  filterDiscoveryError: string | null;
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
  const queryClient = useQueryClient();
  const [retrieval, setRetrieval] = useState({
    epoch: 0,
    generation: null as string | null,
    recovering: false,
  });
  const liveRequest = useRef({ token: "", epoch: 0, retiring: false });
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
  useLayoutEffect(() => {
    liveRequest.current = {
      token: activeSearchExecutionToken,
      epoch: retrieval.epoch,
      retiring: retrieval.recovering,
    };
    return () => {
      liveRequest.current.retiring = true;
    };
  }, [activeSearchExecutionToken, retrieval.epoch, retrieval.recovering]);
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

  const resultsKey = [
    "results",
    activeSearchExecutionToken,
    retrieval.epoch,
    retrieval.generation,
    pageNumber,
  ];
  const resultsQuery = useQuery<SearchResultsPage>({
    queryKey: resultsKey,
    enabled: canRunResultSearch && !retrieval.recovering,
    staleTime: 20_000,
    placeholderData: (previous, previousQuery) =>
      previousQuery?.queryKey[2] === retrieval.epoch &&
      previousQuery.queryKey[3] === retrieval.generation
        ? previous
        : undefined,
    queryFn: async ({ signal }) => {
      const startedAt = performance.now();
      const isCurrent = () =>
        !signal.aborted &&
        !liveRequest.current.retiring &&
        liveRequest.current.epoch === retrieval.epoch &&
        liveRequest.current.token === activeSearchExecutionToken;
      try {
        const page = await searchPage(
          buildSearchRequest(activeSearch, pageNumber, retrieval.generation),
          signal,
        );
        if (retrieval.generation === null && isCurrent()) {
          // Adopt the returned generation without issuing the same search again.
          queryClient.setQueryData(
            [
              "results",
              activeSearchExecutionToken,
              retrieval.epoch,
              page.artifact_generation,
              pageNumber,
            ],
            page,
          );
          setRetrieval((current) =>
            current.epoch === retrieval.epoch && !current.recovering
              ? { ...current, generation: page.artifact_generation }
              : current,
          );
        }
        return page;
      } finally {
        if (isCurrent())
          setLastResultRequest({
            durationMs: elapsedMilliseconds(startedAt),
            finishedAt: Date.now(),
            kind: "page",
          });
      }
    },
  });
  const resultsPaused = isArtifactGenerationChanged(resultsQuery.error);

  async function refreshResults() {
    if (liveRequest.current.retiring) return;
    // Retire the owner before awaiting cancellation, so a late completion cannot seed.
    liveRequest.current.retiring = true;
    setRetrieval((current) => ({ ...current, recovering: true }));
    await retireImmutableReferenceQueries(queryClient);
    dispatch({ type: "resultPage.changed", pageNumber: 1 });
    setRetrieval((current) => ({
      epoch: current.epoch + 1,
      generation: null,
      recovering: false,
    }));
  }

  const filterDiscoveryContext = useMemo(
    () => buildFilterDiscoveryContext(activeSearch),
    [activeSearch],
  );

  const filterDiscovery = useFilterDiscovery({
    context: filterDiscoveryContext,
    enabled: enabled && !retrieval.recovering,
    countsEnabled: !activeSearch.similarSeed,
    queryKeyPrefix: ["filter-discovery", retrieval.epoch, activeSearchExecutionToken],
    placeholderIdentity: [retrieval.epoch],
  });

  const detailQuery = useQuery({
    ...recordDetailQueryOptions(selectedRecordKey, undefined, retrieval.epoch),
    queryFn: async ({ signal }) => {
      const startedAt = performance.now();
      try {
        return await getRecordDetail(selectedRecordKey!, undefined, signal);
      } finally {
        if (!signal.aborted)
          setLastDetailRequest({
            durationMs: elapsedMilliseconds(startedAt),
            finishedAt: Date.now(),
          });
      }
    },
    enabled: enabled && selectedRecordKey !== null && !retrieval.recovering,
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
    // Keyword entry intentionally replaces similarity, while ordinary filters retain it.
    if (next.query.trim()) next = { ...next, similarSeed: null, mode: "text_search" };
    else next = { ...next, mode: next.similarSeed ? "similar_records" : "browse" };
    if (
      next.filterError === SIMILAR_QUERY_CONFLICT &&
      (next.query !== search.query || next.similarSeed !== search.similarSeed)
    )
      next = { ...next, filterError: null };
    const kind = filterDiscovery.filterEditor?.groups
      .flatMap((group) => group.fields)
      .find((field) => field.definition.path === "record.kind");
    if (
      kind &&
      JSON.stringify(commonCondition(search.filter, kind.definition)) !==
        JSON.stringify(commonCondition(next.filter, kind.definition))
    )
      rememberKindPreference(next.filter, kind.definition);
    dispatch({ type: "search.changed", search: next });
    const url = `${searchPath(selectedRecordKey)}${searchStateQueryString(next)}`;
    if (search.similarSeed !== next.similarSeed) history.pushState(null, "", url);
    else history.replaceState(null, "", url);
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
    messageFromError(resultsQuery.error) ?? messageFromError(readiness.error);
  const searchDebouncing = activeSearchExecutionToken !== searchExecutionToken;
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
    setPageNumber: (pageNumber) => {
      if (!resultsPaused && !retrieval.recovering)
        dispatch({ type: "resultPage.changed", pageNumber });
    },
    resultPage: canRunResultSearch ? resultsQuery.data : undefined,
    recordDetail: detailQuery.data,
    detailError: detailQuery.error,
    retryDetail: () => {
      void detailQuery.refetch();
    },
    filterEditor: filterDiscovery.filterEditor,
    filterCountsByField: filterDiscovery.filterCountsByField,
    filterDiscoveryContext: filterDiscovery.filterDiscoveryContext,
    vocabularyScope: filterDiscovery.vocabularyScope,
    filterDiscoveryError: filterDiscovery.errorMessage,
    readiness,
    resultsLoading: canRunResultSearch && (resultsQuery.isLoading || searchDebouncing),
    resultsRefreshing,
    resultsPaused,
    resultsRecovering: retrieval.recovering,
    refreshResults,
    detailLoading: detailQuery.isLoading || detailQuery.isFetching,
    filterDiscoveryLoading: filterDiscovery.loading,
    diagnostics: {
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
      if (resultsPaused) {
        void refreshResults();
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
