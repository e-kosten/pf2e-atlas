import { buildEffectiveDerivedTagTranslationRecord } from "../../translations/publication.js";
import type { DerivedTagTranslationReviewSession } from "../types.js";

export function lintDerivedTagTranslationReviewSession(session: DerivedTagTranslationReviewSession): void {
  const seenKeys = new Set<string>();
  for (const row of session.rows) {
    if (seenKeys.has(row.key)) {
      throw new Error(`Duplicate translation review row key ${row.key}.`);
    }
    seenKeys.add(row.key);
    const effective = buildEffectiveDerivedTagTranslationRecord(row.base, row.draftOverride);
    if (
      (effective.translationStatus === "mapped" || effective.translationStatus === "provisional") &&
      !effective.targetProjectionId?.trim()
    ) {
      throw new Error(`Expected ${row.key} targetProjectionId to be non-empty for ${effective.translationStatus}.`);
    }
  }
}
