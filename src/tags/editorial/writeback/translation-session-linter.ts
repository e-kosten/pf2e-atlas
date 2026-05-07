import { applyDerivedTagTranslationOverride } from "../../translations/record-utils.js";
import type { DerivedTagTranslationReviewSession } from "../types.js";

function assertNonEmpty(value: string | undefined, context: string): void {
  if (!value || !value.trim()) {
    throw new Error(`Expected ${context} to be non-empty.`);
  }
}

export function lintDerivedTagTranslationReviewSession(session: DerivedTagTranslationReviewSession): void {
  const seenKeys = new Set<string>();
  for (const row of session.rows) {
    if (seenKeys.has(row.key)) {
      throw new Error(`Duplicate translation review row key ${row.key}.`);
    }
    seenKeys.add(row.key);
    const effective = applyDerivedTagTranslationOverride(row.base, row.draftOverride);
    if (effective.translationStatus !== "dropped") {
      assertNonEmpty(effective.canonicalConceptId, `${row.key} canonicalConceptId`);
      assertNonEmpty(effective.canonicalConceptLabel, `${row.key} canonicalConceptLabel`);
      assertNonEmpty(effective.projectionAxis, `${row.key} projectionAxis`);
      assertNonEmpty(effective.projectionFamily, `${row.key} projectionFamily`);
    }
  }
}

