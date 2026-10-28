import {
  failedFilterTransport,
  searchPage as urlSearchPage,
} from "../../app/routeSearch";
import { totalPages } from "./pageMetrics";
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
import { getSimilarityParts, searchPage } from "../../api/atlasApi";
import {
  assertSearchIdentity,
  assertSimilarityIdentity,
  assertSimilarityInventoryIdentity,
} from "../../shared/search/searchIdentity";
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
  type ReadingRequest,
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
  DEFAULT_SEARCH_PREFERENCES,
  type SearchPreferences,
} from "../../shared/search/searchPreferences";
import {
  hasCoreSourcePreference,
  preferredSourceSections,
  similarityFromPreferences,
} from "../../shared/search/similarityState";
import {
  isArtifactGenerationChanged,
  retireImmutableReferenceQueries,
} from "../../shared/queries/immutableReferenceQueries";
import { readingNavigationForUrl } from "../../shared/records/readingUrlNavigation";
import { recordNavigationIdentity } from "../../shared/records/recordNavigationIdentity";
function readingRequestIdentity(request: ReadingRequest | null) {
  return JSON.stringify([
    recordNavigationIdentity(
      request?.navigation.record_key ?? null,
      request?.navigation,
    ),
    recordNavigationIdentity(request?.match?.record_key ?? null, request?.match),
    recordNavigationIdentity(request?.landing?.record_key ?? null, request?.landing),
  ]);
}

function urlReadingIdentity(route: ReturnType<typeof currentSearchLocation>) {
  return JSON.stringify([
    recordNavigationIdentity(route?.selectedRecordKey ?? null, route?.selection),
    recordNavigationIdentity(route?.landing?.record_key ?? null, route?.landing),
  ]);
}

const SEARCH_REQUEST_DEBOUNCE_MS = 300;
export type SearchWorkspaceState = {
  search: SearchFormState;
  reading: ReferenceReading;
  positions: BrowsePositions;
  setSearch: (next: SearchFormState) => void;
  applySettings?: (next: SearchFormState, profile: string) => void;
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
  searchPreferences?: SearchPreferences;
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
  searchPreferences = DEFAULT_SEARCH_PREFERENCES,
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
    searchPreferences,
    initialWorkspaceInteractionState,
  );
  const { search, selectedRecordKey, activeSearch } = interaction;
  const queryClient = useQueryClient();
  const seedPreparation = useRef(0);
  const [preparingSeed, setPreparingSeed] = useState(false);
  const [sourcePreparationError, setSourcePreparationError] = useState<string | null>(
    null,
  );
  if (!enabled && preparingSeed) setPreparingSeed(false);
  useEffect(() => {
    return () => {
      seedPreparation.current += 1;
    };
  }, [enabled]);
  const [retrieval, setRetrieval] = useState({
    epoch: 0,
    generation: null as string | null,
    recovering: false,
  });
  const location = useRouterState({ select: (state) => state.location });
  const previousProfile = useRef(searchProfile);
  const profileChanged =
    previousProfile.current !== null &&
    previousProfile.current !== searchProfile &&
    atlasRouter.latestLocation.search.search_profile !== searchProfile;
  const pageNumber = profileChanged
    ? 1
    : urlSearchPage(atlasRouter.latestLocation.search);
  useLayoutEffect(() => {
    if (previousProfile.current === searchProfile) return;
    const shouldReset =
      previousProfile.current !== null &&
      atlasRouter.latestLocation.search.search_profile !== searchProfile;
    previousProfile.current = searchProfile;
    if (enabled && shouldReset && currentSearchLocation())
      void atlasRouter.navigate({
        to: ".",
        hash: true,
        search: (search) => ({ ...search, page: undefined }),
        replace: true,
        resetScroll: false,
        state: true,
      });
  }, [enabled, searchProfile]);
  const initialRoute = currentSearchLocation();
  const failedTransport =
    initialRoute !== null &&
    failedFilterTransport("search", atlasRouter.latestLocation.search);
  const previousUrlCriteria = useRef(urlCriteria(atlasRouter.latestLocation.searchStr));
  const previousReset = useRef(atlasRouter.latestLocation.state.atlasSearchReset);
  const resetPending =
    previousReset.current !== atlasRouter.latestLocation.state.atlasSearchReset;
  const previousUrlReading = useRef(urlReadingIdentity(initialRoute));
  const pendingUrlReading = useRef<{ incoming: string; previous: string } | null>(null);
  const reading = useReferenceReading(
    initialRoute?.selectedRecordKey
      ? {
          navigation:
            initialRoute.selection ?? rootNavigation(initialRoute.selectedRecordKey),
          landing: initialRoute.landing,
        }
      : null,
    enabled &&
      initialRoute !== null &&
      !failedTransport &&
      !resetPending &&
      !search.filterWasReset &&
      !retrieval.recovering,
    retrieval.epoch,
  );
  const inspectReading = reading.select;
  const positions = useBrowsePositions();
  const liveRequest = useRef({
    token: "",
    epoch: 0,
    retiring: false,
    generation: null as string | null,
  });
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
    initialRoute !== null &&
    !failedTransport &&
    !resetPending &&
    searchProfile !== null &&
    (activeSearch.query.trim() ? searchProfileUsable : searchProfileStored) &&
    hasExecutableSearch(activeSearch);
  useLayoutEffect(() => {
    liveRequest.current = {
      token: activeSearchExecutionToken,
      epoch: retrieval.epoch,
      retiring: retrieval.recovering,
      generation: retrieval.generation,
    };
    return () => {
      liveRequest.current.retiring = true;
    };
  }, [
    activeSearchExecutionToken,
    retrieval.epoch,
    retrieval.recovering,
    retrieval.generation,
  ]);
  const restoreLocation = useEffectEvent(() => {
    if (!enabled) return;
    const route = currentSearchLocation();
    if (!route) return;
    const restored = initialWorkspaceInteractionState(searchPreferences);
    const incomingCriteria = urlCriteria(atlasRouter.latestLocation.searchStr);
    const resetChanged =
      previousReset.current !== atlasRouter.latestLocation.state.atlasSearchReset;
    previousReset.current = atlasRouter.latestLocation.state.atlasSearchReset;
    const criteriaChanged =
      previousUrlCriteria.current !== incomingCriteria || resetChanged;
    if (criteriaChanged) {
      seedPreparation.current += 1;
      setPreparingSeed(false);
      setSourcePreparationError(null);
    }
    previousUrlCriteria.current = incomingCriteria;
    // Decoding fills an omitted ranking; that alone must not commit a local edit.
    if (
      criteriaChanged &&
      (resetChanged ||
        searchStateQueryString(restored.search) !==
          searchStateQueryString({
            ...search,
            ranking: search.ranking ?? searchPreferences.ranking,
          }) ||
        restored.search.filterError !== search.filterError ||
        restored.search.filterWasReset !== search.filterWasReset)
    ) {
      dispatch({
        type: "url.restored",
        search: restored.search,
        selectedRecordKey: route.selectedRecordKey ?? null,
      });
    } else {
      if (selectedRecordKey !== route.selectedRecordKey)
        dispatch({
          type: "record.selected",
          recordKey: route.selectedRecordKey ?? null,
        });
    }
    const request = route.selectedRecordKey
      ? {
          navigation: route.selection ?? rootNavigation(route.selectedRecordKey),
          landing: route.landing,
        }
      : null;
    const incomingReading = urlReadingIdentity(route);
    const readingChanged = previousUrlReading.current !== incomingReading;
    previousUrlReading.current = incomingReading;
    const currentSelection = reading.detail
      ? readingNavigationForUrl(reading.detail.selected, reading.request)
      : (reading.request?.match ?? reading.request?.navigation);
    const currentLanding = reading.landing ?? reading.request?.landing;
    if (
      readingChanged &&
      (recordNavigationIdentity(
        currentSelection?.record_key ?? null,
        currentSelection,
      ) !==
        recordNavigationIdentity(
          request?.navigation.record_key ?? null,
          request?.navigation,
        ) ||
        recordNavigationIdentity(currentLanding?.record_key ?? null, currentLanding) !==
          recordNavigationIdentity(
            request?.landing?.record_key ?? null,
            request?.landing,
          ))
    ) {
      pendingUrlReading.current = {
        incoming: readingRequestIdentity(request),
        previous: readingRequestIdentity(reading.request),
      };
      inspectReading(request);
    } else if (readingChanged) {
      pendingUrlReading.current = null;
    }
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
        activeSearch.similarityGeneration ?? retrieval.generation,
        effectiveResultSort(activeSearch),
        !activeSearch.traitReferenceSeed &&
          (activeSearch.similarSeed || activeSearch.query.trim())
          ? (activeSearch.searchContent ?? "all")
          : undefined,
      );
      if (activeSearch.similarSeed)
        assertSimilarityIdentity(
          page,
          activeSearch.similarity,
          activeSearch.sourceSections,
          activeSearch.searchContent ?? "all",
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
  useEffect(() => {
    const response = resultsQuery.data?.page;
    const route = currentSearchLocation();

    if (
      !canRunResultSearch ||
      retrieval.recovering ||
      !route ||
      !response ||
      !resultsQuery.isSuccess ||
      resultsQuery.isFetching ||
      resultsQuery.isPlaceholderData ||
      activeSearchExecutionToken !== searchExecutionToken ||
      urlSearchPage(atlasRouter.latestLocation.search) !== pageNumber ||
      response.page.number !== pageNumber ||
      response.page.size !== activeSearch.pageSize ||
      response.artifact_generation !== retrieval.generation ||
      response.search_profile !== searchProfile ||
      urlCriteria(route.query ?? "") !==
        urlCriteria(searchStateQueryString(activeSearch, urlProfile)) ||
      (atlasRouter.latestLocation.search.search_profile ?? null) !== urlProfile
    )
      return;
    const lastPage = response.page.total === 0n ? 1n : totalPages(response.page);
    if (lastPage < 1n || BigInt(pageNumber) <= lastPage) return;
    // Only the settled response for this exact request may reduce the URL page.
    void atlasRouter.navigate({
      ...searchDestination({ ...route, page: Number(lastPage) }),
      hash: true,
      replace: true,
      resetScroll: false,
      state: true,
    });
  }, [
    canRunResultSearch,
    retrieval.recovering,
    retrieval.generation,
    resultsQuery.data,
    resultsQuery.isSuccess,
    resultsQuery.isFetching,
    resultsQuery.isPlaceholderData,
    activeSearchExecutionToken,
    searchExecutionToken,
    activeSearch,
    pageNumber,
    searchProfile,
    urlProfile,
  ]);
  const resultsPaused = isArtifactGenerationChanged(resultsQuery.error);
  async function refreshResults() {
    if (liveRequest.current.retiring) return;
    // Retire the owner before awaiting cancellation, so a late completion cannot seed.
    liveRequest.current.retiring = true;
    setRetrieval((current) => ({ ...current, recovering: true }));
    await atlasRouter.navigate({
      to: ".",
      hash: true,
      search: (search) => ({ ...search, page: undefined }),
      replace: true,
      resetScroll: false,
      state: true,
    });
    await retireImmutableReferenceQueries(queryClient);
    setRetrieval((current) => ({
      epoch: current.epoch + 1,
      generation: null,
      recovering: false,
    }));
  }
  useEffect(() => {
    const pending = pendingUrlReading.current;
    if (pending) {
      const current = readingRequestIdentity(reading.request);
      if (current !== pending.incoming && current === pending.previous) return;
      pendingUrlReading.current = null;
    }
    const view = currentSearchLocation();
    if (
      !enabled ||
      !reading.detail ||
      search.filterWasReset ||
      !view ||
      view.selectedRecordKey !== reading.detail.record.record_key
    )
      return;
    const selected = readingNavigationForUrl(reading.detail.selected, reading.request);
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
      query: view.query,
      selection,
      landing: reading.landing ?? reading.request?.landing,
      view: view.view,
      tab: view.tab,
      page: urlSearchPage(atlasRouter.latestLocation.search),
    });
    if (atlasRouter.buildLocation(destination).href !== atlasRouter.latestLocation.href)
      void atlasRouter.navigate({
        ...destination,
        hash: true,
        replace: true,
        resetScroll: false,
        state: true,
      });
  }, [
    enabled,
    reading.detail,
    reading.request,
    reading.landing,
    search,
    urlProfile,
    pageNumber,
  ]);

  useEffect(() => {
    if (
      !enabled ||
      search.filterWasReset ||
      failedTransport ||
      resetPending ||
      urlProfile === null ||
      !currentSearchLocation()
    )
      return;
    if (atlasRouter.latestLocation.search.search_profile === urlProfile) return;
    void atlasRouter.navigate({
      to: ".",
      hash: true,
      search: (search) => ({ ...search, search_profile: urlProfile }),
      replace: true,
      resetScroll: false,
      state: true,
    });
  }, [enabled, urlProfile, search.filterWasReset, failedTransport, resetPending]);

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
    enabled:
      enabled &&
      initialRoute !== null &&
      !failedTransport &&
      !resetPending &&
      !search.filterWasReset &&
      !retrieval.recovering,
    queryKeyPrefix: ["filter-discovery", retrieval.epoch],
    catalogIdentity: [retrieval.epoch],
  });
  function setSearch(next: SearchFormState) {
    const version = ++seedPreparation.current;
    setSourcePreparationError(null);
    setPreparingSeed(false);
    if (next.similarSeed !== search.similarSeed && next.similarSeed) {
      const nextSeed = next.similarSeed;
      const trait = nextSeed.startsWith("trait:");
      const preferences = trait
        ? searchPreferences.traitSimilarity
        : searchPreferences.recordSimilarity;
      next = {
        ...next,
        sourceSections: preferences.method === "baseline" ? undefined : "all",
      };
      if (preferences.method !== "baseline" && trait)
        next.sourceSections = preferredSourceSections(searchPreferences, "trait");
      if (
        preferences.method !== "baseline" &&
        !trait &&
        hasCoreSourcePreference(searchPreferences) &&
        searchProfile !== null
      ) {
        const seed = { kind: "record" as const, record_key: nextSeed };
        const nextSearch = next;
        const epoch = retrieval.epoch;
        const generation = retrieval.generation;
        const token = liveRequest.current.token;
        setPreparingSeed(true);
        void queryClient
          .fetchQuery({
            queryKey: ["similarity-parts", seed, searchProfile, generation, 0],
            staleTime: Infinity,
            queryFn: ({ signal }) =>
              getSimilarityParts(
                {
                  seed,
                  search_profile: searchProfile,
                  expected_artifact_generation: generation,
                },
                signal,
              ),
          })
          .then((inventory) => {
            if (
              version !== seedPreparation.current ||
              epoch !== liveRequest.current.epoch ||
              token !== liveRequest.current.token ||
              liveRequest.current.retiring
            )
              return;
            assertSimilarityInventoryIdentity(
              inventory,
              seed,
              searchProfile,
              liveRequest.current.generation,
            );
            if (liveRequest.current.generation === null)
              setRetrieval((current) =>
                current.generation === null && current.epoch === epoch
                  ? { ...current, generation: inventory.artifact_generation }
                  : current,
              );
            commitSearch({
              ...nextSearch,
              sourceSections: preferredSourceSections(
                searchPreferences,
                inventory.core.family,
              ),
            });
          })
          .catch((error: unknown) => {
            if (version === seedPreparation.current)
              setSourcePreparationError(
                error instanceof Error
                  ? error.message
                  : "Unable to load source sections. Choose this record again.",
              );
          })
          .finally(() => {
            if (version === seedPreparation.current) setPreparingSeed(false);
          });
        return;
      }
    }
    commitSearch(next);
  }
  function commitSearch(next: SearchFormState) {
    if (next.similarSeed !== search.similarSeed)
      next = {
        ...next,
        similarity: next.similarSeed
          ? similarityFromPreferences(
              next.similarSeed.startsWith("trait:")
                ? searchPreferences.traitSimilarity
                : searchPreferences.recordSimilarity,
              next.similarSeed.startsWith("trait:"),
            )
          : undefined,
        similarityGeneration: undefined,
        sourceSections: next.similarSeed ? (next.sourceSections ?? "all") : undefined,
      };
    // A submitted canonical address is navigation, independent of search eligibility.
    const recordKey = next.query.trim();
    if (next.query !== search.query && /^[^:\s]+:[^:\s]+$/u.test(recordKey)) {
      selectRecord(recordKey);
      return;
    }
    // Keyword entry intentionally replaces similarity, while ordinary filters retain it.
    if (next.query.trim()) next = { ...next, similarSeed: null, mode: "text_search" };
    else next = { ...next, mode: next.similarSeed ? "similar_records" : "browse" };
    next = normalizeSortTransition(search, { ...next, query: next.query.trim() });
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
        ...currentSearchLocation(),
        selectedRecordKey,
        query: searchStateQueryString(next, urlProfile),
        page: 1,
      }),
      replace: search.similarSeed === next.similarSeed,
      resetScroll: false,
      state: true,
    });
  }
  function selectRecord(recordKey: string | null) {
    dispatch({ type: "record.selected", recordKey });
    const searchQuery = searchStateQueryString(search, urlProfile);
    void atlasRouter.navigate({
      ...searchDestination({
        selectedRecordKey: recordKey,
        query: searchQuery,
        page: pageNumber,
      }),
      replace: true,
      resetScroll: false,
      state: true,
    });
    reading.select(recordKey ? { navigation: rootNavigation(recordKey) } : null);
  }
  const errorMessage = sourcePreparationError ?? messageFromError(resultsQuery.error);
  const searchDebouncing = activeSearchExecutionToken !== searchExecutionToken;
  const resultsRefreshing =
    canRunResultSearch &&
    (searchDebouncing ||
      resultsQuery.isPlaceholderData ||
      (resultsQuery.isFetching && !resultsQuery.isLoading));
  return {
    search,
    setSearch,
    applySettings: async (next, profile) => {
      seedPreparation.current += 1;
      setPreparingSeed(false);
      setSourcePreparationError(null);
      const committed = normalizeSortTransition(search, next);
      const nextGeneration = committed.similarityGeneration;
      const collectionChanged =
        nextGeneration !== undefined && nextGeneration !== retrieval.generation;
      if (collectionChanged) {
        // Refreshed parts belong to a new immutable collection. Retire A before
        // committing its replacement so late A reads cannot seed the B owner.
        liveRequest.current.retiring = true;
        setRetrieval((current) => ({ ...current, recovering: true }));
      }
      dispatch({ type: "search.settingsApplied", search: committed });
      void atlasRouter.navigate({
        ...searchDestination({
          ...currentSearchLocation(),
          selectedRecordKey,
          query: searchStateQueryString(committed, profile),
          selection: reading.detail
            ? readingNavigationForUrl(reading.detail.selected, reading.request)
            : (reading.request?.match ?? reading.request?.navigation),
          landing: reading.landing ?? reading.request?.landing,
          page: 1,
        }),
        replace: true,
        resetScroll: false,
        state: true,
      });
      if (collectionChanged) {
        await retireImmutableReferenceQueries(queryClient);
        setRetrieval((current) => ({
          epoch: current.epoch + 1,
          generation: nextGeneration,
          recovering: false,
        }));
      }
    },
    recoverFilterUrl: (restored) => {
      dispatch({ type: "url.restored", search: restored, selectedRecordKey: null });
      reading.select(null);
    },

    reading,
    positions,
    selectedRecordKey,
    selectRecord,
    pageNumber,
    setPageNumber: (nextPage) => {
      if (!resultsPaused && !retrieval.recovering && nextPage !== pageNumber) {
        void atlasRouter.navigate({
          ...searchDestination({ ...currentSearchLocation(), page: nextPage }),
          replace: false,
          resetScroll: false,
          state: true,
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
    resultsRefreshing: resultsRefreshing || preparingSeed,
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
