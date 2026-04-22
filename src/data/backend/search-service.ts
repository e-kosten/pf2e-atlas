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
import type { SearchRequest } from "../../domain/search-request-types.js";
import { coerceSearchRequest } from "../../domain/search-request-compat.js";
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
    const normalizedFilters = this.normalizeExecutionFilters(query);
    validateSearchFilters(normalizedFilters, "list");
    return this.catalog.listFilterValues(query, normalizedFilters);
  }

  listRecords(request: SearchRequest): SearchResult {
    const normalizedFilters = this.normalizeRequest(coerceSearchRequest(request, "browse"));
    validateSearchFilters(normalizedFilters, "list");
    return listRecordsRuntime(normalizedFilters, this.runtimeSearchDependencies());
  }

  async countRecords(
    request: SearchRequest,
    options: { lexicalOnly?: boolean } = {},
  ): Promise<SearchCountResult> {
    const normalizedRequest = coerceSearchRequest(request, "search");
    const runtime = this.runtimeSearchDependencies();
    const executionFilters = this.compileRequest({
      ...normalizedRequest,
      offset: 0,
      limit: 1,
    });

    if (normalizedRequest.intent === "browse") {
      const normalizedFilters = this.normalizeExecutionFilters(executionFilters);
      validateSearchFilters(normalizedFilters, "list");
      return countStructuredSearchRuntime(normalizedFilters, runtime);
    }

    if (options.lexicalOnly && executionFilters.query?.trim()) {
      executionFilters.searchProfile = "lexical";
    }

    const normalizedFilters = this.normalizeExecutionFilters(executionFilters);
    validateSearchFilters(normalizedFilters, "search");

    if (normalizedRequest.intent === "lookup") {
      return countStructuredSearchRuntime(normalizedFilters, runtime);
    }

    return countSearchResultsRuntime(executionFilters, normalizedFilters, runtime);
  }

  async openSearchWindow(request: SearchRequest): Promise<SearchWindowPage> {
    const normalizedRequest = coerceSearchRequest(request, "search");
    const runtime = this.runtimeSearchDependencies();
    const normalizedFilters = this.normalizeRequest(normalizedRequest);

    if (normalizedRequest.intent === "browse") {
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
    const executionFilters = this.compileRequest(coerceSearchRequest(request, "search"));
    const normalizedFilters = this.normalizeExecutionFilters(executionFilters);
    validateSearchFilters(normalizedFilters, "search");
    return searchRuntime(executionFilters, normalizedFilters, this.runtimeSearchDependencies());
  }

  lookup(
    name: string,
    options: LookupOptions = {},
  ): { match: NormalizedRecord | null; alternatives: NormalizedRecord[] } {
    const filters = this.normalizeRequest({
      intent: "lookup",
      text: name,
      pack: options.pack,
      category: options.category,
      parts: options.subcategory
        ? [
            {
              kind: "subcategory",
              subcategory: options.subcategory,
            },
          ]
        : [],
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
          intent: "lookup",
          text: query.name,
          pack: query.pack,
          category: query.category,
          parts: [
            ...(query.subcategory
              ? [
                  {
                    kind: "subcategory" as const,
                    subcategory: query.subcategory,
                  },
                ]
              : []),
            {
              kind: "metadataPredicate" as const,
              predicate: { field: "sourceCategory", op: "eq", value: "core" },
            },
          ],
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
    const normalizedFilters = this.normalizeRequest(coerceSearchRequest(request, "search"));
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
