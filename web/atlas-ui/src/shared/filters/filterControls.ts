import type { FilterEditorView, FilterValueListView } from "../../generated/atlas";
import type { SearchFormState } from "./searchState";
export type FilterPanelState = {
  search: SearchFormState;
  setSearch: (next: SearchFormState) => void;
  filterEditor: FilterEditorView | undefined;
  filterValuesByField: Record<string, FilterValueListView | undefined>;
  filterCountsByField?: Record<
    string,
    import("../../generated/atlas").FilterCountsView | undefined
  >;
  filterDiscoveryLoading?: boolean;
  errorMessage?: string | null;
};
export function clearAllFilters(search: SearchFormState): SearchFormState {
  return {
    ...search,
    filter: null,
    filterError: null,
    visibleFilterIds: [],
    hiddenFilterIds: [],
  };
}
