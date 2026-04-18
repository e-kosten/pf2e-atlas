import { mkdir, rename } from "node:fs/promises";
import path from "node:path";
import { DatabaseSync } from "node:sqlite";

import {
  createEmbeddingProvider,
  EmbeddingProvider,
} from "../embeddings.js";
import {
  categorySupportsSubcategory,
  getSearchCategoryErrorMessage,
  getSearchSubcategoryErrorMessage,
  normalizeSearchCategory,
  normalizeSearchSubcategory,
} from "../domain/categories.js";
import { NormalizedSearchFilters } from "./service-types.js";
import { DEFAULT_RANKING_CONFIG, RankingConfigStore } from "../search/ranking-config.js";
import { normalizeMetadataFilterNode } from "../search/metadata-filters.js";
import {
  CollectRuleQuestionContextInput,
  CollectRuleQuestionContextResult,
  EmbeddingConfig,
  FilterValueResult,
  FilterValueQuery,
  LinkedRecordSummary,
  LookupOptions,
  LookupQuery,
  LookupResult,
  NormalizedRecord,
  PackInfo,
  RuleGraphCollectionResult,
  SearchCountResult,
  SearchFilters,
  SearchSort,
  SearchResult,
  SearchWindow,
  SearchWindowPage,
} from "../types.js";
import { normalizeSearchScope } from "../search/sql.js";
import { formatInteger } from "../shared/format.js";
import { fileExists } from "../shared/fs.js";
import {
  getLookupMatchType,
  rowToRecord,
  sqliteRowCount,
} from "./rows.js";
import {
  buildMissingIndexError,
  buildStaleIndexError,
  createSchema,
  defaultEmbeddingConfig,
  defaultIndexPath,
  getEmbeddingReuseInvalidReason,
  getIndexInvalidReason,
  loadAliasesByRecordKey,
  loadLegacyLinksByRecordKey,
  loadPacksFromIndex,
  openDatabase,
} from "./schema.js";
import {
  hasStructuredFilterSignal,
  resolveSearchMode,
} from "../search/ranking.js";
import { buildIndex, buildReusableEmbeddingLookup, computeSourceSignature, removeIndexFiles } from "./indexer.js";
import type { StageTiming } from "./index-types.js";
import {
  fetchCandidateCount,
  fetchCandidateRecordKeys,
  fetchCandidates,
  fetchLexicalRetrievalRows,
  fetchPagedCandidates,
  fetchRecordRow,
  fetchRecordRowsByKeys,
  fetchReferenceEdgeRows,
  fetchSemanticRetrievalRows,
} from "./record-queries.js";
import {
  collectRuleQuestionContext as collectRuleQuestionContextRuntime,
  getRuleGraph as getRuleGraphRuntime,
} from "./rule-runtime.js";
import {
  buildSearchWindowSnapshot,
  countSearchResults as countSearchResultsRuntime,
  countStructuredSearch as countStructuredSearchRuntime,
  listRecords as listRecordsRuntime,
  lookup as lookupRuntime,
  search as searchRuntime,
  searchStructured as searchStructuredRuntime,
} from "../search/runtime-search.js";
import {
  getPack as getPackRuntime,
  getSearchVocabulary as getSearchVocabularyRuntime,
  listFilterValues as listFilterValuesRuntime,
  listPacks as listPacksRuntime,
  SearchVocabularyResult,
} from "./vocabulary.js";

type LoadOptions = {
  indexPath?: string;
  embedding?: EmbeddingConfig;
  embeddingProviderFactory?: (
    config: EmbeddingConfig,
  ) => Promise<{ provider: EmbeddingProvider; warnings: string[] }>;
  rankingConfigStore?: RankingConfigStore;
  progressLogger?: (message: string) => void;
  progressStatusLogger?: (message: string) => void;
  reuseEmbeddings?: boolean;
  vectorExtensionLoader?: (db: DatabaseSync) => void;
};

function formatDurationMs(durationMs: number): string {
  const totalSeconds = Math.max(0, Math.round(durationMs / 1000));
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  if (hours > 0) {
    return `${hours}h ${minutes}m ${seconds}s`;
  }
  if (minutes > 0) {
    return `${minutes}m ${seconds}s`;
  }

  return `${seconds}s`;
}

async function moveIndexFiles(sourcePath: string, targetPath: string): Promise<void> {
  await rename(sourcePath, targetPath);

  for (const suffix of ["-wal", "-shm"]) {
    const sourceSidecar = `${sourcePath}${suffix}`;
    if (await fileExists(sourceSidecar)) {
      await rename(sourceSidecar, `${targetPath}${suffix}`);
    }
  }
}

function validateFilters(filters: NormalizedSearchFilters, context: "list" | "search"): void {
  const mode = resolveSearchMode(filters, context);

  if (context === "list" && filters.searchProfile) {
    throw new Error("searchProfile is only supported for pf2e_search.");
  }

  if (context === "list" && mode !== "structured") {
    throw new Error("List mode only supports structured retrieval.");
  }

  if (context === "list" && filters.query) {
    throw new Error("query is only supported for pf2e_search.");
  }

  if (context === "list" && filters.excludeQuery) {
    throw new Error("excludeQuery is only supported for pf2e_search.");
  }

  if (mode === "structured" && filters.query) {
    throw new Error("query requires a themed search profile such as balanced or concept.");
  }

  if (context === "search" && !filters.query?.trim() && !filters.nameQuery?.trim() && !hasStructuredFilterSignal(filters)) {
    throw new Error("pf2e_search requires search text and/or at least one structured filter.");
  }

  if (filters.linksTo !== undefined && filters.linksTo.length === 0) {
    throw new Error("linksTo must contain at least one record key.");
  }

  if (filters.excludeLinksTo !== undefined && filters.excludeLinksTo.length === 0) {
    throw new Error("excludeLinksTo must contain at least one record key.");
  }

  if (filters.linksToMode && (!filters.linksTo || filters.linksTo.length === 0)) {
    throw new Error("linksToMode requires linksTo.");
  }

  if (filters.scopes && filters.scopes.length > 0 && (filters.category || filters.subcategory)) {
    throw new Error("scopes can't be combined with top-level category or subcategory filters.");
  }

  if (filters.category && filters.subcategory && !categorySupportsSubcategory(filters.category, filters.subcategory)) {
    throw new Error(`Subcategory "${filters.subcategory}" does not belong to category "${filters.category}".`);
  }

  if (filters.scopes) {
    for (const scope of filters.scopes) {
      for (const subcategory of scope.subcategories ?? []) {
        if (!categorySupportsSubcategory(scope.category, subcategory)) {
          throw new Error(`Subcategory "${subcategory}" does not belong to category "${scope.category}".`);
        }
      }
    }
  }
}

function normalizeRecordKeyFilter(values: string[] | undefined): string[] | undefined {
  if (values === undefined) {
    return undefined;
  }

  const normalized = values
    .map((value) => value.trim())
    .filter((value) => value.length > 0);
  return [...new Set(normalized)];
}

type RuntimeSearchWindow =
  {
    id: string;
    kind: "recordKeys";
    mode: SearchWindow["mode"];
    searchProfile: SearchWindow["searchProfile"];
    sort: SearchSort;
    sortSeed: number | null;
    total: number;
    orderedRecordKeys: string[];
  };

const MAX_SEARCH_WINDOWS = 24;

export class Pf2eDataService {
  readonly packs: PackInfo[];
  readonly warnings: string[];

  private readonly db: DatabaseSync;
  private readonly embeddingProvider: EmbeddingProvider;
  private readonly indexPath: string;
  private readonly recordCount: number;
  private readonly rankingConfigStore: RankingConfigStore | null;
  private readonly aliasesByRecordKey: Map<string, string[]>;
  private readonly legacyLinksByRecordKey: Map<string, LinkedRecordSummary[]>;
  private readonly searchWindows: Map<string, RuntimeSearchWindow>;
  private searchWindowCounter: number;

  private constructor(
    db: DatabaseSync,
    packs: PackInfo[],
    warnings: string[],
    recordCount: number,
    indexPath: string,
    embeddingProvider: EmbeddingProvider,
    rankingConfigStore: RankingConfigStore | null,
  ) {
    this.db = db;
    this.packs = packs;
    this.warnings = warnings;
    this.recordCount = recordCount;
    this.indexPath = indexPath;
    this.embeddingProvider = embeddingProvider;
    this.rankingConfigStore = rankingConfigStore;
    this.aliasesByRecordKey = loadAliasesByRecordKey(db);
    this.legacyLinksByRecordKey = loadLegacyLinksByRecordKey(db);
    this.searchWindows = new Map();
    this.searchWindowCounter = 0;
  }

  static async load(rootPath: string, manifestPath: string, options: LoadOptions = {}): Promise<Pf2eDataService> {
    const indexPath = options.indexPath ?? defaultIndexPath(manifestPath);
    const embeddingConfig: EmbeddingConfig = options.embedding ?? defaultEmbeddingConfig(indexPath);
    const embeddingProviderFactory = options.embeddingProviderFactory ?? createEmbeddingProvider;
    const embeddingRuntime = await embeddingProviderFactory(embeddingConfig);
    const embeddingProvider = embeddingRuntime.provider;
    const sourceSignature = await computeSourceSignature(rootPath, manifestPath);
    if (!(await fileExists(indexPath))) {
      throw buildMissingIndexError(indexPath);
    }

    const existingDb = openDatabase(indexPath, {
      vectorExtensionLoader: options.vectorExtensionLoader,
    });
    const invalidReason = getIndexInvalidReason(existingDb, sourceSignature, embeddingProvider);
    if (invalidReason) {
      existingDb.close();
      throw buildStaleIndexError(indexPath, invalidReason);
    }

    const packs = loadPacksFromIndex(existingDb);
    const recordCount = sqliteRowCount(
      existingDb.prepare("SELECT COUNT(*) AS total FROM records").get() as Record<string, unknown> | undefined,
    );
    return new Pf2eDataService(
      existingDb,
      packs,
      [...embeddingRuntime.warnings, ...(options.rankingConfigStore?.warnings ?? [])],
      recordCount,
      indexPath,
      embeddingProvider,
      options.rankingConfigStore ?? null,
    );
  }

  static async rebuildIndex(rootPath: string, manifestPath: string, options: LoadOptions = {}): Promise<Pf2eDataService> {
    const rebuildStartTime = Date.now();
    const indexPath = options.indexPath ?? defaultIndexPath(manifestPath);
    const tempIndexPath = `${indexPath}.rebuild-${process.pid}-${Date.now()}`;
    const embeddingConfig: EmbeddingConfig = options.embedding ?? defaultEmbeddingConfig(indexPath);
    const embeddingProviderFactory = options.embeddingProviderFactory ?? createEmbeddingProvider;
    options.progressLogger?.("Loading the configured embedding provider.");
    const embeddingProviderLoadStartTime = Date.now();
    const embeddingRuntime = await embeddingProviderFactory(embeddingConfig);
    const embeddingProviderLoadDurationMs = Date.now() - embeddingProviderLoadStartTime;
    const embeddingProvider = embeddingRuntime.provider;
    options.progressLogger?.(
      `Embedding provider ready: ${embeddingProvider.identity.model} (${embeddingProvider.identity.dimensions} dimensions).`,
    );
    options.progressLogger?.("Computing the PF2E source signature.");
    const sourceSignatureStartTime = Date.now();
    const sourceSignature = await computeSourceSignature(rootPath, manifestPath);
    const sourceSignatureDurationMs = Date.now() - sourceSignatureStartTime;
    options.progressLogger?.(`Preparing index output at ${indexPath}.`);
    const prepareOutputStartTime = Date.now();
    await mkdir(path.dirname(indexPath), { recursive: true });
    await removeIndexFiles(tempIndexPath);

    let previousDb: DatabaseSync | null = null;
    let reusableEmbeddingLookup = null;
    if (options.reuseEmbeddings) {
      if (await fileExists(indexPath)) {
        try {
          previousDb = openDatabase(indexPath, {
            vectorExtensionLoader: options.vectorExtensionLoader,
          });
          const reuseInvalidReason = getEmbeddingReuseInvalidReason(previousDb, embeddingProvider);
          if (reuseInvalidReason) {
            options.progressLogger?.(`Embedding reuse unavailable: ${reuseInvalidReason}. Regenerating all canonical embeddings.`);
            previousDb.close();
            previousDb = null;
          } else {
            options.progressLogger?.("Reusing unchanged canonical embeddings from the existing index when semantic inputs match.");
            reusableEmbeddingLookup = buildReusableEmbeddingLookup(previousDb);
          }
        } catch (error) {
          options.progressLogger?.(`Embedding reuse unavailable: ${(error as Error).message}. Regenerating all canonical embeddings.`);
          previousDb?.close();
          previousDb = null;
        }
      } else {
        options.progressLogger?.("Embedding reuse unavailable: no existing index found. Regenerating all canonical embeddings.");
      }
    }

    const prepareOutputDurationMs = Date.now() - prepareOutputStartTime;

    let tempDb: DatabaseSync | null = null;
    let finalDb: DatabaseSync | null = null;

    try {
      tempDb = openDatabase(tempIndexPath, {
        vectorExtensionLoader: options.vectorExtensionLoader,
      });
      options.progressLogger?.("Creating SQLite schema.");
      const schemaCreationStartTime = Date.now();
      createSchema(tempDb, embeddingProvider.identity.dimensions);
      const schemaCreationDurationMs = Date.now() - schemaCreationStartTime;
      const {
        packs,
        warnings,
        recordCount,
        stageTimings,
      } = await buildIndex(
        tempDb,
        rootPath,
        manifestPath,
        embeddingProvider,
        sourceSignature,
        options.progressLogger,
        options.progressStatusLogger,
        reusableEmbeddingLookup,
      );
      options.progressLogger?.(
        `Finished writing ${formatInteger(recordCount)} records across ${formatInteger(packs.length)} packs.`,
      );
      tempDb.close();
      tempDb = null;
      previousDb?.close();
      previousDb = null;
      await removeIndexFiles(indexPath);
      await moveIndexFiles(tempIndexPath, indexPath);

      finalDb = openDatabase(indexPath, {
        vectorExtensionLoader: options.vectorExtensionLoader,
      });
      const rebuildDurationMs = Date.now() - rebuildStartTime;
      const summaryTimings: StageTiming[] = [
        { label: "Embedding provider load", durationMs: embeddingProviderLoadDurationMs },
        { label: "Source signature", durationMs: sourceSignatureDurationMs },
        { label: "Prepare index output", durationMs: prepareOutputDurationMs },
        { label: "Create SQLite schema", durationMs: schemaCreationDurationMs },
        ...stageTimings,
        { label: "Total rebuild time", durationMs: rebuildDurationMs },
      ];
      options.progressLogger?.("Index rebuild stage timings:");
      for (const timing of summaryTimings) {
        options.progressLogger?.(`- ${timing.label}: ${formatDurationMs(timing.durationMs)}`);
      }
      return new Pf2eDataService(
        finalDb,
        packs,
        [...embeddingRuntime.warnings, ...warnings, ...(options.rankingConfigStore?.warnings ?? [])],
        recordCount,
        indexPath,
        embeddingProvider,
        options.rankingConfigStore ?? null,
      );
    } catch (error) {
      finalDb?.close();
      tempDb?.close();
      previousDb?.close();
      await removeIndexFiles(tempIndexPath);
      throw error;
    }
  }

  getStats(): { packCount: number; recordCount: number } {
    return {
      packCount: this.packs.length,
      recordCount: this.recordCount,
    };
  }

  getSearchVocabulary(options: { traitLimitPerCategory?: number } = {}): SearchVocabularyResult {
    return getSearchVocabularyRuntime(this.db, options);
  }

  getRankingConfigStatus() {
    return this.rankingConfigStore?.getStatus() ?? {
      path: "<defaults>",
      source: "default" as const,
      revision: 1,
      loadedAt: new Date(0).toISOString(),
      lastError: null,
    };
  }

  getPack(packValue: string): PackInfo | undefined {
    return getPackRuntime(this.packs, packValue);
  }

  listPacks(): PackInfo[] {
    return listPacksRuntime(this.packs);
  }

  listFilterValues(query: FilterValueQuery): FilterValueResult {
    const normalizedFilters = this.normalizeSearchFilters(query);
    validateFilters(normalizedFilters, "list");
    return listFilterValuesRuntime(this.db, query, normalizedFilters);
  }

  close(): void {
    this.rankingConfigStore?.close();
    this.db.close();
  }

  private decorateRecord(record: NormalizedRecord): NormalizedRecord {
    return {
      ...record,
      aliases: this.aliasesByRecordKey.get(record.recordKey) ?? [],
      legacyRecordLinks: this.legacyLinksByRecordKey.get(record.recordKey) ?? [],
    };
  }

  private normalizeSearchFilters(filters: SearchFilters): NormalizedSearchFilters {
    const normalizedCategory = filters.category !== undefined
      ? normalizeSearchCategory(filters.category)
      : null;
    if (filters.category !== undefined && !normalizedCategory) {
      throw new Error(getSearchCategoryErrorMessage(String(filters.category)));
    }

    const normalizedSubcategory = filters.subcategory !== undefined
      ? normalizeSearchSubcategory(filters.subcategory)
      : null;
    if (filters.subcategory !== undefined && !normalizedSubcategory) {
      throw new Error(getSearchSubcategoryErrorMessage(String(filters.subcategory)));
    }

    const normalizedScopes = filters.scopes?.map((scope) => normalizeSearchScope(scope));
    const pack = filters.pack ? this.getPack(filters.pack) : undefined;
    const linksTo = normalizeRecordKeyFilter(filters.linksTo);
    const excludeLinksTo = normalizeRecordKeyFilter(filters.excludeLinksTo);

    return {
      ...filters,
      pack: pack?.name ?? filters.pack,
      linksTo,
      excludeLinksTo,
      category: normalizedCategory ?? undefined,
      subcategory: normalizedSubcategory ?? undefined,
      metadata: filters.metadata ? normalizeMetadataFilterNode(filters.metadata) : undefined,
      scopes: normalizedScopes,
    };
  }

  getRecord(recordKeyOrPack: string, maybeId?: string): NormalizedRecord | undefined {
    const row = fetchRecordRow(this.db, recordKeyOrPack, maybeId);

    if (!row) {
      return undefined;
    }

    return this.decorateRecord(rowToRecord(row));
  }

  getRecordsByKeys(recordKeys: string[]): NormalizedRecord[] {
    const rows = fetchRecordRowsByKeys(this.db, [...new Set(recordKeys)]);
    const byKey = new Map(rows.map((row) => [row.recordKey, this.decorateRecord(rowToRecord(row))]));
    return [...new Set(recordKeys)].map((recordKey) => byKey.get(recordKey)).filter((record): record is NormalizedRecord => Boolean(record));
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

  getRuleGraph(
    recordKeys: string[],
    {
      coreOnly,
      includeOutgoing,
      includeBacklinks,
      maxOutgoingPerPrimary,
      maxBacklinksPerPrimary,
    }: {
      coreOnly?: boolean;
      includeOutgoing?: boolean;
      includeBacklinks?: boolean;
      maxOutgoingPerPrimary?: number;
      maxBacklinksPerPrimary?: number;
    } = {},
  ): RuleGraphCollectionResult {
    return getRuleGraphRuntime(recordKeys, {
      coreOnly,
      includeOutgoing,
      includeBacklinks,
      maxOutgoingPerPrimary,
      maxBacklinksPerPrimary,
    }, {
      fetchReferenceEdgeRows: (direction, keys, options) => fetchReferenceEdgeRows(this.db, direction, keys, options),
      getRecordsByKeys: (keys) => this.getRecordsByKeys(keys),
      lookupMany: (queries, options) => this.lookupMany(queries, options),
    });
  }

  collectRuleQuestionContext(input: CollectRuleQuestionContextInput): CollectRuleQuestionContextResult {
    return collectRuleQuestionContextRuntime(input, {
      fetchReferenceEdgeRows: (direction, keys, options) => fetchReferenceEdgeRows(this.db, direction, keys, options),
      getRecordsByKeys: (keys) => this.getRecordsByKeys(keys),
      lookupMany: (queries, options) => this.lookupMany(queries, options),
    });
  }

  private runtimeSearchDependencies() {
    return {
      embeddingProvider: this.embeddingProvider,
      rankingConfig: this.rankingConfigStore?.getConfig() ?? DEFAULT_RANKING_CONFIG,
      rankingConfigStatus: this.getRankingConfigStatus(),
      decorateRecord: (record: NormalizedRecord) => this.decorateRecord(record),
      fetchCandidateCount: (filters: NormalizedSearchFilters, options: { recordKeys?: string[] } = {}) =>
        fetchCandidateCount(this.db, filters, options),
      fetchPagedCandidates: (filters: NormalizedSearchFilters, sort: SearchSort, offset: number, limit: number) =>
        fetchPagedCandidates(this.db, filters, sort, offset, limit),
      getAliases: (recordKey: string) => this.aliasesByRecordKey.get(recordKey) ?? [],
      fetchCandidates: (
        filters: NormalizedSearchFilters,
        includeSearchText = false,
        includeEmbedding = false,
        options: { recordKeys?: string[] } = {},
      ) => fetchCandidates(this.db, filters, includeSearchText, includeEmbedding, options),
      fetchLexicalRetrievalRows: (filters: NormalizedSearchFilters, ftsQuery: string, limit: number) =>
        fetchLexicalRetrievalRows(this.db, filters, ftsQuery, limit),
      fetchSemanticRetrievalRows: (filters: NormalizedSearchFilters, queryVector: Float32Array, limit: number) =>
        fetchSemanticRetrievalRows(this.db, filters, queryVector, limit),
    };
  }

  private createSearchWindowId(): string {
    this.searchWindowCounter += 1;
    return `search-window-${this.searchWindowCounter}`;
  }

  private rememberSearchWindow(window: RuntimeSearchWindow): RuntimeSearchWindow {
    this.searchWindows.set(window.id, window);
    while (this.searchWindows.size > MAX_SEARCH_WINDOWS) {
      const oldestId = this.searchWindows.keys().next().value as string | undefined;
      if (!oldestId) {
        break;
      }
      this.searchWindows.delete(oldestId);
    }
    return window;
  }

  private createSearchWindowPage(
    window: RuntimeSearchWindow,
    offset: number,
    limit: number,
    records: NormalizedRecord[],
  ): SearchWindowPage {
    const hasMore = offset + records.length < window.total;
    return {
      id: window.id,
      searchProfile: window.searchProfile,
      mode: window.mode,
      sort: window.sort,
      sortSeed: window.sortSeed,
      total: window.total,
      offset,
      limit,
      hasMore,
      nextOffset: hasMore ? offset + records.length : null,
      records,
    };
  }

  private readSearchWindowRecords(window: RuntimeSearchWindow, offset: number, limit: number): SearchWindowPage {
    const recordKeys = window.orderedRecordKeys.slice(offset, offset + limit);
    return this.createSearchWindowPage(window, offset, limit, this.getRecordsByKeys(recordKeys));
  }

  private hashSearchWindowKey(recordKey: string, seed: number): number {
    let hash = seed | 0;
    for (let index = 0; index < recordKey.length; index += 1) {
      hash = Math.imul(hash ^ recordKey.charCodeAt(index), 16777619);
    }
    return hash >>> 0;
  }

  private searchStructured(filters: SearchFilters): SearchResult {
    const normalizedFilters = this.normalizeSearchFilters(filters);
    validateFilters(normalizedFilters, "search");
    return searchStructuredRuntime(normalizedFilters, this.runtimeSearchDependencies());
  }

  listRecords(filters: SearchFilters): SearchResult {
    const normalizedFilters = this.normalizeSearchFilters(filters);
    validateFilters(normalizedFilters, "list");
    return listRecordsRuntime(normalizedFilters, this.runtimeSearchDependencies());
  }

  async countRecords(
    filters: SearchFilters,
    options: { mode?: "browse" | "search" | "lookup"; lexicalOnly?: boolean } = {},
  ): Promise<SearchCountResult> {
    const mode = options.mode ?? "search";

    if (mode === "browse") {
      const normalizedFilters = this.normalizeSearchFilters({
        ...filters,
        offset: 0,
        limit: 1,
      });
      validateFilters(normalizedFilters, "list");
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

    const normalizedFilters = this.normalizeSearchFilters(searchFilters);
    validateFilters(normalizedFilters, "search");

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
    const normalizedFilters = this.normalizeSearchFilters(filters);

    if (mode === "browse") {
      validateFilters(normalizedFilters, "list");
      const sort = normalizedFilters.sort === "ranked" || !normalizedFilters.sort
        ? "alphabetical"
        : normalizedFilters.sort;
      const sortSeed = sort === "random" ? (normalizedFilters.sortSeed ?? 0) : null;
      const orderedRecordKeys = sort === "random"
        ? fetchCandidateRecordKeys(this.db, normalizedFilters)
          .sort((left, right) => {
            const leftHash = this.hashSearchWindowKey(left, sortSeed ?? 0);
            const rightHash = this.hashSearchWindowKey(right, sortSeed ?? 0);
            return leftHash - rightHash || left.localeCompare(right);
          })
        : fetchCandidateRecordKeys(this.db, normalizedFilters, sort);
      const window = this.rememberSearchWindow({
        id: this.createSearchWindowId(),
        kind: "recordKeys",
        mode: "structured",
        searchProfile: null,
        sort,
        sortSeed,
        total: orderedRecordKeys.length,
        orderedRecordKeys,
      });
      return this.readSearchWindowRecords(window, normalizedFilters.offset ?? 0, normalizedFilters.limit ?? 20);
    }

    validateFilters(normalizedFilters, "search");
    const snapshot = await buildSearchWindowSnapshot(filters, normalizedFilters, runtime);
    const window = this.rememberSearchWindow({
      id: this.createSearchWindowId(),
      kind: "recordKeys",
      mode: snapshot.mode,
      searchProfile: snapshot.searchProfile,
      sort: snapshot.sort,
      sortSeed: normalizedFilters.sortSeed ?? null,
      total: snapshot.records.length,
      orderedRecordKeys: snapshot.records.map((record) => record.recordKey),
    });
    return this.readSearchWindowRecords(window, normalizedFilters.offset ?? 0, normalizedFilters.limit ?? 20);
  }

  readSearchWindowPage(windowId: string, offset: number, limit: number): SearchWindowPage {
    const window = this.searchWindows.get(windowId);
    if (!window) {
      throw new Error(`Search window "${windowId}" is no longer available.`);
    }
    return this.readSearchWindowRecords(window, offset, limit);
  }

  closeSearchWindow(windowId: string): void {
    this.searchWindows.delete(windowId);
  }

  async search(filters: SearchFilters): Promise<SearchResult> {
    const normalizedFilters = this.normalizeSearchFilters(filters);
    validateFilters(normalizedFilters, "search");
    return searchRuntime(filters, normalizedFilters, this.runtimeSearchDependencies());
  }

  lookup(name: string, options: LookupOptions = {}): { match: NormalizedRecord | null; alternatives: NormalizedRecord[] } {
    const filters = this.normalizeSearchFilters({
      nameQuery: name,
      pack: options.pack,
      category: options.category,
      subcategory: options.subcategory,
      limit: 5,
    });
    validateFilters(filters, "search");
    return lookupRuntime(name, filters, this.runtimeSearchDependencies());
  }
}
