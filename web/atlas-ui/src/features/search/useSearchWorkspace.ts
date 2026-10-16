import {
  useEffect,
  useLayoutEffect,
  useMemo,
  useReducer,
  useRef,
  useState,
} from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { searchPage } from "../../api/atlasApi";
import { assertSearchIdentity } from "../../shared/search/searchIdentity";
import type { FilterEditorView, SearchResultsPage } from "../../generated/atlas";
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

export type SearchWorkspaceState = {
  search: SearchFormState;
  reading: ReferenceReading;
  positions: BrowsePositions;
  setSearch: (next: SearchFormState) => void;
  selectedRecordKey: string | null;
  selectRecord: (recordKey: string | null) => void;
  pageNumber: number;
  setPageNumber: (page: number) => void;
  resultPage: SearchResultsPage | undefined;
  filterEditor: FilterEditorView | undefined;
  filterCountsByField?: Record<
    string,
    import("../../generated/atlas").FilterCountsView | undefined
  >;
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
  // This leaves the reader's artifact epoch and all draft owners untouched.
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
  const initialRoute = currentAtlasRoute();
  const reading = useReferenceReading(
    initialRoute.kind === "search" && initialRoute.selectedRecordKey
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
  useEffect(() => {
    const onUrlStateChange = () => {
      const route = currentAtlasRoute();
      if (route.kind !== "search") return;
      dispatch({
        type: "url.restored",
        search: initialWorkspaceInteractionState().search,
        selectedRecordKey: route.selectedRecordKey,
        pageNumber: history.state?.atlasSearchPage ?? 1,
      });
      inspectReading(
        route.selectedRecordKey
          ? {
              navigation: route.selection ?? rootNavigation(route.selectedRecordKey),
              landing: route.landing,
            }
          : null,
      );
    };
    window.addEventListener("popstate", onUrlStateChange);
    window.addEventListener(ATLAS_ROUTE_CHANGE_EVENT, onUrlStateChange);
    return () => {
      window.removeEventListener("popstate", onUrlStateChange);
      window.removeEventListener(ATLAS_ROUTE_CHANGE_EVENT, onUrlStateChange);
    };
  }, [inspectReading]);

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
  const resultsQuery = useQuery<SearchResultsPage>({
    queryKey: resultsKey,
    enabled: canRunResultSearch && !retrieval.recovering,
    staleTime: 20_000,
    placeholderData: (previous, previousQuery) =>
      previous?.search_profile === searchProfile &&
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
      assertSearchIdentity(page, searchProfile, retrieval.generation);
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
    if (!enabled || !reading.detail || currentAtlasRoute().kind !== "search") return;
    const url = new URL(
      searchPath(reading.detail.record.record_key) +
        searchStateQueryString(search, urlProfile),
      window.location.origin,
    );
    const selected = reading.detail.selected;
    if (
      selected.owners.length ||
      selected.field ||
      selected.passage ||
      selected.source_fingerprint
    )
      url.searchParams.set("selection", JSON.stringify(selected));
    if (reading.landing)
      url.searchParams.set("landing", JSON.stringify(reading.landing));
    history.replaceState(
      { ...history.state, atlasSearchPage: pageNumber },
      "",
      url.pathname + url.search,
    );
  }, [enabled, reading.detail, reading.landing, search, urlProfile, pageNumber]);

  useEffect(() => {
    if (!enabled || urlProfile === null || currentAtlasRoute().kind !== "search")
      return;
    const url = new URL(window.location.href);
    url.searchParams.set("search_profile", urlProfile);
    history.replaceState(
      { ...history.state, atlasSearchPage: pageNumber },
      "",
      url.pathname + url.search,
    );
  }, [enabled, urlProfile, pageNumber]);

  const filterDiscoveryContext = useMemo(
    () =>
      buildFilterDiscoveryContext(
        activeSearch,
        searchProfile ?? undefined,
        retrieval.generation ?? undefined,
      ),
    [activeSearch, searchProfile, retrieval.generation],
  );

  const filterDiscovery = useFilterDiscovery({
    context: filterDiscoveryContext,
    enabled: enabled && !retrieval.recovering,
    queryKeyPrefix: ["filter-discovery", retrieval.epoch, activeSearchExecutionToken],
    catalogIdentity: [retrieval.epoch],
  });

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
    const url = `${searchPath(selectedRecordKey)}${searchStateQueryString(next, urlProfile)}`;
    if (search.similarSeed !== next.similarSeed) history.pushState(null, "", url);
    else history.replaceState({ ...history.state, atlasSearchPage: 1 }, "", url);
  }

  function selectRecord(recordKey: string | null) {
    dispatch({ type: "record.selected", recordKey });
    const searchQuery = searchStateQueryString(search, urlProfile);
    const url = `${searchPath(recordKey)}${searchQuery}`;
    history.replaceState({ ...history.state, atlasSearchPage: pageNumber }, "", url);
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
    reading,
    positions,
    selectedRecordKey,
    selectRecord,
    pageNumber,
    setPageNumber: (pageNumber) => {
      if (!resultsPaused && !retrieval.recovering) {
        dispatch({ type: "resultPage.changed", pageNumber });
        history.replaceState(
          { ...history.state, atlasSearchPage: pageNumber },
          "",
          window.location.href,
        );
      }
    },
    resultPage: canRunResultSearch ? resultsQuery.data : undefined,
    filterEditor: filterDiscovery.filterEditor,
    filterCountsByField: filterDiscovery.filterCountsByField,
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
