import { SEARCH_REQUEST_VOCABULARY } from "../../domain/search-request-types.js";
import type { SearchRequest } from "../../domain/search-request-types.js";
import { SEARCH_VOCABULARY } from "../../domain/search-types.js";
import type { SearchProfile, SearchSort } from "../../domain/search-types.js";
import type { Pf2eTerminalSearchQuery } from "./service-types.js";
import type { Pf2eTerminalSearchSort } from "./service-types.js";
import { buildLookupSortSpec, isLookupSort } from "./service-options.js";

export function buildSearchRequest(
  query: Pf2eTerminalSearchQuery,
  options: {
    limit?: number;
    offset?: number;
    text?: string;
    exclude?: string;
    searchProfile?: SearchProfile;
    sort?: Pf2eTerminalSearchSort;
    sortSeed?: number | null;
  } = {},
): SearchRequest {
  const sort =
    options.sort === undefined
      ? undefined
      : options.sort === SEARCH_REQUEST_VOCABULARY.SORT_KIND.RANDOM
        ? ({ kind: SEARCH_REQUEST_VOCABULARY.SORT_KIND.RANDOM, seed: options.sortSeed ?? undefined } as const)
        : options.sort === SEARCH_VOCABULARY.SORT_KIND.RANKED || isLookupSort(options.sort)
          ? undefined
          : ({ kind: options.sort satisfies SearchSort } as const);

  if (query.mode === SEARCH_REQUEST_VOCABULARY.MODE.BROWSE) {
    return {
      mode: SEARCH_REQUEST_VOCABULARY.MODE.BROWSE,
      filter: query.filter,
      limit: options.limit ?? query.limit,
      offset: options.offset ?? 0,
      sort,
    };
  }

  if (query.mode === SEARCH_REQUEST_VOCABULARY.MODE.LOOKUP) {
    return {
      mode: SEARCH_REQUEST_VOCABULARY.MODE.LOOKUP,
      search: {
        query: options.text ?? query.search.query,
      },
      filter: query.filter,
      limit: options.limit ?? query.limit,
      offset: options.offset ?? 0,
      sort: options.sort && isLookupSort(options.sort) ? buildLookupSortSpec(options.sort) : undefined,
    };
  }

  return {
    mode: SEARCH_REQUEST_VOCABULARY.MODE.SEARCH,
    search: {
      query: options.text ?? query.search.query,
      ...(options.exclude ?? query.search.exclude ? { exclude: options.exclude ?? query.search.exclude } : {}),
      profile: options.searchProfile ?? query.search.profile,
    },
    explain: false,
    filter: query.filter,
    limit: options.limit ?? query.limit,
    offset: options.offset ?? 0,
  };
}
