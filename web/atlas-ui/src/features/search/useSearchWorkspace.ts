import { searchDestination } from "../../app/destinations";
import {
  useEffect,
  useEffectEvent,
  useLayoutEffect,
  useMemo,
  useReducer,
  useRef,
  useState,
} from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { searchPage } from "../../api/atlasApi";
import { assertSearchIdentity } from "../../shared/search/searchIdentity";
import type {
  FilterEditorView,
  SearchResultsPage,
  QueryPredicate,
} from "../../generated/atlas";
import {
  decodeSearchStateFromParams,
  buildFilterDiscoveryContext,
  buildSearchRequest,
  effectiveResultSort,
  normalizeSortTransition,
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
import {
  rootNavigation,
  useReferenceReading,
  type ReferenceReading,
} from "../../shared/records/useReferenceReading";
import {
  useBrowsePositions,
  type BrowsePositions,
} from "../../shared/records/useBrowsePositions";
import {
  initialWorkspaceInteractionState,
  rememberKindPreference,
  workspaceInteractionReducer,
} from "./workspaceState";
import { atlasRouter } from "../../app/router";
import { useRouterState } from "@tanstack/react-router";
import { currentSearchLocation } from "../../app/searchLocation";
import {
  isArtifactGenerationChanged,
  retireImmutableReferenceQueries,
} from "../../shared/queries/immutableReferenceQueries";
const SEARCH_REQUEST_DEBOUNCE_MS = 300;
export type SearchWorkspaceState = {
  search: SearchFormState;
  reading: ReferenceReading;
  positions: BrowsePositions;
  setSearch: (next: SearchFormState) => void;
  recoverFilterUrl: (restored: SearchFormState) => void;
  selectedRecordKey: string | null;
  selectRecord: (recordKey: string | null) => void;
  pageNumber: number;
  setPageNumber: (page: number) => void;
  resultPage: SearchResultsPage | undefined;
  resultFilter: QueryPredicate | null;
  filterEditor: FilterEditorView | undefined;
  filterCountsEnabled?: boolean;
  filterCountQueryKey?: readonly unknown[];
  resultsLoading: boolean;
  resultsRefreshing: boolean;
  resultsPaused?: boolean;
  resultsRecovering?: boolean;
  refreshResults: () => Promise<void>;
  filterDiscoveryLoading: boolean;
  filterDiscoveryContext: import("../../generated/atlas").FilterDiscoveryContext;
  vocabularyScope: import("../../generated/atlas").FilterVocabularyScope;
  filterDiscoveryError: string | null;
  errorMessage: string | null;
  refresh: () => void;
};
type UseAtlasWorkspaceOptions = {
  enabled?: boolean;
  searchProfile: string | null;
  pendingUrlProfile?: string | null;
  searchProfileStored?: boolean;
  searchProfileUsable?: boolean;
  includeSearchProfileInUrl?: boolean;
};
function urlCriteria(query: string) {
  const decoded = decodeSearchStateFromParams(new URLSearchParams(query));
  return JSON.stringify([
    searchStateQueryString(decoded),
    decoded.filterError,
    decoded.filterWasReset,
  ]);
}

export function useSearchWorkspace({
  enabled = true,
  searchProfile,
  pendingUrlProfile = null,
  searchProfileStored = true,
  searchProfileUsable = true,
  includeSearchProfileInUrl = true,
}: UseAtlasWorkspaceOptions): SearchWorkspaceState {
  // Unresolved metadata is distinct from deliberate single-profile URL omission.
  // This literal comes from the existing selection owner and never enables reads.
  const urlProfile =
    searchProfile === null
      ? pendingUrlProfile
      : includeSearchProfileInUrl
        ? searchProfile
        : null;
  const [interaction, dispatch] = useReducer(
    workspaceInteractionReducer,
    undefined,
    initialWorkspaceInteractionState,
  );
  const { search, selectedRecordKey, activeSearch } = interaction;
  const [pagingProfile, setPagingProfile] = useState(searchProfile);
  const profileChanged = pagingProfile !== searchProfile;
  // Adjust the execution page in this render, before any new-profile query can run.
  // This leaves the reader's artifact epoch and all draft embedded_path untouched.
  if (profileChanged) {
    setPagingProfile(searchProfile);
    if (pagingProfile !== null) dispatch({ type: "resultPage.changed", pageNumber: 1 });
  }
  const pageNumber =
    profileChanged && pagingProfile !== null ? 1 : interaction.pageNumber;
  const queryClient = useQueryClient();
  const [retrieval, setRetrieval] = useState({
    epoch: 0,
    generation: null as string | null,
    recovering: false,
  });
  const location = useRouterState({ select: (state) => state.location });
  const initialRoute = currentSearchLocation();
  const previousUrlCriteria = useRef(urlCriteria(atlasRouter.latestLocation.searchStr));
  const previousUrlReading = useRef(
    JSON.stringify([
      initialRoute?.selectedRecordKey,
      initialRoute?.selection,
      initialRoute?.landing,
    ]),
  );
  const reading = useReferenceReading(
    initialRoute?.selectedRecordKey
      ? {
          navigation:
            initialRoute.selection ?? rootNavigation(initialRoute.selectedRecordKey),
          landing: initialRoute.landing,
        }
      : null,
    enabled && !retrieval.recovering,
    retrieval.epoch,
  );
  const inspectReading = reading.select;
  const positions = useBrowsePositions();
  const liveRequest = useRef({ token: "", epoch: 0, retiring: false });
  const searchToken = useMemo(() => encodeSearchState(search), [search]);
  const searchExecutionToken = useMemo(
    () => encodeSearchExecutionState(search, searchProfile),
    [search, searchProfile],
  );
  const activeSearchExecutionToken = useMemo(
    () => encodeSearchExecutionState(activeSearch, searchProfile),
    [activeSearch, searchProfile],
  );
  const canRunResultSearch =
    enabled &&
    searchProfile !== null &&
    (activeSearch.query.trim() ? searchProfileUsable : searchProfileStored) &&
    hasExecutableSearch(activeSearch);
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
  const restoreLocation = useEffectEvent(() => {
    if (!enabled) return;
    const route = currentSearchLocation();
    if (!route) return;
    const restored = initialWorkspaceInteractionState();
    const incomingCriteria = urlCriteria(atlasRouter.latestLocation.searchStr);
    const criteriaChanged = previousUrlCriteria.current !== incomingCriteria;
    previousUrlCriteria.current = incomingCriteria;
    if (
      criteriaChanged &&
      (searchStateQueryString(restored.search) !== searchStateQueryString(search) ||
        restored.search.filterError !== search.filterError ||
        restored.search.filterWasReset !== search.filterWasReset)
    ) {
      dispatch({
        type: "url.restored",
        search: restored.search,
        selectedRecordKey: route.selectedRecordKey ?? null,
        pageNumber: restored.pageNumber,
      });
    } else {
      if (selectedRecordKey !== route.selectedRecordKey)
        dispatch({
          type: "record.selected",
          recordKey: route.selectedRecordKey ?? null,
        });
      if (pageNumber !== restored.pageNumber)
        dispatch({ type: "resultPage.changed", pageNumber: restored.pageNumber });
    }
    const request = route.selectedRecordKey
      ? {
          navigation: route.selection ?? rootNavigation(route.selectedRecordKey),
          landing: route.landing,
        }
      : null;
    const incomingReading = JSON.stringify([
      route.selectedRecordKey,
      route.selection,
      route.landing,
    ]);
    const readingChanged = previousUrlReading.current !== incomingReading;
    previousUrlReading.current = incomingReading;
    if (
      readingChanged &&
      (JSON.stringify(reading.request?.navigation ?? null) !==
        JSON.stringify(request?.navigation ?? null) ||
        JSON.stringify(reading.request?.landing) !== JSON.stringify(request?.landing))
    )
      inspectReading(request);
  });
  useLayoutEffect(() => {
    restoreLocation();
  }, [enabled, location]);

  useEffect(() => {
    const timeout = window.setTimeout(() => {
      dispatch({ type: "search.executionCommitted", search });
    }, SEARCH_REQUEST_DEBOUNCE_MS);
    return () => window.clearTimeout(timeout);
  }, [search, searchToken]);
  const resultsKey = [
    "results",
    activeSearchExecutionToken,
    retrieval.epoch,
    retrieval.generation,
    pageNumber,
  ];
  const resultsQuery = useQuery<{
    page: SearchResultsPage;
    filter: QueryPredicate | null;
  }>({
    queryKey: resultsKey,
    enabled: canRunResultSearch && !retrieval.recovering,
    staleTime: 20000,
    placeholderData: (previous, previousQuery) =>
      previous?.page.search_profile === searchProfile &&
      previous.page.ranking === (activeSearch.ranking ?? "score") &&
      previousQuery?.queryKey[2] === retrieval.epoch &&
      previousQuery.queryKey[3] === retrieval.generation
        ? previous
        : undefined,
    queryFn: async ({ signal }) => {
      const isCurrent = () =>
        !signal.aborted &&
        !liveRequest.current.retiring &&
        liveRequest.current.epoch === retrieval.epoch &&
        liveRequest.current.token === activeSearchExecutionToken;
      const page = await searchPage(
        buildSearchRequest(
          activeSearch,
          pageNumber,
          retrieval.generation,
          searchProfile ?? undefined,
        ),
        signal,
      );
      assertSearchIdentity(
        page,
        searchProfile,
        retrieval.generation,
        effectiveResultSort(activeSearch),
      );
      if (
        activeSearch.query.trim() &&
        page.ranking !== (activeSearch.ranking ?? "score")
      )
        throw new Error("Search response used a different ranking");
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
          { page, filter: activeSearch.filter },
        );
        setRetrieval((current) =>
          current.epoch === retrieval.epoch && !current.recovering
            ? { ...current, generation: page.artifact_generation }
            : current,
        );
      }
      return { page, filter: activeSearch.filter };
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
  useEffect(() => {
    if (
      !enabled ||
      !reading.detail ||
      search.filterWasReset ||
      !currentSearchLocation()
    )
      return;
    const selected = reading.detail.selected;
    const selection =
      selected.spell_rank !== undefined ||
      selected.embedded_path.length ||
      selected.field ||
      selected.passage ||
      selected.source_fingerprint
        ? selected
        : undefined;
    const destination = searchDestination({
      selectedRecordKey: reading.detail.record.record_key,
      query: searchStateQueryString(search, urlProfile),
      selection,
      landing: reading.landing,
    });
    if (atlasRouter.buildLocation(destination).href !== atlasRouter.latestLocation.href)
      void atlasRouter.navigate({
        ...destination,
        replace: true,
        resetScroll: false,
        state: (state) => ({ ...state, atlasSearchPage: pageNumber }),
      });
  }, [enabled, reading.detail, reading.landing, search, urlProfile, pageNumber]);

  useEffect(() => {
    if (!enabled || urlProfile === null || !currentSearchLocation()) return;
    if (atlasRouter.latestLocation.search.search_profile === urlProfile) return;
    void atlasRouter.navigate({
      to: ".",
      search: (search) => ({ ...search, search_profile: urlProfile }),
      replace: true,
      resetScroll: false,
      state: (state) => ({ ...state, atlasSearchPage: pageNumber }),
    });
  }, [enabled, urlProfile, pageNumber]);

  const filterDiscoveryContext = useMemo(
    () =>
      buildFilterDiscoveryContext(
        search,
        searchProfile ?? undefined,
        retrieval.generation ?? undefined,
      ),
    [search, searchProfile, retrieval.generation],
  );
  const filterDiscovery = useFilterDiscovery({
    context: filterDiscoveryContext,
    enabled: enabled && !retrieval.recovering,
    queryKeyPrefix: ["filter-discovery", retrieval.epoch],
    catalogIdentity: [retrieval.epoch],
  });
  function setSearch(next: SearchFormState) {
    // A submitted canonical address is navigation, independent of search eligibility.
    const recordKey = next.query.trim();
    if (next.query !== search.query && /^[^:\s]+:[^:\s]+$/u.test(recordKey)) {
      selectRecord(recordKey);
      return;
    }
    // Keyword entry intentionally replaces similarity, while ordinary filters retain it.
    if (next.query.trim()) next = { ...next, similarSeed: null, mode: "text_search" };
    else next = { ...next, mode: next.similarSeed ? "similar_records" : "browse" };
    next = normalizeSortTransition(search, next);
    if (
      next.filterError === SIMILAR_QUERY_CONFLICT &&
      (next.query !== search.query || next.similarSeed !== search.similarSeed)
    )
      next = { ...next, filterError: null };
    const kind = filterDiscovery.filterEditor?.groups
      .flatMap((group) => group.fields)
      .find((field) => field.definition.path === "kind");
    if (
      kind &&
      JSON.stringify(commonCondition(search.filter, kind.definition)) !==
        JSON.stringify(commonCondition(next.filter, kind.definition))
    )
      rememberKindPreference(next.filter, kind.definition);
    dispatch({ type: "search.changed", search: next });
    void atlasRouter.navigate({
      ...searchDestination({
        selectedRecordKey,
        query: searchStateQueryString(next, urlProfile),
      }),
      replace: search.similarSeed === next.similarSeed,
      resetScroll: false,
      state: (state) => ({ ...state, atlasSearchPage: 1 }),
    });
  }
  function selectRecord(recordKey: string | null) {
    dispatch({ type: "record.selected", recordKey });
    const searchQuery = searchStateQueryString(search, urlProfile);
    void atlasRouter.navigate({
      ...searchDestination({ selectedRecordKey: recordKey, query: searchQuery }),
      replace: true,
      resetScroll: false,
      state: (state) => ({ ...state, atlasSearchPage: pageNumber }),
    });
    reading.select(recordKey ? { navigation: rootNavigation(recordKey) } : null);
  }
  const errorMessage = messageFromError(resultsQuery.error);
  const searchDebouncing = activeSearchExecutionToken !== searchExecutionToken;
  const resultsRefreshing =
    canRunResultSearch &&
    (searchDebouncing ||
      resultsQuery.isPlaceholderData ||
      (resultsQuery.isFetching && !resultsQuery.isLoading));
  return {
    search,
    setSearch,
    recoverFilterUrl: (restored) =>
      dispatch({
        type: "url.restored",
        search: restored,
        selectedRecordKey,
        pageNumber: 1,
      }),
    reading,
    positions,
    selectedRecordKey,
    selectRecord,
    pageNumber,
    setPageNumber: (pageNumber) => {
      if (!resultsPaused && !retrieval.recovering) {
        dispatch({ type: "resultPage.changed", pageNumber });
        void atlasRouter.navigate({
          to: ".",
          search: true,
          replace: true,
          resetScroll: false,
          state: (state) => ({ ...state, atlasSearchPage: pageNumber }),
        });
      }
    },
    resultPage: canRunResultSearch ? resultsQuery.data?.page : undefined,
    resultFilter: canRunResultSearch ? (resultsQuery.data?.filter ?? null) : null,
    filterEditor: filterDiscovery.filterEditor,
    filterCountsEnabled:
      filterDiscovery.filterCountsEnabled &&
      !search.traitReferenceSeed &&
      searchProfile !== null &&
      searchProfileStored,
    filterCountQueryKey: filterDiscovery.filterCountQueryKey,
    filterDiscoveryContext: filterDiscovery.filterDiscoveryContext,
    vocabularyScope: filterDiscovery.vocabularyScope,
    filterDiscoveryError: filterDiscovery.errorMessage,
    resultsLoading:
      enabled &&
      searchProfile !== null &&
      (search.query.trim() ? searchProfileUsable : searchProfileStored) &&
      hasExecutableSearch(search) &&
      (resultsQuery.isLoading || searchDebouncing),
    resultsRefreshing,
    resultsPaused,
    resultsRecovering: retrieval.recovering,
    refreshResults,
    filterDiscoveryLoading: filterDiscovery.loading,
    errorMessage,
    refresh: () => {
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
      reading.retry();
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
export function resetSearchState(): SearchFormState {
  return DEFAULT_SEARCH_STATE;
}
