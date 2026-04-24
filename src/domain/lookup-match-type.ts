import type { NormalizedRecord } from "./record-types.js";
import type { LookupResult } from "./search-types.js";
import { normalizeText } from "../shared/utils.js";

export type LookupMatchType = LookupResult["matchType"];

export function getLookupMatchType(
  query: string,
  record: Pick<NormalizedRecord, "name"> | null,
): LookupMatchType {
  if (!record) {
    return "none";
  }

  if (normalizeText(record.name) === normalizeText(query)) {
    return query.trim() === record.name ? "exact" : "normalized_exact";
  }

  return "fuzzy";
}
