import type { SearchFilters, SearchWindowPage } from "../../domain/search-types.js";
import { buildSearchFilters } from "./filter-building.js";
import type {
  Pf2eTerminalSearchQuery,
  Pf2eTerminalSearchSession,
  Pf2eTerminalSearchSort,
} from "./service-types.js";

function withWindowLimit(query: Pf2eTerminalSearchQuery, limit: number): Pf2eTerminalSearchQuery {
  return query.limit === limit
    ? query
    : {
        ...query,
        limit,
      };
}

export function createSearchSessionFromWindow(
  query: Pf2eTerminalSearchQuery,
  result: SearchWindowPage,
): Pf2eTerminalSearchSession {
  return {
    windowId: result.id,
    query: withWindowLimit(query, result.limit),
    results: result.records,
    windowOffset: result.offset,
    resultMode: result.mode,
    total: result.total,
    loadedCount: result.records.length,
    hasMore: result.hasMore,
    nextOffset: result.nextOffset,
    searchProfile: result.searchProfile,
    sort: result.sort,
    sortSeed: result.sortSeed,
  };
}

export function buildSearchWindowFilters(
  query: Pf2eTerminalSearchQuery,
  options: {
    sort: Pf2eTerminalSearchSort;
    sortSeed: number | null;
    limit: number;
    offset?: number;
  },
): SearchFilters {
  const offset = options.offset ?? 0;
  if (query.mode === "lookup") {
    return buildSearchFilters(query, {
      limit: options.limit,
      offset,
      nameQuery: query.queryText,
      sort: options.sort,
      sortSeed: options.sortSeed,
    });
  }

  return buildSearchFilters(query, {
    limit: options.limit,
    offset,
    query: query.mode === "search" ? query.queryText : undefined,
    searchProfile: query.mode === "search" ? query.searchProfile : undefined,
    sort: options.sort,
    sortSeed: options.sortSeed,
  });
}

export function appendSearchSessionWindowPage(
  session: Pf2eTerminalSearchSession,
  result: SearchWindowPage,
): Pf2eTerminalSearchSession {
  return {
    ...session,
    query: withWindowLimit(session.query, result.limit),
    results: [...session.results, ...result.records],
    total: result.total,
    loadedCount: session.results.length + result.records.length,
    hasMore: result.hasMore,
    nextOffset: result.nextOffset,
    resultMode: result.mode,
    searchProfile: result.searchProfile,
  };
}

export function replaceSearchSessionWindowPage(
  session: Pf2eTerminalSearchSession,
  result: SearchWindowPage,
): Pf2eTerminalSearchSession {
  return {
    ...session,
    query: withWindowLimit(session.query, result.limit),
    results: result.records,
    windowOffset: result.offset,
    total: result.total,
    loadedCount: result.records.length,
    hasMore: result.hasMore,
    nextOffset: result.nextOffset,
    resultMode: result.mode,
    searchProfile: result.searchProfile,
  };
}
