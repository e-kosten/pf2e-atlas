import type {
  FilterDiscoveryContext,
  SearchRanking,
  SearchResultsRequest,
  QueryPredicate,
  RelationshipConstraintView,
} from "../../generated/atlas";
import { decodeFilterUrl, encodeFilterUrl, UNSUPPORTED_FILTER_URL } from "./filterUrl";
import { decodeRelationships, encodeRelationships } from "./relationshipUrl";
export const SIMILAR_QUERY_CONFLICT = "Choose either text or a similar-record seed";
export const UNKNOWN_SEARCH_RANKING = "Unknown search ranking";
export type SearchFormState = {
  ranking?: SearchRanking;
  relationships?: RelationshipConstraintView[];
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
  search_profile?: string,
): SearchResultsRequest {
  if (state.filterError) throw new Error(state.filterError);
  if (state.similarSeed && state.query.trim()) throw new Error(SIMILAR_QUERY_CONFLICT);
  const page = { number, size: state.pageSize };
  const relationshipRequest = state.relationships?.length
    ? { relationships: state.relationships }
    : {};
  if (state.similarSeed)
    return {
      search_profile,
      ...relationshipRequest,
      page,
      expected_artifact_generation,
      criteria: {
        kind: "similar_records",
        seed_record_key: state.similarSeed,
        filter: state.filter,
      },
    };
  return {
    search_profile,
    ...relationshipRequest,
    page,
    expected_artifact_generation,
    ranking: state.ranking ?? "score",
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
    (!!state.similarSeed ||
      state.query.trim().length > 0 ||
      state.filter !== null ||
      !!state.relationships?.length)
  );
}
export function buildFilterDiscoveryContext(
  state: SearchFormState,
  search_profile?: string,
  expected_artifact_generation?: string,
): FilterDiscoveryContext {
  return {
    search_profile,
    expected_artifact_generation,
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
  return {
    ...buildFilterDiscoveryContext(state),
    kind: "saved_list",
    list_ref,
    mode: "lexical",
  };
}
export function encodeSearchState(state: SearchFormState): string {
  return encodeURIComponent(JSON.stringify(state));
}
export function encodeSearchExecutionState(
  state: SearchFormState,
  search_profile?: string | null,
): string {
  return encodeURIComponent(
    JSON.stringify({
      search_profile,
      ranking: state.ranking ?? "score",
      query: state.query,
      similarSeed: state.similarSeed,
      relationships: state.relationships,
      mode: state.mode,
      filter: state.filter,
      filterError: state.filterError,
      pageSize: state.pageSize,
    }),
  );
}
export function searchStateQueryString(
  state: SearchFormState,
  search_profile?: string | null,
): string {
  const params = new URLSearchParams();
  if (search_profile != null) params.set("search_profile", search_profile);
  if (state.ranking && state.ranking !== "score") params.set("ranking", state.ranking);
  if (state.query) params.set("q", state.query);
  if (state.similarSeed) params.set("similar", state.similarSeed);
  if (state.relationships?.length)
    params.set("relationships", encodeRelationships(state.relationships));
  if (state.filter) params.set("filter", encodeFilterUrl(state.filter));
  if (state.pageSize !== 25) params.set("limit", String(state.pageSize));
  return params.size ? "?" + params.toString() : "";
}
export function decodeSearchStateFromParams(params: URLSearchParams): SearchFormState {
  let filter: QueryPredicate | null = null;
  let filterError: string | null = null;
  try {
    const encoded = params.get("filter");
    if (encoded) filter = decodeFilterUrl(encoded);
  } catch (error) {
    filterError =
      error instanceof Error && error.message === UNSUPPORTED_FILTER_URL
        ? UNSUPPORTED_FILTER_URL
        : "Invalid structured filter URL";
  }
  let relationships: RelationshipConstraintView[] | undefined;
  if (params.has("relationships")) {
    try {
      relationships = decodeRelationships(params.get("relationships")!);
    } catch (error) {
      filterError =
        error instanceof Error ? error.message : "Invalid relationship filter URL";
    }
  }
  const ranking = params.get("ranking") ?? "score";
  if (!filterError && ranking !== "rrf" && ranking !== "score")
    filterError = UNKNOWN_SEARCH_RANKING;
  const pageSize = Number(params.get("limit") || 25);
  const similarSeed = params.get("similar")?.trim() || null;
  if (similarSeed && params.get("q")?.trim()) filterError = SIMILAR_QUERY_CONFLICT;
  return {
    ...DEFAULT_SEARCH_STATE,
    ranking: ranking === "rrf" ? "rrf" : "score",
    query: params.get("q") || "",
    similarSeed,
    ...(relationships ? { relationships } : {}),
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
