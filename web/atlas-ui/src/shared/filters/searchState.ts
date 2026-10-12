import type {
  FilterDiscoveryContext,
  SearchResultsRequest,
  QueryPredicate,
} from "../../generated/atlas";
export const SIMILAR_QUERY_CONFLICT = "Choose either text or a similar-record seed";
export type SearchFormState = {
  query: string;
  mode: "browse" | "text_search" | "similar_records";
  similarSeed: string | null;
  filter: QueryPredicate | null;
  filterError: string | null;
  pageSize: number;
};
export const DEFAULT_SEARCH_STATE: SearchFormState = {
  query: "",
  similarSeed: null,
  mode: "browse",
  filter: null,
  filterError: null,
  pageSize: 25,
};
export function buildFilter(state: SearchFormState): QueryPredicate | null {
  return state.filter;
}
export function buildSearchRequest(
  state: SearchFormState,
  number: number,
  expected_artifact_generation: string | null = null,
): SearchResultsRequest {
  if (state.filterError) throw new Error(state.filterError);
  if (state.similarSeed && state.query.trim()) throw new Error(SIMILAR_QUERY_CONFLICT);
  const page = { number, size: state.pageSize };
  if (state.similarSeed)
    return {
      page,
      expected_artifact_generation,
      criteria: {
        kind: "similar_records",
        seed_record_key: state.similarSeed,
        filter: state.filter,
      },
    };
  return {
    page,
    expected_artifact_generation,
    criteria: state.query.trim()
      ? {
          kind: "text_search",
          query: state.query.trim(),
          filter: state.filter,
          mode: "hybrid",
        }
      : { kind: "list_records", filter: state.filter },
  };
}
export function hasExecutableSearch(state: SearchFormState): boolean {
  return (
    !state.filterError &&
    (!!state.similarSeed || state.query.trim().length > 0 || state.filter !== null)
  );
}
export function buildFilterDiscoveryContext(
  state: SearchFormState,
): FilterDiscoveryContext {
  return {
    kind: "filtered",
    filter: state.filter,
    text: state.query.trim() || null,
    mode: "hybrid",
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
      similarSeed: state.similarSeed,
      mode: state.mode,
      filter: state.filter,
      filterError: state.filterError,
      pageSize: state.pageSize,
    }),
  );
}
export function searchStateQueryString(state: SearchFormState): string {
  const params = new URLSearchParams();
  if (state.query) params.set("q", state.query);
  if (state.similarSeed) params.set("similar", state.similarSeed);
  if (state.mode !== "browse") params.set("mode", state.mode);
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
  const similarSeed = params.get("similar")?.trim() || null;
  if (similarSeed && params.get("q")?.trim()) filterError = SIMILAR_QUERY_CONFLICT;
  return {
    ...DEFAULT_SEARCH_STATE,
    query: params.get("q") || "",
    similarSeed,
    mode: similarSeed
      ? "similar_records"
      : params.get("q")?.trim()
        ? "text_search"
        : "browse",
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
        similar: typeof parsed.similarSeed === "string" ? parsed.similarSeed : "",
        mode: parsed.mode === "text_search" ? "text_search" : "browse",
        filter: parsed.filter ? JSON.stringify(parsed.filter) : "",
        limit: String(parsed.pageSize || 25),
      }),
    );
  } catch {
    return { ...DEFAULT_SEARCH_STATE, filterError: "Invalid search state URL" };
  }
}
