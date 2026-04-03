import { mkdir } from "node:fs/promises";
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
  SearchFilters,
  SearchResult,
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
import { buildIndex, computeSourceSignature, removeIndexFiles } from "./indexer.js";
import type { StageTiming } from "./index-types.js";
import {
  fetchCandidates,
  fetchLexicalRetrievalRows,
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

  if (mode === "structured" && filters.query) {
    throw new Error("query requires a themed search profile such as balanced or concept.");
  }

  if (context === "search" && !filters.query?.trim() && !filters.nameQuery?.trim() && !hasStructuredFilterSignal(filters)) {
    throw new Error("pf2e_search requires search text and/or at least one structured filter.");
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
    await removeIndexFiles(indexPath);
    const prepareOutputDurationMs = Date.now() - prepareOutputStartTime;

    const db = openDatabase(indexPath, {
      vectorExtensionLoader: options.vectorExtensionLoader,
    });
    options.progressLogger?.("Creating SQLite schema.");
    const schemaCreationStartTime = Date.now();
    createSchema(db, embeddingProvider.identity.dimensions);
    const schemaCreationDurationMs = Date.now() - schemaCreationStartTime;
    const { packs, warnings, recordCount, stageTimings } = await buildIndex(
      db,
      rootPath,
      manifestPath,
      embeddingProvider,
      sourceSignature,
      options.progressLogger,
      options.progressStatusLogger,
    );
    options.progressLogger?.(
      `Finished writing ${formatInteger(recordCount)} records across ${formatInteger(packs.length)} packs.`,
    );
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
      db,
      packs,
      [...embeddingRuntime.warnings, ...warnings, ...(options.rankingConfigStore?.warnings ?? [])],
      recordCount,
      indexPath,
      embeddingProvider,
      options.rankingConfigStore ?? null,
    );
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
    return listFilterValuesRuntime(this.db, query.field, normalizedFilters);
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

    return {
      ...filters,
      pack: pack?.name ?? filters.pack,
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
