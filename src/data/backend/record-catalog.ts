import type { DatabaseSync } from "node:sqlite";

import type {
  FilterValueQuery,
  FilterValueResult,
  LinkedRecordSummary,
  NormalizedRecord,
  PackInfo,
  RankingConfigStatus,
} from "../../domain/index.js";
import type { NormalizedSearchFilters } from "../../search/contracts.js";
import type { RankingConfigStore } from "../../search/ranking-config.js";
import type { SearchVocabularyResult } from "../vocabulary.js";
import {
  getPack as getPackRuntime,
  getSearchVocabulary as getSearchVocabularyRuntime,
  listFilterValues as listFilterValuesRuntime,
  listPacks as listPacksRuntime,
} from "../vocabulary.js";
import { fetchRecordRow, fetchRecordRowsByKeys } from "../record-queries.js";
import { rowToRecord } from "../rows.js";
import { loadAliasesByRecordKey, loadLegacyLinksByRecordKey } from "../schema.js";

export class Pf2eRecordCatalog {
  private readonly aliasesByRecordKey: Map<string, string[]>;
  private readonly legacyLinksByRecordKey: Map<string, LinkedRecordSummary[]>;

  constructor(
    private readonly db: DatabaseSync,
    private readonly packs: PackInfo[],
    private readonly rankingConfigStore: RankingConfigStore | null,
  ) {
    this.aliasesByRecordKey = loadAliasesByRecordKey(db);
    this.legacyLinksByRecordKey = loadLegacyLinksByRecordKey(db);
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
