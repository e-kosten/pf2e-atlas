import { useEffect, useMemo, useState } from "react";
import type { FilterEditorView, FilterValueListView } from "../../generated/atlas";
import {
  buildSavedListFilterDiscoveryContext,
  encodeSearchExecutionState,
  type SearchFormState,
} from "../../shared/filters/searchState";
import { useFilterDiscovery } from "../../shared/filters/useFilterDiscovery";

const LIST_SEARCH_REQUEST_DEBOUNCE_MS = 300;

export function useSavedListFilterDiscovery(
  listRef: string,
  filters: SearchFormState,
): {
  filterEditor: FilterEditorView | undefined;
  filterValuesByField: Record<string, FilterValueListView | undefined>;
  loading: boolean;
  errorMessage: string | null;
} {
  const filterToken = useMemo(() => encodeSearchExecutionState(filters), [filters]);
  const context = useMemo(
    () => buildSavedListFilterDiscoveryContext(listRef, filters),
    [listRef, filters],
  );
  return useFilterDiscovery({
    context,
    hiddenFieldIds: filters.hiddenFilterIds,
    queryKeyPrefix: ["saved-list-filter-discovery", listRef, filterToken],
    retainedValueQueryKeyPrefix: ["saved-list-filter-discovery", listRef],
    selectedFieldIds: filters.visibleFilterIds,
    visibleFieldIds: filters.visibleFilterIds,
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
