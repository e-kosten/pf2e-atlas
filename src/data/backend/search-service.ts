import type { DatabaseSync } from "node:sqlite";

import type {
  FilterValueQuery,
  FilterValueResult,
  LookupResult,
  LookupOptions,
  LookupQuery,
  SearchCountResult,
  SearchResultRecord,
  SearchResult,
  SearchWindowPage,
} from "../../domain/search-types.js";
import type { NormalizedRecord } from "../../domain/record-types.js";
import type { EmbeddingProvider } from "../../embeddings.js";
import type { RankingConfigStore } from "../../search/ranking-config.js";
import { DEFAULT_RANKING_CONFIG } from "../../search/ranking-config.js";
import {
  buildSearchWindowSnapshot,
  countSearchResults as countSearchResultsRuntime,
  countStructuredSearch as countStructuredSearchRuntime,
  listRecords as listRecordsRuntime,
  lookup as lookupRuntime,
  search as searchRuntime,
  searchStructured as searchStructuredRuntime,
} from "../../search/runtime-search.js";
import type { NormalizedSearchFilters, RuntimeSearchDependencies } from "../../search/contracts.js";
import { compileSearchRequest } from "../../search/request-compilation.js";
import { buildAllOfFilter, buildAnyOfFilter, buildScopeFilter, type SearchRequest } from "../../domain/search-request-types.js";
import { fetchCandidateRecordKeys } from "../record-queries.js";
import { getLookupMatchType } from "../../domain/lookup-match-type.js";
import { normalizeSearchFilters, validateSearchFilters } from "../../search/filters/normalization.js";
import { hashRecordSortSeed } from "../../search/runtime-search-sorting.js";
import type { SearchTraceSink } from "../../search/trace.js";
import { traceAsync, traceSync } from "../../search/trace.js";
import { createSearchRetrievalPort } from "./search-retrieval.js";
import { Pf2eRecordCatalog } from "./record-catalog.js";
import { Pf2eSearchWindowStore } from "./search-window-store.js";
import { SEARCH_REQUEST_VOCABULARY } from "../../domain/search-request-types.js";
import { SEARCH_VOCABULARY } from "../../domain/search-types.js";

type AnnotatedLookupRecord = SearchResultRecord & {
  matchType: LookupResult["matchType"];
};

const DISCOVERY_RECORD_KEY_CACHE_LIMIT = 24;

function buildDiscoveryRecordKeyCacheKey(
  normalizedFilters: NormalizedSearchFilters,
): string {
  return JSON.stringify({
    filter: normalizedFilters.filter ?? null,
    query: normalizedFilters.query ?? "",
    nameQuery: normalizedFilters.nameQuery ?? "",
    excludeQuery: normalizedFilters.excludeQuery ?? "",
    searchProfile: normalizedFilters.searchProfile ?? null,
  });
}

function applyLookupTieredOrdering(
  records: readonly AnnotatedLookupRecord[],
): AnnotatedLookupRecord[] {
  const buckets: Record<Exclude<LookupResult["matchType"], "none">, AnnotatedLookupRecord[]> = {
    exact: [],
    normalized_exact: [],
    fuzzy: [],
  };

  for (const record of records) {
    const matchType = record.matchType;
    if (matchType !== "none") {
      buckets[matchType].push(record);
    }
  }

  return [...buckets.exact, ...buckets.normalized_exact, ...buckets.fuzzy];
}

function annotateLookupRecords(
  query: string,
  records: readonly SearchResultRecord[],
): AnnotatedLookupRecord[] {
  return records.map((record) => ({
    ...record,
    matchType: getLookupMatchType(query, record),
  }));
}

export class Pf2eSearchBackendService {
  private readonly searchWindows: Pf2eSearchWindowStore;
  private readonly discoveryRecordKeysByRequest = new Map<string, Promise<string[]>>();
  private trace: SearchTraceSink | undefined;

  constructor(
    private readonly db: DatabaseSync,
    private readonly catalog: Pf2eRecordCatalog,
    private readonly embeddingProvider: EmbeddingProvider,
    private readonly rankingConfigStore: RankingConfigStore | null,
  ) {
    this.searchWindows = new Pf2eSearchWindowStore((recordKeys) => this.catalog.getRecordsByKeys(recordKeys));
  }

  setTraceSink(trace: SearchTraceSink | undefined): void {
    this.trace = trace;
  }

  listFilterValues(query: FilterValueQuery): FilterValueResult {
    return traceSync(
      this.trace,
      "backend.listFilterValues",
      { field: query.field, category: query.category ?? "any" },
      () => {
        const normalizedFilters = this.normalizeRequest({
          mode: SEARCH_REQUEST_VOCABULARY.MODE.BROWSE,
          filter:
            query.scopes && query.scopes.length > 0
              ? buildAnyOfFilter(
                  query.scopes.map((scope) => buildScopeFilter(scope.category, scope.subcategories?.[0] ?? null)),
                )
              : query.category
                ? buildScopeFilter(query.category, query.subcategory ?? null)
                : undefined,
        });
        validateSearchFilters(normalizedFilters, "list");
        return this.catalog.listFilterValues(query, normalizedFilters);
      },
      (result) => ({ values: result.values.length }),
    );
  }

  async discoverFilterValues(
    query: FilterValueQuery,
    request: Readonly<SearchRequest>,
  ): Promise<FilterValueResult> {
    return traceAsync(
      this.trace,
      "backend.discoverFilterValues",
      { field: query.field, requestMode: request.mode, category: query.category ?? "any" },
      async () => {
        const normalizedFilters = this.normalizeRequest(request);
        validateSearchFilters(
          normalizedFilters,
          request.mode === SEARCH_REQUEST_VOCABULARY.MODE.BROWSE ? "list" : "search",
        );
        const recordKeys = await this.resolveDiscoveryRecordKeys(request, normalizedFilters);
        return this.catalog.listFilterValues(query, normalizedFilters, {
          ...(recordKeys ? { recordKeys } : {}),
        });
      },
      (result) => ({ values: result.values.length }),
    );
  }

  listRecords(request: SearchRequest): SearchResult {
    const normalizedFilters = this.normalizeRequest(request);
    validateSearchFilters(normalizedFilters, "list");
    return listRecordsRuntime(normalizedFilters, this.runtimeSearchDependencies());
  }

  async countRecords(
    request: SearchRequest,
    options: { lexicalOnly?: boolean } = {},
  ): Promise<SearchCountResult> {
    const normalizedRequest = request;
    const runtime = this.runtimeSearchDependencies();
    const executionFilters = this.compileRequest({
      ...normalizedRequest,
      offset: 0,
      limit: 1,
    });

    if (normalizedRequest.mode === SEARCH_REQUEST_VOCABULARY.MODE.BROWSE) {
      const normalizedFilters = this.normalizeExecutionFilters(executionFilters);
      validateSearchFilters(normalizedFilters, "list");
      return countStructuredSearchRuntime(normalizedFilters, runtime);
    }

    if (options.lexicalOnly && executionFilters.query?.trim()) {
      executionFilters.searchProfile = SEARCH_VOCABULARY.PROFILE.LEXICAL;
    }

    const normalizedFilters = this.normalizeExecutionFilters(executionFilters);
    validateSearchFilters(normalizedFilters, "search");

    if (normalizedRequest.mode === SEARCH_REQUEST_VOCABULARY.MODE.LOOKUP) {
      return countStructuredSearchRuntime(normalizedFilters, runtime);
    }

    return countSearchResultsRuntime(executionFilters, normalizedFilters, runtime);
  }

  async openSearchWindow(request: SearchRequest): Promise<SearchWindowPage> {
    const normalizedRequest = request;
    const runtime = this.runtimeSearchDependencies();
    const normalizedFilters = this.normalizeRequest(normalizedRequest);

    if (normalizedRequest.mode === SEARCH_REQUEST_VOCABULARY.MODE.BROWSE) {
      validateSearchFilters(normalizedFilters, "list");
      const sort =
        normalizedFilters.sort === SEARCH_VOCABULARY.SORT_KIND.RANKED || !normalizedFilters.sort
          ? SEARCH_VOCABULARY.SORT_KIND.ALPHABETICAL
          : normalizedFilters.sort;
      const sortSeed = sort === SEARCH_VOCABULARY.SORT_KIND.RANDOM ? (normalizedFilters.sortSeed ?? 0) : null;
      const orderedRecordKeys =
        sort === SEARCH_VOCABULARY.SORT_KIND.RANDOM
          ? fetchCandidateRecordKeys(this.db, normalizedFilters).sort((left, right) => {
              const leftHash = hashRecordSortSeed(left, sortSeed ?? 0);
              const rightHash = hashRecordSortSeed(right, sortSeed ?? 0);
              return leftHash - rightHash || left.localeCompare(right);
            })
          : fetchCandidateRecordKeys(this.db, normalizedFilters, sort);
      const window = this.searchWindows.openWindow({
        kind: "recordKeys",
        mode: SEARCH_VOCABULARY.MODE.STRUCTURED,
        searchProfile: null,
        sort,
        sortSeed,
        total: orderedRecordKeys.length,
        orderedRecords: orderedRecordKeys.map((recordKey) => ({ recordKey })),
      });
      return this.searchWindows.readWindowPage(window.id, normalizedFilters.offset ?? 0, normalizedFilters.limit ?? 20);
    }

    validateSearchFilters(normalizedFilters, "search");
    const snapshot = await buildSearchWindowSnapshot(normalizedFilters, runtime);
    const lookupSortPolicy =
      normalizedRequest.mode === SEARCH_REQUEST_VOCABULARY.MODE.LOOKUP
        ? normalizedRequest.sort?.policy ?? SEARCH_REQUEST_VOCABULARY.LOOKUP_SORT_POLICY.TIERED
        : SEARCH_REQUEST_VOCABULARY.LOOKUP_SORT_POLICY.TIERED;
    const orderedRecords =
      normalizedRequest.mode === SEARCH_REQUEST_VOCABULARY.MODE.LOOKUP
        ? (() => {
            const annotatedRecords = annotateLookupRecords(normalizedRequest.search.query, snapshot.records);
            return lookupSortPolicy === SEARCH_REQUEST_VOCABULARY.LOOKUP_SORT_POLICY.TIERED
              ? applyLookupTieredOrdering(annotatedRecords)
              : annotatedRecords;
          })()
        : snapshot.records;
    const window = this.searchWindows.openWindow({
      kind: "recordKeys",
      mode: snapshot.mode,
      searchProfile: snapshot.searchProfile,
      sort: snapshot.sort,
      sortSeed: normalizedFilters.sortSeed ?? null,
      total: orderedRecords.length,
      orderedRecords: orderedRecords.map((record) => ({
        recordKey: record.recordKey,
        matchType: record.matchType,
      })),
    });
    return this.searchWindows.readWindowPage(window.id, normalizedFilters.offset ?? 0, normalizedFilters.limit ?? 20);
  }

  readSearchWindowPage(windowId: string, offset: number, limit: number): SearchWindowPage {
    return this.searchWindows.readWindowPage(windowId, offset, limit);
  }

  closeSearchWindow(windowId: string): void {
    this.searchWindows.closeWindow(windowId);
  }

  async search(request: SearchRequest): Promise<SearchResult> {
    const executionFilters = this.compileRequest(request);
    const normalizedFilters = this.normalizeExecutionFilters(executionFilters);
    validateSearchFilters(normalizedFilters, "search");
    const result = await searchRuntime(executionFilters, normalizedFilters, this.runtimeSearchDependencies());
    if (request.mode !== SEARCH_REQUEST_VOCABULARY.MODE.LOOKUP) {
      return result;
    }

    return {
      ...result,
      records: annotateLookupRecords(request.search.query, result.records),
    };
  }

  lookup(
    name: string,
    options: LookupOptions = {},
  ): { match: NormalizedRecord | null; alternatives: NormalizedRecord[]; matchType: LookupResult["matchType"] } {
    const filters = this.normalizeRequest({
      mode: SEARCH_REQUEST_VOCABULARY.MODE.LOOKUP,
      search: { query: name },
      filter: buildAllOfFilter([
        options.pack
          ? { kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.PACK, value: options.pack }
          : undefined,
        options.category ? buildScopeFilter(options.category, options.subcategory ?? null) : undefined,
      ]),
      limit: 5,
    });
    validateSearchFilters(filters, "search");
    const lookup = lookupRuntime(name, filters, this.runtimeSearchDependencies());
    return {
      ...lookup,
      matchType: getLookupMatchType(name, lookup.match),
    };
  }

  lookupMany(queries: LookupQuery[], options: { coreOnly?: boolean } = {}): LookupResult[] {
    return queries.map((query) => {
      const lookup = (() => {
        if (!options.coreOnly) {
          return this.lookup(query.name, query);
        }

        const results = this.searchStructured({
          mode: SEARCH_REQUEST_VOCABULARY.MODE.LOOKUP,
          search: { query: query.name },
          filter: buildAllOfFilter([
            query.pack
              ? { kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.PACK, value: query.pack }
              : undefined,
            query.category ? buildScopeFilter(query.category, query.subcategory ?? null) : undefined,
            {
              kind: SEARCH_REQUEST_VOCABULARY.FILTER_NODE_KIND.METADATA_PREDICATE,
              predicate: { field: "sourceCategory", op: "eq", value: "core" },
            },
          ]),
          limit: 5,
        }).records;
        return {
          match: results[0] ?? null,
          alternatives: results.slice(1),
          matchType: getLookupMatchType(query.name, results[0] ?? null),
        };
      })();

      return {
        query,
        match: lookup.match,
        alternatives: lookup.alternatives,
        matchType: lookup.matchType,
      };
    });
  }

  private searchStructured(request: SearchRequest): SearchResult {
    const normalizedFilters = this.normalizeRequest(request);
    validateSearchFilters(normalizedFilters, "search");
    return searchStructuredRuntime(normalizedFilters, this.runtimeSearchDependencies());
  }

  private compileRequest(request: SearchRequest) {
    return compileSearchRequest(request);
  }

  private normalizeExecutionFilters(filters: Parameters<typeof normalizeSearchFilters>[0]) {
    return normalizeSearchFilters(filters, (packValue) => this.catalog.getPack(packValue)?.name);
  }

  private normalizeRequest(request: SearchRequest) {
    return this.normalizeExecutionFilters(this.compileRequest(request));
  }

  private async resolveDiscoveryRecordKeys(
    request: Readonly<SearchRequest>,
    normalizedFilters: ReturnType<Pf2eSearchBackendService["normalizeRequest"]>,
  ): Promise<string[] | undefined> {
    if (request.mode === SEARCH_REQUEST_VOCABULARY.MODE.BROWSE) {
      return undefined;
    }

    if (!normalizedFilters.query && !normalizedFilters.nameQuery && !normalizedFilters.excludeQuery) {
      return undefined;
    }

    const cacheKey = buildDiscoveryRecordKeyCacheKey(normalizedFilters);
    const cached = this.discoveryRecordKeysByRequest.get(cacheKey);
    if (cached) {
      this.trace?.startSpan("backend.resolveDiscoveryRecordKeys", { cache: "hit" }).end();
      return cached;
    }

    const promise = traceAsync(
      this.trace,
      "backend.resolveDiscoveryRecordKeys",
      {
        cache: "miss",
        mode: normalizedFilters.query ? SEARCH_REQUEST_VOCABULARY.MODE.SEARCH : SEARCH_VOCABULARY.MODE.STRUCTURED,
        profile: normalizedFilters.searchProfile ?? "default",
      },
      () =>
        buildSearchWindowSnapshot(normalizedFilters, this.runtimeSearchDependencies()).then((snapshot) =>
          snapshot.records.map((record) => record.recordKey),
        ),
      (recordKeys) => ({ recordKeys: recordKeys.length }),
    );
    this.discoveryRecordKeysByRequest.set(cacheKey, promise);
    if (this.discoveryRecordKeysByRequest.size > DISCOVERY_RECORD_KEY_CACHE_LIMIT) {
      const oldestKey = this.discoveryRecordKeysByRequest.keys().next().value;
      if (oldestKey) {
        this.discoveryRecordKeysByRequest.delete(oldestKey);
      }
    }
    void promise.catch(() => {
      if (this.discoveryRecordKeysByRequest.get(cacheKey) === promise) {
        this.discoveryRecordKeysByRequest.delete(cacheKey);
      }
    });
    return promise;
  }

  private runtimeSearchDependencies(): RuntimeSearchDependencies {
    return {
      ...createSearchRetrievalPort({
        db: this.db,
        decorateRecord: (record) => this.catalog.decorateRecord(record),
        trace: this.trace,
      }),
      embeddingProvider: this.embeddingProvider,
      rankingConfig: this.rankingConfigStore?.getConfig() ?? DEFAULT_RANKING_CONFIG,
      rankingConfigStatus: this.catalog.getRankingConfigStatus(),
      trace: this.trace,
      getAliases: (recordKey) => this.catalog.getAliases(recordKey),
    };
  }
}
