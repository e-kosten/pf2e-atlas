import type { DatabaseSync } from "node:sqlite";

import type {
  FilterValueQuery,
  FilterValueResult,
  LookupOptions,
  LookupQuery,
  LookupResult,
  NormalizedRecord,
  SearchCountResult,
  SearchFilters,
  SearchResult,
  SearchWindowPage,
} from "../../domain/index.js";
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
import { fetchCandidateRecordKeys } from "../record-queries.js";
import { getLookupMatchType } from "../rows.js";
import { createRuntimeSearchDependencies } from "./runtime-search-dependencies.js";
import { normalizeSearchFilters, validateSearchFilters } from "./filter-normalization.js";
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
    const normalizedFilters = this.normalizeFilters(query);
    validateSearchFilters(normalizedFilters, "list");
    return this.catalog.listFilterValues(query, normalizedFilters);
  }

  listRecords(filters: SearchFilters): SearchResult {
    const normalizedFilters = this.normalizeFilters(filters);
    validateSearchFilters(normalizedFilters, "list");
    return listRecordsRuntime(normalizedFilters, this.runtimeSearchDependencies());
  }

  async countRecords(
    filters: SearchFilters,
    options: { mode?: "browse" | "search" | "lookup"; lexicalOnly?: boolean } = {},
  ): Promise<SearchCountResult> {
    const mode = options.mode ?? "search";

    if (mode === "browse") {
      const normalizedFilters = this.normalizeFilters({
        ...filters,
        offset: 0,
        limit: 1,
      });
      validateSearchFilters(normalizedFilters, "list");
      return countStructuredSearchRuntime(normalizedFilters, this.runtimeSearchDependencies());
    }

    const searchFilters: SearchFilters = {
      ...filters,
      offset: 0,
      limit: 1,
    };
    if (options.lexicalOnly && searchFilters.query?.trim()) {
      searchFilters.searchProfile = "lexical";
    }

    const normalizedFilters = this.normalizeFilters(searchFilters);
    validateSearchFilters(normalizedFilters, "search");

    if (mode === "lookup") {
      return countStructuredSearchRuntime(normalizedFilters, this.runtimeSearchDependencies());
    }

    return countSearchResultsRuntime(searchFilters, normalizedFilters, this.runtimeSearchDependencies());
  }

  async openSearchWindow(
    filters: SearchFilters,
    options: { mode?: "browse" | "search" | "lookup" } = {},
  ): Promise<SearchWindowPage> {
    const mode = options.mode ?? "search";
    const runtime = this.runtimeSearchDependencies();
    const normalizedFilters = this.normalizeFilters(filters);

    if (mode === "browse") {
      validateSearchFilters(normalizedFilters, "list");
      const sort =
        normalizedFilters.sort === "ranked" || !normalizedFilters.sort ? "alphabetical" : normalizedFilters.sort;
      const sortSeed = sort === "random" ? (normalizedFilters.sortSeed ?? 0) : null;
      const orderedRecordKeys =
        sort === "random"
          ? fetchCandidateRecordKeys(this.db, normalizedFilters).sort((left, right) => {
              const leftHash = this.hashSearchWindowKey(left, sortSeed ?? 0);
              const rightHash = this.hashSearchWindowKey(right, sortSeed ?? 0);
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

  async search(filters: SearchFilters): Promise<SearchResult> {
    const normalizedFilters = this.normalizeFilters(filters);
    validateSearchFilters(normalizedFilters, "search");
    return searchRuntime(filters, normalizedFilters, this.runtimeSearchDependencies());
  }

  lookup(
    name: string,
    options: LookupOptions = {},
  ): { match: NormalizedRecord | null; alternatives: NormalizedRecord[] } {
    const filters = this.normalizeFilters({
      nameQuery: name,
      pack: options.pack,
      category: options.category,
      subcategory: options.subcategory,
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
          nameQuery: query.name,
          pack: query.pack,
          category: query.category,
          subcategory: query.subcategory,
          metadata: { field: "sourceCategory", op: "eq", value: "core" },
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

  private searchStructured(filters: SearchFilters): SearchResult {
    const normalizedFilters = this.normalizeFilters(filters);
    validateSearchFilters(normalizedFilters, "search");
    return searchStructuredRuntime(normalizedFilters, this.runtimeSearchDependencies());
  }

  private normalizeFilters(filters: SearchFilters) {
    return normalizeSearchFilters(filters, (packValue) => this.catalog.getPack(packValue)?.name);
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

  private hashSearchWindowKey(recordKey: string, seed: number): number {
    let hash = seed | 0;
    for (let index = 0; index < recordKey.length; index += 1) {
      hash = Math.imul(hash ^ recordKey.charCodeAt(index), 16777619);
    }
    return hash >>> 0;
  }
}
