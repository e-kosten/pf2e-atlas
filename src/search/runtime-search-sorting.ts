import type { NormalizedRecord } from "../domain/record-types.js";
import type { SearchSort } from "../domain/search-types.js";
import { normalizeText } from "../shared/utils.js";
import { scoreNameCandidate } from "./ranking.js";

export function nameScore(query: string, record: NormalizedRecord, aliases: string[] = []): number {
  let best = scoreNameCandidate(query, record.normalizedName);
  for (const alias of aliases) {
    best = Math.max(best, scoreNameCandidate(query, normalizeText(alias)));
  }
  return best;
}

export function sortRecords(left: NormalizedRecord, right: NormalizedRecord): number {
  return (
    left.name.localeCompare(right.name) ||
    left.packLabel.localeCompare(right.packLabel) ||
    left.id.localeCompare(right.id)
  );
}

function compareNullableLevel(left: number | null, right: number | null): number {
  if (left === null && right === null) {
    return 0;
  }
  if (left === null) {
    return 1;
  }
  if (right === null) {
    return -1;
  }
  return left - right;
}

function hashRecordSortSeed(recordKey: string, seed: number): number {
  let hash = seed | 0;
  for (let index = 0; index < recordKey.length; index += 1) {
    hash = Math.imul(hash ^ recordKey.charCodeAt(index), 16777619);
  }
  return hash >>> 0;
}

export function compareRecordsForSort(
  left: NormalizedRecord,
  right: NormalizedRecord,
  sort: SearchSort,
  sortSeed: number,
): number {
  switch (sort) {
    case "levelAsc":
      return compareNullableLevel(left.level, right.level) || sortRecords(left, right);
    case "levelDesc":
      return compareNullableLevel(right.level, left.level) || sortRecords(left, right);
    case "random":
      return (
        hashRecordSortSeed(left.recordKey, sortSeed) - hashRecordSortSeed(right.recordKey, sortSeed) ||
        sortRecords(left, right)
      );
    case "alphabetical":
    case "ranked":
    default:
      return sortRecords(left, right);
  }
}
