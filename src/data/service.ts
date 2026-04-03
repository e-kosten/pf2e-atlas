import { mkdir, stat } from "node:fs/promises";
import path from "node:path";
import { DatabaseSync } from "node:sqlite";

import {
  createEmbeddingProvider,
  EmbeddingProvider,
} from "../embeddings.js";
import { DERIVED_TAG_CATALOG } from "../tags/index.js";
import {
  categorySupportsSubcategory,
  getSearchCategoryErrorMessage,
  getSearchSubcategoryErrorMessage,
  normalizeSearchCategory,
  normalizeSearchSubcategory,
} from "../domain/categories.js";
import { NormalizedSearchFilters, NormalizedSearchScope, SqlValue } from "./service-types.js";
import { DEFAULT_RANKING_CONFIG, RankingConfig, RankingConfigStore } from "../search/ranking-config.js";
import { normalizeMetadataFilterNode } from "../search/metadata-filters.js";
import {
  CollectRuleQuestionContextInput,
  CollectRuleQuestionContextResult,
  DerivedTagCatalogEntry,
  EmbeddingConfig,
  FilterValueField,
  FilterValueResult,
  FilterValueQuery,
  LinkedRecordSummary,
  LookupOptions,
  LookupQuery,
  LookupResult,
  NormalizedRecord,
  PackInfo,
  PackManifestEntry,
  RuleGraphCollectionResult,
  RuleGraphResult,
  RuleReferenceEdge,
  SearchCategory,
  SearchExplainResult,
  SearchFilters,
  SearchRecordExplanation,
  SearchResult,
  SearchSubcategory,
  SourceCategory,
} from "../types.js";
import {
  buildCandidateQuery,
  buildFilterValueQuery,
  buildLexicalRetrievalQuery,
  buildSemanticRetrievalQuery,
  normalizeSearchScope,
  recordMatchesFilters,
  semanticQueryLimit,
} from "../search/sql.js";
import {
  bigramDice,
  clampLimit,
  clampOffset,
  normalizeText,
} from "../utils.js";
import { buildLiteralQueryWeights, buildSearchQueryAnalysis } from "../search-query-analysis.js";
import {
  backlinkTypeRank,
  buildPlaceholders,
  CandidateRow,
  edgeRowToReferenceEdge,
  extractQuestionRuleNames,
  getLookupMatchType,
  ReferenceEdgeRow,
  rowToRecord,
  sourceCategoryRank,
  sqliteRowCount,
  ValueCountRow,
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
  buildFusionConfigSummary,
  buildLexicalSignal,
  buildNormalizedRankScoreMap,
  buildRankMap,
  buildRerankAdjustments,
  compareOptionalRanks,
  computeWeightedRrfScore,
  hasStructuredFilterSignal,
  LexicalRetrievalRow,
  SemanticRetrievalRow,
  packQualityScore,
  rarityPreferenceScore,
  resolveHybridFusionProfile,
  resolveSearchMode,
  resolveSearchProfile,
  sourcePenaltyScore,
  sourceQualityScore,
  sumRerankAdjustments,
} from "../search/ranking.js";
import { buildIndex, computeSourceSignature, removeIndexFiles } from "./indexer.js";
import type { StageTiming } from "./index-types.js";

const LOOKUP_LEXICAL_TOP_K = 100;

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

const INTEGER_FORMATTER = new Intl.NumberFormat("en-US");

function encodeVector(vector: Float32Array): Buffer {
  return Buffer.from(vector.buffer.slice(vector.byteOffset, vector.byteOffset + vector.byteLength));
}

function decodeVector(blob: Uint8Array | null | undefined): Float32Array {
  if (!blob || blob.byteLength === 0) {
    return new Float32Array(0);
  }

  const copy = Uint8Array.from(blob);
  return new Float32Array(copy.buffer);
}

function cosineSimilarity(left: Float32Array, right: Float32Array): number {
  if (left.length === 0 || right.length === 0 || left.length !== right.length) {
    return 0;
  }

  let total = 0;
  for (let index = 0; index < left.length; index += 1) {
    total += left[index]! * right[index]!;
  }

  return total;
}

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

function getPackAliasValues(pack: PackInfo, value: string): boolean {
  const normalized = normalizeText(value);
  return normalized === normalizeText(pack.name) || normalized === normalizeText(pack.label);
}

function scoreNameCandidate(query: string, normalizedName: string): number {
  const normalizedQuery = normalizeText(query);
  if (!normalizedQuery) {
    return 0.5;
  }

  if (normalizedName === normalizedQuery) {
    return 1;
  }

  if (normalizedName.startsWith(normalizedQuery)) {
    return 0.95;
  }

  if (normalizedName.includes(normalizedQuery)) {
    return 0.9;
  }

  const queryTokens = normalizedQuery.split(" ").filter(Boolean);
  const nameTokens = new Set(normalizedName.split(" ").filter(Boolean));
  const overlap = queryTokens.filter((token) => nameTokens.has(token)).length;
  const tokenScore = queryTokens.length > 0 ? overlap / queryTokens.length : 0;
  const dice = bigramDice(normalizedQuery, normalizedName);
  return Math.max(tokenScore * 0.8, dice * 0.75);
}

function nameScore(query: string, record: NormalizedRecord, aliases: string[] = []): number {
  let best = scoreNameCandidate(query, record.normalizedName);
  for (const alias of aliases) {
    best = Math.max(best, scoreNameCandidate(query, normalizeText(alias)));
  }
  return best;
}

function sortRecords(left: NormalizedRecord, right: NormalizedRecord): number {
  return left.name.localeCompare(right.name) || left.packLabel.localeCompare(right.packLabel) || left.id.localeCompare(right.id);
}

function queryTextScore(query: string, haystack: string): number {
  const normalizedQuery = normalizeText(query);
  if (!normalizedQuery) {
    return 0;
  }

  const queryTokens = normalizedQuery.split(" ").filter(Boolean);
  const targetTokens = new Set(normalizeText(haystack).split(" ").filter(Boolean));
  if (queryTokens.length === 0 || targetTokens.size === 0) {
    return 0;
  }

  const overlap = queryTokens.filter((token) => targetTokens.has(token)).length;
  return overlap / queryTokens.length;
}

function buildFtsQuery(query: string): string | null {
  const tokens = normalizeText(query).split(" ").filter(Boolean);
  if (tokens.length === 0) {
    return null;
  }

  return tokens.map((token) => `"${token}"*`).join(" OR ");
}

function formatInteger(value: number): string {
  return INTEGER_FORMATTER.format(value);
}

async function fileExists(targetPath: string): Promise<boolean> {
  try {
    await stat(targetPath);
    return true;
  } catch {
    return false;
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

  getSearchVocabulary(options: { traitLimitPerCategory?: number } = {}): {
    categories: Array<{ value: SearchCategory; count: number }>;
    subcategories: Array<{ value: string; count: number }>;
    rarities: Array<{ value: string; count: number }>;
    sizes: Array<{ value: string; count: number }>;
    traditions: Array<{ value: string; count: number }>;
    spellKinds: Array<{ value: string; count: number }>;
    sourceCategories: Array<{ value: SourceCategory; count: number }>;
    commonTraitsByCategory: Array<{ category: SearchCategory; traits: Array<{ value: string; count: number }> }>;
    commonDerivedTagsByCategory: Array<{ category: SearchCategory; tags: Array<{ value: string; count: number }> }>;
    derivedTagCatalog: DerivedTagCatalogEntry[];
  } {
    const traitLimit = Math.max(3, Math.min(options.traitLimitPerCategory ?? 12, 25));
    const categories = this.db
      .prepare(
        `
          SELECT r.category AS value, COUNT(*) AS count
          FROM records r
          WHERE r.is_search_canonical = 1
          GROUP BY r.category
          ORDER BY count DESC, value ASC
        `,
      )
      .all() as Array<{ value: SearchCategory; count: number }>;
    const subcategories = this.db
      .prepare(
        `
          SELECT r.subcategory AS value, COUNT(*) AS count
          FROM records r
          WHERE r.is_search_canonical = 1 AND r.subcategory IS NOT NULL AND r.subcategory <> ''
          GROUP BY r.subcategory
          ORDER BY count DESC, value ASC
        `,
      )
      .all() as Array<{ value: string; count: number }>;
    const sourceCategories = this.db
      .prepare(
        `
          SELECT r.source_category AS value, COUNT(*) AS count
          FROM records r
          WHERE r.is_search_canonical = 1
          GROUP BY r.source_category
          ORDER BY count DESC, value ASC
        `,
      )
      .all() as Array<{ value: SourceCategory; count: number }>;
    const rarities = this.db
      .prepare(
        `
          SELECT r.rarity AS value, COUNT(*) AS count
          FROM records r
          WHERE r.is_search_canonical = 1 AND r.rarity IS NOT NULL AND r.rarity <> ''
          GROUP BY r.rarity
          ORDER BY count DESC, value ASC
        `,
      )
      .all() as Array<{ value: string; count: number }>;
    const sizes = this.db
      .prepare(
        `
          SELECT a.size AS value, COUNT(*) AS count
          FROM actor_records a
          JOIN records r ON r.record_key = a.record_key
          WHERE r.is_search_canonical = 1 AND a.size IS NOT NULL AND a.size <> ''
          GROUP BY a.size
          ORDER BY count DESC, value ASC
        `,
      )
      .all() as Array<{ value: string; count: number }>;
    const categoryTraitRows = this.db
      .prepare(
        `
          SELECT r.category AS category, rt.trait AS value, COUNT(*) AS count
          FROM record_traits rt
          JOIN records r ON r.record_key = rt.record_key
          WHERE r.is_search_canonical = 1
          GROUP BY r.category, rt.trait
          ORDER BY r.category ASC, count DESC, value ASC
        `,
      )
      .all() as Array<{ category: SearchCategory; value: string; count: number }>;
    const commonTraitsByCategory = (() => {
      const grouped = new Map<SearchCategory, Array<{ value: string; count: number }>>();
      for (const row of categoryTraitRows) {
        const bucket = grouped.get(row.category) ?? [];
        if (bucket.length < traitLimit) {
          bucket.push({ value: row.value, count: row.count });
          grouped.set(row.category, bucket);
        }
      }
      return [...grouped.entries()]
        .sort((left, right) => left[0].localeCompare(right[0]))
        .map(([category, traits]) => ({ category, traits }));
    })();
    const categoryDerivedTagRows = this.db
      .prepare(
        `
          SELECT r.category AS category, rdt.tag AS value, COUNT(*) AS count
          FROM record_derived_tags rdt
          JOIN records r ON r.record_key = rdt.record_key
          WHERE r.is_search_canonical = 1
          GROUP BY r.category, rdt.tag
          ORDER BY r.category ASC, count DESC, value ASC
        `,
      )
      .all() as Array<{ category: SearchCategory; value: string; count: number }>;
    const commonDerivedTagsByCategory = (() => {
      const grouped = new Map<SearchCategory, Array<{ value: string; count: number }>>();
      for (const row of categoryDerivedTagRows) {
        const bucket = grouped.get(row.category) ?? [];
        if (bucket.length < traitLimit) {
          bucket.push({ value: row.value, count: row.count });
          grouped.set(row.category, bucket);
        }
      }
      return [...grouped.entries()]
        .sort((left, right) => left[0].localeCompare(right[0]))
        .map(([category, tags]) => ({ category, tags }));
    })();
    const traditionCounts = new Map<string, number>();
    const traditionRows = this.db
      .prepare(`
        SELECT s.traditions_json AS traditionsJson
        FROM spell_records s
        JOIN records r ON r.record_key = s.record_key
        WHERE r.is_search_canonical = 1
      `)
      .all() as Array<{ traditionsJson: string }>;
    for (const row of traditionRows) {
      const traditions = JSON.parse(row.traditionsJson) as string[];
      for (const tradition of traditions) {
        const normalized = normalizeText(tradition);
        if (!normalized) {
          continue;
        }

        traditionCounts.set(normalized, (traditionCounts.get(normalized) ?? 0) + 1);
      }
    }
    const traditions = [...traditionCounts.entries()]
      .sort((left, right) => right[1] - left[1] || left[0].localeCompare(right[0]))
      .map(([value, count]) => ({ value, count }));
    const spellKindCounts = new Map<string, number>();
    const spellKindRows = this.db
      .prepare(`
        SELECT s.spell_kinds_json AS spellKindsJson
        FROM spell_records s
        JOIN records r ON r.record_key = s.record_key
        WHERE r.is_search_canonical = 1
      `)
      .all() as Array<{ spellKindsJson: string }>;
    for (const row of spellKindRows) {
      const spellKinds = JSON.parse(row.spellKindsJson) as string[];
      for (const spellKind of spellKinds) {
        const normalized = normalizeText(spellKind);
        if (!normalized) {
          continue;
        }

        spellKindCounts.set(normalized, (spellKindCounts.get(normalized) ?? 0) + 1);
      }
    }
    const spellKinds = [...spellKindCounts.entries()]
      .sort((left, right) => right[1] - left[1] || left[0].localeCompare(right[0]))
      .map(([value, count]) => ({ value, count }));

    return {
      categories,
      subcategories,
      rarities,
      sizes,
      traditions,
      spellKinds,
      sourceCategories,
      commonTraitsByCategory,
      commonDerivedTagsByCategory,
      derivedTagCatalog: DERIVED_TAG_CATALOG,
    };
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
    return this.packs.find((pack) => getPackAliasValues(pack, packValue));
  }

  listPacks(): PackInfo[] {
    return this.packs;
  }

  listFilterValues(query: FilterValueQuery): FilterValueResult {
    const normalizedFilters = this.normalizeSearchFilters(query);
    validateFilters(normalizedFilters, "list");
    const { sql, params } = buildFilterValueQuery(query.field, normalizedFilters);
    const values = this.db.prepare(sql).all(...params) as ValueCountRow[];
    return {
      field: query.field,
      values,
    };
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

  private fetchCandidates(
    filters: NormalizedSearchFilters,
    includeSearchText = false,
    includeEmbedding = false,
    options: { recordKeys?: string[] } = {},
  ): CandidateRow[] {
    const { sql, params } = buildCandidateQuery(filters, includeSearchText, includeEmbedding, options);
    return this.db.prepare(sql).all(...params) as CandidateRow[];
  }

  private fetchLexicalRetrievalRows(filters: NormalizedSearchFilters, query: string, limit: number): LexicalRetrievalRow[] {
    const ftsQuery = buildFtsQuery(query);
    if (!ftsQuery) {
      return [];
    }

    const { sql, params } = buildLexicalRetrievalQuery(filters, ftsQuery, limit);
    return this.db.prepare(sql).all(...params) as LexicalRetrievalRow[];
  }

  private fetchSemanticRetrievalRows(filters: NormalizedSearchFilters, queryVector: Float32Array, limit: number): SemanticRetrievalRow[] {
    if (queryVector.length === 0) {
      return [];
    }

    const encodedQuery = encodeVector(queryVector);
    const { sql, params } = buildSemanticRetrievalQuery(filters, limit);
    return this.db.prepare(sql).all(encodedQuery, ...params) as SemanticRetrievalRow[];
  }

  private fetchRecordRowsByKeys(recordKeys: string[]): CandidateRow[] {
    if (recordKeys.length === 0) {
      return [];
    }

    const placeholders = buildPlaceholders(recordKeys);
    return this.db
      .prepare(
        `
          SELECT
            r.record_key AS recordKey,
            r.id AS id,
            r.name AS name,
            r.normalized_name AS normalizedName,
            r.record_type AS type,
            r.category AS category,
            r.subcategory AS subcategory,
            r.pack_name AS packName,
            r.pack_label AS packLabel,
            r.document_type AS documentType,
            r.level AS level,
            r.rarity AS rarity,
            r.traits_json AS traitsJson,
            r.derived_tags_json AS derivedTagsJson,
            r.publication_title AS publicationTitle,
            r.publication_remaster AS publicationRemaster,
            r.description_text AS descriptionText,
            r.has_description AS hasDescription,
            r.description_snippet AS descriptionSnippet,
            r.source_category AS sourceCategory,
            r.folder_id AS folderId,
            r.families_json AS familiesJson,
            r.source_path AS sourcePath,
            r.is_unique AS isUnique,
            r.is_search_canonical AS isSearchCanonical,
            a.size AS size,
            a.languages_json AS languagesJson,
            a.speed_types_json AS speedTypesJson,
            a.immunities_json AS immunitiesJson,
            a.resistances_json AS resistancesJson,
            a.weaknesses_json AS weaknessesJson,
            i.item_category AS itemCategory,
            i.price_cp AS priceCp,
            i.bulk_value AS bulkValue,
            i.usage_text AS usage,
            i.hands AS hands,
            COALESCE(s.damage_types_json, i.damage_types_json) AS damageTypesJson,
            i.weapon_group AS weaponGroup,
            i.armor_group AS armorGroup,
            COALESCE(s.action_cost, i.action_cost) AS actionCost,
            s.traditions_json AS traditionsJson,
            s.spell_kinds_json AS spellKindsJson,
            s.range_value AS rangeValue
          FROM records r
          LEFT JOIN actor_records a ON a.record_key = r.record_key
          LEFT JOIN item_records i ON i.record_key = r.record_key
          LEFT JOIN spell_records s ON s.record_key = r.record_key
          WHERE r.record_key IN (${placeholders})
        `,
      )
      .all(...recordKeys) as CandidateRow[];
  }

  private fetchReferenceEdgeRows(
    direction: RuleReferenceEdge["direction"],
    recordKeys: string[],
    {
      coreOnly = false,
      maxPerPrimary = 4,
    }: { coreOnly?: boolean; maxPerPrimary?: number } = {},
  ): RuleGraphResult {
    if (recordKeys.length === 0) {
      return { records: [], edges: [] };
    }

    const placeholders = buildPlaceholders(recordKeys);
    const targetFilter = direction === "outgoing"
      ? (coreOnly ? "AND target.source_category = 'core'" : "")
      : (coreOnly ? "AND re.from_source_category = 'core'" : "AND re.from_source_category IN ('core', 'rules')");
    const backlinkFilter = direction === "backlink"
      ? "AND (re.from_record_type = 'action' OR re.from_record_type = 'feat' OR LOWER(re.from_pack_name) = 'classfeatures')"
      : "";
    const keyColumn = direction === "outgoing" ? "re.from_record_key" : "re.to_record_key";

    const rows = this.db
      .prepare(
        `
          SELECT
            re.from_record_key AS fromRecordKey,
            re.to_record_key AS toRecordKey,
            re.display_text AS displayText,
            re.reference_text AS referenceText,
            re.from_pack_name AS fromPackName,
            re.from_record_type AS fromRecordType,
            re.from_document_type AS fromDocumentType,
            re.from_source_category AS fromSourceCategory
          FROM reference_edges re
          JOIN records target ON target.record_key = re.to_record_key
          WHERE ${keyColumn} IN (${placeholders})
          ${targetFilter}
          ${backlinkFilter}
        `,
      )
      .all(...recordKeys) as ReferenceEdgeRow[];

    const grouped = new Map<string, ReferenceEdgeRow[]>();
    for (const row of rows) {
      const groupKey = direction === "outgoing" ? row.fromRecordKey : row.toRecordKey;
      const bucket = grouped.get(groupKey) ?? [];
      bucket.push(row);
      grouped.set(groupKey, bucket);
    }

    const keptRows: ReferenceEdgeRow[] = [];
    for (const primaryKey of recordKeys) {
      const bucket = grouped.get(primaryKey) ?? [];
      bucket.sort((left, right) => {
        const leftTypeRank =
          left.fromPackName === "classfeatures" ? 2 : backlinkTypeRank(left.fromRecordType);
        const rightTypeRank =
          right.fromPackName === "classfeatures" ? 2 : backlinkTypeRank(right.fromRecordType);
        const leftLabel = left.displayText ?? (direction === "outgoing" ? left.toRecordKey : left.fromRecordKey);
        const rightLabel = right.displayText ?? (direction === "outgoing" ? right.toRecordKey : right.fromRecordKey);
        return (
          sourceCategoryRank(left.fromSourceCategory) - sourceCategoryRank(right.fromSourceCategory) ||
          leftTypeRank - rightTypeRank ||
          leftLabel.localeCompare(rightLabel) ||
          left.referenceText.localeCompare(right.referenceText)
        );
      });
      keptRows.push(...bucket.slice(0, Math.max(1, maxPerPrimary)));
    }

    const relatedRecordKeys = [
      ...new Set(keptRows.map((row) => direction === "outgoing" ? row.toRecordKey : row.fromRecordKey)),
    ];
    return {
      records: this.getRecordsByKeys(relatedRecordKeys),
      edges: keptRows.map((row) => edgeRowToReferenceEdge(row, direction)),
    };
  }

  getRecord(recordKeyOrPack: string, maybeId?: string): NormalizedRecord | undefined {
    const row = maybeId
      ? (this.db
          .prepare(
            `
              SELECT
                r.record_key AS recordKey,
                r.id AS id,
                r.name AS name,
                r.normalized_name AS normalizedName,
                r.record_type AS type,
                r.category AS category,
                r.subcategory AS subcategory,
                r.pack_name AS packName,
                r.pack_label AS packLabel,
                r.document_type AS documentType,
            r.level AS level,
            r.rarity AS rarity,
            r.traits_json AS traitsJson,
            r.derived_tags_json AS derivedTagsJson,
            r.publication_title AS publicationTitle,
            r.publication_remaster AS publicationRemaster,
            r.description_text AS descriptionText,
                r.has_description AS hasDescription,
                r.description_snippet AS descriptionSnippet,
                r.source_category AS sourceCategory,
                r.folder_id AS folderId,
                r.families_json AS familiesJson,
            r.source_path AS sourcePath,
            r.is_unique AS isUnique,
            r.is_search_canonical AS isSearchCanonical,
            a.size AS size,
                a.languages_json AS languagesJson,
                a.speed_types_json AS speedTypesJson,
                a.immunities_json AS immunitiesJson,
                a.resistances_json AS resistancesJson,
                a.weaknesses_json AS weaknessesJson,
                i.item_category AS itemCategory,
                i.price_cp AS priceCp,
                i.bulk_value AS bulkValue,
                i.usage_text AS usage,
                i.hands AS hands,
                COALESCE(s.damage_types_json, i.damage_types_json) AS damageTypesJson,
                i.weapon_group AS weaponGroup,
                i.armor_group AS armorGroup,
                COALESCE(s.action_cost, i.action_cost) AS actionCost,
                s.traditions_json AS traditionsJson,
                s.spell_kinds_json AS spellKindsJson,
                s.range_value AS rangeValue,
                r.raw_json AS rawJson
              FROM records r
              LEFT JOIN actor_records a ON a.record_key = r.record_key
              LEFT JOIN item_records i ON i.record_key = r.record_key
              LEFT JOIN spell_records s ON s.record_key = r.record_key
              WHERE r.pack_name = ? AND r.id = ?
            `,
          )
          .get(recordKeyOrPack, maybeId) as CandidateRow | undefined)
      : (this.db
          .prepare(
            `
              SELECT
                r.record_key AS recordKey,
                r.id AS id,
                r.name AS name,
                r.normalized_name AS normalizedName,
                r.record_type AS type,
                r.category AS category,
                r.subcategory AS subcategory,
                r.pack_name AS packName,
                r.pack_label AS packLabel,
                r.document_type AS documentType,
            r.level AS level,
            r.rarity AS rarity,
            r.traits_json AS traitsJson,
            r.derived_tags_json AS derivedTagsJson,
            r.publication_title AS publicationTitle,
            r.publication_remaster AS publicationRemaster,
            r.description_text AS descriptionText,
                r.has_description AS hasDescription,
                r.description_snippet AS descriptionSnippet,
                r.source_category AS sourceCategory,
                r.folder_id AS folderId,
                r.families_json AS familiesJson,
            r.source_path AS sourcePath,
            r.is_unique AS isUnique,
            r.is_search_canonical AS isSearchCanonical,
            a.size AS size,
                a.languages_json AS languagesJson,
                a.speed_types_json AS speedTypesJson,
                a.immunities_json AS immunitiesJson,
                a.resistances_json AS resistancesJson,
                a.weaknesses_json AS weaknessesJson,
                i.item_category AS itemCategory,
                i.price_cp AS priceCp,
                i.bulk_value AS bulkValue,
                i.usage_text AS usage,
                i.hands AS hands,
                COALESCE(s.damage_types_json, i.damage_types_json) AS damageTypesJson,
                i.weapon_group AS weaponGroup,
                i.armor_group AS armorGroup,
                COALESCE(s.action_cost, i.action_cost) AS actionCost,
                s.traditions_json AS traditionsJson,
                s.spell_kinds_json AS spellKindsJson,
                s.range_value AS rangeValue,
                r.raw_json AS rawJson
              FROM records r
              LEFT JOIN actor_records a ON a.record_key = r.record_key
              LEFT JOIN item_records i ON i.record_key = r.record_key
              LEFT JOIN spell_records s ON s.record_key = r.record_key
              WHERE r.record_key = ?
            `,
          )
          .get(recordKeyOrPack) as CandidateRow | undefined);

    if (!row) {
      return undefined;
    }

    return this.decorateRecord(rowToRecord(row));
  }

  getRecordsByKeys(recordKeys: string[]): NormalizedRecord[] {
    const rows = this.fetchRecordRowsByKeys([...new Set(recordKeys)]);
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
    const uniqueRecordKeys = [...new Set(recordKeys)];
    const directionsSpecified = includeOutgoing !== undefined || includeBacklinks !== undefined;
    const shouldIncludeOutgoing = directionsSpecified ? includeOutgoing === true : true;
    const shouldIncludeBacklinks = directionsSpecified ? includeBacklinks === true : false;
    const emptyGraph: RuleGraphResult = { records: [], edges: [] };
    const outgoing = shouldIncludeOutgoing
      ? this.fetchReferenceEdgeRows("outgoing", uniqueRecordKeys, {
          coreOnly,
          maxPerPrimary: maxOutgoingPerPrimary,
        })
      : emptyGraph;
    const backlinks = shouldIncludeBacklinks
      ? this.fetchReferenceEdgeRows("backlink", uniqueRecordKeys, {
          coreOnly,
          maxPerPrimary: maxBacklinksPerPrimary,
        })
      : emptyGraph;

    return {
      outgoing,
      backlinks,
      edges: [...outgoing.edges, ...backlinks.edges],
    };
  }

  collectRuleQuestionContext(input: CollectRuleQuestionContextInput): CollectRuleQuestionContextResult {
    const explicitRules = (input.rules ?? []).map((rule) => rule.trim()).filter((rule) => rule.length > 0);
    const derivedRules = explicitRules.length > 0
      ? explicitRules
      : input.question
        ? extractQuestionRuleNames(input.question)
        : [];
    const primary = this.lookupMany(derivedRules.map((name) => ({ name })), { coreOnly: input.coreOnly });
    const primaryKeys = primary
      .map((result) => result.match?.recordKey ?? null)
      .filter((recordKey): recordKey is string => Boolean(recordKey));
    const graph = this.getRuleGraph(primaryKeys, {
      coreOnly: input.coreOnly,
      includeOutgoing: true,
      includeBacklinks: input.includeBacklinks,
      maxOutgoingPerPrimary: input.maxOutgoingPerPrimary ?? 4,
      maxBacklinksPerPrimary: input.maxBacklinksPerPrimary ?? 4,
    });

    return {
      primary,
      ...graph,
    };
  }

  private searchStructured(filters: SearchFilters): SearchResult {
    const normalizedFilters = this.normalizeSearchFilters(filters);
    validateFilters(normalizedFilters, "search");
    const limit = clampLimit(normalizedFilters.limit);
    const offset = clampOffset(normalizedFilters.offset);
    const mode = resolveSearchMode(normalizedFilters, "search");
    const searchProfile = resolveSearchProfile(normalizedFilters, "search", mode);
    const rankingConfig = this.rankingConfigStore?.getConfig() ?? DEFAULT_RANKING_CONFIG;
    const candidates = this.fetchCandidates(normalizedFilters);
    const scored = candidates
      .map((candidate) => {
        const record = this.decorateRecord(rowToRecord(candidate));
        const packQuality = packQualityScore(record, rankingConfig);
        const sourceQuality = sourceQualityScore(record, rankingConfig);
        const rarityPreference = rarityPreferenceScore(record, normalizedFilters, rankingConfig);
        const sourcePenalty = sourcePenaltyScore(record, normalizedFilters, rankingConfig);
        const score =
          (normalizedFilters.nameQuery ? nameScore(normalizedFilters.nameQuery, record, this.aliasesByRecordKey.get(record.recordKey) ?? []) : 0.5) +
          packQuality +
          sourceQuality +
          rarityPreference +
          sourcePenalty;

        return { record, score };
      })
      .filter(({ score }) => {
        if (normalizedFilters.nameQuery) {
          return score >= 0.2;
        }

        return true;
      })
      .sort((left, right) => right.score - left.score || sortRecords(left.record, right.record));

    return {
      searchProfile,
      mode: "structured",
      total: scored.length,
      offset,
      limit,
      records: scored.slice(offset, offset + limit).map(({ record }) => record),
    };
  }

  listRecords(filters: SearchFilters): SearchResult {
    const normalizedFilters = this.normalizeSearchFilters(filters);
    validateFilters(normalizedFilters, "list");
    const limit = clampLimit(normalizedFilters.limit);
    const offset = clampOffset(normalizedFilters.offset);
    const records = this.fetchCandidates(normalizedFilters).map((row) => this.decorateRecord(rowToRecord(row)));
    records.sort((left, right) => sortRecords(left, right));
    return {
      searchProfile: null,
      mode: "structured",
      total: records.length,
      offset,
      limit,
      records: records.slice(offset, offset + limit),
    };
  }

  async search(filters: SearchFilters): Promise<SearchResult> {
    const normalizedFilters = this.normalizeSearchFilters(filters);
    validateFilters(normalizedFilters, "search");
    const limit = clampLimit(normalizedFilters.limit);
    const offset = clampOffset(normalizedFilters.offset);
    const mode = resolveSearchMode(normalizedFilters, "search");
    const searchProfile = resolveSearchProfile(normalizedFilters, "search", mode);
    const rawSemanticQuery = normalizedFilters.query?.trim() || "";
    const rawLexicalQuery = normalizedFilters.query?.trim() || normalizedFilters.nameQuery?.trim() || "";
    const rankingConfig = this.rankingConfigStore?.getConfig() ?? DEFAULT_RANKING_CONFIG;
    const hybridFusion = resolveHybridFusionProfile(searchProfile, mode, rankingConfig);
    const queryAnalysis = rawLexicalQuery
      ? buildSearchQueryAnalysis(rawLexicalQuery)
      : null;
    const literalQueryWeights = queryAnalysis
      ? buildLiteralQueryWeights(queryAnalysis)
      : null;
    const lexicalQuery = queryAnalysis?.normalizedQuery ?? rawLexicalQuery;
    const semanticVector = hybridFusion && rawSemanticQuery
      ? await this.embeddingProvider.embed(rawSemanticQuery)
      : null;
    const lexicalRetrievalRows = lexicalQuery
      ? this.fetchLexicalRetrievalRows(
          normalizedFilters,
          lexicalQuery,
          Math.max(mode === "lexical" ? LOOKUP_LEXICAL_TOP_K : (hybridFusion?.config.lexicalTopK ?? 0), (offset + limit) * 5),
        )
      : [];
    const lexicalRetrievedKeys = lexicalRetrievalRows.map((row) => row.recordKey);
    const lexicalRetrievalRanks = buildRankMap(lexicalRetrievedKeys);
    const lexicalMatches = buildNormalizedRankScoreMap(lexicalRetrievedKeys);

    const semanticRetrievalRows = semanticVector && hybridFusion
      ? this.fetchSemanticRetrievalRows(
          normalizedFilters,
          semanticVector,
          semanticQueryLimit(Math.max(hybridFusion.config.semanticTopK, (offset + limit) * 5), normalizedFilters),
        )
      : [];
    const semanticRetrievedKeys = semanticRetrievalRows.map((row) => row.recordKey);
    const semanticRetrievalRanks = buildRankMap(semanticRetrievedKeys);

    const candidateKeys = mode === "structured"
      ? []
      : [...new Set([...lexicalRetrievedKeys, ...semanticRetrievedKeys])];
    const candidateRows = mode === "structured"
      ? []
      : this.fetchCandidates(normalizedFilters, false, false, { recordKeys: candidateKeys });
    const candidateRecords = candidateRows
      .map((row) => this.decorateRecord(rowToRecord(row)))
      .filter((record) => recordMatchesFilters(record, normalizedFilters));
    const candidatesByKey = new Map(candidateRecords.map((record) => [record.recordKey, record]));

    const scored = (() => {
      if (mode === "structured") {
        return this.fetchCandidates(normalizedFilters)
          .map((candidate) => {
            const record = this.decorateRecord(rowToRecord(candidate));
            const rerankAdjustments = buildRerankAdjustments(record, normalizedFilters, rankingConfig);
            const totalScore =
              (normalizedFilters.nameQuery
                ? nameScore(normalizedFilters.nameQuery, record, this.aliasesByRecordKey.get(record.recordKey) ?? [])
                : 0.5) +
              sumRerankAdjustments(rerankAdjustments);
            const explanation: SearchRecordExplanation = {
              recordKey: record.recordKey,
              name: record.name,
              totalScore,
              fusionScore: null,
              lexicalRank: null,
              semanticRank: null,
              lexicalRerankScore: null,
              matchedTraits: [],
              matchedNameTokens: [],
              rerankAdjustments,
            };

            return { record, totalScore, explanation };
          })
          .filter(({ totalScore }) => {
            if (normalizedFilters.nameQuery) {
              return totalScore >= 0.2;
            }

            return true;
          })
          .sort((left, right) => right.totalScore - left.totalScore || sortRecords(left.record, right.record));
      }

      if (mode === "lexical") {
        return lexicalRetrievedKeys
          .map((recordKey) => candidatesByKey.get(recordKey))
          .filter((record): record is NormalizedRecord => Boolean(record))
          .map((record) => {
            const lexicalSignal = buildLexicalSignal(record, lexicalQuery, literalQueryWeights, lexicalMatches, rankingConfig);
            const rerankAdjustments = buildRerankAdjustments(record, normalizedFilters, rankingConfig);
            const totalScore = lexicalSignal.lexicalScore + sumRerankAdjustments(rerankAdjustments);
            const explanation: SearchRecordExplanation = {
              recordKey: record.recordKey,
              name: record.name,
              totalScore,
              fusionScore: null,
              lexicalRank: lexicalRetrievalRanks.get(record.recordKey) ?? null,
              semanticRank: null,
              lexicalRerankScore: lexicalSignal.lexicalScore,
              matchedTraits: lexicalSignal.matchedTraits,
              matchedNameTokens: lexicalSignal.matchedNameTokens,
              rerankAdjustments,
            };

            return {
              record,
              totalScore,
              lexicalRank: lexicalRetrievalRanks.get(record.recordKey) ?? null,
              lexicalRerankScore: lexicalSignal.lexicalScore,
              explanation,
            };
          })
          .filter(({ totalScore }) => {
            if (lexicalQuery) {
              return totalScore > 0;
            }

            return true;
          })
          .sort((left, right) => {
            return (
              right.totalScore - left.totalScore ||
              right.lexicalRerankScore - left.lexicalRerankScore ||
              compareOptionalRanks(left.lexicalRank, right.lexicalRank) ||
              sortRecords(left.record, right.record)
            );
          });
      }

      const fusionConfig = hybridFusion!.config;
      const rerankedLexical = lexicalRetrievedKeys
        .map((recordKey) => candidatesByKey.get(recordKey))
        .filter((record): record is NormalizedRecord => Boolean(record))
        .map((record) => ({
          record,
          lexicalSignal: buildLexicalSignal(record, lexicalQuery, literalQueryWeights, lexicalMatches, rankingConfig),
        }))
        .filter(({ lexicalSignal }) => lexicalSignal.lexicalScore > 0)
        .sort((left, right) => {
          return (
            right.lexicalSignal.lexicalScore - left.lexicalSignal.lexicalScore ||
            compareOptionalRanks(
              semanticRetrievalRanks.get(left.record.recordKey) ?? null,
              semanticRetrievalRanks.get(right.record.recordKey) ?? null,
            ) ||
            sortRecords(left.record, right.record)
          );
        })
        .slice(0, fusionConfig.lexicalTopK);
      const rerankedLexicalRanks = buildRankMap(rerankedLexical.map(({ record }) => record.recordKey));
      const semanticRanks = buildRankMap(
        semanticRetrievedKeys
          .filter((recordKey) => candidatesByKey.has(recordKey))
          .slice(0, fusionConfig.semanticTopK),
      );

      return candidateRecords
        .filter((record) => rerankedLexicalRanks.has(record.recordKey) || semanticRanks.has(record.recordKey))
        .map((record) => {
          const lexicalSignal = buildLexicalSignal(record, lexicalQuery, literalQueryWeights, lexicalMatches, rankingConfig);
          const rerankAdjustments = buildRerankAdjustments(record, normalizedFilters, rankingConfig);
          const lexicalRank = rerankedLexicalRanks.get(record.recordKey) ?? null;
          const semanticRank = semanticRanks.get(record.recordKey) ?? null;
          const fusionScore = computeWeightedRrfScore(
            lexicalRank,
            semanticRank,
            fusionConfig,
            rankingConfig.hybridFusion.rrfK,
          );
          const totalScore = fusionScore + sumRerankAdjustments(rerankAdjustments);
          const explanation: SearchRecordExplanation = {
            recordKey: record.recordKey,
            name: record.name,
            totalScore,
            fusionScore,
            lexicalRank,
            semanticRank,
            lexicalRerankScore: lexicalSignal.lexicalScore,
            matchedTraits: lexicalSignal.matchedTraits,
            matchedNameTokens: lexicalSignal.matchedNameTokens,
            rerankAdjustments,
          };

          return {
            record,
            totalScore,
            fusionScore,
            lexicalRank,
            semanticRank,
            lexicalRerankScore: lexicalSignal.lexicalScore,
            explanation,
          };
        })
        .sort((left, right) => {
          return (
            right.totalScore - left.totalScore ||
            right.fusionScore - left.fusionScore ||
            compareOptionalRanks(left.semanticRank, right.semanticRank) ||
            compareOptionalRanks(left.lexicalRank, right.lexicalRank) ||
            right.lexicalRerankScore - left.lexicalRerankScore ||
            sortRecords(left.record, right.record)
          );
        });
    })();

    const page = scored.slice(offset, offset + limit);
    const explain: SearchExplainResult | undefined = filters.explain
      ? {
          searchProfile,
          mode,
          fusionMethod: hybridFusion ? "weightedRrf" : null,
          fusionProfile: hybridFusion?.profile ?? null,
          fusionConfig: buildFusionConfigSummary(hybridFusion?.profile ?? null, hybridFusion?.config ?? null, rankingConfig),
          lexicalQuery,
          semanticQuery: rawSemanticQuery,
          query: queryAnalysis
            ? {
                rawQuery: queryAnalysis.rawQuery,
                normalizedQuery: queryAnalysis.normalizedQuery,
                queryTokens: queryAnalysis.queryTokens,
              }
            : null,
          rankingConfig: this.getRankingConfigStatus(),
          records: page.map(({ explanation }) => explanation),
        }
      : undefined;

    return {
      searchProfile,
      mode,
      total: scored.length,
      offset,
      limit,
      records: page.map(({ record }) => record),
      explain,
    };
  }

  lookup(name: string, options: LookupOptions = {}): { match: NormalizedRecord | null; alternatives: NormalizedRecord[] } {
    const results = this.searchStructured({
      nameQuery: name,
      pack: options.pack,
      category: options.category,
      subcategory: options.subcategory,
      limit: 5,
    }).records;

    return {
      match: results[0] ?? null,
      alternatives: results.slice(1),
    };
  }
}
