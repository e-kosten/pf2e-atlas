import type { SearchWindowPage } from "../../domain/search-types.js";
import type { SearchRequest } from "../../domain/search-request-types.js";
import { getLookupMatchType } from "../../domain/lookup-match-type.js";
import { buildSearchRequest } from "./filter-building.js";
import type {
  Pf2eTerminalSearchQuery,
  Pf2eTerminalSearchResultRecord,
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

function annotateWindowRecords(
  query: Pf2eTerminalSearchQuery,
  records: SearchWindowPage["records"],
): Pf2eTerminalSearchResultRecord[] {
  if (query.mode !== "lookup") {
    return records;
  }

  return records.map((record) => ({
    ...record,
    matchType: getLookupMatchType(query.search.query, record),
  }));
}

export function createSearchSessionFromWindow(
  query: Pf2eTerminalSearchQuery,
  result: SearchWindowPage,
  options: {
    sort: Pf2eTerminalSearchSort;
    sortSeed: number | null;
  },
): Pf2eTerminalSearchSession {
  return {
    windowId: result.id,
    query: withWindowLimit(query, result.limit),
    results: annotateWindowRecords(query, result.records),
    windowOffset: result.offset,
    resultMode: result.mode,
    total: result.total,
    loadedCount: result.records.length,
    hasMore: result.hasMore,
    nextOffset: result.nextOffset,
    searchProfile: result.searchProfile,
    sort: options.sort,
    sortSeed: options.sortSeed,
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
): SearchRequest {
  const offset = options.offset ?? 0;
  return buildSearchRequest(query, {
    limit: options.limit,
    offset,
    text: query.mode === "browse" ? undefined : query.search.query,
    exclude: query.mode === "search" ? query.search.exclude : undefined,
    searchProfile: query.mode === "search" ? query.search.profile : undefined,
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
    results: [...session.results, ...annotateWindowRecords(session.query, result.records)],
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
    results: annotateWindowRecords(session.query, result.records),
    windowOffset: result.offset,
    total: result.total,
    loadedCount: result.records.length,
    hasMore: result.hasMore,
    nextOffset: result.nextOffset,
    resultMode: result.mode,
    searchProfile: result.searchProfile,
  };
}
