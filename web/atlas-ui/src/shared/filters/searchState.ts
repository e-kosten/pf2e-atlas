import type {
  FilterDiscoveryContext,
  OpenResultWindowRequest,
  QueryPredicate,
  RetrievalModeView,
} from "../../generated/atlas";
export type SearchFormState = {
  query: string;
  mode: "browse" | "text_search";
  retrievalMode: RetrievalModeView;
  filter: QueryPredicate | null;
  filterError: string | null;
  visibleFilterIds: string[];
  hiddenFilterIds: string[];
  pageSize: number;
};
export const DEFAULT_SEARCH_STATE: SearchFormState = {
  query: "",
  mode: "browse",
  retrievalMode: "hybrid",
  filter: null,
  filterError: null,
  visibleFilterIds: [],
  hiddenFilterIds: [],
  pageSize: 25,
};
export function buildFilter(state: SearchFormState): QueryPredicate | null {
  return state.filter;
}
export function buildOpenRequest(
  state: SearchFormState,
  number: number,
): OpenResultWindowRequest {
  if (state.filterError) throw new Error(state.filterError);
  const page = { number, size: state.pageSize };
  return {
    page,
    mode:
      state.mode === "text_search" && state.query.trim()
        ? {
            kind: "text_search",
            query: state.query.trim(),
            filter: state.filter,
            mode: state.retrievalMode,
          }
        : { kind: "list_records", filter: state.filter },
  };
}
export function hasExecutableSearch(state: SearchFormState): boolean {
  return !state.filterError && (state.query.trim().length > 0 || state.filter !== null);
}
export function buildFilterDiscoveryContext(
  state: SearchFormState,
): FilterDiscoveryContext {
  return {
    kind: "filtered",
    filter: state.filter,
    text: state.mode === "text_search" ? state.query.trim() || null : null,
    mode: state.retrievalMode,
  };
}
export function buildSavedListFilterDiscoveryContext(
  list_ref: string,
  state: SearchFormState,
): FilterDiscoveryContext {
  return { ...buildFilterDiscoveryContext(state), kind: "saved_list", list_ref };
}
export function encodeSearchState(state: SearchFormState): string {
  return encodeURIComponent(JSON.stringify(state));
}
export function encodeSearchExecutionState(state: SearchFormState): string {
  return encodeURIComponent(
    JSON.stringify({
      query: state.query,
      mode: state.mode,
      retrievalMode: state.retrievalMode,
      filter: state.filter,
      filterError: state.filterError,
      pageSize: state.pageSize,
    }),
  );
}
export function searchStateQueryString(state: SearchFormState): string {
  const params = new URLSearchParams();
  if (state.query) params.set("q", state.query);
  if (state.mode !== "browse") params.set("mode", state.mode);
  if (state.retrievalMode !== "hybrid") params.set("retrieval", state.retrievalMode);
  if (state.filter) params.set("filter", JSON.stringify(state.filter));
  if (state.pageSize !== 25) params.set("limit", String(state.pageSize));
  return params.size ? "?" + params.toString() : "";
}
export function decodeSearchStateFromParams(params: URLSearchParams): SearchFormState {
  let filter: QueryPredicate | null = null;
  let filterError: string | null = null;
  try {
    const json = params.get("filter");
    if (json) {
      const value = JSON.parse(json);
      if (typeof value !== "object" || value === null || !("kind" in value))
        throw new Error("Invalid structured filter URL");
      filter = value as QueryPredicate;
    }
  } catch {
    filterError = "Invalid structured filter URL";
  }
  const pageSize = Number(params.get("limit") || 25);
  const retrieval = params.get("retrieval");
  return {
    ...DEFAULT_SEARCH_STATE,
    query: params.get("q") || "",
    mode: params.get("mode") === "text_search" ? "text_search" : "browse",
    retrievalMode:
      retrieval === "semantic" || retrieval === "lexical" ? retrieval : "hybrid",
    filter,
    filterError,
    pageSize:
      Number.isInteger(pageSize) && pageSize >= 1 && pageSize <= 100 ? pageSize : 25,
  };
}
export function decodeSearchState(value: string | null): SearchFormState {
  if (!value) return DEFAULT_SEARCH_STATE;
  try {
    const parsed = JSON.parse(decodeURIComponent(value));
    return decodeSearchStateFromParams(
      new URLSearchParams({
        q: typeof parsed.query === "string" ? parsed.query : "",
        mode: parsed.mode === "text_search" ? "text_search" : "browse",
        retrieval: parsed.retrievalMode || "hybrid",
        filter: parsed.filter ? JSON.stringify(parsed.filter) : "",
        limit: String(parsed.pageSize || 25),
      }),
    );
  } catch {
    return { ...DEFAULT_SEARCH_STATE, filterError: "Invalid search state URL" };
  }
}
