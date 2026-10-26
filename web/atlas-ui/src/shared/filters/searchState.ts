import type {
  FilterDiscoveryContext,
  SearchRanking,
  SearchResultsRequest,
  QueryPredicate,
  ResultSort,
  SortField,
  SortDirection,
  SimilarityOptions,
} from "../../generated/atlas";
import { decodeFilterUrl, encodeFilterUrl } from "./filterUrl";
import {
  DEFAULT_SEARCH_PREFERENCES,
  type SearchPreferences,
} from "../search/searchPreferences";
export const SIMILAR_QUERY_CONFLICT = "Choose either text or a similar-record seed";
export const SEARCH_SCOPE_CONFLICT = "Choose either Similar or trait references";
export const UNKNOWN_SEARCH_RANKING = "Unknown search ranking";
export const UNKNOWN_RESULT_SORT = "Unknown result order";
export const INVALID_RESULT_SORT = "Choose an order available for this search";
export const INVALID_SIMILARITY_SETTINGS =
  "Invalid similarity settings in this search link";
export type SearchFormState = {
  sort?: ResultSort;
  ranking?: SearchRanking;
  similarity?: SimilarityOptions;
  similarityGeneration?: string;
  query: string;
  mode: "browse" | "text_search" | "similar_records";
  similarSeed: string | null;
  traitReferenceSeed?: string | null;
  filter: QueryPredicate | null;
  filterError: string | null;
  /** Transient URL recovery state, excluded from requests and execution identity. */
  filterWasReset?: boolean;
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
  if (state.similarSeed && state.traitReferenceSeed)
    throw new Error(SEARCH_SCOPE_CONFLICT);
  if (state.similarSeed && state.query.trim()) throw new Error(SIMILAR_QUERY_CONFLICT);
  const page = { number, size: state.pageSize };
  const sortRequest = state.sort ? { sort: state.sort } : {};
  if (state.traitReferenceSeed)
    return {
      search_profile,
      ...sortRequest,
      page,
      expected_artifact_generation,
      ranking: state.ranking ?? "score",
      criteria: {
        kind: "trait_references",
        trait_key: state.traitReferenceSeed,
        query: state.query.trim() || null,
        filter: state.filter,
      },
    };
  if (state.similarSeed)
    return {
      search_profile,
      ...sortRequest,
      page,
      expected_artifact_generation:
        state.similarityGeneration ?? expected_artifact_generation,
      criteria: state.similarSeed.startsWith("trait:")
        ? {
            kind: "similar_traits",
            seed_trait_key: state.similarSeed,
            filter: state.filter,
            similarity: state.similarity,
          }
        : {
            kind: "similar_records",
            seed_record_key: state.similarSeed,
            filter: state.filter,
            similarity: state.similarity,
          },
    };
  return {
    search_profile,
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
      !!state.traitReferenceSeed ||
      state.query.trim().length > 0 ||
      state.filter !== null)
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
      similarity: state.similarSeed ? state.similarity : undefined,
      similarityGeneration: state.similarSeed ? state.similarityGeneration : undefined,
      sort: state.sort,
      query: state.query,
      similarSeed: state.similarSeed,
      traitReferenceSeed: state.traitReferenceSeed,
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
  if (state.ranking) params.set("ranking", state.ranking);
  encodeResultSort(params, state.sort);
  if (state.query) params.set("q", state.query);
  if (state.similarSeed) params.set("similar", state.similarSeed);
  if (state.similarSeed && state.similarity) {
    const settings = state.similarity;
    params.set("similarity_method", settings.method);
    if (settings.method !== "baseline") {
      params.set("candidate_parts", settings.candidate_parts);
      if (settings.method === "passages")
        params.set("similarity_decay", String(settings.decay));
      if (settings.parts)
        for (const part of settings.parts) params.append("similarity_part", part);
      if (state.similarityGeneration)
        params.set("similarity_generation", state.similarityGeneration);
    }
  }
  if (state.traitReferenceSeed) params.set("trait_refs", state.traitReferenceSeed);
  if (state.filter) params.set("filter", encodeFilterUrl(state.filter));
  if (state.pageSize !== 25) params.set("limit", String(state.pageSize));
  return params.size ? "?" + params.toString() : "";
}
export function decodeSearchStateFromParams(
  params: URLSearchParams,
  surface: "search" | "saved_list" = "search",
  preferences: SearchPreferences = DEFAULT_SEARCH_PREFERENCES,
): SearchFormState {
  let filter: QueryPredicate | null = null;
  let filterError: string | null = null;
  let filterWasReset = false;
  try {
    const encoded = params.get("filter");
    if (encoded !== null) filter = decodeFilterUrl(encoded);
  } catch {
    filterWasReset = true;
  }
  if (params.has("relationships")) {
    filter = null;
    filterWasReset = true;
  }
  const ranking = params.get("ranking") ?? preferences.ranking;
  if (!filterError && ranking !== "rrf" && ranking !== "score")
    filterError = UNKNOWN_SEARCH_RANKING;
  const pageSize = Number(params.get("limit") || 25);
  const similarSeed = params.get("similar")?.trim() || null;
  let similarity: SimilarityOptions | undefined;
  if (similarSeed) {
    try {
      similarity = decodeSimilarityOptions(
        params,
        preferences,
        similarSeed.startsWith("trait:"),
      );
    } catch {
      filterError = INVALID_SIMILARITY_SETTINGS;
    }
  }
  if (similarSeed && params.get("q")?.trim()) filterError = SIMILAR_QUERY_CONFLICT;
  if (similarSeed && params.get("trait_refs")) filterError = SEARCH_SCOPE_CONFLICT;
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
    ...(similarity ? { similarity } : {}),
    ...(params.get("similarity_generation")
      ? { similarityGeneration: params.get("similarity_generation")! }
      : {}),
    ...(params.get("trait_refs")
      ? { traitReferenceSeed: params.get("trait_refs") }
      : {}),
    mode: similarSeed
      ? "similar_records"
      : params.get("q")?.trim()
        ? "text_search"
        : "browse",
    filter,
    filterError,
    ...(filterWasReset ? { filterWasReset: true } : {}),
    pageSize:
      Number.isInteger(pageSize) && pageSize >= 1 && pageSize <= 100 ? pageSize : 25,
  };
}

function decodeSimilarityOptions(
  params: URLSearchParams,
  preferences: SearchPreferences,
  trait: boolean,
): SimilarityOptions {
  const preferred = trait ? preferences.traitSimilarity : preferences.recordSimilarity;
  const method = params.get("similarity_method") ?? preferred.method;
  const hasContentOptions =
    params.has("candidate_parts") ||
    params.has("similarity_decay") ||
    params.has("similarity_part") ||
    params.has("similarity_generation");
  if (method === "baseline") {
    if (hasContentOptions) throw new Error(INVALID_SIMILARITY_SETTINGS);
    return { method };
  }
  if (method !== "passages" && method !== "content-average")
    throw new Error(INVALID_SIMILARITY_SETTINGS);
  const candidate_parts =
    params.get("candidate_parts") ?? (trait ? "any" : preferred.candidateParts);
  if (
    (candidate_parts !== "any" && candidate_parts !== "corresponding") ||
    (trait && candidate_parts !== "any")
  )
    throw new Error(INVALID_SIMILARITY_SETTINGS);
  const parts = params.has("similarity_part") ? params.getAll("similarity_part") : null;
  if (parts?.some((part) => !part) || (parts && !params.get("similarity_generation")))
    throw new Error(INVALID_SIMILARITY_SETTINGS);
  if (method === "content-average") {
    if (params.has("similarity_decay")) throw new Error(INVALID_SIMILARITY_SETTINGS);
    return { method, parts, candidate_parts };
  }
  const rawDecay = params.get("similarity_decay");
  const decay = rawDecay === null ? preferred.decay : Number(rawDecay);
  if (rawDecay?.trim() === "" || !Number.isFinite(decay) || decay < 0 || decay > 1)
    throw new Error(INVALID_SIMILARITY_SETTINGS);
  return { method, parts, candidate_parts, decay };
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
