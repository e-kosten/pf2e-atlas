import type { DatabaseSync } from "node:sqlite";

import type {
  FilterValueQuery,
  FilterValueResult,
  RankingConfigStatus,
} from "../../domain/search-types.js";
import type { LinkedRecordSummary, NormalizedRecord, PackInfo } from "../../domain/record-types.js";
import type { NormalizedSearchFilters } from "../../search/contracts.js";
import type { RankingConfigStore } from "../../search/ranking-config.js";
import type {
  SearchCategorySummaryResult,
  SearchSemanticsBootstrapSummaryResult,
  SearchVocabularyResult,
} from "../vocabulary.js";
import {
  getPack as getPackRuntime,
  getSearchCategorySummary as getSearchCategorySummaryRuntime,
  getSearchSemanticsBootstrapSummary as getSearchSemanticsBootstrapSummaryRuntime,
  getSearchVocabulary as getSearchVocabularyRuntime,
  listFilterValues as listFilterValuesRuntime,
  listPacks as listPacksRuntime,
  normalizeTraitLimitPerCategory,
} from "../vocabulary.js";
import { fetchRecordRow, fetchRecordRowsByKeys } from "../record-queries.js";
import { rowToRecord } from "../rows.js";
import { loadAliasesByRecordKey, loadLegacyLinksByRecordKey } from "../schema.js";

export class Pf2eRecordCatalog {
  private readonly aliasesByRecordKey: Map<string, string[]>;
  private readonly legacyLinksByRecordKey: Map<string, LinkedRecordSummary[]>;
  private searchCategorySummaryCache: SearchCategorySummaryResult | null = null;
  private readonly searchSemanticsBootstrapSummaryCache = new Map<number, SearchSemanticsBootstrapSummaryResult>();

  constructor(
    private readonly db: DatabaseSync,
    private readonly packs: PackInfo[],
    private readonly rankingConfigStore: RankingConfigStore | null,
  ) {
    this.aliasesByRecordKey = loadAliasesByRecordKey(db);
    this.legacyLinksByRecordKey = loadLegacyLinksByRecordKey(db);
  }

  getSearchCategorySummary(): SearchCategorySummaryResult {
    if (!this.searchCategorySummaryCache) {
      this.searchCategorySummaryCache = getSearchCategorySummaryRuntime(this.db);
    }
    return this.searchCategorySummaryCache;
  }

  getSearchSemanticsBootstrapSummary(
    options: { traitLimitPerCategory?: number } = {},
  ): SearchSemanticsBootstrapSummaryResult {
    const traitLimit = normalizeTraitLimitPerCategory(options.traitLimitPerCategory);
    const cached = this.searchSemanticsBootstrapSummaryCache.get(traitLimit);
    if (cached) {
      return cached;
    }

    const summary = getSearchSemanticsBootstrapSummaryRuntime(this.db, { traitLimitPerCategory: traitLimit });
    this.searchSemanticsBootstrapSummaryCache.set(traitLimit, summary);
    return summary;
  }

  getSearchVocabulary(options: { traitLimitPerCategory?: number } = {}): SearchVocabularyResult {
    return getSearchVocabularyRuntime(this.db, options);
  }

  getRankingConfigStatus(): RankingConfigStatus {
    return (
      this.rankingConfigStore?.getStatus() ?? {
        path: "<defaults>",
        source: "default" as const,
        revision: 1,
        loadedAt: new Date(0).toISOString(),
        lastError: null,
      }
    );
  }

  getPack(packValue: string): PackInfo | undefined {
    return getPackRuntime(this.packs, packValue);
  }

  listPacks(): PackInfo[] {
    return listPacksRuntime(this.packs);
  }

  listFilterValues(query: FilterValueQuery, normalizedFilters: NormalizedSearchFilters): FilterValueResult {
    return listFilterValuesRuntime(this.db, query, normalizedFilters);
  }

  getAliases(recordKey: string): string[] {
    return this.aliasesByRecordKey.get(recordKey) ?? [];
  }

  decorateRecord(record: NormalizedRecord): NormalizedRecord {
    return {
      ...record,
      aliases: this.aliasesByRecordKey.get(record.recordKey) ?? [],
      legacyRecordLinks: this.legacyLinksByRecordKey.get(record.recordKey) ?? [],
    };
  }

  getRecord(recordKeyOrPack: string, maybeId?: string): NormalizedRecord | undefined {
    const row = fetchRecordRow(this.db, recordKeyOrPack, maybeId);
    return row ? this.decorateRecord(rowToRecord(row)) : undefined;
  }

  getRecordsByKeys(recordKeys: string[]): NormalizedRecord[] {
    const dedupedRecordKeys = [...new Set(recordKeys)];
    const rows = fetchRecordRowsByKeys(this.db, dedupedRecordKeys);
    const byKey = new Map(rows.map((row) => [row.recordKey, this.decorateRecord(rowToRecord(row))]));
    return dedupedRecordKeys
      .map((recordKey) => byKey.get(recordKey))
      .filter((record): record is NormalizedRecord => Boolean(record));
  }
}
