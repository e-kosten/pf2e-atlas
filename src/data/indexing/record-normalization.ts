import type { BuildSourceEntry, PackBuildInfo } from "../index-types.js";
import {
  normalizeIndexRecord,
  parseActorIndexData,
  parseItemIndexData,
  parseSpellIndexData,
  shouldExcludeRecordFromIndex,
} from "../record-normalization.js";
import { extractRulesReferences } from "../references.js";

export function normalizeSourceEntry(
  pack: PackBuildInfo,
  filePath: string,
  raw: Record<string, unknown>,
): BuildSourceEntry {
  if (shouldExcludeRecordFromIndex(pack, filePath, raw)) {
    return {
      pack,
      filePath,
      raw,
      record: null,
      actorData: null,
      itemData: null,
      spellData: null,
      references: [],
      resolvedReferences: [],
    };
  }

  const record = normalizeIndexRecord(pack, filePath, raw);
  return {
    pack,
    filePath,
    raw,
    record,
    actorData: pack.documentType === "Actor" ? parseActorIndexData(raw) : null,
    itemData: pack.documentType === "Item" ? parseItemIndexData(raw) : null,
    spellData: record.type === "spell" ? parseSpellIndexData(raw) : null,
    references: extractRulesReferences(raw),
    resolvedReferences: [],
  };
}
