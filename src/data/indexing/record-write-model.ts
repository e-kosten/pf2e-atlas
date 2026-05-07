import { uniqueSorted } from "../../shared/utils.js";
import type {
  IndexedBuildSourceEntry,
  RecordAliasRow,
  RecordLegacyLinkRow,
  WritableIndexEntry,
} from "../index-types.js";
import type { DerivedAfflictionBuild } from "../derived-afflictions.js";

export type RecordWriteModel = {
  writableEntries: WritableIndexEntry[];
};

export function createRecordWriteModel(input: {
  indexedEntries: IndexedBuildSourceEntry[];
  derivedAfflictions: DerivedAfflictionBuild;
  aliasRows: RecordAliasRow[];
  legacyLinkRows: RecordLegacyLinkRow[];
}): RecordWriteModel {
  const suppressedRecordKeys = new Set(input.legacyLinkRows.map((row) => row.legacyRecordKey));
  const aliasesByCanonicalRecordKey = new Map<string, string[]>();
  for (const alias of input.aliasRows) {
    const bucket = aliasesByCanonicalRecordKey.get(alias.canonicalRecordKey) ?? [];
    bucket.push(alias.aliasText);
    aliasesByCanonicalRecordKey.set(alias.canonicalRecordKey, uniqueSorted(bucket));
  }

  return {
    writableEntries: [
      ...input.indexedEntries.map((entry) => ({
        record: entry.record,
        raw: entry.raw,
        actorData: entry.actorData,
        itemData: entry.itemData,
        spellData: entry.spellData,
        resolvedReferences: entry.resolvedReferences,
        aliasTexts: aliasesByCanonicalRecordKey.get(entry.record.recordKey) ?? [],
        isSearchCanonical: !suppressedRecordKeys.has(entry.record.recordKey),
      })),
      ...input.derivedAfflictions.records.map((entry) => ({
        record: entry.record,
        raw: entry.raw,
        actorData: entry.actorData,
        itemData: entry.itemData,
        spellData: entry.spellData,
        resolvedReferences: entry.resolvedReferences,
        aliasTexts: [],
        isSearchCanonical: entry.isSearchCanonical,
      })),
    ],
  };
}
