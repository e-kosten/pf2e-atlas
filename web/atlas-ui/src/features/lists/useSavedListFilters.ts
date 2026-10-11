import { useEffect, useMemo, useState } from "react";
import {
  buildSavedListFilterDiscoveryContext,
  encodeSearchExecutionState,
  type SearchFormState,
} from "../../shared/filters/searchState";
import { useFilterDiscovery } from "../../shared/filters/useFilterDiscovery";

const LIST_SEARCH_REQUEST_DEBOUNCE_MS = 300;

export function useSavedListFilterDiscovery(listRef: string, filters: SearchFormState) {
  const filterToken = useMemo(() => encodeSearchExecutionState(filters), [filters]);
  const context = useMemo(
    () => buildSavedListFilterDiscoveryContext(listRef, filters),
    [listRef, filters],
  );
  return useFilterDiscovery({
    context,
    queryKeyPrefix: ["saved-list-filter-discovery", listRef, filterToken],
    vocabularyScope: { kind: "saved_list", list_ref: listRef },
  });
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
