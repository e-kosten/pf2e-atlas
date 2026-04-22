import type { SearchRequest } from "../../domain/search-request-types.js";
import type { SearchProfile, SearchSort } from "../../domain/search-types.js";
import {
  getSearchQueryCategory,
} from "./query-state.js";
import type { Pf2eTerminalSearchQuery } from "./service-types.js";

export function buildSearchRequest(
  query: Pf2eTerminalSearchQuery,
  options: {
    limit?: number;
    offset?: number;
    text?: string;
    searchProfile?: SearchProfile;
    sort?: SearchSort;
    sortSeed?: number | null;
  } = {},
): SearchRequest {
  return {
    intent: query.mode,
    text: options.text ?? (query.mode === "browse" ? undefined : query.queryText),
    category: getSearchQueryCategory(query) ?? undefined,
    parts: query.filters.parts,
    limit: options.limit ?? query.limit,
    offset: options.offset ?? 0,
    searchProfile: options.searchProfile ?? (query.mode === "search" ? query.searchProfile : undefined),
    sort: options.sort,
    sortSeed: options.sortSeed ?? undefined,
  };
}
