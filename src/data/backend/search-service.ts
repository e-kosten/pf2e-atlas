import type { DatabaseSync } from "node:sqlite";

import type {
  FilterValueQuery,
  FilterValueResult,
  LookupOptions,
  LookupQuery,
  LookupResult,
  SearchCountResult,
  SearchResult,
  SearchWindowPage,
} from "../../domain/search-types.js";
import type { NormalizedRecord } from "../../domain/record-types.js";
import type { EmbeddingProvider } from "../../embeddings.js";
import type { RankingConfigStore } from "../../search/ranking-config.js";
import {
  buildSearchWindowSnapshot,
  countSearchResults as countSearchResultsRuntime,
  countStructuredSearch as countStructuredSearchRuntime,
  listRecords as listRecordsRuntime,
  lookup as lookupRuntime,
  search as searchRuntime,
  searchStructured as searchStructuredRuntime,
} from "../../search/runtime-search.js";
import { compileSearchRequest } from "../../search/request-compilation.js";
import { buildAllOfFilter, buildAnyOfFilter, buildScopeFilter, type SearchRequest } from "../../domain/search-request-types.js";
import { fetchCandidateRecordKeys } from "../record-queries.js";
import { getLookupMatchType } from "../rows.js";
import { normalizeSearchFilters, validateSearchFilters } from "../../search/filters/normalization.js";
import { hashRecordSortSeed } from "../../search/runtime-search-sorting.js";
import { createRuntimeSearchDependencies } from "./runtime-search-dependencies.js";
import { Pf2eRecordCatalog } from "./record-catalog.js";
import { Pf2eSearchWindowStore } from "./search-window-store.js";

export class Pf2eSearchBackendService {
  private readonly searchWindows: Pf2eSearchWindowStore;

  constructor(
    private readonly db: DatabaseSync,
    private readonly catalog: Pf2eRecordCatalog,
    private readonly embeddingProvider: EmbeddingProvider,
    private readonly rankingConfigStore: RankingConfigStore | null,
  ) {
    this.searchWindows = new Pf2eSearchWindowStore((recordKeys) => this.catalog.getRecordsByKeys(recordKeys));
  }

  listFilterValues(query: FilterValueQuery): FilterValueResult {
    const normalizedFilters = this.normalizeRequest({
      mode: "browse",
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
  }

  async discoverFilterValues(
    query: FilterValueQuery,
    request: Readonly<SearchRequest>,
  ): Promise<FilterValueResult> {
    const normalizedFilters = this.normalizeRequest(request);
    validateSearchFilters(normalizedFilters, request.mode === "browse" ? "list" : "search");
    const recordKeys = await this.resolveDiscoveryRecordKeys(request, normalizedFilters);
    return this.catalog.listFilterValues(query, normalizedFilters, {
      ...(recordKeys ? { recordKeys } : {}),
    });
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

    if (normalizedRequest.mode === "browse") {
      const normalizedFilters = this.normalizeExecutionFilters(executionFilters);
      validateSearchFilters(normalizedFilters, "list");
      return countStructuredSearchRuntime(normalizedFilters, runtime);
    }

    if (options.lexicalOnly && executionFilters.query?.trim()) {
      executionFilters.searchProfile = "lexical";
    }

    const normalizedFilters = this.normalizeExecutionFilters(executionFilters);
    validateSearchFilters(normalizedFilters, "search");

    if (normalizedRequest.mode === "lookup") {
      return countStructuredSearchRuntime(normalizedFilters, runtime);
    }

    return countSearchResultsRuntime(executionFilters, normalizedFilters, runtime);
  }

  async openSearchWindow(request: SearchRequest): Promise<SearchWindowPage> {
    const normalizedRequest = request;
    const runtime = this.runtimeSearchDependencies();
    const normalizedFilters = this.normalizeRequest(normalizedRequest);

    if (normalizedRequest.mode === "browse") {
      validateSearchFilters(normalizedFilters, "list");
      const sort =
        normalizedFilters.sort === "ranked" || !normalizedFilters.sort ? "alphabetical" : normalizedFilters.sort;
      const sortSeed = sort === "random" ? (normalizedFilters.sortSeed ?? 0) : null;
      const orderedRecordKeys =
        sort === "random"
          ? fetchCandidateRecordKeys(this.db, normalizedFilters).sort((left, right) => {
              const leftHash = hashRecordSortSeed(left, sortSeed ?? 0);
              const rightHash = hashRecordSortSeed(right, sortSeed ?? 0);
              return leftHash - rightHash || left.localeCompare(right);
            })
          : fetchCandidateRecordKeys(this.db, normalizedFilters, sort);
      const window = this.searchWindows.openWindow({
        kind: "recordKeys",
        mode: "structured",
        searchProfile: null,
        sort,
        sortSeed,
        total: orderedRecordKeys.length,
        orderedRecordKeys,
      });
      return this.searchWindows.readWindowPage(window.id, normalizedFilters.offset ?? 0, normalizedFilters.limit ?? 20);
    }

    validateSearchFilters(normalizedFilters, "search");
    const snapshot = await buildSearchWindowSnapshot(normalizedFilters, runtime);
    const window = this.searchWindows.openWindow({
      kind: "recordKeys",
      mode: snapshot.mode,
      searchProfile: snapshot.searchProfile,
      sort: snapshot.sort,
      sortSeed: normalizedFilters.sortSeed ?? null,
      total: snapshot.records.length,
      orderedRecordKeys: snapshot.records.map((record) => record.recordKey),
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
    return searchRuntime(executionFilters, normalizedFilters, this.runtimeSearchDependencies());
  }

  lookup(
    name: string,
    options: LookupOptions = {},
  ): { match: NormalizedRecord | null; alternatives: NormalizedRecord[] } {
    const filters = this.normalizeRequest({
      mode: "lookup",
      search: { query: name },
      filter: buildAllOfFilter([
        options.pack ? { kind: "pack", value: options.pack } : undefined,
        options.category ? buildScopeFilter(options.category, options.subcategory ?? null) : undefined,
      ]),
      limit: 5,
    });
    validateSearchFilters(filters, "search");
    return lookupRuntime(name, filters, this.runtimeSearchDependencies());
  }

  lookupMany(queries: LookupQuery[], options: { coreOnly?: boolean } = {}): LookupResult[] {
    return queries.map((query) => {
      const lookup = (() => {
        if (!options.coreOnly) {
          return this.lookup(query.name, query);
        }

        const results = this.searchStructured({
          mode: "lookup",
          search: { query: query.name },
          filter: buildAllOfFilter([
            query.pack ? { kind: "pack", value: query.pack } : undefined,
            query.category ? buildScopeFilter(query.category, query.subcategory ?? null) : undefined,
            {
              kind: "metadataPredicate",
              predicate: { field: "sourceCategory", op: "eq", value: "core" },
            },
          ]),
          limit: 5,
        }).records;
        return {
          match: results[0] ?? null,
          alternatives: results.slice(1),
        };
      })();

      return {
        query,
        match: lookup.match,
        alternatives: lookup.alternatives,
        matchType: getLookupMatchType(query.name, lookup.match),
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
    if (request.mode === "browse") {
      return undefined;
    }

    if (!normalizedFilters.query && !normalizedFilters.nameQuery && !normalizedFilters.excludeQuery) {
      return undefined;
    }

    const snapshot = await buildSearchWindowSnapshot(normalizedFilters, this.runtimeSearchDependencies());
    return snapshot.records.map((record) => record.recordKey);
  }

  private runtimeSearchDependencies() {
    return createRuntimeSearchDependencies({
      db: this.db,
      embeddingProvider: this.embeddingProvider,
      rankingConfigStore: this.rankingConfigStore,
      decorateRecord: (record) => this.catalog.decorateRecord(record),
      getAliases: (recordKey) => this.catalog.getAliases(recordKey),
      getRankingConfigStatus: () => this.catalog.getRankingConfigStatus(),
    });
  }
}
