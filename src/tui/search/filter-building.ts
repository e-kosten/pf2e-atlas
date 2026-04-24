import type { SearchRequest } from "../../domain/search-request-types.js";
import type { SearchProfile, SearchSort } from "../../domain/search-types.js";
import type { Pf2eTerminalSearchQuery } from "./service-types.js";

export function buildSearchRequest(
  query: Pf2eTerminalSearchQuery,
  options: {
    limit?: number;
    offset?: number;
    text?: string;
    exclude?: string;
    searchProfile?: SearchProfile;
    sort?: SearchSort;
    sortSeed?: number | null;
  } = {},
): SearchRequest {
  const sort =
    options.sort === undefined
      ? undefined
      : options.sort === "random"
        ? ({ kind: "random", seed: options.sortSeed ?? undefined } as const)
        : options.sort === "ranked"
          ? undefined
          : ({ kind: options.sort } as const);

  if (query.mode === "browse") {
    return {
      mode: "browse",
      filter: query.filter,
      limit: options.limit ?? query.limit,
      offset: options.offset ?? 0,
      sort,
    };
  }

  if (query.mode === "lookup") {
    return {
      mode: "lookup",
      search: {
        query: options.text ?? query.search.query,
      },
      filter: query.filter,
      limit: options.limit ?? query.limit,
      offset: options.offset ?? 0,
      sort:
        options.sort && options.sort !== "random" && options.sort !== "ranked"
          ? ({ kind: options.sort, policy: "tiered" } as const)
          : undefined,
    };
  }

  return {
    mode: "search",
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
