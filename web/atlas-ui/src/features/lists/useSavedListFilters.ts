import { useEffect, useMemo, useState } from "react";
import {
  buildSavedListFilterDiscoveryContext,
  encodeSearchExecutionState,
  type SearchFormState,
} from "../../shared/filters/searchState";
import { useFilterDiscovery } from "../../shared/filters/useFilterDiscovery";
import { useSearchProfile } from "../../shared/search/SearchProfileContext";

const LIST_SEARCH_REQUEST_DEBOUNCE_MS = 300;

export function useSavedListFilterDiscovery(listRef: string, filters: SearchFormState) {
  const profile = useSearchProfile();
  const filterToken = useMemo(() => encodeSearchExecutionState(filters), [filters]);
  const context = useMemo(
    () => ({
      ...buildSavedListFilterDiscoveryContext(listRef, filters),
      search_profile: profile.id ?? undefined,
    }),
    [listRef, filters, profile.id],
  );
  const discovery = useFilterDiscovery({
    context,
    queryKeyPrefix: ["saved-list-filter-discovery", listRef, filterToken],
    vocabularyScope: { kind: "saved_list", list_ref: listRef },
  });
  return {
    ...discovery,
    filterCountsEnabled: profile.id !== null && profile.storedVectors,
  };
}

export function listSearchQuery(filters: SearchFormState): string | undefined {
  const query = filters.query.trim();
  return filters.mode === "text_search" && query.length > 0 ? query : undefined;
}

export function useDebouncedSearchFilters(filters: SearchFormState): SearchFormState {
  const [activeFilters, setActiveFilters] = useState(filters);

  useEffect(() => {
    const timeout = window.setTimeout(() => {
      setActiveFilters(filters);
    }, LIST_SEARCH_REQUEST_DEBOUNCE_MS);
    return () => window.clearTimeout(timeout);
  }, [filters]);

  return activeFilters;
}
