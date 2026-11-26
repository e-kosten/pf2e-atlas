import { useAtlasRouter } from "../../app/router";
import {
  userErrorMessage,
  isSharedFailure,
  lexicalInputExplanation,
} from "../../shared/feedback/errorMessages";
import {
  failedFilterTransport,
  searchPage as urlSearchPage,
} from "../../app/routeSearch";
import { totalPages } from "./pageMetrics";
import { searchDestination } from "../../app/destinations";
import {
  useEffect,
  useEffectEvent,
  useId,
  useLayoutEffect,
  useMemo,
  useReducer,
  useState,
  useRef,
} from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { searchPage } from "../../api/atlasApi";
import type { WorkspaceSearchDefault } from "../../shared/search/SearchProfileContext";
import { searchExecution } from "../../shared/search/searchExecution";
import {
  useReferenceGeneration,
  useReferenceGenerationAdoption,
} from "../../shared/search/useReferenceGeneration";
import { useReferenceQueryRetirement } from "../../shared/queries/useReferenceQueryRetirement";
import {
  assertKeywordSearch,
  assertSearchIdentity,
  assertSimilarityIdentity,
  assertSimilarityResultSource,
  matchesSimilaritySource,
} from "../../shared/search/searchIdentity";
import {
  rememberRankedSimilaritySource,
  useSimilaritySource,
} from "../../shared/search/similarityQueries";
import { sameSimilaritySeed, searchSeed } from "../../shared/search/similaritySource";
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
import {
  commonCondition,
  type FilterPanelState,
  type SearchSubmission,
  type SearchUpdateIntent,
} from "../../shared/filters/filterControls";
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
import { useRouterState } from "@tanstack/react-router";
import { currentSearchLocation } from "../../app/searchLocation";
import {
  DEFAULT_SEARCH_PREFERENCES,
  effectiveMatchDisplay,
  type SearchPreferences,
} from "../../shared/search/searchPreferences";
import {
  isArtifactGenerationChanged,
  referenceQueryMeta,
} from "../../shared/queries/referenceQueries";
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
type ResultData = { page: SearchResultsPage; filter: QueryPredicate | null };

export type SearchWorkspaceState = {
  search: SearchFormState;
  reading: ReferenceReading;
  positions: BrowsePositions;
  setSearch: FilterPanelState["setSearch"];
  acceptedSearchSubmission?: SearchSubmission;
  applySettings?: (next: SearchFormState, profile: string | null) => void;
  recoverFilterUrl: (restored: SearchFormState) => void;
  selectedRecordKey: string | null;
  selectRecord: (recordKey: string | null, request?: ReadingRequest) => void;
  pageNumber: number;
  setPageNumber: (page: number) => void;
  resultPage: SearchResultsPage | undefined;
  resultFilter: QueryPredicate | null;
  filterEditor: FilterEditorView | undefined;
  refreshFilterInformation?: () => Promise<void>;
  filterApplicabilityCurrent: boolean;
  filterCountsEnabled?: boolean;
  filterCountQueryKey?: readonly unknown[];
  evidencePending?: boolean;
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
  searchInputError?: string;
  resultsFailed?: boolean;
  resultsError?: unknown;
  referenceGeneration?: string | null;
  similaritySource?: ReturnType<typeof useSimilaritySource>;
  refresh: () => void;
  rerunSemantic?: () => void;
};
type UseAtlasWorkspaceOptions = {
  enabled?: boolean;
  searchProfile: string | null;
  pendingUrlProfile?: string | null;
  searchProfileStored?: boolean;
  searchProfileUsable?: boolean;
  keywordSearch?: boolean;
  automaticKeyword?: boolean;
  semanticDefault?: string;
  promoteKeyword?: (id: string) => void;
  resolveWorkspaceDefault?: () => Promise<WorkspaceSearchDefault | undefined>;
  adoptWorkspaceDefault?: (id: string | null) => void;
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
  keywordSearch = false,
  automaticKeyword = false,
  semanticDefault,
  promoteKeyword,
  resolveWorkspaceDefault,
  adoptWorkspaceDefault,
  includeSearchProfileInUrl = true,
  searchPreferences = DEFAULT_SEARCH_PREFERENCES,
}: UseAtlasWorkspaceOptions): SearchWorkspaceState {
  const atlasRouter = useAtlasRouter();
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
    (preferences) => initialWorkspaceInteractionState(preferences, atlasRouter),
  );
  const { search, selectedRecordKey, activeSearch } = interaction;
  const searchVersion = useRef(0);
  useEffect(
    () => () => {
      searchVersion.current += 1;
    },
    [enabled],
  );
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
    if (enabled && shouldReset && currentSearchLocation(atlasRouter))
      void atlasRouter.navigate({
        to: ".",
        hash: true,
        search: (search) => ({ ...search, page: undefined }),
        replace: true,
        resetScroll: false,
        state: true,
      });
  }, [atlasRouter, enabled, searchProfile]);
  const initialRoute = currentSearchLocation(atlasRouter);
  const failedTransport =
    initialRoute !== null &&
    failedFilterTransport("search", atlasRouter.latestLocation.search);
  const previousUrlCriteria = useRef(urlCriteria(atlasRouter.latestLocation.searchStr));
  const previousReset = useRef(atlasRouter.latestLocation.state.atlasSearchReset);
  const resetPending =
    previousReset.current !== atlasRouter.latestLocation.state.atlasSearchReset;
  const previousUrlReading = useRef(urlReadingIdentity(initialRoute));
  const pendingUrlReading = useRef<{ incoming: string; previous: string } | null>(null);
  const searchToken = useMemo(() => encodeSearchState(search), [search]);
  const availability = {
    id: searchProfile,
    storedVectors: searchProfileStored,
    usable: searchProfileUsable,
    keywordSearch,
  };
  const browseReady = searchExecution(availability, "list_records").ready;
  const similarReady = searchExecution(availability, "similar_records").ready;
  const {
    profile: requestProfile,
    mode: textMode,
    ready: textReady,
  } = searchExecution(availability, "text_search");
  const searchReady = search.similarSeed
    ? similarReady
    : search.query.trim()
      ? textReady
      : browseReady;
  const activeSearchReady = activeSearch.similarSeed
    ? similarReady
    : activeSearch.query.trim()
      ? textReady
      : browseReady;
  const searchExecutionToken = encodeSearchExecutionState(
    search,
    requestProfile,
    textMode,
  );
  const activeSearchExecutionToken = encodeSearchExecutionState(
    activeSearch,
    requestProfile,
    textMode,
  );
  const retrieval = useReferenceGeneration(
    activeSearchExecutionToken,
    enabled && initialRoute !== null && !failedTransport && !resetPending,
  );
  const visibleSource = searchSeed(search);
  const activeSource = searchSeed(activeSearch);
  const sameSource = sameSimilaritySeed(visibleSource, activeSource);
  const similaritySource = useSimilaritySource(
    retrieval.active && !retrieval.recovering && !search.filterError
      ? visibleSource
      : null,
    retrieval.generation,
  );
  // A cold tab resolves its title before admitting its own expensive ranking job.
  // A transient title failure still permits ranking to supply a verified descriptor.
  const awaitingSource =
    activeSource !== null && (!sameSource || similaritySource.isPending);
  const canRunResultSearch =
    enabled &&
    initialRoute !== null &&
    !failedTransport &&
    !resetPending &&
    activeSearchReady &&
    hasExecutableSearch(activeSearch) &&
    sameSource &&
    !awaitingSource;
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
  const restoreLocation = useEffectEvent(() => {
    if (!enabled) return;
    const route = currentSearchLocation(atlasRouter);
    if (!route) return;
    const restored = initialWorkspaceInteractionState(searchPreferences, atlasRouter);
    const incomingCriteria = urlCriteria(atlasRouter.latestLocation.searchStr);
    const resetChanged =
      previousReset.current !== atlasRouter.latestLocation.state.atlasSearchReset;
    previousReset.current = atlasRouter.latestLocation.state.atlasSearchReset;
    const criteriaChanged =
      previousUrlCriteria.current !== incomingCriteria || resetChanged;
    if (criteriaChanged) {
      searchVersion.current += 1;
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
      inspectReading(request, retrieval.generation);
    } else if (readingChanged) {
      pendingUrlReading.current = null;
    }
  });
  useLayoutEffect(() => {
    restoreLocation();
  }, [enabled, location]);

  useEffect(() => {
    if (search === activeSearch) return;
    const timeout = window.setTimeout(() => {
      dispatch({ type: "search.executionCommitted", search });
    }, SEARCH_REQUEST_DEBOUNCE_MS);
    return () => window.clearTimeout(timeout);
  }, [search, searchToken, activeSearch]);
  const baseResultsKey = (generation: string | null) => [
    "results",
    activeSearchExecutionToken,
    retrieval.epoch,
    generation,
    pageNumber,
    requestProfile ?? null,
  ];
  const client = useQueryClient();
  const workspaceId = useId();
  const visible =
    effectiveMatchDisplay(
      searchPreferences.matchDisplay,
      Boolean(activeSearch.similarSeed),
    ) !== "hidden";
  const context = JSON.stringify(baseResultsKey(null));
  const [projection, setProjection] = useState({
    context,
    visible,
    detail: visible ? ("evidence" as const) : ("records" as const),
    revision: 0,
    retained: undefined as ResultData | undefined,
  });
  const resultsKeyForGeneration = (generation: string | null) => [
    ...baseResultsKey(generation),
    projection.detail,
    projection.detail === "evidence" && projection.revision
      ? [workspaceId, projection.revision]
      : null,
  ];
  const resultsKey = resultsKeyForGeneration(retrieval.generation);
  if (projection.context !== context || projection.visible !== visible) {
    const retained =
      projection.context === context
        ? (client.getQueryData<ResultData>(resultsKey) ?? projection.retained)
        : undefined;
    setProjection({
      context,
      visible,
      // Hiding during the first fetch lets that request supply the page.
      detail:
        visible || (!retained && projection.context === context)
          ? "evidence"
          : "records",
      revision: projection.revision + Number(visible && !projection.visible),
      retained,
    });
  }
  const heldPage =
    projection.retained?.page.artifact_generation === retrieval.generation &&
    !retrieval.recovering
      ? projection.retained
      : undefined;
  const [generationFailure, setGenerationFailure] = useState<{
    epoch: number;
    error: Error;
  } | null>(null);
  const pausedError =
    generationFailure?.epoch === retrieval.epoch ? generationFailure.error : null;
  const resultsQuery = useQuery<ResultData>({
    queryKey: resultsKey,
    meta: referenceQueryMeta,
    enabled:
      canRunResultSearch &&
      !retrieval.recovering &&
      !pausedError &&
      (visible || !heldPage),
    staleTime: 20000,
    placeholderData: (previous, previousQuery) =>
      !heldPage &&
      previous &&
      matchesSimilaritySource(previous.page, visibleSource) &&
      previousQuery?.queryKey[6] === projection.detail &&
      previousQuery.queryKey[5] === (requestProfile ?? null) &&
      (previous.page.mode.kind !== "text_search" ||
        previous.page.mode.mode === textMode) &&
      previous.page.ranking === (activeSearch.ranking ?? "score") &&
      previousQuery?.queryKey[2] === retrieval.epoch &&
      previousQuery.queryKey[3] === retrieval.generation
        ? previous
        : undefined,
    queryFn: async ({ signal }) => {
      const ticket = retrieval.capture(signal);
      const version = searchVersion.current;
      const page = await searchPage(
        {
          ...buildSearchRequest(
            activeSearch,
            pageNumber,
            retrieval.generation,
            requestProfile,
            textMode,
          ),
          ...(projection.detail === "evidence" && { result_detail: "evidence" }),
        },
        signal,
      );
      assertSearchIdentity(
        page,
        requestProfile,
        activeSearch.similarityGeneration ?? retrieval.generation,
        effectiveResultSort(activeSearch),
        activeSearch.similarSeed || activeSearch.query.trim()
          ? (activeSearch.searchContent ?? "all")
          : undefined,
      );
      const source = searchSeed(activeSearch);
      if (source) {
        assertSimilarityResultSource(page, source);
        assertSimilarityIdentity(
          page,
          activeSearch.similarity,
          activeSearch.sourceSections,
          activeSearch.searchContent ?? "all",
        );
      }
      if (textMode === "lexical" && activeSearch.query.trim())
        assertKeywordSearch(page);
      if (
        activeSearch.query.trim() &&
        page.ranking !== (activeSearch.ranking ?? "score")
      )
        throw new Error("Search response used a different ranking");
      const owner = retrieval.current(ticket);
      if (
        source &&
        page.mode.kind === "similar_records" &&
        owner &&
        (!owner.generation || owner.generation === page.artifact_generation) &&
        version === searchVersion.current
      ) {
        rememberRankedSimilaritySource(
          client,
          {
            artifact_generation: page.artifact_generation,
            source: page.mode.source ?? { seed: source, title: page.mode.seed.title },
          },
          retrieval.generation,
        );
      }
      retrieval.adopt(ticket, page.artifact_generation, {
        queryKey: resultsKeyForGeneration(page.artifact_generation),
        data: { page, filter: activeSearch.filter },
      });
      return { page, filter: activeSearch.filter };
    },
  });
  useReferenceQueryRetirement(resultsKey);
  useReferenceGenerationAdoption(
    retrieval,
    resultsQuery,
    resultsQuery.data?.page.artifact_generation,
    resultsKeyForGeneration(resultsQuery.data?.page.artifact_generation ?? null),
  );
  useEffect(() => {
    const response = resultsQuery.data?.page;
    const route = currentSearchLocation(atlasRouter);

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
      (requestProfile !== undefined && response.search_profile !== requestProfile) ||
      urlCriteria(route.query ?? "") !==
        urlCriteria(searchStateQueryString(activeSearch, urlProfile)) ||
      (atlasRouter.latestLocation.search.search_profile ?? null) !== urlProfile
    )
      return;
    const lastPage = response.page.total === 0 ? 1 : totalPages(response.page);
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
    requestProfile,
    urlProfile,
    atlasRouter,
  ]);
  // A copied Custom search can pin an old inventory even in a current workspace.
  // Keep that failure with the URL so it cannot pause an unrelated next search.
  const changed = [
    activeSearch.similarityGeneration ? null : resultsQuery.error,
    reading.error,
  ].find(isArtifactGenerationChanged);
  if (!pausedError && changed)
    setGenerationFailure({ epoch: retrieval.epoch, error: changed });
  const resultsError = sameSource ? (pausedError ?? resultsQuery.error) : null;
  const resultsPaused = isArtifactGenerationChanged(resultsError);
  function refreshResults() {
    return retrieval.refresh(async () => {
      await atlasRouter.navigate({
        to: ".",
        hash: true,
        search: (search) => ({ ...search, page: undefined }),
        replace: true,
        resetScroll: false,
        state: true,
      });
    });
  }
  useEffect(() => {
    const pending = pendingUrlReading.current;
    if (pending) {
      const current = readingRequestIdentity(reading.request);
      if (current !== pending.incoming && current === pending.previous) return;
      pendingUrlReading.current = null;
    }
    const view = currentSearchLocation(atlasRouter);
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
    atlasRouter,
  ]);

  useEffect(() => {
    if (
      !enabled ||
      search.filterWasReset ||
      failedTransport ||
      resetPending ||
      (urlProfile === null && !keywordSearch) ||
      !currentSearchLocation(atlasRouter)
    )
      return;
    if ((atlasRouter.latestLocation.search.search_profile ?? null) === urlProfile)
      return;
    void atlasRouter.navigate({
      to: ".",
      hash: true,
      search: (search) => ({ ...search, search_profile: urlProfile ?? undefined }),
      replace: true,
      resetScroll: false,
      state: true,
    });
  }, [
    enabled,
    urlProfile,
    keywordSearch,
    search.filterWasReset,
    failedTransport,
    resetPending,
    atlasRouter,
  ]);

  const filterDiscoveryContext = buildFilterDiscoveryContext(
    search,
    requestProfile,
    retrieval.generation ?? undefined,
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
  function setSearch(
    next: SearchFormState,
    intent: SearchUpdateIntent = "edit",
    submission?: SearchSubmission,
    rerunSemantic = false,
  ) {
    searchVersion.current += 1;
    if ((!next.similarSeed || next.query.trim()) && search.similarSeed)
      next = {
        ...next,
        similarity: undefined,
        similarLocation: undefined,
        similarityGeneration: undefined,
        sourceSections: undefined,
      };
    // A submitted canonical address is navigation, independent of search eligibility.
    const recordKey = next.query.trim();
    if (next.query !== search.query && /^[^:\s]+:[^:\s]+$/u.test(recordKey)) {
      selectRecord(recordKey, undefined, submission);
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
    const model =
      keywordSearch &&
      next.query &&
      !next.filterError &&
      (rerunSemantic || (intent === "submit" && automaticKeyword))
        ? semanticDefault
        : undefined;
    const commit = async (
      selected: string | null,
      resolved?: { id: string | null },
    ) => {
      if (resolved) adoptWorkspaceDefault?.(resolved.id);
      else if (model) promoteKeyword?.(model);
      if (intent === "submit")
        dispatch({ type: "search.submitted", search: next, submission });
      else dispatch({ type: "search.changed", search: next });
      await atlasRouter.navigate({
        ...searchDestination({
          ...currentSearchLocation(atlasRouter),
          selectedRecordKey,
          query: searchStateQueryString(next, selected),
          page: 1,
        }),
        replace: sameSimilaritySeed(searchSeed(search), searchSeed(next)),
        resetScroll: false,
        state: true,
      });
    };
    if (
      intent === "submit" &&
      next.query &&
      !next.filterError &&
      resolveWorkspaceDefault
    ) {
      const submission = searchVersion.current;
      const href = atlasRouter.latestLocation.href;
      void resolveWorkspaceDefault().then((current) => {
        if (
          submission !== searchVersion.current ||
          href !== atlasRouter.latestLocation.href
        )
          return;
        const apply = () => {
          if (
            submission !== searchVersion.current ||
            href !== atlasRouter.latestLocation.href
          )
            return;
          if (!current) return commit(urlProfile);
          return commit(
            current.id === searchProfile ? urlProfile : current.id,
            current,
          );
        };
        void retrieval.applyGeneration(current?.generation, apply);
      });
    } else if (model && promoteKeyword) {
      void retrieval.refresh(() => commit(model));
    } else {
      void commit(urlProfile);
    }
  }
  function selectRecord(
    recordKey: string | null,
    request?: ReadingRequest,
    submission?: SearchSubmission,
  ) {
    dispatch({ type: "record.selected", recordKey, submission });
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
    reading.select(
      recordKey ? (request ?? { navigation: rootNavigation(recordKey) }) : null,
      result?.page.artifact_generation ?? retrieval.generation,
    );
  }
  const canDisplayResult =
    canRunResultSearch ||
    (enabled &&
      initialRoute !== null &&
      !failedTransport &&
      !resetPending &&
      !activeSearchReady &&
      hasExecutableSearch(activeSearch));
  const candidateResult = resultsQuery.data ?? heldPage;
  const result =
    candidateResult && matchesSimilaritySource(candidateResult.page, visibleSource)
      ? candidateResult
      : undefined;
  const evidencePending =
    visible &&
    (resultsQuery.data?.page.result_detail !== "evidence" ||
      resultsQuery.isPlaceholderData);
  const searchInputError =
    activeSearchExecutionToken === searchExecutionToken
      ? lexicalInputExplanation(resultsError)
      : undefined;
  const errorMessage = searchInputError ? null : messageFromError(resultsError);
  const searchDebouncing = activeSearchExecutionToken !== searchExecutionToken;
  const resultsRefreshing =
    canRunResultSearch &&
    (searchDebouncing ||
      resultsQuery.isPlaceholderData ||
      (resultsQuery.isFetching && !resultsQuery.isLoading));
  return {
    search,
    setSearch,
    acceptedSearchSubmission: interaction.acceptedSearchSubmission,
    rerunSemantic:
      keywordSearch &&
      semanticDefault &&
      promoteKeyword &&
      search.query.trim() &&
      !search.filterError &&
      !retrieval.recovering &&
      result?.page.mode.kind === "text_search" &&
      result.page.mode.mode === "lexical"
        ? () => setSearch(search, "submit", undefined, true)
        : undefined,
    applySettings: async (next, profile) => {
      searchVersion.current += 1;
      const committed = normalizeSortTransition(search, next);
      const nextGeneration = committed.similarityGeneration;
      const recoveringCustom = resultsPaused && Boolean(search.similarityGeneration);
      await retrieval.applyGeneration(
        nextGeneration,
        () => {
          dispatch({ type: "search.settingsApplied", search: committed });
          void atlasRouter.navigate({
            ...searchDestination({
              ...currentSearchLocation(atlasRouter),
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
        },
        recoveringCustom,
      );
    },
    recoverFilterUrl: (restored) => {
      dispatch({ type: "url.restored", search: restored, selectedRecordKey: null });
      reading.select(null);
    },

    reading: isArtifactGenerationChanged(reading.error)
      ? { ...reading, retry: () => void refreshResults() }
      : reading,
    positions,
    selectedRecordKey,
    selectRecord,
    pageNumber,
    setPageNumber: (nextPage) => {
      if (!resultsPaused && !retrieval.recovering && nextPage !== pageNumber) {
        void atlasRouter.navigate({
          ...searchDestination({
            ...currentSearchLocation(atlasRouter),
            page: nextPage,
          }),
          replace: false,
          resetScroll: false,
          state: true,
        });
      }
    },
    resultPage: canDisplayResult ? result?.page : undefined,
    resultFilter: canDisplayResult ? (result?.filter ?? null) : null,
    filterEditor: filterDiscovery.filterEditor,
    refreshFilterInformation: refreshResults,
    filterApplicabilityCurrent: filterDiscovery.filterApplicabilityCurrent,
    filterCountsEnabled: filterDiscovery.filterCountsEnabled && browseReady,
    filterCountQueryKey: filterDiscovery.filterCountQueryKey,
    filterDiscoveryContext: filterDiscovery.filterDiscoveryContext,
    vocabularyScope: filterDiscovery.vocabularyScope,
    filterDiscoveryError: filterDiscovery.errorMessage,
    evidencePending,
    resultsLoading:
      enabled &&
      searchReady &&
      hasExecutableSearch(search) &&
      ((resultsQuery.isLoading && !heldPage) || awaitingSource || searchDebouncing),
    resultsRefreshing,
    resultsPaused,
    resultsRecovering: retrieval.recovering,
    refreshResults,
    filterDiscoveryLoading: filterDiscovery.loading,
    errorMessage,
    searchInputError,
    similaritySource,
    resultsFailed: !!resultsError,
    resultsError,
    referenceGeneration: retrieval.generation,
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
      if (!(visible && heldPage && resultsQuery.error)) reading.retry();
    },
  };
}
function messageFromError(error: unknown): string | null {
  if (!error) {
    return null;
  }
  if (isSharedFailure(error)) return null;
  return userErrorMessage(error, "Try searching again.");
}
export function resetSearchState(): SearchFormState {
  return DEFAULT_SEARCH_STATE;
}
