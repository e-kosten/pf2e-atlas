import type {
  FilterDiscoveryContext,
  SearchRanking,
  SearchResultsRequest,
  QueryPredicate,
  RelationshipConstraintView,
  ResultSort,
  SortField,
  SortDirection,
} from "../../generated/atlas";
import { decodeFilterUrl, encodeFilterUrl, UNSUPPORTED_FILTER_URL } from "./filterUrl";
import { decodeRelationships, encodeRelationships } from "./relationshipUrl";
export const SIMILAR_QUERY_CONFLICT = "Choose either text or a similar-record seed";
export const UNKNOWN_SEARCH_RANKING = "Unknown search ranking";
export const UNKNOWN_RESULT_SORT = "Unknown result order";
export const INVALID_RESULT_SORT = "Choose an order available for this search";
export type SearchFormState = {
  sort?: ResultSort;
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
  const sortRequest = state.sort ? { sort: state.sort } : {};
  if (state.similarSeed)
    return {
      search_profile,
      ...relationshipRequest,
      ...sortRequest,
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
    ...sortRequest,
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
    ...(state.relationships?.length ? { relationships: state.relationships } : {}),
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
      sort: state.sort,
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
  encodeResultSort(params, state.sort);
  if (state.query) params.set("q", state.query);
  if (state.similarSeed) params.set("similar", state.similarSeed);
  if (state.relationships?.length)
    params.set("relationships", encodeRelationships(state.relationships));
  if (state.filter) params.set("filter", encodeFilterUrl(state.filter));
  if (state.pageSize !== 25) params.set("limit", String(state.pageSize));
  return params.size ? "?" + params.toString() : "";
}
export function decodeSearchStateFromParams(
  params: URLSearchParams,
  surface: "search" | "saved_list" = "search",
): SearchFormState {
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
  const { sort, sortError } = decodeResultSort(
    params,
    surface,
    Boolean(similarSeed || params.get("q")?.trim()),
  );
  filterError ??= sortError;
  return {
    ...DEFAULT_SEARCH_STATE,
    ranking: ranking === "rrf" ? "rrf" : "score",
    ...(sort ? { sort } : {}),
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

export const INVALID_SORT_SEED =
  "Random seed must be a whole number from 0 to 4294967295";
export const UNEXPECTED_SORT_SEED = "Seed is only available with Random order";

export function parseRandomSeed(value: string): number | null {
  if (!/^\d+$/.test(value)) return null;
  const seed = Number(value);
  return Number.isInteger(seed) && seed >= 0 && seed <= 4294967295 ? seed : null;
}

export function encodeResultSort(params: URLSearchParams, sort?: ResultSort): void {
  params.delete("sort");
  params.delete("seed");
  if (!sort) return;
  if (sort.kind === "attributes") {
    for (const rule of sort.rules)
      params.append("sort", `${rule.field}_${rule.direction}`);
  } else {
    params.set("sort", sort.kind);
    if (sort.kind === "random") params.set("seed", String(sort.seed));
  }
}

export function decodeResultSort(
  params: URLSearchParams,
  surface: "search" | "saved_list",
  semantic: boolean,
): { sort?: ResultSort; sortError: string | null } {
  const tokens = params.getAll("sort");
  const seeds = params.getAll("seed");
  if (tokens.length === 1 && tokens[0] === "random") {
    const seed = seeds.length === 1 ? parseRandomSeed(seeds[0]) : null;
    return seed === null
      ? { sortError: INVALID_SORT_SEED }
      : { sort: { kind: "random", seed }, sortError: null };
  }
  if (seeds.length) return { sortError: UNEXPECTED_SORT_SEED };
  if (!tokens.length) return { sortError: null };
  if (
    tokens.length === 1 &&
    (tokens[0] === "relevance" || tokens[0] === "saved_order")
  ) {
    const kind = tokens[0];
    return {
      sort: { kind },
      sortError:
        surface === "saved_list"
          ? kind === "relevance"
            ? INVALID_RESULT_SORT
            : null
          : kind === "saved_order" || !semantic
            ? INVALID_RESULT_SORT
            : null,
    };
  }
  const rules = tokens.map((token) => {
    const match = /^(name|level|unit_price)_(asc|desc)$/.exec(token);
    return match
      ? { field: match[1] as SortField, direction: match[2] as SortDirection }
      : null;
  });
  if (rules.some((rule) => !rule)) return { sortError: UNKNOWN_RESULT_SORT };
  const validRules = rules.filter((rule) => rule !== null);
  if (
    validRules.length > 3 ||
    new Set(validRules.map((rule) => rule.field)).size !== validRules.length
  )
    return { sortError: INVALID_RESULT_SORT };
  return { sort: { kind: "attributes", rules: validRules }, sortError: null };
}

export function isResultSortError(error: string | null): boolean {
  return [
    UNKNOWN_RESULT_SORT,
    INVALID_RESULT_SORT,
    INVALID_SORT_SEED,
    UNEXPECTED_SORT_SEED,
  ].some((value) => value === error);
}

/** Resolve display/response identity without changing optional URL intent. */
export function effectiveResultSort(
  state: Pick<SearchFormState, "sort" | "query" | "similarSeed">,
  surface: "search" | "saved_list" = "search",
): ResultSort {
  return (
    state.sort ??
    (surface === "saved_list"
      ? { kind: "saved_order" }
      : state.query.trim() || state.similarSeed
        ? { kind: "relevance" }
        : { kind: "attributes", rules: [{ field: "name", direction: "asc" }] })
  );
}

/** Removing a semantic criterion retires Relevance; explicit attributes stay sticky. */
export function normalizeSortTransition(
  previous: SearchFormState,
  next: SearchFormState,
): SearchFormState {
  return next.sort?.kind === "relevance" &&
    !next.query.trim() &&
    !next.similarSeed &&
    Boolean(previous.query.trim() || previous.similarSeed)
    ? { ...next, sort: undefined }
    : next;
}
